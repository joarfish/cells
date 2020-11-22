use super::{lights::LightsResources, meshes::MeshResources, utils::GpuVector3};
use crate::renderer::meshes::Mesh;
use crate::renderer::command_queue::CommandQueue;
use crate::renderer::utils::{GpuMatrix4BGA, GpuVector3BGA};

use wgpu::util::*;

pub struct RenderShadowMeshCommand {
    pub mesh: Mesh,
    pub distance: u8
}

impl From<u32> for RenderShadowMeshCommand {
    fn from(other: u32) -> Self {
        RenderShadowMeshCommand {
            mesh: Mesh {
                pool_index: ((0b1111_0000_0000_0000_0000_0000_0000_0000 & other) >> 28) as u16,
                geometry_index: ((0b0000_1111_1111_0000_0000_0000_0000_0000 & other) >> 20) as u32,
                object_index: (0b0000_0000_0000_0000_1111_1111_1111_1111 & other) as u32,
            },
            distance: ((0b0000_0000_0000_1111_0000_0000_0000_0000 & other) >> 16) as u8,
        }
    }
}

impl Into<u32> for RenderShadowMeshCommand {
    fn into(self) -> u32 {
        (self.mesh.pool_index as u32) << 28 |
            (self.mesh.geometry_index << 20) |
            (self.distance as u32) << 16 |
            self.mesh.object_index
    }
}

use cgmath::SquareMatrix;

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone)]
pub struct GpuLightView {
    pub view_matrix: cgmath::Matrix4<f32>,
}

impl Default for GpuLightView {
    fn default() -> Self {
        let view_matrix = cgmath::perspective(cgmath::Deg(45.0), 1.333334, 10.0, 20.0)//cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 1.0, 100.0)
            * cgmath::Matrix4::look_at(
                cgmath::Point3::new(5.0, 15.0, -5.0),
                cgmath::Point3::new(0.0, 0.0, 0.0),
                cgmath::Vector3::unit_z()
            );

        GpuLightView {
            view_matrix,
        }
    }
}

unsafe impl bytemuck::Pod for GpuLightView {}
unsafe impl bytemuck::Zeroable for GpuLightView {}

pub struct ShadowPasses {
    shadow_texture: wgpu::Texture,
    pub shadow_texture_view: wgpu::TextureView,
    pub shadow_sampler: wgpu::Sampler,
    shadow_light_buffer: wgpu::Buffer,
    pub shadow_light_bind_group: wgpu::BindGroup,
    pub shadow_light_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl ShadowPasses {
    pub fn new(device: &wgpu::Device, mesh_resources: &MeshResources, window_width: u32, window_height: u32) -> Self {

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadows Texture"),
            format: wgpu::TextureFormat::Depth32Float,
            dimension: wgpu::TextureDimension::D2,
            sample_count: 1,
            mip_level_count: 1,
            size: wgpu::Extent3d { 
                width: window_width * 2,
                height: window_height * 2,
                depth: 1
            },
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED
        });
    
        let shadow_texture_view = shadow_texture.create_view(& wgpu::TextureViewDescriptor::default());

        let shadow_light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            contents: bytemuck::cast_slice(&[ GpuLightView::default() ])
        });

        let shadow_light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        });

        let shadow_light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(shadow_light_buffer.slice(..))
                }
            ],
            layout: &shadow_light_bind_group_layout
        });

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });

        // create vertex shader:
    
        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;

            layout(set=0, binding=0)
            uniform SceneUniforms {
                mat4 light_view_matrix;
            };

            layout(set=1, binding=0)
            uniform ModelUniforms {
                mat4 u_transform;
            };

            void main() {        
                gl_Position = light_view_matrix * u_transform * vec4(a_position, 1.0);
            }        
        ".to_string();

        let vs_spirv = compiler.compile_into_spirv(
                &vs_code,
                shaderc::ShaderKind::Vertex,
                "shadow_pass.vert",
                "main",
                None,
            )
            .unwrap();

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(vs_spirv.as_binary()),
        ));

        // Create the render pipeline

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &shadow_light_bind_group_layout,
                &mesh_resources.buffer_bind_group_layout
            ],
            push_constant_ranges: &[],
        });
    
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_shader_module,
                entry_point: "main",
            },
            fragment_stage: None,
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 2, // corresponds to bilinear filtering
                depth_bias_slope_scale: 2.0,
                depth_bias_clamp: 0.0,
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[ wgpu::VertexBufferDescriptor {
                    attributes: &[ 
                        wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float3
                        },
                    ],
                    step_mode: wgpu::InputStepMode::Vertex,
                    stride: (std::mem::size_of::<GpuVector3>()) as wgpu::BufferAddress
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
    
        ShadowPasses {
            shadow_texture,
            shadow_texture_view,
            shadow_sampler,
            shadow_light_buffer,
            shadow_light_bind_group_layout,
            shadow_light_bind_group,
            pipeline
        }
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, mesh_resources: &MeshResources, shadow_mesh_commands: &mut CommandQueue<RenderShadowMeshCommand>) {

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[ ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.shadow_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None
                }),
            });

            render_pass.push_debug_group("Begin Shadow Pass");

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.shadow_light_bind_group, &[]);

            while let Some(command) = shadow_mesh_commands.pop_command() {

                let mesh_pool = mesh_resources.mesh_pools.get(command.mesh.pool_index as usize).unwrap();
                let geometry = mesh_resources.geometries.get(command.mesh.geometry_index as usize).unwrap();

                render_pass.set_bind_group(1, &mesh_pool.bind_group,
                       &[
                           command.mesh.object_index * (std::mem::size_of::<GpuMatrix4BGA>() as u32),
                           command.mesh.object_index * (std::mem::size_of::<GpuVector3BGA>() as u32)
                       ]
                );

                render_pass.set_vertex_buffer(0, geometry.positions_buffer.slice(..));
                render_pass.set_vertex_buffer(1, geometry.normals_buffer.slice(..));

                render_pass.set_index_buffer(geometry.index_buffer.slice(..));
                render_pass.draw_indexed(0..geometry.index_count, 0, 0..1);
            }

            render_pass.pop_debug_group();
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

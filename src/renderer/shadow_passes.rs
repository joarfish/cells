use super::{lights::LightsResources, meshes::MeshResources, utils::GpuVector3};
use crate::renderer::command_queue::{CommandQueue, Batch, Command};
use crate::renderer::utils::{GpuMatrix4BGA, GpuVector3BGA};
use std::ops::Not;

use wgpu::util::*;

#[derive(Clone)]
pub struct RenderShadowMeshCommand {
    pub mesh_type: u8,
    pub object_index: u16,
    pub order: u16,
}

impl Command for RenderShadowMeshCommand {
    fn is_compatible(&self, other: &Self) -> bool {
        self.mesh_type == other.mesh_type
    }
}

impl From<u32> for RenderShadowMeshCommand {
    fn from(other: u32) -> Self {
        RenderShadowMeshCommand {
            mesh_type: ((0b1111_1110_0000_0000_0000_0000_0000_0000 & other) >> 25) as u8,
            object_index: ((0b0000_0000_0000_1111_1111_1100_0000_0000 & other) >> 10) as u16,
            order: (0b0000_0000_0000_0000_0000_0011_1111_1111 & other) as u16,
        }
    }
}

impl Into<u32> for RenderShadowMeshCommand {
    fn into(self) -> u32 {
        (self.mesh_type as u32) << 25 |
        (self.object_index as u32) << 10 |
        (self.order as u32)
    }
}

pub struct RenderShadowBatch {
    pub object_indices: Vec<u32>,
    pub mesh_type: u16,
}

impl Batch<RenderShadowMeshCommand> for RenderShadowBatch {
    fn new(first_command: RenderShadowMeshCommand) -> Self {
        RenderShadowBatch {
            object_indices: Vec::new(),
            mesh_type: first_command.mesh_type as u16,
        }
    }

    fn add_command(&mut self, command: &RenderShadowMeshCommand) -> bool {
        if command.mesh_type == self.mesh_type as u8 {
            if !self.object_indices.contains(&(command.object_index as u32)) {
                self.object_indices.push(command.object_index as u32);
            }
            true
        } else {
            false
        }
    }
}

use cgmath::SquareMatrix;
use crate::renderer::material::MaterialResources;

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone)]
pub struct GpuLightView {
    pub view_matrix: cgmath::Matrix4<f32>,
}

impl Default for GpuLightView {
    fn default() -> Self {
        let view_matrix = //cgmath::perspective(cgmath::Deg(45.0), 1.333334, 10.0, 20.0)//cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 1.0, 100.0)
            cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.0, 25.0)
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
    pub shadow_result_bind_group: wgpu::BindGroup,
    pub shadow_result_bind_group_layout: wgpu::BindGroupLayout,
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
                width: window_width,
                height: window_height,
                depth_or_array_layers: 1
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let shadow_texture_view = shadow_texture.create_view(& wgpu::TextureViewDescriptor::default());

        let shadow_light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shadow Light Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            contents: bytemuck::cast_slice(&[ GpuLightView::default() ])
        });

        let shadow_light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                },
            ]
        });

        let shadow_light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(shadow_light_buffer.as_entire_buffer_binding())
                },
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
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });

        let shadow_result_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None
                },
            ]
        });

        let shadow_result_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow_texture_view)
                }
            ],
            layout: &shadow_result_bind_group_layout
        });


        // create vertex shader:

        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;
            layout(location=1) in vec4 a_model_matrix_1;
            layout(location=2) in vec4 a_model_matrix_2;
            layout(location=3) in vec4 a_model_matrix_3;
            layout(location=4) in vec4 a_model_matrix_4;

            layout(set=0, binding=0)
            uniform SceneUniforms {
                mat4 light_view_matrix;
            };

            layout(set=1, binding=0)
            uniform ModelUniforms {
                mat4 u_transform;
            };

            void main() {
                mat4 a_model_matrix = mat4(a_model_matrix_1, a_model_matrix_2, a_model_matrix_3, a_model_matrix_4);
                gl_Position = light_view_matrix * a_model_matrix * vec4(a_position, 1.0);
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

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Pass Vertex Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(vs_spirv.as_binary())),
        });

        // Create the render pipeline

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &shadow_light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x3
                            },
                        ],
                        step_mode: wgpu::VertexStepMode::Vertex,
                        array_stride: (std::mem::size_of::<GpuVector3>()) as wgpu::BufferAddress
                    },
                    wgpu::VertexBufferLayout {
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x4
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x4
                            },
                        ],
                        step_mode: wgpu::VertexStepMode::Instance,
                        array_stride: 64,
                    },
                ]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: device.features().contains(wgpu::Features::DEPTH_CLIP_CONTROL),
                polygon_mode: Default::default(),
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            fragment: None,
            multiview: None,
            cache: None,
        });

        ShadowPasses {
            shadow_texture,
            shadow_texture_view,
            shadow_sampler,
            shadow_light_buffer,
            shadow_light_bind_group_layout,
            shadow_light_bind_group,
            shadow_result_bind_group_layout,
            shadow_result_bind_group,
            pipeline
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh_resources: &MeshResources,
        shadow_mesh_commands: &mut CommandQueue<RenderShadowMeshCommand, RenderShadowBatch>
    ) {

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shadow Pass"),
                color_attachments: &[ ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.push_debug_group("Begin Shadow Pass");

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.shadow_light_bind_group, &[]);

            while let Some(batch) = shadow_mesh_commands.pop_next_batch() {

                let mesh_type = mesh_resources.mesh_types.get(batch.mesh_type as usize).unwrap();

                render_pass.set_vertex_buffer(0, mesh_type.gpu_geometry.positions_buffer.slice(..));
                render_pass.set_vertex_buffer(1, mesh_type.model_matrix_buffer.slice(..));

                let instances = batch.object_indices.len() as u32;

                mesh_type.prepare_instances(&queue, &batch.object_indices);

                render_pass.set_index_buffer(mesh_type.gpu_geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh_type.gpu_geometry.index_count, 0, 0..instances);
            }

            render_pass.pop_debug_group();
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

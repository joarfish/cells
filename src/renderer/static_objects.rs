use specs::prelude::*;
use specs::Component;
use wgpu::{util::*};
use std::vec::Vec;

use crate::scene::scene_graph::Transformation;

use super::{DeltaTimer, mesh::create_cube_mesh, renderer::{RenderCommand, RendererCommandsQueue}, resources::{BindGroupBufferHandle, PipelineHandle, RendererResources}, resources::{BufferSlice, RenderObject, RenderObjectHandle}, scene_base::SceneBaseResources, utils::GpuMatrix4, utils::GpuMatrix4BGA, utils::{GpuVector3, GpuVector3BGA}};

struct StaticObjectData {
    model_matrix_index: u64,
    color_index: u64,
    render_object_handle: RenderObjectHandle
}

pub struct StaticObjectsResources {
    model_matrices_buffer: BindGroupBufferHandle,
    free_model_matrix_index: u64,
    uniforms_bind_group_layout: wgpu::BindGroupLayout,
    colors_buffer: BindGroupBufferHandle,
    free_color_index: u64,
    pipeline: PipelineHandle,
    free_render_objects: Vec<StaticObjectData>,
}

impl StaticObjectsResources {
    pub fn new(device: &wgpu::Device, base_scene_object: &SceneBaseResources, resources: &mut RendererResources) -> Self {
        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;
            layout(location=1) in vec3 a_normal;

            layout(set=0, binding=0)
            uniform SceneUniforms {
                mat4 u_view_proj;
            };

            layout(set=1, binding=0)
            uniform ModelUniforms {
                mat4 u_transform;
            };

            layout(location=0) out vec3 world_position;
            layout(location=1) out vec3 normal;

            void main() {        
                vec4 position = u_transform * vec4(a_position, 1.0);
                world_position = position.xyz;
                normal = (u_transform * vec4(a_position + a_normal, 1.0) - position).xyz;
                gl_Position = u_view_proj * position;
            }        
        ".to_string();

        let fs_code = "
            #version 450

            layout(location=0) in vec3 world_position;
            layout(location=1) in vec3 normal;

            layout(location=0) out vec4 f_albedo;
            layout(location=1) out vec3 f_position;
            layout(location=2) out vec3 f_normal;

            layout(set=1, binding=1)
            uniform ModelUniforms {
                vec3 u_color;
            };

            void main() {
                f_position = world_position;
                f_normal = normal;
                f_albedo = vec4(0.25, 0.25, 0.25, 1.0);
            }
        ".to_string();

        let vs_spirv = compiler.compile_into_spirv(
                &vs_code,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();

        let fs_spirv = compiler
            .compile_into_spirv(
                &fs_code,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(vs_spirv.as_binary()),
        ));
        let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(fs_spirv.as_binary()),
        ));

        let uniforms_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("StaticObject Uniforms"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
        });

        // We create a buffer for 50 model matrices:
        let matrices_buffer_data = vec![GpuMatrix4BGA::empty();50];

        let model_matrices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&matrices_buffer_data),
            label: Some("ModelMatricesBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        // The same for colors:
        let colors_buffer_data = vec![GpuVector3BGA::empty();50];

        let colors_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&colors_buffer_data),
            label: Some("ColorsBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let view_matrix_bind_group_layout = resources.bind_group_layouts.get(
            &base_scene_object.view_matrix_bind_group_layout
        ).unwrap();

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&view_matrix_bind_group_layout, &uniforms_bind_group_layout],
                push_constant_ranges: &[],
                label: None
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vertex_shader_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fragment_shader_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                },
                wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Rgba32Float,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::RED |  wgpu::ColorWrite::GREEN |  wgpu::ColorWrite::BLUE,
                },
                wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Rgba32Float,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::RED |  wgpu::ColorWrite::GREEN |  wgpu::ColorWrite::BLUE,
                }
            ],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: Some(
                wgpu::DepthStencilStateDescriptor {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilStateDescriptor::default(),
                }
            ),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[ wgpu::VertexBufferDescriptor {
                    attributes: &[ 
                        wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float3
                        },
                        wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float3
                        }
                    ],
                    step_mode: wgpu::InputStepMode::Vertex,
                    stride: (std::mem::size_of::<GpuVector3>()) as wgpu::BufferAddress
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        StaticObjectsResources {
            free_render_objects: Vec::new(),
            pipeline: resources.render_pipelines.add_entry(pipeline),
            colors_buffer: resources.bind_group_buffers.add_entry(colors_buffer),
            uniforms_bind_group_layout,
            free_color_index: 0,
            model_matrices_buffer: resources.bind_group_buffers.add_entry(model_matrices_buffer),
            free_model_matrix_index: 0,
        }
    }

    fn create_render_object(&mut self, device: &wgpu::Device, resources: &mut RendererResources, scene_base: &SceneBaseResources) -> RenderObject {

        let model_matrices_buffer_slice = {
            let model_matrices_buffer = resources.bind_group_buffers.get(&self.model_matrices_buffer).unwrap();

            let size = std::mem::size_of::<GpuMatrix4BGA>() as u64;
            let offset = self.free_model_matrix_index * size;
            self.free_model_matrix_index += 1;

            model_matrices_buffer.slice(offset..(offset+size))
        };

        let color_buffer_slice = {

            let colors_buffer = resources.bind_group_buffers.get(&self.colors_buffer).unwrap();

            let size = std::mem::size_of::<GpuVector3BGA>() as u64;
            let offset = self.free_color_index * size;
            self.free_color_index += 1;

            colors_buffer.slice(offset..(offset+size))
        };

        let uniforms_bind_group = resources.bind_groups.add_entry(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniforms"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(model_matrices_buffer_slice),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(color_buffer_slice),
                }
            ],
            layout: &self.uniforms_bind_group_layout,
        }));

        let mesh = create_cube_mesh();

        let positions = resources.vertex_buffers.add_entry(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsage::VERTEX
        }));

        let normals = resources.vertex_buffers.add_entry(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mesh.normals),
            usage: wgpu::BufferUsage::VERTEX
        }));

        let index_count = mesh.indices.len() as u64;
        let indices = resources.index_buffers.add_entry(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsage::INDEX,
        }));

        RenderObject {
            bind_groups: vec![scene_base.view_matrix_bind_group.clone(), uniforms_bind_group],
            pipeline: self.pipeline.clone(),
            vertex_buffers: vec![
                BufferSlice { buffer: positions, range: 0..(mesh.vertices.len() as u64) },
                BufferSlice { buffer: normals, range: 0..(mesh.normals.len() as u64) },
            ],
            indices: BufferSlice { buffer: indices, range: 0..(index_count * 2) },
            index_count: index_count as u32
        }
    }

    pub fn create_static_object(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, resources: &mut RendererResources, scene_base: &SceneBaseResources, transformation: Transformation) -> StaticObject {
        
        let matrix = 
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(transformation.position.x, transformation.position.y, transformation.position.z)) *
                    cgmath::Matrix4::from_nonuniform_scale(transformation.scale.x, transformation.scale.y, transformation.scale.z) *
                    cgmath::Matrix4::from_angle_x(transformation.rotation.x) *
                    cgmath::Matrix4::from_angle_y(transformation.rotation.y) *
                    cgmath::Matrix4::from_angle_z(transformation.rotation.z);

        

        let static_object = if let Some(do_data) = self.free_render_objects.pop() {
            let renderer_object = do_data.render_object_handle;
            StaticObject {
                model_matrix_index: do_data.model_matrix_index,
                color_index: do_data.color_index,
                renderer_object
            }
        } else {
            let renderer_object = self.create_render_object(device, resources, scene_base);
            let handle = resources.render_objects.add_entry(renderer_object);
            
            StaticObject {
                color_index: self.free_color_index - 1,
                model_matrix_index: self.free_model_matrix_index - 1,
                renderer_object: handle
            }
        };

        // Update model matrix for this new render object:

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        let model_matrices_buffer = resources.bind_group_buffers.get(&self.model_matrices_buffer).expect("");

        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[GpuMatrix4BGA::new(matrix)]),
            usage: wgpu::BufferUsage::COPY_SRC
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer, 0, 
            &model_matrices_buffer, static_object.model_matrix_index * (std::mem::size_of::<GpuMatrix4BGA>() as u64),
            (std::mem::size_of::<GpuMatrix4BGA>() ) as wgpu::BufferAddress
        );

        queue.submit(std::iter::once(encoder.finish()));

        static_object
    }
}


#[derive(Component)]
pub struct StaticObject {
    color_index: u64,
    model_matrix_index: u64,
    pub renderer_object: RenderObjectHandle
}

pub struct StaticObjectsSystem;


impl<'a> System<'a> for StaticObjectsSystem {
    type SystemData = (
        WriteExpect<'a, RendererCommandsQueue>,
        ReadStorage<'a, StaticObject>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (
            mut commands_queue,
            static_objects
        ) = data;

        for static_object in (&static_objects).join() {
            commands_queue.push_render_command(&RenderCommand {
                object: static_object.renderer_object.clone(),
                layer: 1,
                distance: 1
            });
        }
    }
}
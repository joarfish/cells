use specs::prelude::*;
use specs::Component;
use wgpu::{util::*};

use crate::scene::scene_graph::Transformation;

use super::{DeltaTimer, resources::{BindGroupBufferHandle, PipelineHandle, RendererResources}, resources::{BufferSlice, RenderObject, RenderObjectHandle}, scene_base::SceneBaseResources, utils::GpuMatrix4, utils::GpuMatrix4BGA, utils::{GpuVector3, GpuVector3BGA}};

struct DynamicObjectData {
    model_matrix_index: u64,
    color_index: u64,
    render_object_handle: RenderObjectHandle
}

pub struct DynamicObjectsResources {
    model_matrices_buffer: BindGroupBufferHandle,
    free_model_matrix_index: u64,
    uniforms_bind_group_layout: wgpu::BindGroupLayout,
    colors_buffer: BindGroupBufferHandle,
    free_color_index: u64,
    pipeline: PipelineHandle,
    free_render_objects: Vec<DynamicObjectData>,
}

impl DynamicObjectsResources {
    pub fn new(device: &wgpu::Device, base_scene_object: &SceneBaseResources, resources: &mut RendererResources) -> Self {
        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;

            layout(set=0, binding=0)
            uniform SceneUniforms {
                mat4 u_view_proj;
            };

            layout(set=1, binding=0)
            uniform ModelUniforms {
                mat4 u_transform;
            };

            void main() {               
                gl_Position = u_view_proj * (u_transform * vec4(a_position, 1.0));
            }        
        ".to_string();

        let fs_code = "
            #version 450

            layout(location=0) out vec4 f_color;

            layout(set=1, binding=1)
            uniform ModelUniforms {
                vec3 u_color;
            };

            void main() {
                f_color = vec4(u_color, 1.0);
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
                label: Some("DynamicObject Uniforms"),
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
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8Unorm,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[ wgpu::VertexBufferDescriptor {
                    attributes: &[ 
                        wgpu::VertexAttributeDescriptor {
                            offset: 0,
                            shader_location: 0,
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

        DynamicObjectsResources {
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

        let positions = resources.vertex_buffers.add_entry(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                GpuVector3 { vector: cgmath::vec3(0.25, 0.25, 0.0) },
                GpuVector3 { vector: cgmath::vec3(-0.5, -0.25, 0.0) },
                GpuVector3 { vector: cgmath::vec3(0.5, -0.25, 0.0) }
            ]),
            usage: wgpu::BufferUsage::VERTEX
        }));

        let index_data = &[0 as u16, 1, 2];
        let index_count = index_data.len() as u32;
        let indices = resources.index_buffers.add_entry(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(index_data),
            usage: wgpu::BufferUsage::INDEX,
        }));

        RenderObject {
            bind_groups: vec![scene_base.view_matrix_bind_group.clone(), uniforms_bind_group],
            pipeline: self.pipeline.clone(),
            vertex_buffers: vec![ BufferSlice { buffer: positions, range: 0..3 } ],
            indices: BufferSlice { buffer: indices, range: 0..6 },
            index_count
        }
    }

    pub fn create_dynamic_object(&mut self, device: &wgpu::Device, resources: &mut RendererResources, scene_base: &SceneBaseResources) -> DynamicObject {
        // How should be mapping be here? What we need for the DO-System is actually just buffer indices for model-matrix and color
        
        if let Some(do_data) = self.free_render_objects.pop() {
            // Reuse RenderObject:
            // - add model-matrix and color indices to DynamicObject
            // - check if the render object is still registered with the renderer?
            // More generally: How do we make sure all referenced resources are still available? This is a big @toto
            do_data.render_object_handle; // <-- use this to check if it is still alive
            DynamicObject {
                model_matrix_index: do_data.model_matrix_index,
                color_index: do_data.color_index
            }
        } else {
            let render_object = self.create_render_object(device, resources, scene_base);
            resources.render_objects.add_entry(render_object);
            
            DynamicObject {
                color_index: self.free_color_index - 1,
                model_matrix_index: self.free_model_matrix_index - 1
            }
        }
    }
}

pub struct TransformationTests;

impl<'a> System<'a> for TransformationTests {
    type SystemData = (
        WriteStorage<'a, Transformation>,
        ReadExpect<'a, DeltaTimer>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut transformations, delta_timer) = data;
        let d = &delta_timer.get_duration_f32();

        for transformation in (&mut transformations).join() {
            (*transformation).rotation.z += cgmath::Deg(d*25.0);    
        }
    }
}


#[derive(Component)]
pub struct DynamicObject {
    color_index: u64,
    model_matrix_index: u64
}

#[derive(Component)]
pub struct Color {
    pub r: f32,
    pub g: f32, 
    pub b: f32
}

pub struct DynamicObjectsSystem;

impl DynamicObjectsSystem {

    pub fn update_data(&self, device: &wgpu::Device, queue: &wgpu::Queue, resources: &RendererResources, do_resources: &DynamicObjectsResources, model_matrices: &[GpuMatrix4BGA], colors: &[GpuVector3BGA]) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let model_matrices_buffer = resources.bind_group_buffers.get(&do_resources.model_matrices_buffer).expect("");

            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(model_matrices),
                usage: wgpu::BufferUsage::COPY_SRC
            });

            encoder.copy_buffer_to_buffer(
                &staging_buffer, 0, 
                &model_matrices_buffer, 0,
                (std::mem::size_of::<GpuMatrix4BGA>() * 50) as wgpu::BufferAddress
            );
        }

        {
            let colors_buffer = resources.bind_group_buffers.get(&do_resources.colors_buffer).expect("");

            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(colors),
                usage: wgpu::BufferUsage::COPY_SRC
            });

            encoder.copy_buffer_to_buffer(
                &staging_buffer, 0, 
                &colors_buffer, 0,
                (std::mem::size_of::<GpuVector3BGA>() * 50) as wgpu::BufferAddress
            );
        }
        
        queue.submit(std::iter::once(encoder.finish()));
    }
}

impl<'a> System<'a> for DynamicObjectsSystem {
    type SystemData = (
        ReadExpect<'a, DynamicObjectsResources>,
        ReadExpect<'a, RendererResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
        ReadStorage<'a, DynamicObject>,
        ReadStorage<'a, Transformation>,
        ReadStorage<'a, Color>,
    );

    fn run(&mut self, data: Self::SystemData) {

        let (do_resources, renderer_resources, device, queue, dynamic_objects, transformations, color) = data;
        let mut matrices : Vec<GpuMatrix4BGA> = vec![GpuMatrix4BGA::empty(); 50];
        let mut colors : Vec<GpuVector3BGA> = vec![GpuVector3BGA::empty(); 50];

        for (object, transformation, color) in (&dynamic_objects, &transformations, &color).join() {
            // Use object data to set indices properly
            let position = transformation.position;
            let scale = transformation.scale;
            let rotation = transformation.rotation;

            let matrix = 
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(position.x, position.y, position.z)) *
                    cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) *
                    cgmath::Matrix4::from_angle_x(rotation.x) *
                    cgmath::Matrix4::from_angle_y(rotation.y) *
                    cgmath::Matrix4::from_angle_z(rotation.z);


            matrices[object.model_matrix_index as usize] = GpuMatrix4BGA::new(matrix);
            colors[object.color_index as usize] = GpuVector3BGA::new(color.r, color.g, color.b);
        }

        self.update_data(&device, &queue, &renderer_resources, &do_resources, &matrices, &colors);
    }
}
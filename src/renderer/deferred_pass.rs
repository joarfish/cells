use super::{
    command_queue::{CommandQueue, RenderMeshCommand},
    meshes::MeshResources,
    scene_base::SceneBaseResources,
    utils::GpuMatrix4BGA,
    utils::GpuVector3,
    utils::GpuVector3BGA,
};
use crate::renderer::command_queue::RenderBatch;
use crate::renderer::material::MaterialResources;
use crate::renderer::utils::GpuMatrix4;

pub struct DeferredPass {
    pub pipeline: wgpu::RenderPipeline,
    pub msaa_diffuse_view: wgpu::TextureView,
    pub diffuse_texture_view: wgpu::TextureView,
    pub position_texture_view: wgpu::TextureView,
    pub normal_texture_view: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,
    pub gbuffer_bind_group_layout: wgpu::BindGroupLayout,
    pub gbuffer_bind_group: wgpu::BindGroup,
}

impl DeferredPass {
    pub fn new(
        device: &wgpu::Device,
        material_resources: &MaterialResources,
        scene_base_resources: &SceneBaseResources,
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        // Setup textures for color attachments:

        let base_texture_descriptor = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: screen_width,
                height: screen_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
            view_formats: &[],
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            sample_count: 1,
            ..base_texture_descriptor
        });

        let msaa_diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            sample_count: 4,
            ..base_texture_descriptor
        });

        let msaa_diffuse_view =
            msaa_diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let position_texture = device.create_texture(&wgpu::TextureDescriptor {
            sample_count: 1,
            format: wgpu::TextureFormat::Rgba16Float,
            ..base_texture_descriptor
        });

        let normal_texture = device.create_texture(&wgpu::TextureDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            ..base_texture_descriptor
        });

        // Setup texture for depth

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            format: wgpu::TextureFormat::Depth32Float,
            label: None,
            ..base_texture_descriptor
        });

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let position_texture_view =
            position_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_texture_view =
            normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // GBUffer Bindgroup (can be used by other passes):

        let gbuffer_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("gBuffer Uniforms"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });

        let gbuffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gBufferBindGroup"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(
                        &wgpu::SamplerDescriptor {
                            label: None,
                            address_mode_u: wgpu::AddressMode::ClampToEdge,
                            address_mode_v: wgpu::AddressMode::ClampToEdge,
                            address_mode_w: wgpu::AddressMode::ClampToEdge,
                            mag_filter: wgpu::FilterMode::Nearest,
                            min_filter: wgpu::FilterMode::Nearest,
                            mipmap_filter: wgpu::FilterMode::Nearest,
                            lod_min_clamp: 0.0,
                            lod_max_clamp: 0.0,
                            compare: None,
                            anisotropy_clamp: 1,
                            border_color: None,
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&position_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_texture_view),
                },
            ],
            layout: &gbuffer_bind_group_layout,
        });

        // Setup shaders:

        let compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;
            layout(location=1) in vec3 a_normal;
            layout(location=2) in uint a_part_id;
            layout(location=3) in vec4 a_model_matrix_1;
            layout(location=4) in vec4 a_model_matrix_2;
            layout(location=5) in vec4 a_model_matrix_3;
            layout(location=6) in vec4 a_model_matrix_4;

            layout(set=0, binding=0)
            uniform SceneUniforms {
                mat4 u_view;
                mat4 u_projection;
            };

            layout(location=0) out vec3 world_position;
            layout(location=1) out vec3 normal;
            layout(location=2) out flat uint part_id;

            mat3 inverseNoExt(mat3 m) {
              float a00 = m[0][0], a01 = m[0][1], a02 = m[0][2];
              float a10 = m[1][0], a11 = m[1][1], a12 = m[1][2];
              float a20 = m[2][0], a21 = m[2][1], a22 = m[2][2];

              float b01 = a22 * a11 - a12 * a21;
              float b11 = -a22 * a10 + a12 * a20;
              float b21 = a21 * a10 - a11 * a20;

              float det = a00 * b01 + a01 * b11 + a02 * b21;

              return mat3(b01, (-a22 * a01 + a02 * a21), (a12 * a01 - a02 * a11),
                          b11, (a22 * a00 - a02 * a20), (-a12 * a00 + a02 * a10),
                          b21, (-a21 * a00 + a01 * a20), (a11 * a00 - a01 * a10)) / det;
            }

            void main() {
                mat4 a_model_matrix = mat4(a_model_matrix_1, a_model_matrix_2, a_model_matrix_3, a_model_matrix_4);
                vec4 position = u_view * a_model_matrix * vec4(a_position, 1.0);
                mat3 normal_matrix = transpose(inverseNoExt(mat3(u_view * a_model_matrix)));
                normal = normal_matrix * a_normal;
                world_position = position.xyz;
                part_id = a_part_id;
                gl_Position = u_projection * position;
            }
        ".to_string();

        let fs_code = "
            #version 450

            layout(location=0) in vec3 world_position;
            layout(location=1) in vec3 normal;
            layout(location=2) in flat uint part_id;

            layout(location=0) out vec4 f_albedo;
            layout(location=1) out vec4 f_position;
            layout(location=2) out vec4 f_normal;

            struct Material {
                vec4 primary;
                vec4 secondary;
                vec4 tertiary;
                vec4 quaternary;
            };

            layout(set=1, binding=0)
            uniform MaterialUniforms {
                Material material;
            };

            void main() {
                f_position = vec4(world_position, 1.0);
                f_normal = vec4(normalize(normal) * 0.5 + 0.5, 1.0);
                f_albedo = material.primary;
            }
        "
        .to_string();

        let vs_spirv = compiler
            .compile_into_spirv(
                &vs_code,
                shaderc::ShaderKind::Vertex,
                "deferred.vert",
                "main",
                None,
            )
            .unwrap();

        let fs_spirv = compiler
            .compile_into_spirv(
                &fs_code,
                shaderc::ShaderKind::Fragment,
                "deferred.frag",
                "main",
                None,
            )
            .unwrap();

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Vertex Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(vs_spirv.as_binary())),
        });
        let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Fragment Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(fs_spirv.as_binary())),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &scene_base_resources.bind_group_layout,
                &material_resources.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: true,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                        step_mode: wgpu::VertexStepMode::Vertex,
                        array_stride: (std::mem::size_of::<GpuVector3>()) as wgpu::BufferAddress,
                    },
                    wgpu::VertexBufferLayout {
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        }],
                        step_mode: wgpu::VertexStepMode::Vertex,
                        array_stride: (std::mem::size_of::<GpuVector3>()) as wgpu::BufferAddress,
                    },
                    wgpu::VertexBufferLayout {
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Uint32,
                        }],
                        step_mode: wgpu::VertexStepMode::Vertex,
                        array_stride: (std::mem::size_of::<u32>()) as wgpu::BufferAddress,
                    },
                    wgpu::VertexBufferLayout {
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 32,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            wgpu::VertexAttribute {
                                offset: 48,
                                shader_location: 6,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                        step_mode: wgpu::VertexStepMode::Instance,
                        array_stride: (64) as wgpu::BufferAddress,
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::RED
                            | wgpu::ColorWrites::GREEN
                            | wgpu::ColorWrites::BLUE,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::RED
                            | wgpu::ColorWrites::GREEN
                            | wgpu::ColorWrites::BLUE,
                    }),
                ],
            }),
            multiview: None,
            cache: None,
        });

        DeferredPass {
            msaa_diffuse_view,
            diffuse_texture_view,
            position_texture_view,
            normal_texture_view,
            depth_texture_view,
            pipeline,
            gbuffer_bind_group_layout,
            gbuffer_bind_group,
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene_base_resources: &SceneBaseResources,
        mesh_resources: &MeshResources,
        material_resources: &MaterialResources,
        mesh_commands: &mut CommandQueue<RenderMeshCommand, RenderBatch>,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Deferred Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.diffuse_texture_view,
                        resolve_target: None, //Some(&self.diffuse_texture_view),
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.position_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.normal_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.push_debug_group("Begin Deferred Pass");

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &scene_base_resources.bind_group, &[]);

            // todo: Check if this works properly
            let ubo_align = device.limits().min_uniform_buffer_offset_alignment as u64;

            while let Some(batch) = mesh_commands.pop_next_batch() {
                let mesh_type = mesh_resources
                    .mesh_types
                    .get(batch.mesh_type as usize)
                    .unwrap();

                render_pass.set_bind_group(
                    1,
                    &material_resources.bind_group,
                    &[align_up(batch.material as u64, ubo_align) as u32],
                );

                mesh_type.prepare_instances(&queue, &batch.object_indices);

                render_pass.set_vertex_buffer(0, mesh_type.gpu_geometry.positions_buffer.slice(..));
                render_pass.set_vertex_buffer(1, mesh_type.gpu_geometry.normals_buffer.slice(..));
                render_pass.set_vertex_buffer(2, mesh_type.gpu_geometry.parts_buffer.slice(..));
                render_pass.set_vertex_buffer(3, mesh_type.model_matrix_buffer.slice(..));

                let instances = batch.object_indices.len() as u32;

                render_pass.set_index_buffer(
                    mesh_type.gpu_geometry.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(0..mesh_type.gpu_geometry.index_count, 0, 0..instances);
            }

            render_pass.pop_debug_group();
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

fn align_up(value: u64, align: u64) -> u64 {
    (value + align - 1) & !(align - 1)
}

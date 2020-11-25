use super::{utils::GpuVector3BGA, command_queue::{CommandQueue, RenderMeshCommand}, meshes::{MeshResources}, scene_base::SceneBaseResources, utils::GpuMatrix4BGA, utils::GpuVector3};

pub struct DeferredPass {
    pub pipeline: wgpu::RenderPipeline,
    pub msaa_diffuse_view: wgpu::TextureView,
    pub diffuse_texture_view: wgpu::TextureView,
    pub position_texture_view: wgpu::TextureView,
    pub normal_texture_view: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,
    pub gbuffer_bind_group_layout: wgpu::BindGroupLayout,
    pub gbuffer_bind_group: wgpu::BindGroup
}

impl DeferredPass {
    pub fn new(
        device: &wgpu::Device,
        mesh_resources: &MeshResources,
        scene_base_resources: &SceneBaseResources,
        screen_width: u32,
        screen_height: u32
    ) -> Self {

        // Setup textures for color attachments:

        let base_texture_descriptor = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: screen_width,
                height: screen_height,
                depth: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
            label: None,
        };

        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            sample_count: 1,
            ..base_texture_descriptor
        });

        let msaa_diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            sample_count: 8,
            ..base_texture_descriptor
        });

        let msaa_diffuse_view = msaa_diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        let position_texture = device.create_texture(&wgpu::TextureDescriptor {
            sample_count: 1,
            format: wgpu::TextureFormat::Rgba32Float,
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

        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let position_texture_view = position_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let normal_texture_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // GBUffer Bindgroup (can be used by other passes):

        let gbuffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gBuffer Uniforms"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false
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
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor {
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
                        anisotropy_clamp: None,
                    }))
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&position_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_texture_view)
                },
            ],
            layout: &gbuffer_bind_group_layout
        });

        // Setup shaders:

        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;
            layout(location=1) in vec3 a_normal;

            layout(set=0, binding=0)
            uniform SceneUniforms {
                mat4 u_view;
                mat4 u_projection;
            };

            layout(set=1, binding=0)
            uniform ModelUniforms {
                mat4 u_transform;
            };

            layout(location=0) out vec3 world_position;
            layout(location=1) out vec3 normal;

            void main() {
                vec4 position = u_view * u_transform * vec4(a_position, 1.0);
                mat3 normal_matrix = transpose(inverse(mat3(u_view * u_transform)));
                normal = normal_matrix * a_normal;
                world_position = position.xyz;
                gl_Position = u_projection * position;
            }        
        ".to_string();

        let fs_code = "
            #version 450

            layout(location=0) in vec3 world_position;
            layout(location=1) in vec3 normal;

            layout(location=0) out vec4 f_albedo;
            layout(location=1) out vec3 f_position;
            layout(location=2) out vec4 f_normal;

            layout(set=1, binding=1)
            uniform ModelUniforms {
                vec3 u_color;
            };

            void main() {
                f_position = world_position;
                f_normal = vec4(normalize(normal) * 0.5 + 0.5, 1.0);
                f_albedo = vec4(u_color, 1.0);
            }
        ".to_string();

        let vs_spirv = compiler.compile_into_spirv(
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

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(vs_spirv.as_binary()),
        ));
        let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleSource::SpirV(
            std::borrow::Cow::Borrowed(fs_spirv.as_binary()),
        ));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[ &scene_base_resources.bind_group_layout, &mesh_resources.buffer_bind_group_layout ],
            push_constant_ranges: &[ ]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
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
                    vertex_buffers: &[
                        wgpu::VertexBufferDescriptor {
                        attributes: &[
                            wgpu::VertexAttributeDescriptor {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float3
                            },
                        ],
                        step_mode: wgpu::InputStepMode::Vertex,
                        stride: (std::mem::size_of::<GpuVector3>()) as wgpu::BufferAddress
                    }
                ],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        DeferredPass {
            msaa_diffuse_view,
            diffuse_texture_view,
            position_texture_view,
            normal_texture_view,
            depth_texture_view,
            pipeline,
            gbuffer_bind_group_layout,
            gbuffer_bind_group
        }
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, scene_base_resources: &SceneBaseResources, mesh_resources: &MeshResources, mesh_commands: &mut CommandQueue<RenderMeshCommand>) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.diffuse_texture_view,
                            resolve_target: None,//Some(&self.diffuse_texture_view),
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: true,
                            },
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.position_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: true,
                            },
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &self.normal_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &self.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: false,
                        }),
                        stencil_ops: None
                    }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &scene_base_resources.bind_group, &[]);
    
            while let Some(command) = mesh_commands.pop_command() {

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
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
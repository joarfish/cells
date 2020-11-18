use wgpu::util::*;

use super::{deferred_pass::DeferredPass, utils::GpuVector3, lights::LightsResources};

pub struct CompositionPass {
    pub pipeline: wgpu::RenderPipeline,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub g_buffer_bind_group: wgpu::BindGroup,
}

impl CompositionPass {
    pub fn new(device: &wgpu::Device, deferred_pass: &DeferredPass, light_resources: &LightsResources) -> CompositionPass {

        let composition_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                }
            ],
        });
    
        let g_buffer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gBufferBindGroup"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor {
                        label: None,
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        mipmap_filter: wgpu::FilterMode::Linear,
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 0.0,
                        compare: None,
                        anisotropy_clamp: None,
                    }))
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&deferred_pass.diffuse_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&deferred_pass.position_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&deferred_pass.normal_texture_view)
                }
            ],
            layout: &composition_bind_group_layout
        });
    
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                GpuVector3::new(-1.0, -1.0, 0.0),
                GpuVector3::new(1.0, -1.0, 0.0),
                GpuVector3::new(1.0, 1.0, 0.0),
                GpuVector3::new(-1.0, 1.0, 0.0)
            ]),
            usage: wgpu::BufferUsage::VERTEX
        });
    
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0 as u16, 2, 3, 0, 1, 2]),
            usage: wgpu::BufferUsage::INDEX
        });
    
        let mut compiler = shaderc::Compiler::new().unwrap();
    
        let vs_code = "
            #version 450
    
            layout(location=0) in vec3 a_position;
    
            layout(location=0) out vec2 tex_coord;
    
            void main() {
                tex_coord = (a_position.xy + vec2(1.0,1.0)) * vec2(0.5, 0.5);
                gl_Position = vec4(a_position, 1.0);
            }        
        ".to_string();
    
        let fs_code = "
            #version 450
    
            layout(location=0) in vec2 tex_coord;
    
            layout(location=0) out vec4 f_color;
    
            layout(set=0, binding=0) uniform sampler layer_sampler;
            layout(set=0, binding=1) uniform texture2D gAlbedo;
            layout(set=0, binding=2) uniform texture2D gPosition;
            layout(set=0, binding=3) uniform texture2D gNormal;
    
            struct GpuLight {
                mat4 view_matrix;
                vec4 position;
                vec4 color;
                float intensity;
                float radius;
                int enabled;
            };
    
            layout(set = 1, binding = 0) uniform Lights {
                GpuLight u_point_lights[20];
            };
    
            void main() {
                vec4 f_albedo = texture(sampler2D(gAlbedo, layer_sampler), tex_coord);
                vec3 f_position = texture(sampler2D(gPosition, layer_sampler), tex_coord).xyz;
                vec3 f_normal = texture(sampler2D(gNormal, layer_sampler), tex_coord).xyz;
    
                vec4 color = f_albedo * vec4(0.25, 0.25, 0.25, 1.0);
    
                for(int i=0; i < 20; ++i) {
                    GpuLight light = u_point_lights[i];
                    if (light.enabled>0) {
                        vec3 light_dir = light.position.xyz - f_position;
                        color += vec4(max(0.0, dot(f_normal, light_dir)) * light.color.xyz * f_albedo.xyz * light.intensity, 1.0);
                    }
                }
    
                f_color = color;
            }
        ".to_string();
    
        let vs_spirv = compiler.compile_into_spirv(
                &vs_code,
                shaderc::ShaderKind::Vertex,
                "composition.vert",
                "main",
                None,
            )
            .unwrap();
    
        let fs_spirv = compiler
            .compile_into_spirv(
                &fs_code,
                shaderc::ShaderKind::Fragment,
                "composition.frag",
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
    
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&composition_bind_group_layout, &light_resources.lights_bind_group_layout ],
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
                    stride: wgpu::VertexFormat::Float3.size() as wgpu::BufferAddress
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
    
        CompositionPass {
            vertices,
            indices,
            pipeline,
            g_buffer_bind_group,
        }
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, swap_chain: &mut wgpu::SwapChain, light_resources: &LightsResources) {

        let screen_frame = swap_chain.get_current_frame().expect("Could not aquire frame for rendering.").output;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &screen_frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.5,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: None
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.g_buffer_bind_group, &[]);
            render_pass.set_bind_group(1, &light_resources.lights_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..));
            render_pass.draw_indexed(0..8, 0, 0..1)
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
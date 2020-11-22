use wgpu::util::*;

use super::{deferred_pass::DeferredPass, utils::GpuVector3, lights::LightsResources};
use crate::renderer::shadow_passes::ShadowPasses;
use crate::renderer::scene_base::SceneBaseResources;
use cgmath::InnerSpace;

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug)]
struct HemisphereSamples {
    points: [[f32; 3]; 64]
}

unsafe impl bytemuck::Pod for HemisphereSamples {}
unsafe impl bytemuck::Zeroable for HemisphereSamples {}

pub struct CompositionPass {
    pub pipeline: wgpu::RenderPipeline,
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub g_buffer_bind_group: wgpu::BindGroup,
    pub ssao_bind_group: wgpu::BindGroup
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

impl CompositionPass {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        deferred_pass: &DeferredPass,
        shadow_passes: &ShadowPasses,
        light_resources: &LightsResources,
        scene_base_resources: &SceneBaseResources
    ) -> CompositionPass {

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
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: true
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
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

        // Generate Hemisphere Sample Point:
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut samples = [[1.0 as f32, 1.0, 1.0]; 64];
        for i in 0..64 {
            let x = rng.gen_range(-1.0 as f32, 1.0 as f32);
            let y = rng.gen_range(-1.0 as f32, 1.0 as f32);
            let z = rng.gen_range(0.0 as f32, 1.0 as f32);

            let scale : f32 = i as f32 / 64.0;
            let lerp = lerp(0.1, 1.0, scale * scale);

            samples[i] = [x * lerp,y * lerp,z * lerp];
        }

        let hemisphere = HemisphereSamples { points: samples };

        let random_vector_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST
        });

        {
            let mut data = [[0.0; 4]; 16];

            for i in 0..16 {
                data[i] = [
                    rng.gen_range(-1.0, 1.0 as f32),
                    rng.gen_range(-1.0, 1.0 as f32),
                    0.0, 1.0 ]
            }

            queue.write_texture(
                wgpu::TextureCopyView {
                    texture: &random_vector_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO
                },
                bytemuck::cast_slice(&data),
                wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * 4 * 4,
                    rows_per_image: 4
                },
                wgpu::Extent3d {
                    width: 4,
                    height: 4,
                    depth: 1
                }
            );
        }

        let ssao_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[hemisphere]),
            usage: wgpu::BufferUsage::UNIFORM
        });

        let ssao_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false, min_binding_size: None },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility:wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false, min_binding_size: None },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility:wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false, min_binding_size: None },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false
                    },
                    count: None
                }
            ]
        });

        let ssao_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &ssao_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(ssao_buffer.slice(..))
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(scene_base_resources.view_matrix_buffer.slice(..))
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(scene_base_resources.projection_matrix_buffer.slice(..))
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor {
                        label: None,
                        address_mode_u: wgpu::AddressMode::Repeat,
                        address_mode_v: wgpu::AddressMode::Repeat,
                        address_mode_w: wgpu::AddressMode::Repeat,
                        mag_filter: wgpu::FilterMode::Nearest,
                        min_filter: wgpu::FilterMode::Nearest,
                        mipmap_filter: Default::default(),
                        lod_min_clamp: 0.0,
                        lod_max_clamp: 0.0,
                        compare: None,
                        anisotropy_clamp: None
                    }))
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&random_vector_texture.create_view(&wgpu::TextureViewDescriptor::default()))
                }
            ]
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
                    resource: wgpu::BindingResource::TextureView(&deferred_pass.diffuse_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&deferred_pass.position_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&deferred_pass.normal_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&shadow_passes.shadow_sampler)
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&shadow_passes.shadow_texture_view)
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

            struct GpuLight {
                vec4 position; // 4 * 4 = 16
                vec4 color; // 4 * 4 = 16
                float intensity; // 4
                float radius; // 4
                float enabled; // 4
            };

            layout(set = 0, binding = 0) uniform Lights {
                GpuLight u_point_lights[20];
            };

            layout(set=1, binding=0) uniform sampler layer_sampler;
            layout(set=1, binding=1) uniform texture2D gAlbedo;
            layout(set=1, binding=2) uniform texture2D gPosition;
            layout(set=1, binding=3) uniform texture2D gNormal;
            layout(set=1, binding=4) uniform samplerShadow shadow_sampler;
            layout(set=1, binding=5) uniform texture2D shadow;

            layout(set = 2, binding = 0) uniform SceneUniforms {
                mat4 light_view_mat;
            };

            layout(set = 3, binding = 0) uniform Hemisphere { vec3 sample_points[64]; };
            layout(set = 3, binding = 1) uniform SceneBaseView { mat4 view_mat; };
            layout(set = 3, binding = 2) uniform SceneBaseProjection { mat4 projection_mat; };
            layout(set = 3, binding = 3) uniform sampler random_vec_sampler;
            layout(set = 3, binding = 4) uniform texture2D random_vec_texture;

            float fetch_shadow(vec4 homogeneous_coords) {
                if (homogeneous_coords.w <= 0.0) {
                    return 1.0;
                }

                const vec2 flip_correction = vec2(0.5, -0.5);
                // compute texture coordinates for shadow lookup
                vec3 light_local = vec3(
                    homogeneous_coords.xy * flip_correction/homogeneous_coords.w + 0.5,
                    homogeneous_coords.z / homogeneous_coords.w
                );
                // do the lookup, using HW PCF and comparison
                return texture(sampler2DShadow(shadow, shadow_sampler), light_local);
            }

            void main() {
                vec4 f_albedo = texture(sampler2D(gAlbedo, layer_sampler), tex_coord);
                vec3 f_position = texture(sampler2D(gPosition, layer_sampler), tex_coord).xyz;
                vec3 f_normal = normalize(texture(sampler2D(gNormal, layer_sampler), tex_coord).rgb * 2.0 - 1.0);
/*
                // Calculate a bias to avoid self-shadowing:
                vec3 shadow_light_dir = normalize(f_position - vec3(5.0, 15.0, -5.0));
                vec3 bias = (1.0 - dot(f_normal, shadow_light_dir)) * shadow_light_dir * 0.01; //Todo: Provide light position via uniform
                float shadow_f = fetch_shadow(light_view_mat * vec4(f_position - bias, 1.0)) + 0.25;
                shadow_f = 1.0; // shadows off
*/

                // Lambert Lighting

                vec4 color = f_albedo * vec4(0.05, 0.05, 0.05, 1.0);

                for(int i=0; i < 20; ++i) {
                    GpuLight light = u_point_lights[i];
                    if (light.enabled>0) {
                        vec4 view_space_light_pos = view_mat * light.position;
                        vec3 light_dir = normalize(view_space_light_pos.xyz - f_position);
                        color += vec4(max(0.0, dot(f_normal, light_dir)) * light.color.xyz * light.intensity, 1.0);
                    }
                }

                // Screen Space Ambient Occlusion

                const vec2 noise_scale = vec2( 1024.0 / 4.0, 768.0 / 4.0 ); // scale to cover whole screen

                vec3 random_vector = normalize(texture(sampler2D(random_vec_texture, random_vec_sampler), tex_coord * noise_scale).xyz);
                vec3 tangent = normalize( random_vector - f_normal * dot(random_vector, f_normal) );
                vec3 bitangent = cross(f_normal, tangent);
                mat3 tbn = mat3(tangent, bitangent, f_normal);

                float radius = 0.8;
                float ssao_bias = 0.0125;
                float occ = 0.0;
                vec3 debug = vec3(0.0, 0.0, 0.0);

                for(int i=0; i < 64; ++i) {
                    vec3 point = tbn * sample_points[i];
                    point = f_position + point * radius;

                    vec4 offset = vec4(point, 1.0);
                    offset = projection_mat * offset;
                    offset.xyz /= offset.w;
                    offset.xy = offset.xy * vec2(0.5, -0.5) + 0.5;

                    vec3 occluder_position = texture(sampler2D(gPosition, layer_sampler), offset.xy).xyz;

                    if(i==32) { debug = occluder_position; }

                    float rangeCheck = smoothstep(0.0, 1.0, radius / abs(point.z - occluder_position.z));
                    occ += (occluder_position.z >= (point.z + ssao_bias) ? 1.0 : 0.0) * rangeCheck;
                }

                //f_color = color * shadow_f * occ;
                //f_color = vec4(1.0, 1.0, 1.0, 1.0) * (1.0 - occ/64.0);
                //f_color = vec4(debug, 1.0);
                f_color = color * (1.0 - occ/64.0);
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
                bind_group_layouts: &[
                    &light_resources.lights_bind_group_layout,
                    &composition_bind_group_layout,
                    &shadow_passes.shadow_light_bind_group_layout,
                    &ssao_bind_group_layout
                ],
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
            ssao_bind_group
        }
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, swap_chain: &mut wgpu::SwapChain, light_resources: &LightsResources, shadow_passes: &ShadowPasses) {

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
            render_pass.set_bind_group(0, &light_resources.lights_bind_group, &[]);
            render_pass.set_bind_group(1, &self.g_buffer_bind_group, &[]);
            render_pass.set_bind_group(2, &shadow_passes.shadow_light_bind_group, &[]);
            render_pass.set_bind_group(3, &self.ssao_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..));
            render_pass.draw_indexed(0..8, 0, 0..1)
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
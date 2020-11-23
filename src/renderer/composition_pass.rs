use wgpu::util::*;

use super::{deferred_pass::DeferredPass, utils::GpuVector3, lights::LightsResources};
use crate::renderer::shadow_passes::ShadowPasses;
use crate::renderer::scene_base::SceneBaseResources;
use cgmath::InnerSpace;
use crate::renderer::ssao_pass::SSAOPass;

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
        ssao_pass: &SSAOPass,
        light_resources: &LightsResources,
        scene_base_resources: &SceneBaseResources
    ) -> CompositionPass {

    
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
            #extension GL_EXT_samplerless_texture_functions : require

            layout(location=0) in vec2 tex_coord;
            layout(location=0) out vec4 f_color;

            struct GpuLight {
                vec4 position; // 4 * 4 = 16
                vec4 color; // 4 * 4 = 16
                float intensity; // 4
                float radius; // 4
                float enabled; // 4
            };

            layout(set = 0, binding = 0) uniform SceneBase {
                mat4 view_mat;
                mat4 projection_mat;
                vec2 window_size;
            };

            layout(set = 1, binding = 0) uniform Lights {
                GpuLight u_point_lights[20];
            };

            layout(set=2, binding=0) uniform sampler layer_sampler;
            layout(set=2, binding=1) uniform texture2D gAlbedo;
            layout(set=2, binding=2) uniform texture2D gPosition;
            layout(set=2, binding=3) uniform texture2D gNormal;

            layout(set = 3, binding = 0) uniform ShadowUniforms {
                mat4 light_view_mat;
            };
            layout(set=4, binding=0) uniform samplerShadow shadow_sampler;
            layout(set=4, binding=1) uniform texture2D shadow;

            layout(set = 5, binding = 0) uniform texture2D ssao_texture;

            float fetch_shadow(vec4 homogeneous_coords) {
                if (homogeneous_coords.w <= 0.0) {
                    return 1.0;
                }

                const vec2 flip_correction = vec2(0.5, -0.5);

                vec3 light_local = vec3(
                    homogeneous_coords.xy * flip_correction/homogeneous_coords.w + 0.5,
                    homogeneous_coords.z / homogeneous_coords.w
                );

                return texture(sampler2DShadow(shadow, shadow_sampler), light_local);
            }

            void main() {
                vec4 f_albedo = texture(sampler2D(gAlbedo, layer_sampler), tex_coord);
                vec3 f_position = texture(sampler2D(gPosition, layer_sampler), tex_coord).xyz;
                vec3 f_normal = normalize(texture(sampler2D(gNormal, layer_sampler), tex_coord).xyz);

                //*** SHADOW MAPPING ***///

                float shadow_f = 1.0;

                // Calculate a bias to avoid self-shadowing:
                vec4 light_position = view_mat * vec4(5.0, 15.0, -5.0, 1.0);
                vec3 shadow_light_dir = normalize(f_position - light_position.xyz);

                vec3 bias = (1.0 - dot(f_normal, shadow_light_dir)) * shadow_light_dir * 0.0001; //Todo: Provide light position via uniform
                vec3 world_position = (inverse(view_mat) * vec4(f_position, 1.0)).xyz;
                shadow_f = fetch_shadow(light_view_mat * vec4(world_position, 1.0)) + 0.45;


                // Blur the ssao texture:

                vec2 texelSize = 1.0 / vec2(textureSize(ssao_texture, 0));
                vec2 ssao_tex_coord = vec2(0.0, 1.0) - (tex_coord) * vec2(-1.0, 1.0);
                float result = 0.0;

                for(int i=-2; i < 2; i++) {
                    for(int j=-2; j < 2; j++) {
                        vec2 offset = vec2(float(i), float(j)) * texelSize;
                        result += texture(sampler2D(ssao_texture, layer_sampler), ssao_tex_coord + offset).r;
                    }
                }

                float f_occlusion = result / (4.0 * 4.0);

                // Lambert Lighting

                vec4 ambient_light = vec4(0.6, 0.6, 0.6, 1.0);

                vec4 color = f_albedo * ambient_light;

                for(int i=0; i < 20; ++i) {
                    GpuLight light = u_point_lights[i];
                    if (light.enabled>0) {
                        vec4 view_space_light_pos = view_mat * light.position;
                        vec3 light_dir = normalize(view_space_light_pos.xyz - f_position);
                        color += vec4(max(0.0, dot(f_normal, light_dir)) * light.color.xyz * light.intensity, 0.0);
                    }
                }

                f_color = color * shadow_f * f_occlusion;
                //f_color = color * shadow_f;
                //f_color = vec4(1.0, 1.0, 1.0, 1.0) * shadow_f * f_occlusion;
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
                    &scene_base_resources.bind_group_layout,
                    &light_resources.lights_bind_group_layout,
                    &deferred_pass.gbuffer_bind_group_layout,
                    &shadow_passes.shadow_light_bind_group_layout,
                    &shadow_passes.shadow_result_bind_group_layout,
                    &ssao_pass.bind_group_layout
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
        }
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swap_chain: &mut wgpu::SwapChain,
        scene_base: &SceneBaseResources,
        light_resources: &LightsResources,
        deferred_pass: &DeferredPass,
        shadow_passes: &ShadowPasses,
        ssao_pass: &SSAOPass
    ) {

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
            render_pass.set_bind_group(0, &scene_base.bind_group, &[]);
            render_pass.set_bind_group(1, &light_resources.lights_bind_group, &[]);
            render_pass.set_bind_group(2, &deferred_pass.gbuffer_bind_group, &[]);
            render_pass.set_bind_group(3, &shadow_passes.shadow_light_bind_group, &[]);
            render_pass.set_bind_group(4, &shadow_passes.shadow_result_bind_group, &[]);
            render_pass.set_bind_group(5, &ssao_pass.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..));
            render_pass.draw_indexed(0..8, 0, 0..1)
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
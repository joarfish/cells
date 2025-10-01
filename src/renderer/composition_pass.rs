use wgpu::util::*;

use super::{deferred_pass::DeferredPass, lights::LightsResources, utils::GpuVector3};
use crate::renderer::scene_base::SceneBaseResources;
use crate::renderer::shadow_passes::ShadowPasses;
use crate::renderer::ssao_pass::SSAOPass;
use cgmath::InnerSpace;
use std::ops::Not;

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug)]
struct HemisphereSamples {
    points: [[f32; 3]; 64],
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
        scene_base_resources: &SceneBaseResources,
    ) -> CompositionPass {
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CompPass Vertex Buffer"),
            contents: bytemuck::cast_slice(&[
                GpuVector3::new(-1.0, -1.0, 0.0),
                GpuVector3::new(1.0, -1.0, 0.0),
                GpuVector3::new(1.0, 1.0, 0.0),
                GpuVector3::new(-1.0, 1.0, 0.0),
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Comp Pass Index Buffer"),
            contents: bytemuck::cast_slice(&[0 as u16, 2, 3, 0, 1, 2]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;
            layout(location=0) out vec2 tex_coord;

            void main() {
                tex_coord = (vec2(0.0,1.0) - a_position.xy + vec2(1.0,0.0)) * vec2(0.5, 0.5);
                gl_Position = vec4(a_position, 1.0);
            }
        "
        .to_string();

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

            mat4 inverseNoExt(mat4 m) {
              float
                  a00 = m[0][0], a01 = m[0][1], a02 = m[0][2], a03 = m[0][3],
                  a10 = m[1][0], a11 = m[1][1], a12 = m[1][2], a13 = m[1][3],
                  a20 = m[2][0], a21 = m[2][1], a22 = m[2][2], a23 = m[2][3],
                  a30 = m[3][0], a31 = m[3][1], a32 = m[3][2], a33 = m[3][3],

                  b00 = a00 * a11 - a01 * a10,
                  b01 = a00 * a12 - a02 * a10,
                  b02 = a00 * a13 - a03 * a10,
                  b03 = a01 * a12 - a02 * a11,
                  b04 = a01 * a13 - a03 * a11,
                  b05 = a02 * a13 - a03 * a12,
                  b06 = a20 * a31 - a21 * a30,
                  b07 = a20 * a32 - a22 * a30,
                  b08 = a20 * a33 - a23 * a30,
                  b09 = a21 * a32 - a22 * a31,
                  b10 = a21 * a33 - a23 * a31,
                  b11 = a22 * a33 - a23 * a32,

                  det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

              return mat4(
                  a11 * b11 - a12 * b10 + a13 * b09,
                  a02 * b10 - a01 * b11 - a03 * b09,
                  a31 * b05 - a32 * b04 + a33 * b03,
                  a22 * b04 - a21 * b05 - a23 * b03,
                  a12 * b08 - a10 * b11 - a13 * b07,
                  a00 * b11 - a02 * b08 + a03 * b07,
                  a32 * b02 - a30 * b05 - a33 * b01,
                  a20 * b05 - a22 * b02 + a23 * b01,
                  a10 * b10 - a11 * b08 + a13 * b06,
                  a01 * b08 - a00 * b10 - a03 * b06,
                  a30 * b04 - a31 * b02 + a33 * b00,
                  a21 * b02 - a20 * b04 - a23 * b00,
                  a11 * b07 - a10 * b09 - a12 * b06,
                  a00 * b09 - a01 * b07 + a02 * b06,
                  a31 * b01 - a30 * b03 - a32 * b00,
                  a20 * b03 - a21 * b01 + a22 * b00) / det;
            }

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
                vec3 world_position = (inverseNoExt(view_mat) * vec4(f_position, 1.0)).xyz;
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

        let vs_spirv = compiler
            .compile_into_spirv(
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

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Composition Vertex Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(vs_spirv.as_binary())),
        });
        let fragment_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Composition Fragment Shader"),
            source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(fs_spirv.as_binary())),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &scene_base_resources.bind_group_layout,
                    &light_resources.lights_bind_group_layout,
                    &deferred_pass.gbuffer_bind_group_layout,
                    &shadow_passes.shadow_light_bind_group_layout,
                    &shadow_passes.shadow_result_bind_group_layout,
                    &ssao_pass.bind_group_layout,
                ],
                push_constant_ranges: &[],
                label: None,
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 0,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x3,
                    }],
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: Default::default(),
                unclipped_depth: true,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader_module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
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
        surface: &mut wgpu::Surface,
        scene_base: &SceneBaseResources,
        light_resources: &LightsResources,
        deferred_pass: &DeferredPass,
        shadow_passes: &ShadowPasses,
        ssao_pass: &SSAOPass,
    ) {
        let screen_frame = surface
            .get_current_texture()
            .expect("Could not acquire texture for rendering");

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Composition Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &screen_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &scene_base.bind_group, &[]);
            render_pass.set_bind_group(1, &light_resources.lights_bind_group, &[]);
            render_pass.set_bind_group(2, &deferred_pass.gbuffer_bind_group, &[]);
            render_pass.set_bind_group(3, &shadow_passes.shadow_light_bind_group, &[]);
            render_pass.set_bind_group(4, &shadow_passes.shadow_result_bind_group, &[]);
            render_pass.set_bind_group(5, &ssao_pass.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertices.slice(..));
            render_pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..1)
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

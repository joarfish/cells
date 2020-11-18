use std::num::NonZeroU32;

use super::{lights::LightsResources, meshes::MeshResources, utils::GpuVector3};


pub struct ShadowPasses {
    shadow_texture: wgpu::Texture,
    shadow_texture_views: Vec<wgpu::TextureView>,
    pipeline: wgpu::RenderPipeline,
}

impl ShadowPasses {
    pub fn new(device: &wgpu::Device, mesh_resources: &MeshResources, lights_resources: &LightsResources, window_width: u32, window_height: u32) -> Self {

        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadows Texture"),
            format: wgpu::TextureFormat::Depth32Float,
            dimension: wgpu::TextureDimension::D2,
            sample_count: 1,
            mip_level_count: 1,
            size: wgpu::Extent3d { 
                width: window_width,
                height: window_height,
                depth: 20
            },
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED
        });
    
        let mut shadow_texture_views = Vec::with_capacity(20);
    
        for idx in 0..20 as u32 {
            shadow_texture_views.push(
                shadow_texture.create_view(& wgpu::TextureViewDescriptor {
                    label: None,
                    format: Some(wgpu::TextureFormat::Depth32Float),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    level_count: None, 
                    base_array_layer: idx,
                    array_layer_count: NonZeroU32::new(1)
                })
            );
        }

        // create vertex shader:
    
        let mut compiler = shaderc::Compiler::new().unwrap();

        let vs_code = "
            #version 450

            layout(location=0) in vec3 a_position;

            struct GpuLight {
                mat4 view_matrix;
                vec4 position;
                vec4 color;
                float intensity;
                float radius;
                bool enabled;
            };

            layout(set=0, binding=0)
            uniform SceneUniforms {
                GpuLight lights[20];
            };

            layout ( push_constant ) uniform LightBlock {
                int light_index;
            } u_push_constants;

            layout(set=1, binding=0)
            uniform ModelUniforms {
                mat4 u_transform;
            };

            void main() {        
                gl_Position = lights[u_push_constants.light_index].view_matrix * u_transform * vec4(a_position, 1.0);
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
                &lights_resources.lights_bind_group_layout,
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
            shadow_texture_views,
            pipeline
        }
    }
}

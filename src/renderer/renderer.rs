use imgui::MouseCursor;
use specs::prelude::*;

use handled_vec::{Handle};

use wgpu::util::*;

use std::{collections::BinaryHeap, time::Instant};

use super::{DeltaTimer, lights::GpuPointLight, resources::{BindGroupHandle, PipelineHandle, RenderObjectHandle, RendererResources}, utils::GpuVector3};

pub enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None,
}

pub struct CompositionResources {
    pub albedo_texture_view: wgpu::TextureView,
    pub position_texture_view: wgpu::TextureView,
    pub normal_texture_view: wgpu::TextureView,
    pub composition_pipeline: wgpu::RenderPipeline,
    pub composition_vertices: wgpu::Buffer,
    pub composition_indices: wgpu::Buffer,
    pub g_buffer_bind_group: wgpu::BindGroup,
    pub point_lights_bind_group_layout: wgpu::BindGroupLayout,
    pub point_lights_bind_group: wgpu::BindGroup,
    pub point_lights_buffer: wgpu::Buffer
}

pub fn setup_composition_resources(device: &wgpu::Device, width: u32, height: u32) -> CompositionResources {

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

    let albedo_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        label: None,
    });

    let position_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        label: None,
    });

    let normal_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        label: None,
    });

    let albedo_texture_view = albedo_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let position_texture_view = position_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let normal_texture_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                resource: wgpu::BindingResource::TextureView(&albedo_texture_view)
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&position_texture_view)
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&normal_texture_view)
            }
        ],
        layout: &composition_bind_group_layout
    });

    let composition_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[
            GpuVector3::new(-1.0, -1.0, 0.0),
            GpuVector3::new(1.0, -1.0, 0.0),
            GpuVector3::new(1.0, 1.0, 0.0),
            GpuVector3::new(-1.0, 1.0, 0.0)
        ]),
        usage: wgpu::BufferUsage::VERTEX
    });

    let composition_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&[0 as u16, 2, 3, 0, 1, 2]),
        usage: wgpu::BufferUsage::INDEX
    });

    let point_lights_data = vec![GpuPointLight::default(); 20];

    let point_lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Point Lights Buffer"),
        contents: bytemuck::cast_slice(&point_lights_data),
        usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM
    });

    let point_lights_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Point Lights Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None//wgpu::BufferSize::new((std::mem::size_of::<GpuPointLight>() * 20) as wgpu::BufferAddress)
                },
                count: None
            },
        ]
    });

    let point_lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Point Lights Bind Group"),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(point_lights_buffer.slice(..))
            }
        ],
        layout: &point_lights_bind_group_layout
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

        struct GpuPointLight {
            vec4 position;
            vec4 color;
            float intensity;
            float radius;
            bool enabled;
        };

        layout(set = 1, binding = 0) uniform Lights {
            GpuPointLight u_point_lights[20];
        };

        void main() {
            vec4 f_albedo = texture(sampler2D(gAlbedo, layer_sampler), tex_coord);
            vec3 f_position = texture(sampler2D(gPosition, layer_sampler), tex_coord).xyz;
            vec3 f_normal = texture(sampler2D(gNormal, layer_sampler), tex_coord).xyz;

            vec4 color = f_albedo * vec4(0.25, 0.25, 0.25, 1.0);

            for(int i=0; i < 20; ++i) {
                GpuPointLight light = u_point_lights[i];
                if (light.enabled) {
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
            bind_group_layouts: &[&composition_bind_group_layout, &point_lights_bind_group_layout ],
            push_constant_ranges: &[],
            label: None
        });

    let composition_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

    CompositionResources {
        albedo_texture_view,
        position_texture_view,
        normal_texture_view,
        composition_vertices,
        composition_indices,
        composition_pipeline,
        g_buffer_bind_group,
        point_lights_bind_group_layout,
        point_lights_bind_group,
        point_lights_buffer
    }
}

pub struct Renderer {
    pub instance: wgpu::Instance,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub last_cursor: Option<MouseCursor>,
    pub depth_view: wgpu::TextureView,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> (Self, wgpu::Device, wgpu::Queue) {

        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let (size, surface) = unsafe {
            let size = window.inner_size();
            let surface = instance.create_surface(window);
            (size, surface)
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        log::info!("Limits: {:?}", adapter.limits());

        // Todo: Specify required features
        let adapter_features = adapter.features();
        log::info!("Features: {:?}", adapter_features);

        // Todo: Specify limits
        let limits = wgpu::Limits {
            max_bind_groups: 6,
            ..wgpu::Limits::default()
        };
        log::info!("Limits: {:#?}", limits);

        let trace_dir = std::env::var("WGPU_TRACE");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: limits,
                    shader_validation: true,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        });

        ( Renderer {
            swap_chain,
            instance,
            size,
            surface,
            adapter,
            sc_desc,
            last_cursor: None,
            depth_view: depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
        }, device, queue )
    }

    pub fn render(&mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        resources: &RendererResources,
        composition: &CompositionResources,
        command_queue: &mut RendererCommandsQueue
    ) {
        // Sort queue in such a way that we can minimize switch bindings

        // We want to render a frame, so we need a frame:
        let screen_frame = self.swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture!")
            .output;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &composition.albedo_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: true,
                            },
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &composition.position_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &composition.normal_texture_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &self.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: false,
                        }),
                        stencil_ops: None
                    }),
            });
    
            while let Some(command) = command_queue.pop_render_command() {
                let render_object = resources.render_objects.get(&command.object).unwrap();
    
                let render_pipeline = resources.render_pipelines.get(&render_object.pipeline).unwrap();
    
                render_pass.set_pipeline(render_pipeline);
                
                for (bind_group_handle, set) in render_object.bind_groups.iter().zip(0..std::u32::MAX) {
                    let bind_group = resources.bind_groups.get(bind_group_handle).unwrap();
                    render_pass.set_bind_group(set, bind_group, &[]);
                }
    
                for (slice, slot) in render_object.vertex_buffers.iter().zip(0..std::u32::MAX) {
                    let buffer = resources.vertex_buffers.get(&slice.buffer).unwrap();
                    render_pass.set_vertex_buffer(slot, buffer.slice(slice.range.clone()));
                }
    
                let index_buffer = resources.index_buffers.get(&render_object.indices.buffer).unwrap();
                let index_buffer_range = render_object.indices.range.clone();
                render_pass.set_index_buffer(index_buffer.slice(index_buffer_range));
                render_pass.draw_indexed(0..render_object.index_count, 0, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));

        // Composition Pass:

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

            render_pass.set_pipeline(&composition.composition_pipeline);
            render_pass.set_bind_group(0, &composition.g_buffer_bind_group, &[]);
            render_pass.set_bind_group(1, &composition.point_lights_bind_group, &[]);
            render_pass.set_vertex_buffer(0, composition.composition_vertices.slice(..));
            render_pass.set_index_buffer(composition.composition_indices.slice(..));
            render_pass.draw_indexed(0..8, 0, 0..1)
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        device: &wgpu::Device
    ) {

        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = device.create_swap_chain(&self.surface, &self.sc_desc);


        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.sc_desc.width,
                height: self.sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        });

        self.depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

// Render System

impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, RendererEvent>,
        WriteExpect<'a, DeltaTimer>,
        WriteExpect<'a, RendererCommandsQueue>,
        Read<'a, RendererResources>,
        ReadExpect<'a, CompositionResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event,
            mut d_t,
            mut commands_queue,
            resources,
            composition,
            device,
            queue
        ) = data;

        match *event {
            RendererEvent::Render => {

                self.render(&device, &queue, &resources, &composition, &mut commands_queue);

                *event = RendererEvent::None;
                *d_t = DeltaTimer::new(Instant::now() - d_t.get_last_render(), Instant::now());
            }
            RendererEvent::Resize(size) => {
               self.resize(size, &device);
               *event = RendererEvent::None;
            }
            _ => (),
        }
    }
}


pub struct RenderCommand {
    pub object: RenderObjectHandle,
    pub layer: u8,
    pub distance: u8 // coarse value used for sorting within a layer
}

pub struct RendererCommandsQueue {
    queue: BinaryHeap<u32>
}

impl RendererCommandsQueue {
    pub fn new() -> Self {
        RendererCommandsQueue {
            queue: std::collections::BinaryHeap::new()
        }
    }

    pub fn command_count(&self) -> usize {
        self.queue.len()
    }

    pub fn push_render_command(&mut self, command: &RenderCommand) {
        // Assume a maximum of u16::MAX objects
        // only a few generational changes (max: 15)
        // only 15 levels and distance differences per level of 256
        // So:
        // | object.index (16bit) | object.generation (4bit) | layer (4bit) | distance (8bit)
        let index = command.object.get_index() as u32;
        let generation  = (((command.object.get_generation() as u8) << 4) | (command.layer)) as u16;
        self.queue.push(((index << 16) as u32) | ((generation << 8) | (command.distance as u16)) as u32);
    }
    
    pub fn pop_render_command(&mut self) -> Option<RenderCommand> {
        self.queue.pop().map(|command| {
            RenderCommand {
                object: RenderObjectHandle::new(
                    ((0b1111_1111_1111_1111_0000_0000_0000_0000u32 & command) >> 16) as usize,
                ((0b0000_0000_0000_0000_1111_0000_0000_0000u32 & command) >> 12) as usize
                ),
                layer: ((0b0000_0000_0000_0000_0000_1111_0000_0000u32 & command) >> 4) as u8,
                distance: (0b0000_0000_0000_0000_0000_0000_1111_1111u32 & command) as u8
            }
        })
    }
}

pub struct RenderPass {
    pub pipeline: PipelineHandle,
    pub bind_group_layouts: Vec<BindGroupHandle>,
}

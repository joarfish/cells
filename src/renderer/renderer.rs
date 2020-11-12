use imgui::MouseCursor;
use specs::prelude::*;
use specs::Component;
use wgpu::util::*;

use handled_vec::{Handle, HandledStdVec, MarkedHandle};

use std::{collections::BinaryHeap, time::Instant};
use std::vec::Vec;

use super::{DeltaTimer, resources::{BindGroupBufferHandle, BindGroupHandle, BufferHandle, PipelineHandle, RenderObjectHandle, RendererResources}, scene_base::SceneBaseResources, utils::GpuMatrix4, utils::GpuVector3};

pub enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None,
}

pub struct Renderer {
    render_command_queue: BinaryHeap<u32>,
    pub instance: wgpu::Instance,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub swap_chain: wgpu::SwapChain,
    pub last_cursor: Option<MouseCursor>
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

        ( Renderer {
            render_command_queue: BinaryHeap::new(),
            swap_chain,
            instance,
            size,
            surface,
            adapter,
            sc_desc,
            last_cursor: None
        }, device, queue )
    }

    pub fn dispatch_render_command(&mut self, command: u32) {
        self.render_command_queue.push(command);
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, resources: &RendererResources) {
        // Sort queue in such a way that we can minimize switch bindings

        // We want to render a frame, so we need a frame:
        let screen_frame = self.swap_chain
            .get_current_frame()
            .expect("Failed to acquire next swap chain texture!")
            .output;

        //screen_frame.output
        // Record command buffers:
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
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: None,
            });

            // Use this as soon as proper render command submission is implemented:
    
            /*for command in self.render_command_queue.iter() {
                let decoded_command = decode_render_command(command);
                let render_object = resources.render_objects.get(&decoded_command.object).unwrap();
    
                for (bind_group_handle, set) in render_object.bind_groups.iter().zip(1..std::u32::MAX) {
                    let bind_group = resources.bind_groups.get(bind_group_handle).unwrap();
                    render_pass.set_bind_group(set, bind_group, &[])
                }
    
                let render_pipeline = resources.render_pipelines.get(&render_object.pipeline).unwrap();
    
                render_pass.set_pipeline(render_pipeline);
    
                for (slice, slot) in render_object.vertex_buffers.iter().zip(1..std::u32::MAX) {
                    let buffer = resources.vertex_buffers.get(&slice.buffer).unwrap();
                    render_pass.set_vertex_buffer(slot, buffer.slice(slice.range.clone()));
                }
    
                let index_buffer = resources.index_buffers.get(&render_object.indices.buffer).unwrap();
                render_pass.set_index_buffer(index_buffer.slice(render_object.indices.range.clone()));
    
                render_pass.draw_indexed(0..render_object.get_indices_count(), 0, 0..1)
            }*/
            
            // This does not work because you cannot have multiple pipelines and bind groups per pass :/

            for render_object in resources.render_objects.get_iterator() {    
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
                /*render_pass.set_index_buffer(index_buffer.slice(..));
                render_pass.draw_indexed(0..3, 0, 0..1);*/
                render_pass.set_index_buffer(index_buffer.slice(index_buffer_range));
                render_pass.draw_indexed(0..render_object.index_count, 0, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
        device: &wgpu::Device
    ) {
        log::info!("Beginning to recreate swap chain...");
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = device.create_swap_chain(&self.surface, &self.sc_desc);
        log::info!("Finished recreate swap chain.");
    }
}

// Render System

impl<'a> System<'a> for Renderer {
    type SystemData = (
        WriteExpect<'a, RendererEvent>,
        WriteExpect<'a, DeltaTimer>,
        ReadExpect<'a, RendererResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut event,
            mut d_t,
            resources,
            device,
            queue
        ) = data;

        match *event {
            RendererEvent::Render => {

                self.render(&device, &queue, &resources);

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
    object: RenderObjectHandle,
    layer: u8,
    distance: u8 // coarse value used for sorting within a layer
}

pub fn encode_render_command(command: &RenderCommand) -> u32 {
    // Assume a maximum of u16::MAX objects
    // only a few generational changes (max: 15)
    // only 15 levels and distance differences per level of 256
    // So:
    // | object.index (16bit) | object.generation (4bit) | layer (4bit) | distance (8bit)
    let index = command.object.get_index() as u16;
    let generation  = (((command.object.get_generation() as u8) << 4) | (command.layer)) as u16;
    
    ((index << 16) as u32) | ((generation << 8) | (command.distance as u16)) as u32
}

pub fn decode_render_command(command: &u32) -> RenderCommand {
    
    RenderCommand {
        object: RenderObjectHandle::new(
            ((0b1111_1111_1111_1111_0000_0000_0000_0000u32 & command) >> 16) as usize,
        ((0b0000_0000_0000_0000_1111_0000_0000_0000u32 & command) >> 12) as usize
        ),
        layer: ((0b0000_0000_0000_0000_0000_1111_0000_0000u32 & command) >> 4) as u8,
        distance: (0b0000_0000_0000_0000_0000_0000_1111_1111u32 & command) as u8
    }
}

struct RendererCommandQueue {
    queue: std::collections::VecDeque<RendererEvent>
}

impl RendererCommandQueue {
    pub fn new() -> Self {
        RendererCommandQueue {
            queue: std::collections::VecDeque::new()
        }
    }

    pub fn enqueue_command(&mut self, command: RendererEvent) {
        if self.queue.len() < 5 {
            self.queue.push_back(command);
        } else {
            match command {
                RendererEvent::Render => {

                },
                _ => {
                    self.queue.push_back(command);
                }
            }
        }
    }
}
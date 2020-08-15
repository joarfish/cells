use winit::event_loop::EventLoop;
use winit::event_loop::ControlFlow;
use winit::event;
use winit::event::WindowEvent;
use winit::event::KeyboardInput;
use std::time::{Duration, Instant};
use wgpu::Instance;
use wgpu::util::DeviceExt;

mod renderer;

use renderer::render_object_type::{ RenderObjectType, RenderObject };
use renderer::static_mesh::{ StaticMesh, StaticMeshObject };
use renderer::camera::*;
use renderer::base::Vertex;

use specs::prelude::*;

struct RendererState {
    instance: wgpu::Instance,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    object_types : std::vec::Vec<RenderObject> 
}

impl RendererState {
    async fn new<'a>(window: &'a winit::window::Window) -> Self {
        
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

        let static_mesh_type = StaticMesh::new(&device, sc_desc.format);

        RendererState {
            instance,
            adapter,
            size,
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            object_types: vec![
                RenderObject::StaticMesh(static_mesh_type)
            ]
        }
    }
}

struct Renderer;

impl Renderer {
    fn render<'a>(RendererState { device, swap_chain, queue, object_types, ..}: &mut RendererState, objects : ReadStorage<'a, StaticMeshObject>) {
        let screen_frame = swap_chain.get_current_frame().expect("Failed to acquire next swap chain texture!");
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: std::borrow::Cow::Borrowed(&[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &screen_frame.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: true
                        },
                    }
                ]),
                depth_stencil_attachment: None
            });
            
            for object in objects.join() {
                let object_type = object_types.get(object.object_type()).unwrap();
                render_pass.set_pipeline(object_type.get_pipeline());
                render_pass.set_vertex_buffer(0, object.vertex_buffer());
                render_pass.set_index_buffer(object.index_buffer());
                render_pass.draw_indexed(0..object.get_indices_count(), 0, 0..1)
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn resize(RendererState {
        surface,
        device,
        sc_desc,
        ..
    } : &mut RendererState, new_size: winit::dpi::PhysicalSize<u32>) {
        sc_desc.width = new_size.width;
        sc_desc.height = new_size.height;
        device.create_swap_chain(&surface, &sc_desc);
    }
    
}

enum RendererEvent {
    Render,
    Resize(winit::dpi::PhysicalSize<u32>),
    None
}

impl <'a> System<'a> for Renderer {
    type SystemData = (WriteExpect<'a, RendererEvent>, WriteExpect<'a, RendererState>, ReadStorage<'a, StaticMeshObject>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut event, mut state, static_meshes) = data;

        match *event {
            RendererEvent::Render => {
                Self::render(&mut state, static_meshes);
                *event = RendererEvent::None;
            },
            RendererEvent::Resize(size) => {
                Self::resize(&mut state, size);
                *event = RendererEvent::None;
            },
            _ => ()
        }
    }
}



fn main() {

    env_logger::init();

    let mut world = World::new();

    let (mut pool, _spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };

    log::info!("Setting things up.");

    let event_loop = EventLoop::new();
    let window_builder = winit::window::WindowBuilder::new();
    let window = window_builder.with_title("Cells").build(&event_loop).unwrap();

    
    let renderer_state = futures::executor::block_on(RendererState::new(&window));


    /* Register Components */
    world.register::<StaticMeshObject>();

    /* Create Entities */
    world.create_entity().with(StaticMeshObject::new(
        renderer_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                Vertex::new([0.25, 0.25, 0.0], [1.0, 0.0, 0.0]),
                Vertex::new([-0.5, -0.25, 0.0], [0.0, 1.0, 0.0]),
                Vertex::new([0.5, -0.25, 0.0], [0.0, 0.0, 1.0]),
            ]),
            usage: wgpu::BufferUsage::VERTEX
        }),
        renderer_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                0 as u16,1,2
            ]),
            usage: wgpu::BufferUsage::INDEX
        })
    )).build();

    world.create_entity().with(StaticMeshObject::new(
        renderer_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                Vertex::new([0.75, 0.75, 0.0], [1.0, 0.0, 0.0]),
                Vertex::new([0.25, -0.75, 0.0], [0.0, 1.0, 0.0]),
                Vertex::new([0.99, -0.75, 0.0], [0.0, 0.0, 1.0]),
            ]),
            usage: wgpu::BufferUsage::VERTEX
        }),
        renderer_state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                0 as u16,1,2
            ]),
            usage: wgpu::BufferUsage::INDEX
        })
    )).build();

    /* Add Resources */
    world.insert(renderer_state);
    world.insert(RendererEvent::None);

    

    let mut dispatcher = DispatcherBuilder::new().with_thread_local(Renderer).build();

    event_loop.run(move |event, _, control_flow| {

        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(10))
        };

        let mut last_update_inst = Instant::now();
        {
            let mut renderer_event = world.write_resource::<RendererEvent>();

            match event {
                event::Event::MainEventsCleared => {
                    {
                        if last_update_inst.elapsed() > Duration::from_millis(20) {
                            window.request_redraw();
                            last_update_inst = Instant::now();
                        }

                        pool.run_until_stalled();
                    }
                }
                event::Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    log::info!("Resizing to {:?}", size);
                    *renderer_event = RendererEvent::Resize(window.inner_size());
                }
                event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(event::VirtualKeyCode::Escape),
                                state: event::ElementState::Pressed,
                                ..
                            },
                        ..
                    }
                    | WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        log::info!("Closing Application.");
                    }
                    _ => {
                        
                    }
                },
                event::Event::RedrawRequested(_) => {
                    *renderer_event = RendererEvent::Render;             
                }
                _ => {}
            }
        }

        dispatcher.dispatch(&mut world);
        world.maintain();
    });
}

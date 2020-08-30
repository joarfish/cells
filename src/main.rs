mod input;
mod renderer;

use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use winit::event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use specs::prelude::*;
use renderer::base::Vertex;
use renderer::camera::*;
use renderer::static_mesh::{StaticMesh};
use input::InputMap;
use renderer::render_state::RendererState;

use renderer::gui::{ GUI, GUItWrapper };
use renderer::renderer::RendererEvent;
use renderer::base::DeltaTimer;
use renderer::renderer::Renderer;

fn main() {
    env_logger::init();

    let mut world = World::new();

    /*let (mut pool, _spawner) = {
        let local_pool = futures::executor::LocalPool::new();
        let spawner = local_pool.spawner();
        (local_pool, spawner)
    };*/

    log::info!("Setting things up.");

    let event_loop = EventLoop::new();
    let window_builder = winit::window::WindowBuilder::new();
    let window = window_builder
        .with_title("Cells")
        .build(&event_loop)
        .unwrap();

    let mut renderer_state = futures::executor::block_on(RendererState::new(window));

    let mut gui = GUI::setup(&renderer_state.window, &renderer_state.device, &mut renderer_state.queue, &renderer_state.sc_desc);

    /* Register Components */
    world.register::<StaticMesh>();
    world.register::<Camera>();
    world.register::<ActiveCamera>();

    /* Create Entities */
    world
        .create_entity()
        .with(StaticMesh::new(
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        Vertex::new([0.25, 0.25, 0.0], [1.0, 0.0, 0.0]),
                        Vertex::new([-0.5, -0.25, 0.0], [0.0, 1.0, 0.0]),
                        Vertex::new([0.5, -0.25, 0.0], [0.0, 0.0, 1.0]),
                    ]),
                    usage: wgpu::BufferUsage::VERTEX,
                }),
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[0 as u16, 1, 2]),
                    usage: wgpu::BufferUsage::INDEX,
                }),
        ))
        .build();

    world
        .create_entity()
        .with(StaticMesh::new(
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        Vertex::new([0.75, 0.75, 0.0], [1.0, 0.0, 0.0]),
                        Vertex::new([0.25, -0.75, 0.0], [0.0, 1.0, 0.0]),
                        Vertex::new([0.99, -0.75, 0.0], [0.0, 0.0, 1.0]),
                    ]),
                    usage: wgpu::BufferUsage::VERTEX,
                }),
            renderer_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[0 as u16, 1, 2]),
                    usage: wgpu::BufferUsage::INDEX,
                }),
        ))
        .build();

    world
        .create_entity()
        .with(Camera::new(
            renderer_state.sc_desc.width as f32 / renderer_state.sc_desc.height as f32,
        ))
        .with(ActiveCamera)
        .build();

    /* Add Resources */
    world.insert(RendererEvent::None);
    world.insert(renderer_state);
    world.insert(DeltaTimer::new(
        Duration::from_millis(0),
        Instant::now()
    ));
    world.insert(InputMap::new());
    world.insert(GUItWrapper::new(&mut gui));

    let mut dispatcher = DispatcherBuilder::new()
        .with(CameraSystem, "Camera System", &[])
        .with_thread_local(Renderer)
        .build();

    let mut last_update = Instant::now();
    let mut last_render = Instant::now();

    //let mut last_cursor = None;

    event_loop.run(move |event, _, control_flow| {
        {
            let wrapper = world.read_resource::<GUItWrapper>();
            let RendererState { window, .. } = &*world.read_resource::<RendererState>();
            let gui = wrapper.get();
            let (imgui, platform, _) = gui.get();
            platform.handle_event(imgui.io_mut(), &window, &event);
        }
        {
            match event {
                event::Event::MainEventsCleared => {
                    let RendererState { window, .. } = &*world.read_resource::<RendererState>();
                    window.request_redraw();
                }
                event::Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    log::info!("Resizing to {:?}", size);
                    let mut cameras = world.write_component::<Camera>();
                    for camera in (&mut cameras).join() {
                        (*camera).resize(size);
                    }
                    let mut renderer_event = world.write_resource::<RendererEvent>();
                    *renderer_event = RendererEvent::Resize(size);
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
                    },
                    WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(key_code),
                                state: key_state,
                                ..
                            },
                        ..
                    } => {
                        let mut input_map = world.write_resource::<InputMap>();
                        input_map.update(key_code, key_state);
                    }
                    _ => {}
                },
                event::Event::RedrawRequested(_) => {

                    // Render approx. 60 times a second
                    if (Instant::now() - last_render) >= Duration::from_millis(16) {
                        *world.write_resource::<RendererEvent>() = RendererEvent::Render;
                        last_render = Instant::now();
                    }

                    // Update approx. 250 times per second
                    if (Instant::now() - last_update) >= Duration::from_millis(4) {
                        dispatcher.dispatch(&mut world);
                        world.maintain();
                        last_update = Instant::now();
                    }

                    {
                        let RendererState { window, .. } = &*world.read_resource::<RendererState>();
                        window.request_redraw();
                    }
                }
                _ => {}
            }


        }
    });
}

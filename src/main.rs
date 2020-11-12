mod input;
mod renderer;
mod scene;

use std::time::{Duration, Instant};
use renderer::{DeltaTimer, camera::{ActiveCamera, Camera, CameraSystem}, dynamic_objects::{Color, DynamicObject, DynamicObjectsResources, DynamicObjectsSystem, TransformationTests}, renderer::{Renderer, RendererEvent}, resources::RendererResources, scene_base::SceneBaseResources, scene_base::SceneBaseSystem};
use scene::scene_graph::Transformation;
use winit::{window::Window, event};
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use specs::prelude::*;
use input::InputMap;
use cgmath::prelude::*;

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

    let (renderer, device, queue) = futures::executor::block_on(Renderer::new(&window));
    let mut resources = RendererResources::new();
    let scene_base_resources = SceneBaseResources::new(&device, &mut resources);
    let mut dynamic_objects_resources = DynamicObjectsResources::new(&device, &scene_base_resources, &mut resources);

    /* Register Components */
    
    world.register::<Camera>();
    world.register::<DynamicObject>();
    world.register::<Transformation>();
    world.register::<Color>();

    let active_camera = world
        .create_entity()
        .with(Camera::new(
            renderer.sc_desc.width as f32 / renderer.sc_desc.height as f32,
        ))
        .build();

    world.create_entity()
    .with(
        dynamic_objects_resources.create_dynamic_object(&device, &mut resources, &scene_base_resources)
    )
    .with(
        Transformation {
            position: cgmath::Point3::new(0.0, 0.0, -5.0),
            rotation: cgmath::Euler { x: cgmath::Deg(0.0), y: cgmath::Deg(0.0), z: cgmath::Deg(0.0) },
            scale: cgmath::Point3::new(1.0, 1.0, 1.0)
        }
    )
    .with(Color { r: 0.0, g: 1.0, b: 0.0})
    .build();

    world.create_entity()
    .with(
        dynamic_objects_resources.create_dynamic_object(&device, &mut resources, &scene_base_resources)
    )
    .with(
        Transformation {
            position: cgmath::Point3::new(1.0, 1.0, 0.0),
            rotation: cgmath::Euler { x: cgmath::Deg(0.0), y: cgmath::Deg(0.0), z: cgmath::Deg(45.0) },
            scale: cgmath::Point3::new(1.0, 1.0, 1.0)
        }
    )
    .with(Color { r: 1.0, g: 0.0, b: 0.0})
    .build();

    /* Add Resources */

    world.insert(device);
    world.insert(queue);

    world.insert(dynamic_objects_resources);

    world.insert(scene_base_resources);

    world.insert(resources);

    world.insert(RendererEvent::None);

    world.insert(window);
    
    world.insert(DeltaTimer::new(
        Duration::from_millis(0),
        Instant::now()
    ));
    world.insert(InputMap::new());
    //world.insert(GUItWrapper::new(&mut gui));
    world.insert(ActiveCamera(active_camera));

    let mut dispatcher = DispatcherBuilder::new()
        .with(CameraSystem, "Camera System", &[])
        .with(TransformationTests, "Transformation Tests", &[])
        .with(SceneBaseSystem, "BaseObjectSystem", &["Camera System"])
        .with(DynamicObjectsSystem, "DynamicObjectsSystem", &[])
        .with_thread_local(renderer)
        .build();

    let mut last_update = Instant::now();
    let mut last_render = Instant::now();

    

    //let mut last_cursor = None;

    event_loop.run(move |event, _, control_flow| {
        {
            //let wrapper = world.read_resource::<GUItWrapper>();
            //let RendererState { window, .. } = &*world.read_resource::<RendererState>();
            //let gui = wrapper.get();
            //let (imgui, platform, _) = gui.get();
            //platform.handle_event(imgui.io_mut(), &window, &event);
        }
        {
            match event {
                event::Event::MainEventsCleared => {
                    let window = world.read_resource::<Window>();
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
                        // this should be a queue!
                        let mut render_event= world.write_resource::<RendererEvent>();
                        match *render_event {
                            RendererEvent::None => {
                                *render_event = RendererEvent::Render;
                                last_render = Instant::now();
                            }
                            _ => {

                            }
                        }
                    }

                    // Update approx. 250 times per second
                    if (Instant::now() - last_update) >= Duration::from_millis(4) {
                        dispatcher.dispatch(&mut world);
                        world.maintain();
                        last_update = Instant::now();
                    }

                    {
                        let window = world.read_resource::<Window>();
                        window.request_redraw();
                    }
                }
                _ => {}
            }


        }
    });
}


mod input;
mod renderer;
mod scene;

use std::time::{Duration, Instant};
use renderer::{renderer::RendererEvent, setup_rendering};
use scene::{camera::{ActiveCamera, Camera, CameraSystem}, dynamic_objects::Color, dynamic_objects::{DynamicObjectsSystem, TransformationTests}, lights::{LightSystem, PointLight}, scene_graph::{SceneGraph, Transformation}, setup_scene, spawning::Spawner};
use winit::{window::Window, event};
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use specs::prelude::*;
use input::{InputMap, InputSystem};
use crate::scene::static_objects::{StaticObjectsSystem, StaticObject};
use crate::scene::SceneInfo;
use crate::renderer::meshes::MeshResources;
use crate::scene::playing_field::PlayingField;

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

    let window_size = window.inner_size();

    let renderer = setup_rendering(&mut world, &window);
    setup_scene(&mut world);

    /* Register Components */

    let active_camera = world
        .create_entity()
        .with(Camera::new(
            window_size.width as f32 / window_size.height as f32,
        ))
        .build();

    /* Add Resources */

    world.insert(window);

    world.insert(InputMap::new());
    //world.insert(GUItWrapper::new(&mut gui));
    world.insert(ActiveCamera(active_camera));

    let mut dispatcher = DispatcherBuilder::new()
        .with(CameraSystem, "Camera System", &[])
        .with(TransformationTests, "Transformation Tests", &[])
        .with(DynamicObjectsSystem::default(), "DynamicObjectsSystem", &[])
        .with(StaticObjectsSystem::default(), "StaticObjectsSystem", &[])
        .with(Spawner::default(), "Test Spawner", &[])
        .with(SceneGraph::default(), "Scene", &[])
        .with(LightSystem::default(), "Light System", &[])
        .with(PlayingField::new(), "Playing Field System", &["StaticObjectsSystem"])
        .with(InputSystem, "InputSystem", &["Camera System"])
        .with_thread_local(renderer)
        .build();

    dispatcher.setup(&mut world);

    let mut last_update = Instant::now();
    let mut last_render = Instant::now();

    world.create_entity().with(PointLight {
        position: cgmath::Vector3::new(2.0, 15.0, 2.0),
        color: cgmath::Vector3::new(1.0, 1.0, 1.0),
        intensity: 0.2625,
        radius: 40.0,
        light_index: 0
    }).build();

    world.create_entity().with(PointLight {
        position: cgmath::Vector3::new(8.0, 3.0, 8.0),
        color: cgmath::Vector3::new(1.0, 1.0, 1.0),
        intensity: 0.2625,
        radius: 20.0,
        light_index: 1
    }).build();

    world.create_entity().with(PointLight {
        position: cgmath::Vector3::new(-8.0, 3.0, 8.0),
        color: cgmath::Vector3::new(1.0, 1.0, 1.0),
        intensity: 0.2625,
        radius: 20.0,
        light_index: 2
    }).build();


    world.create_entity().with(PointLight {
        position: cgmath::Vector3::new(-8.0, 3.0, -8.0),
        color: cgmath::Vector3::new(1.0, 1.0, 1.0),
        intensity: 0.2625,
        radius: 20.0,
        light_index: 3
    }).build();


    world.create_entity().with(PointLight {
        position: cgmath::Vector3::new(8.0, 3.0, -8.0),
        color: cgmath::Vector3::new(1.0, 1.0, 1.0),
        intensity: 0.2625,
        radius: 20.0,
        light_index: 4
    }).build();

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
                    },
                    WindowEvent::MouseWheel {
                        delta: event::MouseScrollDelta::LineDelta(_, delta_y),
                        ..
                    } => {
                        let mut input_map = world.write_resource::<InputMap>();
                        input_map.update_mouse_wheel(delta_y);
                    },
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


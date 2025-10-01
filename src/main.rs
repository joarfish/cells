mod input;
mod renderer;
mod scene;

use crate::renderer::meshes::MeshResources;
use crate::scene::playing_field::PlayingField;
use crate::scene::solid_object::{SolidObject, SolidObjectSystem};
use imgui::Key;
use input::{InputMap, InputSystem};
use renderer::{renderer::RendererEvent, setup_rendering};
use scene::{
    camera::{ActiveCamera, Camera, CameraSystem},
    lights::{LightSystem, PointLight},
    scene_graph::{SceneGraph, Transformation},
    setup_scene,
    spawning::Spawner,
};
use specs::prelude::*;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode, NamedKey};
use winit::window::WindowId;
use winit::{event, keyboard, window::Window};

struct App<'a, 'b> {
    world: Option<World>,
    last_render: Instant,
    last_update: Instant,
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> Default for App<'a, 'b> {
    fn default() -> Self {
        App {
            world: None,
            last_render: Instant::now(),
            last_update: Instant::now(),
            dispatcher: None,
        }
    }
}

impl<'a, 'b> ApplicationHandler for App<'a, 'b> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.world.is_none() {
            let attrs = Window::default_attributes().with_title("Cells");
            let window = std::sync::Arc::new(event_loop.create_window(attrs).expect("Create Window"));

            log::info!("Setting things up.");

            let window_size = window.inner_size();

            let mut world = World::new();
            let renderer = setup_rendering(&mut world, window.clone());
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
                .with(Spawner::default(), "Test Spawner", &[])
                .with(SceneGraph::default(), "Scene", &[])
                .with(LightSystem::default(), "Light System", &[])
                .with(SolidObjectSystem::new(), "Solid Objects System", &[])
                .with(
                    PlayingField::new(),
                    "Playing Field System",
                    &["Solid Objects System"],
                )
                .with(InputSystem, "InputSystem", &["Camera System"])
                .with_thread_local(renderer)
                .build();

            dispatcher.setup(&mut world);

            let mut last_update = Instant::now();
            let mut last_render = Instant::now();

            world
                .create_entity()
                .with(PointLight {
                    position: cgmath::Vector3::new(2.0, 15.0, 2.0),
                    color: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    intensity: 0.2625,
                    radius: 40.0,
                    light_index: 0,
                })
                .build();

            world
                .create_entity()
                .with(PointLight {
                    position: cgmath::Vector3::new(8.0, 3.0, 8.0),
                    color: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    intensity: 0.2625,
                    radius: 20.0,
                    light_index: 1,
                })
                .build();

            world
                .create_entity()
                .with(PointLight {
                    position: cgmath::Vector3::new(-8.0, 3.0, 8.0),
                    color: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    intensity: 0.2625,
                    radius: 20.0,
                    light_index: 2,
                })
                .build();

            world
                .create_entity()
                .with(PointLight {
                    position: cgmath::Vector3::new(-8.0, 3.0, -8.0),
                    color: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    intensity: 0.2625,
                    radius: 20.0,
                    light_index: 3,
                })
                .build();

            world
                .create_entity()
                .with(PointLight {
                    position: cgmath::Vector3::new(8.0, 3.0, -8.0),
                    color: cgmath::Vector3::new(1.0, 1.0, 1.0),
                    intensity: 0.2625,
                    radius: 20.0,
                    light_index: 4,
                })
                .build();

            self.world = Some(world);
            self.dispatcher = Some(dispatcher);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                log::info!("Resizing to {:?}", size);
                if let Some(world) = &self.world {
                    let mut cameras = world.write_component::<Camera>();
                    for camera in (&mut cameras).join() {
                        (*camera).resize(size);
                    }
                    let mut renderer_event = world.write_resource::<RendererEvent>();
                    *renderer_event = RendererEvent::Resize(size);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: keyboard::PhysicalKey::Code(KeyCode::Escape),
                        state: event::ElementState::Pressed,
                        ..
                    },
                ..
            }
            | WindowEvent::CloseRequested => {
                event_loop.exit();
                log::info!("Closing Application.");
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: key_code,
                        state: key_state,
                        ..
                    },
                ..
            } => {
                if let Some(world) = &self.world {
                    let mut input_map = world.write_resource::<InputMap>();
                    input_map.update(key_code, key_state);
                }
            }
            WindowEvent::MouseWheel {
                delta: event::MouseScrollDelta::LineDelta(_, delta_y),
                ..
            } => {
                if let Some(world) = &self.world {
                    let mut input_map = world.write_resource::<InputMap>();
                    input_map.update_mouse_wheel(delta_y);
                }
            }

            WindowEvent::RedrawRequested => {
                // Render approx. 60 times a second
                if (Instant::now() - self.last_render) >= Duration::from_millis(16)
                    && let Some(world) = &self.world
                {
                    // this should be a queue!
                    let mut render_event = world.write_resource::<RendererEvent>();
                    match *render_event {
                        RendererEvent::None => {
                            *render_event = RendererEvent::Render;
                            self.last_render = Instant::now();
                        }
                        _ => {}
                    }
                }

                // Update approx. 250 times per second
                if (Instant::now() - self.last_update) >= Duration::from_millis(4)
                    && let Some(dispatcher) = &mut self.dispatcher
                    && let Some(world) = &mut self.world
                {
                    dispatcher.dispatch(world);
                    world.maintain();
                    self.last_update = Instant::now();
                }

                if let Some(world) = &self.world {
                    let window = world.read_resource::<std::sync::Arc<Window>>();
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();

    event_loop.run_app(&mut app).expect("TODO: panic message");
}

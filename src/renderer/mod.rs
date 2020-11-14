use std::time::{Duration, Instant};
use specs::prelude::*;
use crate::renderer::renderer::{Renderer, RendererEvent};
use crate::renderer::resources::RendererResources;
use crate::renderer::scene_base::SceneBaseResources;
use crate::renderer::dynamic_objects::{DynamicObjectsResources, DynamicObject};
use crate::renderer::camera::Camera;

use self::renderer::RendererCommandsQueue;

pub mod renderer;
pub mod camera;
pub mod resources;
pub mod scene_base;
pub mod dynamic_objects;
pub mod mesh;

mod utils;

pub struct DeltaTimer {
    d: Duration,
    last_render: Instant,
}

impl DeltaTimer {

    pub fn new(d : Duration, last_render : Instant) -> Self {
        DeltaTimer {
            d,
            last_render
        }
    }

    pub fn get_duration_f32(&self) -> f32 {
        self.d.as_secs_f32()
    }

    pub fn get_last_render(&self) -> Instant {
        self.last_render
    }
}

pub fn setup_rendering(world: &mut World, window: &winit::window::Window) -> Renderer {
    let (renderer, device, queue) = futures::executor::block_on(Renderer::new(&window));
    let mut resources = RendererResources::new();
    let scene_base_resources = SceneBaseResources::new(&device, &mut resources);
    let dynamic_objects_resources = DynamicObjectsResources::new(&device, &scene_base_resources, &mut resources);

    world.register::<Camera>();
    world.register::<DynamicObject>();

    world.insert(device);
    world.insert(queue);

    world.insert(dynamic_objects_resources);

    world.insert(scene_base_resources);

    world.insert(resources);

    world.insert(RendererEvent::None);

    world.insert(DeltaTimer::new(
        Duration::from_millis(0),
        Instant::now()
    ));

    world.insert(RendererCommandsQueue::new());

    renderer
}
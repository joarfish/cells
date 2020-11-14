use std::time::{Duration, Instant};
use specs::prelude::*;
use crate::{scene::scene_graph::Transformation, renderer::renderer::{Renderer, RendererEvent}};
use crate::renderer::resources::RendererResources;
use crate::renderer::scene_base::SceneBaseResources;
use crate::renderer::dynamic_objects::{DynamicObjectsResources, DynamicObject};
use crate::renderer::camera::Camera;

use self::{dynamic_objects::Color, lights::PointLight, renderer::{RendererCommandsQueue, setup_composition_resources}, static_objects::StaticObject, static_objects::StaticObjectsResources};

pub mod renderer;
pub mod camera;
pub mod resources;
pub mod scene_base;
pub mod dynamic_objects;
pub mod static_objects;
pub mod mesh;
pub mod lights;

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
    let mut static_objects_resources = StaticObjectsResources::new(&device, &scene_base_resources, &mut resources);

    let composition_resources = setup_composition_resources(&device, window.inner_size().width, window.inner_size().height);

    world.register::<Camera>();
    world.register::<DynamicObject>();
    world.register::<StaticObject>();
    world.register::<PointLight>();

    world.create_entity()
        .with(static_objects_resources.create_static_object(&device, &queue, &mut resources, &scene_base_resources, Transformation {
            position: cgmath::Point3::new(0.0, -1.0, 0.0),
            rotation: cgmath::Euler { x: cgmath::Deg(0.0), y: cgmath::Deg(0.0), z: cgmath::Deg(0.0) },
            scale: cgmath::Point3::new(20.0, 1.0, 20.0),
        }))
        .build();

    world.insert(device);
    world.insert(queue);

    world.insert(dynamic_objects_resources);
    world.insert(static_objects_resources);
    world.insert(composition_resources);

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
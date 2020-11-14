use specs::prelude::*;

use crate::{input::{InputMap, KeyState}, renderer::{dynamic_objects::{Color, DynamicObject, DynamicObjectsResources}, resources::RendererResources, scene_base::SceneBaseResources}};

use super::scene_graph::Transformation;
pub struct Spawner {
    last_spawned: std::time::Instant
}

impl Default for Spawner {
    fn default() -> Self {
        Spawner {
            last_spawned: std::time::Instant::now()
        }
    }
}

impl<'a> System<'a> for Spawner {
    type SystemData = (
        WriteExpect<'a, RendererResources>,
        WriteExpect<'a, DynamicObjectsResources>,
        Entities<'a>,
        WriteStorage<'a, DynamicObject>,
        WriteStorage<'a, Color>,
        WriteStorage<'a, Transformation>,
        ReadExpect<'a, SceneBaseResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, InputMap>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut renderer_resources,
            mut dynamic_objects_resources,
            entities,
            mut dynamic_objects,
            mut colors,
            mut transformations,
            scene_base_resources,
            device,
            input_map
        ) = data;

        let now = std::time::Instant::now();

        if now - self.last_spawned > std::time::Duration::from_millis(250) && input_map.key_p == KeyState::Pressed {

            use rand::Rng;

            let mut rng = rand::thread_rng();
            let x = rng.gen_range(-5.0, 5.0);
            let y = 0.0;//rng.gen_range(-2.0, 2.0);
            let z = rng.gen_range(-5.0, 5.0);
            let r = 1.0; // rng.gen_range(0.0, 1.0);
            let g = 1.0; // rng.gen_range(0.0, 1.0);
            let b = 1.0; // rng.gen_range(0.0, 1.0);

            entities.build_entity()
                .with(
                    dynamic_objects_resources.create_dynamic_object(&device, &mut renderer_resources, &scene_base_resources),
                    &mut dynamic_objects
                )
                .with(
                    Transformation {
                        position: cgmath::Point3::new(x, y, z),
                        rotation: cgmath::Euler { x: cgmath::Deg(0.0), y: cgmath::Deg(0.0), z: cgmath::Deg(0.0) },
                        scale: cgmath::Point3::new(1.0, 1.0, 1.0)
                    },
                    &mut transformations
                )
                .with(Color { r, g, b},
                    &mut colors
                )
                .build();

            self.last_spawned = now;
        }

    }
}
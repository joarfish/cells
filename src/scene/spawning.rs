use specs::prelude::*;

use crate::{input::{InputMap, KeyState}, renderer::{meshes::MeshResources, geometry::create_cube_geometry}};

use super::{dynamic_objects::{Color, DynamicObject}, scene_graph::Transformation};
use crate::scene::SceneInfo;

pub struct Spawner {
    last_spawned: std::time::Instant,
    cube_geo_index: u32
}

impl Default for Spawner {
    fn default() -> Self {
        Spawner {
            last_spawned: std::time::Instant::now(),
            cube_geo_index: 0
        }
    }
}

impl<'a> System<'a> for Spawner {
    type SystemData = (
        WriteExpect<'a, MeshResources>,
        Entities<'a>,
        WriteStorage<'a, DynamicObject>,
        WriteStorage<'a, Color>,
        WriteStorage<'a, Transformation>,
        ReadExpect<'a, SceneInfo>,
        ReadExpect<'a, InputMap>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut mesh_resources,
            entities,
            mut dynamic_objects,
            mut colors,
            mut transformations,
            scene_info,
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

            let mesh = mesh_resources.create_mesh(self.cube_geo_index, scene_info.dynamic_objects_pool as u16);
            log::info!("Created Static Object with Mesh: object_index={}, geometry_index={}, pool_index={}", mesh.object_index, mesh.geometry_index, mesh.pool_index);

            entities.build_entity()
                .with(
                    DynamicObject {
                        mesh
                    },
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

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        let mut mesh_resources = world.write_resource::<MeshResources>();
        let device = world.read_resource::<wgpu::Device>();
        let cube_geometry = create_cube_geometry();
        self.cube_geo_index = mesh_resources.add_geometry(&device, &cube_geometry) as u32;
    }
}
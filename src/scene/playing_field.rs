use specs::prelude::*;
use crate::scene::static_objects::StaticObject;
use crate::scene::scene_graph::Transformation;
use crate::scene::dynamic_objects::Color;
use crate::scene::SceneInfo;
use crate::renderer::meshes::MeshResources;

pub struct PlayingField {
    cells_horizontal: u32,
    cells_vertical: u32,
}

impl PlayingField {
    pub fn new() -> Self {
        PlayingField {
            cells_horizontal: 20,
            cells_vertical: 20
        }
    }
}

impl<'a> System<'a> for PlayingField {
    type SystemData = (

    );

    fn run(&mut self, data: Self::SystemData) {

    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        // Setup mesh pool for cells:
        let pool_index = {
            let mut mesh_resources = world.write_resource::<MeshResources>();
            let device = world.read_resource::<wgpu::Device>();
            mesh_resources.add_pool(&device, self.cells_horizontal * self.cells_vertical)
        };

        // Create Cells:

        let mut meshes = vec![];
        let mut transforms = vec![];
        let mut colors = vec![];

        {
            let mut mesh_resources = world.write_resource::<MeshResources>();

            for x in 0..self.cells_horizontal {
                for z in 0..self.cells_vertical {
                    let mesh = mesh_resources.create_mesh(0, pool_index as u16);

                    meshes.push(StaticObject {
                        mesh
                    });

                    transforms.push(
                        Transformation {
                            position: cgmath::Point3::new(x as f32, -1.0, z as f32),
                            rotation: cgmath::Euler { x: cgmath::Deg(0.0), y: cgmath::Deg(0.0), z: cgmath::Deg(0.0) },
                            scale: cgmath::Point3::new(1.0, 0.25, 1.0)
                        }
                    );

                    colors.push(Color { r: 0.0, g: 0.4, b: 0.0});
                }
            }
        }

        for ((static_object, transformation), color) in (meshes.into_iter().zip(transforms.into_iter())).zip(colors.into_iter()) {
            world.create_entity()
                .with(static_object)
                .with(transformation)
                .with(color)
                .build();
        }
    }
}
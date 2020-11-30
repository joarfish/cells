use specs::prelude::*;
use crate::scene::scene_graph::Transformation;
use crate::renderer::meshes::{MeshResources, MeshType};
use crate::renderer::geometry::create_cube_geometry;
use crate::scene::solid_object::SolidObject;
use crate::renderer::material::{MaterialResources, GpuMaterial};

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

        // Setup cell mesh type:
        let (cell_mesh_type, cell_material) = {
            let device = world.read_resource::<wgpu::Device>();
            let queue = world.read_resource::<wgpu::Queue>();
            let mut mesh_resources = world.write_resource::<MeshResources>();
            let mut material_resources = world.write_resource::<MaterialResources>();

            let cell_mesh_type = mesh_resources.add_mesh_type(MeshType::new(
                &device,
                "Cell",
                (self.cells_vertical * self.cells_horizontal) as usize,
                create_cube_geometry()
            ));

            let cell_material = material_resources.add_material(&queue, GpuMaterial {
                primary: cgmath::Vector4::new(0.5, 0.5, 0.5, 1.0),
                secondary: cgmath::Vector4::new(1.0, 0.0, 0.0, 1.0),
                tertiary: cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0),
                quaternary: cgmath::Vector4::new(0.0, 0.0, 0.0, 1.0)
            });

            (cell_mesh_type, cell_material)
        };

        // Create Cells:

        let mut meshes = vec![];
        let mut transforms = vec![];

        {
            let mut mesh_resources = world.write_resource::<MeshResources>();

            for x in 0..self.cells_horizontal {
                for z in 0..self.cells_vertical {
                    let object_index = mesh_resources.create_mesh(cell_mesh_type);

                    meshes.push(SolidObject {
                        mesh_type: cell_mesh_type as u32,
                        object_index: object_index as u32,
                        material: cell_material as u32
                    });

                    transforms.push(
                        Transformation {
                            position: cgmath::Point3::new(x as f32, -0.125, z as f32),
                            rotation: cgmath::Euler { x: cgmath::Deg(0.0), y: cgmath::Deg(0.0), z: cgmath::Deg(0.0) },
                            scale: cgmath::Point3::new(0.9, 0.25, 0.9)
                        }
                    );
                }
            }
        }

        for (solid_object, transformation) in meshes.into_iter().zip(transforms.into_iter()) {
            world.create_entity()
                .with(solid_object)
                .with(transformation)
                .build();
        }
    }
}
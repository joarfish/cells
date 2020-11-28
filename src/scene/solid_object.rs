use specs::prelude::*;
use specs::Component;

use crate::scene::scene_graph::Transformation;

use crate::renderer::meshes::{MeshResources, MeshType};
use crate::renderer::utils::{GpuMatrix4BGA, GpuMatrix4};
use crate::renderer::geometry::create_cube_geometry;


#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct SolidObject {
    pub mesh_type: u32,
    pub object_index: u32,
    pub material: u32
}

#[derive(Default)]
pub struct SolidObjectSystem {
    reader: Option<ReaderId<ComponentEvent>>,
}

impl SolidObjectSystem {
    pub fn new() -> Self {
        SolidObjectSystem {
            reader: None
        }
    }
}

impl<'a> System<'a> for SolidObjectSystem {
    type SystemData = (
        ReadStorage<'a, SolidObject>,
        ReadStorage<'a, Transformation>,
        WriteExpect<'a, MeshResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
    );

    fn run(&mut self, data: Self::SystemData) {

        if self.reader.is_none() {
            return;
        }

        let (
            objects,
            transformations,
            mut mesh_resources,
            device,
            queue
        ) = data;

        let events = objects
            .channel()
            .read(self.reader.as_mut().unwrap());

        // Process parenting updates:

        let mut update_transform : BitSet = BitSet::new();
        let mut removed : BitSet = BitSet::new();

        for event in events {
            match event {
                ComponentEvent::Inserted(id) => {
                    update_transform.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    update_transform.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    removed.add(*id);
                }
            }
        }

        for (object, transformation, _) in (&objects, &transformations, update_transform).join() {
            let position = transformation.position;
            let scale = transformation.scale;
            let rotation = transformation.rotation;

            let matrix = GpuMatrix4::new(
                 cgmath::Matrix4::from_translation(cgmath::Vector3::new(position.x, position.y, position.z)) *
                    cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) *
                    cgmath::Matrix4::from_angle_x(rotation.x) *
                    cgmath::Matrix4::from_angle_y(rotation.y) *
                    cgmath::Matrix4::from_angle_z(rotation.z)
            );

            let mesh_type = mesh_resources.mesh_types.get_mut(object.mesh_type as usize).unwrap();

            mesh_type.update_model_matrix(object.object_index, matrix);
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.reader = Some(
            WriteStorage::<SolidObject>::fetch(&world).register_reader()
        );
    }
}

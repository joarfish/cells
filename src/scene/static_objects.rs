use specs::prelude::*;
use specs::Component;
use wgpu::{util::*};


use crate::scene::scene_graph::Transformation;

use crate::renderer::meshes::{MeshResources, Mesh};
use crate::scene::SceneInfo;
use crate::scene::dynamic_objects::Color;
use crate::renderer::utils::{GpuMatrix4BGA, GpuVector3BGA};
use crate::scene::camera::OPENGL_TO_WGPU_MATRIX;


#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct StaticObject {
    pub mesh: Mesh
}

#[derive(Default)]
pub struct StaticObjectsSystem {
    mesh_pool: Option<u32>,
    reader: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for StaticObjectsSystem {
    type SystemData = (
        ReadStorage<'a, StaticObject>,
        ReadStorage<'a, Transformation>,
        ReadStorage<'a, Color>,
        ReadExpect<'a, MeshResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
    );

    fn run(&mut self, data: Self::SystemData) {

        if self.reader.is_none() {
            return;
        }

        let (
            static_objects,
            transformations,
            colors,
            mesh_resources,
            device,
            queue
        ) = data;

        let events = static_objects
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

        for (static_object, transformation, color, _) in (&static_objects, &transformations, &colors, update_transform).join() {
            let position = transformation.position;
            let scale = transformation.scale;
            let rotation = transformation.rotation;

            let matrix = GpuMatrix4BGA::new(
                 cgmath::Matrix4::from_translation(cgmath::Vector3::new(position.x, position.y, position.z)) *
                    cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z) *
                    cgmath::Matrix4::from_angle_x(rotation.x) *
                    cgmath::Matrix4::from_angle_y(rotation.y) *
                    cgmath::Matrix4::from_angle_z(rotation.z)
            );

            let color_vector = GpuVector3BGA::new(
                color.r,
                color.g,
                color.b
            );

            let mesh_pool = mesh_resources.mesh_pools
                .get(static_object.mesh.pool_index as usize)
                .unwrap();

            mesh_pool.update_world_matrix(&device, &queue, static_object.mesh.object_index, &matrix);
            mesh_pool.update_color(&device, &queue, static_object.mesh.object_index, &color_vector)
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        // Setup pool and create mesh:
        let mut mesh_resources = world.write_resource::<MeshResources>();
        let device = world.read_resource::<wgpu::Device>();
        let mesh_pool = mesh_resources.add_pool(&device, 50);

        let mut scene_info = world.write_resource::<SceneInfo>();
        scene_info.static_objects_pool = mesh_pool;

        self.reader = Some(
            WriteStorage::<StaticObject>::fetch(&world).register_reader()
        );
    }
}
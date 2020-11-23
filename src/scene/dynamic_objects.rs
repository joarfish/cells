use specs::prelude::*;
use specs::Component;

use crate::{renderer::{DeltaTimer, utils::GpuVector3BGA, meshes::{Mesh, MeshResources}, utils::GpuMatrix4BGA}, scene::scene_graph::Transformation};
use crate::scene::SceneInfo;
use crate::scene::camera::OPENGL_TO_WGPU_MATRIX;

pub struct TransformationTests;

impl<'a> System<'a> for TransformationTests {
    type SystemData = (
        WriteStorage<'a, Transformation>,
        ReadExpect<'a, DeltaTimer>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut transformations, delta_timer) = data;
        let d = &delta_timer.get_duration_f32();

        for transformation in (&mut transformations).join() {
            (*transformation).rotation.y += cgmath::Deg(d*5.0);
        }
    }
}


#[derive(Component)]
pub struct DynamicObject {
    pub mesh: Mesh
}

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct Color {
    pub r: f32,
    pub g: f32, 
    pub b: f32
}

#[derive(Default)]
pub struct DynamicObjectsSystem {
    mesh_pool: Option<u32>,
    color_reader: Option<ReaderId<ComponentEvent>>,
}

impl<'a> System<'a> for DynamicObjectsSystem {
    type SystemData = (
        ReadExpect<'a, MeshResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
        ReadStorage<'a, DynamicObject>,
        ReadStorage<'a, Transformation>,
        ReadStorage<'a, Color>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mesh_resources,
            device,
            queue,
            dynamic_objects,
            transformations,
            color
        ) = data;

        if let Some(mesh_pool) = self.mesh_pool {

            let events = color
                .channel()
                .read(self.color_reader.as_mut().unwrap());

            let mut buffer_update_required : BitSet = BitSet::new();

            for event in events {
                match event {
                    ComponentEvent::Inserted(id) => {
                        buffer_update_required.add(*id);
                    }
                    ComponentEvent::Modified(id) => {
                        buffer_update_required.add(*id);
                    }
                    ComponentEvent::Removed(_id) => {

                    }
                }
            }

            let mut matrices : Vec<GpuMatrix4BGA> = vec![GpuMatrix4BGA::empty(); 50];

            for (object, color, _) in (&dynamic_objects, &color, &buffer_update_required).join() {
                log::info!("Updating color buffer for obejct with index {}", object.mesh.object_index);
                mesh_resources.mesh_pools
                    .get(mesh_pool as usize)
                    .expect("Dynamic Objects Mesh Pool not available!")
                    .update_color(
                        &device,
                        &queue,
                        object.mesh.object_index,
                        &GpuVector3BGA::new(color.r, color.g, color.b)
                    );
            }

            for (object, transformation) in (&dynamic_objects, &transformations).join() {

                // Use object data to set indices properly
                let position = transformation.position;
                let scale = transformation.scale;
                let rotation = transformation.rotation;

                let matrix =
                    cgmath::Matrix4::from_translation(cgmath::Vector3::new(position.x, position.y, position.z)) *
                        cgmath::Matrix4::from_angle_x(rotation.x) *
                        cgmath::Matrix4::from_angle_y(rotation.y) *
                        cgmath::Matrix4::from_angle_z(rotation.z) *
                        cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
                    ;


                matrices[object.mesh.object_index as usize] = GpuMatrix4BGA::new(matrix);
            }

            mesh_resources.mesh_pools
                .get(mesh_pool as usize)
                .expect("Dynamic Objects Mesh Pool not available!")
                .update_world_matrices(&device, &queue, &matrices);
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        let mut mesh_resources = world.write_resource::<MeshResources>();
        let device = world.read_resource::<wgpu::Device>();

        let mesh_pool = mesh_resources.add_pool(&device, 50);
        self.mesh_pool = Some(mesh_pool);

        let mut scene_info = world.write_resource::<SceneInfo>();
        scene_info.dynamic_objects_pool = mesh_pool;

        self.color_reader = Some(
            WriteStorage::<Color>::fetch(&world).register_reader()
        );
    }
}
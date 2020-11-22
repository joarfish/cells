use specs::prelude::*;
use specs::Component;

use crate::renderer::lights::{GpuLight, LightsResources};

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct PointLight {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector3<f32>,
    pub intensity: f32,
    pub radius: f32,
    pub light_index: u32,
}

pub struct LightSystem {
    point_lights_reader: Option<ReaderId<ComponentEvent>>,
}

impl Default for LightSystem {
    fn default() -> Self {
        LightSystem {
            point_lights_reader: None
        }
    }
}

impl<'a> System<'a> for LightSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>,
        ReadExpect<'a, LightsResources>,
        ReadStorage<'a, PointLight>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            device,
            queue,
            resources,
            point_lights
        ) = data;

        let events = point_lights
            .channel()
            .read(self.point_lights_reader.as_mut().unwrap());

        // Process parenting updates:

        let mut inserted : BitSet = BitSet::new();
        let mut updated : BitSet = BitSet::new();
        let mut removed : BitSet = BitSet::new();

        for event in events {
            match event {
                ComponentEvent::Inserted(id) => {
                    inserted.add(*id);
                }
                ComponentEvent::Modified(id) => {
                    updated.add(*id);
                }
                ComponentEvent::Removed(id) => {
                    removed.add(*id);
                }
            }
        }

        for (_, point_light) in (&inserted, &point_lights).join() {
            // update buffer...
            log::info!("Adding Point Light!");
            resources.update_light(&device, &queue, point_light.light_index, GpuLight {
                position: [point_light.position.x, point_light.position.y, point_light.position.z, 1.0],
                color: [point_light.color.x, point_light.color.y, point_light.color.z, 1.0],
                intensity_radius_enabled: [ point_light.intensity, point_light.radius, 1.0, 1.0 ]
            });
        }
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.point_lights_reader = Some(
            WriteStorage::<PointLight>::fetch(&world).register_reader()
        );
    }

}
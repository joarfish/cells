use specs::prelude::*;
use specs::Component;

use super::{renderer::CompositionResources, resources::RendererResources};

use wgpu::util::*;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpuPointLight {
    position: [f32;4], // 12 bytes
    color: [f32;4], // 12 bytes
    intensity_radius_enabled: [f32; 4]
}
// total: 36bytes.. alignment needs to be 4. So 36 bytes.

unsafe impl bytemuck::Pod for GpuPointLight {}
unsafe impl bytemuck::Zeroable for GpuPointLight {}

impl Default for GpuPointLight {
    fn default() -> Self {
        GpuPointLight {
            position: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            intensity_radius_enabled: [0.125, 10.0, 0.0, 1.0]
        }
    }
}

#[derive(Component)]
#[storage(FlaggedStorage)]
pub struct PointLight {
    pub position: cgmath::Vector3<f32>,
    pub color: cgmath::Vector3<f32>,
    pub intensity: f32,
    pub radius: f32,
    pub buffer_index: u64,
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
        ReadExpect<'a, RendererResources>,
        ReadExpect<'a, CompositionResources>,
        ReadStorage<'a, PointLight>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            device,
            queue,
            resources,
            composition,
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

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        for (_, point_light) in (&inserted, &point_lights).join() {
            // update buffer...
            log::info!("Adding Point Light!");

            encoder.copy_buffer_to_buffer(
                &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[
                        GpuPointLight {
                            position: [ point_light.position.x, point_light.position.y, point_light.position.z, 1.0 ],
                            color: [ point_light.color.x, point_light.color.y, point_light.color.z, 1.0 ],
                            intensity_radius_enabled: [point_light.intensity, point_light.radius, 1.0, 1.0]
                        }
                    ]),
                    usage: wgpu::BufferUsage::COPY_SRC
                }),
                0,
                &composition.point_lights_buffer,
                point_light.buffer_index * std::mem::size_of::<GpuPointLight>() as u64,
                std::mem::size_of::<GpuPointLight>() as wgpu::BufferAddress
            );
            
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);
        self.point_lights_reader = Some(
            WriteStorage::<PointLight>::fetch(&world).register_reader()
        );
    }

}
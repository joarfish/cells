use wgpu::util::*;
use bytemuck::__core::num::NonZeroU32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpuLight {
    pub position: [f32;4],
    pub color: [f32;4],
    pub intensity_radius_enabled: [f32; 4] // gpu wants 16byte wide fields...
}

unsafe impl bytemuck::Pod for GpuLight {}
unsafe impl bytemuck::Zeroable for GpuLight {}

impl Default for GpuLight {
    fn default() -> Self {
        GpuLight {
            position: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            intensity_radius_enabled: [0.125, 10.0, 0.0, 1.0]
        }
    }
}

pub struct LightsResources {
    pub lights_bind_group_layout: wgpu::BindGroupLayout,
    pub lights_bind_group: wgpu::BindGroup,
    pub lights_buffer: wgpu::Buffer,
    free_light_indices: std::vec::Vec<u32>
}

impl LightsResources {
    pub fn new(device: &wgpu::Device) -> Self {
        let lights_data = vec![GpuLight::default(); 20];

        let lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Lights Buffer"),
            contents: bytemuck::cast_slice(&lights_data),
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM
        });

        let lights_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Point Lights Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None//wgpu::BufferSize::new((std::mem::size_of::<GpuLight>() * 20) as wgpu::BufferAddress)
                    },
                    count: None
                },
            ]
        });

        let lights_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &lights_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(lights_buffer.slice(..))
                }
            ]
        });

        LightsResources {
            lights_buffer,
            lights_bind_group_layout,
            lights_bind_group,
            free_light_indices: (0..20).collect()
        }
    }

    pub fn create_new_light(&mut self) -> Option<u32> {
        self.free_light_indices.pop()
    }

    pub fn update_light(&self, device: &wgpu::Device, queue: &wgpu::Queue, light_index: u32, light: GpuLight) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        encoder.copy_buffer_to_buffer(
            &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[
                    light
                ]),
                usage: wgpu::BufferUsage::COPY_SRC
            }),
            0,
            &self.lights_buffer,
            (light_index as u64) * std::mem::size_of::<GpuLight>() as u64,
            std::mem::size_of::<GpuLight>() as wgpu::BufferAddress
        );

        queue.submit(std::iter::once(encoder.finish()));
    }
}
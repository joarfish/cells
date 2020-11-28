use wgpu::util::DeviceExt;

#[repr(C, align(256))]
#[derive(Debug, Clone, Copy)]
pub struct GpuMaterial {
    pub(crate) primary: cgmath::Vector4<f32>, // 16 bytes
    pub(crate) secondary: cgmath::Vector4<f32>, // 16 bytes
    pub(crate) tertiary: cgmath::Vector4<f32>, // 16 bytes
    pub(crate) quaternary: cgmath::Vector4<f32>, // 16 bytes
}

unsafe impl bytemuck::Pod for GpuMaterial {}
unsafe impl bytemuck::Zeroable for GpuMaterial {}

pub struct MaterialResources {
    pub materials: Vec<GpuMaterial>,
    buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    capacity: u64
}

impl MaterialResources {
    pub fn new(device: &wgpu::Device, capacity: u64) -> MaterialResources {

        let materials = Vec::with_capacity(capacity as usize);

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Material Buffer"),
            size: capacity * std::mem::size_of::<GpuMaterial>() as u64,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
            mapped_at_creation: false
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: true,
                        min_binding_size: std::num::NonZeroU64::new(wgpu::BIND_BUFFER_ALIGNMENT)
                    },
                    count: None
                }
            ]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.slice(..))
                }
            ]
        });

        MaterialResources {
            materials,
            buffer,
            bind_group_layout,
            bind_group,
            capacity
        }
    }

    pub fn add_material(&mut self, queue: &wgpu::Queue, material: GpuMaterial) -> u64 {
        let index = self.materials.len() as u64;

        queue.write_buffer(&self.buffer, std::mem::size_of::<GpuMaterial>() as u64 * index, bytemuck::cast_slice(&[material]));
        self.materials.push(material);

        index
    }
}
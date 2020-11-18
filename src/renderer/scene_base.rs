use wgpu::util::*;

use super::{utils::GpuMatrix4};

pub struct SceneBaseResources {
    pub view_matrix_bind_group: wgpu::BindGroup,
    pub view_matrix_bind_group_layout: wgpu::BindGroupLayout,
    pub view_matrix_buffer: wgpu::Buffer
}

impl SceneBaseResources {
    pub fn new(
        device: &wgpu::Device,
    ) -> Self {

        let view_matrix_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ViewMatrixLayout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[GpuMatrix4::empty()]),
            label: Some("ViewMatrixBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let view_matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ViewMatrix"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(view_matrix_buffer.slice(0..)),
            }],
            layout: &view_matrix_bind_group_layout,
        });
        
        SceneBaseResources {
            view_matrix_bind_group,
            view_matrix_bind_group_layout,
            view_matrix_buffer
        }
    }

    pub fn update_view_matrix(&self, device: &wgpu::Device, queue: &wgpu::Queue, matrix: GpuMatrix4) {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None
            });
    
            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[matrix]),
                usage: wgpu::BufferUsage::COPY_SRC
            });
    
            encoder.copy_buffer_to_buffer(
                &staging_buffer, 0, 
                &self.view_matrix_buffer, 0,
                std::mem::size_of::<GpuMatrix4>() as wgpu::BufferAddress
            );
    
            queue.submit(std::iter::once(encoder.finish()));
    }
}

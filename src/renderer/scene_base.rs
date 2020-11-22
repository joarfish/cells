use wgpu::util::*;

use super::{utils::GpuMatrix4};
use cgmath::{Zero, SquareMatrix};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpuSceneBase {
    pub view_matrix: cgmath::Matrix4<f32>,
}

impl GpuSceneBase {
    pub fn new(matrix: cgmath::Matrix4<f32>) -> Self {
        GpuSceneBase {
            view_matrix: matrix,
        }
    }

    pub fn empty() -> Self {
        GpuSceneBase {
            view_matrix: cgmath::Matrix4::zero(),
        }
    }
}

unsafe impl bytemuck::Pod for GpuSceneBase {}
unsafe impl bytemuck::Zeroable for GpuSceneBase {}


pub struct SceneBaseResources {
    pub view_projection_matrix_bind_group: wgpu::BindGroup,
    pub view_projection_matrix_bind_group_layout: wgpu::BindGroupLayout,
    pub view_matrix_buffer: wgpu::Buffer,
    pub projection_matrix_buffer: wgpu::Buffer
}

impl SceneBaseResources {
    pub fn new(
        device: &wgpu::Device,
    ) -> Self {

        let view_projection_matrix_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ViewProjectionMatrixLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let view_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[GpuSceneBase::empty()]),
            label: Some("ViewMatrixBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let projection_matrix_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[GpuSceneBase::empty()]),
            label: Some("ProjectionMatrixBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let view_projection_matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ViewMatrix"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(view_matrix_buffer.slice(0..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(projection_matrix_buffer.slice(0..)),
                }
            ],
            layout: &view_projection_matrix_bind_group_layout,
        });
        
        SceneBaseResources {
            view_projection_matrix_bind_group,
            view_projection_matrix_bind_group_layout,
            view_matrix_buffer,
            projection_matrix_buffer
        }
    }

    pub fn update_view_matrix(&self, device: &wgpu::Device, queue: &wgpu::Queue, matrix: cgmath::Matrix4<f32>) {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None
            });
    
            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[GpuSceneBase {
                    view_matrix: matrix,
                }]),
                usage: wgpu::BufferUsage::COPY_SRC
            });
    
            encoder.copy_buffer_to_buffer(
                &staging_buffer, 0, 
                &self.view_matrix_buffer, 0,
                std::mem::size_of::<GpuSceneBase>() as wgpu::BufferAddress
            );
    
            queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update_projection_matrix(&self, device: &wgpu::Device, queue: &wgpu::Queue, matrix: cgmath::Matrix4<f32>) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[GpuSceneBase {
                view_matrix: matrix,
            }]),
            usage: wgpu::BufferUsage::COPY_SRC
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer, 0,
            &self.projection_matrix_buffer, 0,
            std::mem::size_of::<GpuSceneBase>() as wgpu::BufferAddress
        );

        queue.submit(std::iter::once(encoder.finish()));
    }
}

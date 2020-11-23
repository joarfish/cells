use wgpu::util::*;

use cgmath::{Zero};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpuSceneBase {
    pub view_matrix: cgmath::Matrix4<f32>,
    pub projection_matrix: cgmath::Matrix4<f32>,
    pub window_size: cgmath::Vector2<f32>,
}

impl GpuSceneBase {
    pub fn new(view_matrix: cgmath::Matrix4<f32>, projection_matrix: cgmath::Matrix4<f32>, window_size: cgmath::Vector2<f32>) -> Self {
        GpuSceneBase {
            view_matrix,
            projection_matrix,
            window_size
        }
    }

    pub fn empty() -> Self {
        GpuSceneBase {
            view_matrix: cgmath::Matrix4::zero(),
            projection_matrix: cgmath::Matrix4::zero(),
            window_size: cgmath::Vector2::new(0.0, 0.0)
        }
    }
}

unsafe impl bytemuck::Pod for GpuSceneBase {}
unsafe impl bytemuck::Zeroable for GpuSceneBase {}

pub struct SceneBaseResources {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub buffer: wgpu::Buffer
}

impl SceneBaseResources {
    pub fn new(
        device: &wgpu::Device,
    ) -> Self {

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ViewProjectionMatrixLayout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[GpuSceneBase::empty()]),
            label: Some("ViewMatrixBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ViewMatrix"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.slice(0..)),
                },
            ],
            layout: &bind_group_layout,
        });
        
        SceneBaseResources {
            bind_group_layout,
            bind_group,
            buffer,
        }
    }

    pub fn update_scene_base(&self, queue: &wgpu::Queue, scene_base: GpuSceneBase) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[scene_base]));
    }
}

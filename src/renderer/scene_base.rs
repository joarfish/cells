use handled_vec::MarkedHandle;
use specs::prelude::*;
use wgpu::util::*;

use super::{camera::{ActiveCamera, Camera}, resources::{BindGroupHandle, RendererResources}, utils::GpuMatrix4};

pub struct SceneBaseResources {
    pub view_matrix_bind_group: BindGroupHandle,
    pub view_matrix_bind_group_layout: MarkedHandle<wgpu::BindGroupLayout>,
    pub view_matrix_buffer: MarkedHandle<wgpu::Buffer>
}

impl SceneBaseResources {
    pub fn new(
        device: &wgpu::Device,
        resources: &mut RendererResources
    ) -> Self {

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            contents: bytemuck::cast_slice(&[GpuMatrix4::empty()]),
            label: Some("ViewMatrixBuffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ViewMatrix"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(buffer.slice(0..)),
            }],
            layout: &bind_group_layout,
        });
        
        SceneBaseResources {
            view_matrix_bind_group: resources.bind_groups.add_entry(bind_group),
            view_matrix_bind_group_layout: resources.bind_group_layouts.add_entry(bind_group_layout),
            view_matrix_buffer: resources.bind_group_buffers.add_entry(buffer)
        }
    }
}

pub struct SceneBaseSystem;

impl<'a> System<'a> for SceneBaseSystem {
    type SystemData = (
        ReadStorage<'a, Camera>,
        ReadExpect<'a, ActiveCamera>,
        ReadExpect<'a, RendererResources>,
        ReadExpect<'a, SceneBaseResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (cameras,
            active_camera_entity,
            resources,
            scene_base,
            device,
            queue
        ) = data;
        
        if let Some(active_camera) = cameras.get(active_camera_entity.0) {
            let updated_matrix = GpuMatrix4::new(active_camera.build_view_projection_matrix());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None
            });
    
            let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[updated_matrix]),
                usage: wgpu::BufferUsage::COPY_SRC
            });
    
            let view_matrix_buffer = resources.bind_group_buffers
                .get(&scene_base.view_matrix_buffer)
                .expect("Could not retrieve ViewMatrix Uniform Buffer!");
    
            encoder.copy_buffer_to_buffer(
                &staging_buffer, 0, 
                &view_matrix_buffer, 0,
                std::mem::size_of::<GpuMatrix4>() as wgpu::BufferAddress
            );
    
            queue.submit(std::iter::once(encoder.finish()));
        }
        
    }
}

use super::{geometry::Geometry, utils::{GpuMatrix4BGA, GpuVector3BGA}};
use wgpu::util::*;

pub struct MeshPool {
    pub world_matrix_buffer: wgpu::Buffer,
    pub color_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    free_indices: std::vec::Vec<u32>,
    capacity: u32
}

#[derive(Copy, Clone)]
pub struct Mesh {
    pub pool_index: u16,
    pub object_index: u32,
    pub geometry_index: u32
}

impl MeshPool {

    fn new(device: &wgpu::Device, capacity: u32, bind_group_layout: &wgpu::BindGroupLayout, pool_index: u32) -> Self {

        log::info!("Creating Mesh pool #{}", pool_index);

        let world_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("MeshPool#{} WorldMatrixBuffer", pool_index)),
            size: (capacity as u64) * std::mem::size_of::<GpuMatrix4BGA>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
            //contents: bytemuck::cast_slice( &vec![GpuMatrix4BGA::empty(); capacity as usize] )
            mapped_at_creation: false
        });

        let color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("MeshPool#{} ColorBuffer", pool_index)),
            size: (capacity as u64) * std::mem::size_of::<GpuVector3BGA>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::UNIFORM,
            mapped_at_creation: false
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("MeshPool#{} BindGroup", pool_index)),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(world_matrix_buffer.slice(0..(std::mem::size_of::<GpuMatrix4BGA>() as u64)))
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(color_buffer.slice(0..(std::mem::size_of::<GpuMatrix4BGA>() as u64))),
                },
            ]
        });

        MeshPool {
            free_indices: (0..capacity).rev().collect(),
            world_matrix_buffer,
            color_buffer,
            bind_group,
            capacity
        }
    }

    pub fn update_world_matrix(&self, device: &wgpu::Device, queue: &wgpu::Queue, object_index: u32, matrix: &GpuMatrix4BGA) {

        log::info!("Updating world matrix for object_index={}", object_index);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[*matrix]),
            usage: wgpu::BufferUsage::COPY_SRC
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.world_matrix_buffer,
            (object_index as u64) * std::mem::size_of::<GpuMatrix4BGA>() as u64,
            std::mem::size_of::<GpuVector3BGA>() as u64
        );

        queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn update_world_matrices(&self, device: &wgpu::Device, queue: &wgpu::Queue, matrices: &[GpuMatrix4BGA]) {

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(matrices),
            usage: wgpu::BufferUsage::COPY_SRC
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer, 
            0, 
            &self.world_matrix_buffer, 
            0, 
            (self.capacity as u64) * (std::mem::size_of::<GpuMatrix4BGA>() as u64)
        );

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update_color(&self, device: &wgpu::Device, queue: &wgpu::Queue, color_index: u32, color: &GpuVector3BGA) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[*color]),
            usage: wgpu::BufferUsage::COPY_SRC
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer, 
            0, 
            &self.color_buffer, 
            (color_index as u64) * std::mem::size_of::<GpuVector3BGA>() as u64,
            std::mem::size_of::<GpuVector3BGA>() as u64
        );

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn get_free_index(&mut self) -> u32 {
        self.free_indices.pop().unwrap()
    }
}

pub struct GpuGeometry {
    pub positions_buffer: wgpu::Buffer,
    pub normals_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct MeshResources {
    pub geometries: std::vec::Vec<GpuGeometry>,
    pub buffer_bind_group_layout: wgpu::BindGroupLayout,
    pub index_push_constant_range: wgpu::PushConstantRange,
    pub mesh_pools: Vec<MeshPool>
}

impl MeshResources {
    pub fn init(device: &wgpu::Device) -> Self {

        let index_push_constant_range = wgpu::PushConstantRange {
            stages: wgpu::ShaderStage::FRAGMENT | wgpu::ShaderStage::VERTEX,
            range: 0..4 // 4 bytes uint32
        };

        let buffer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: true,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GpuMatrix4BGA>() as u64)
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: true,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GpuVector3BGA>() as u64)
                    },
                    count: None
                }
            ]
        });

        MeshResources {
            geometries: vec![],
            mesh_pools: vec![],
            index_push_constant_range,
            buffer_bind_group_layout
        }
    }

    pub fn add_pool(&mut self, device: &wgpu::Device, capacity: u32) -> u32 {
        let pool_index = self.mesh_pools.len();

        self.mesh_pools.push(
            MeshPool::new(device, capacity, &self.buffer_bind_group_layout, pool_index as u32)
        );

        (self.mesh_pools.len() - 1) as u32
    }

    pub fn create_mesh(&mut self, geometry_index: u32, pool_index: u16) -> Mesh {
        Mesh {
            pool_index,
            geometry_index,
            object_index: self.mesh_pools.get_mut(pool_index as usize).unwrap().get_free_index()
        }
    }

    pub fn add_geometry(&mut self, device: &wgpu::Device, geometry: &Geometry) -> usize {
        let positions_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.vertices),
            usage: wgpu::BufferUsage::VERTEX
        });

        let normals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.normals),
            usage: wgpu::BufferUsage::VERTEX
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&geometry.indices),
            usage: wgpu::BufferUsage::INDEX
        });

        self.geometries.push(GpuGeometry {
            positions_buffer,
            normals_buffer,
            index_buffer,
            index_count: geometry.indices.len() as u32
        });

        self.geometries.len() - 1
    }
}
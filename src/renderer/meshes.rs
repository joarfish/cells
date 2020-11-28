use super::{geometry::Geometry};
use wgpu::util::*;
use crate::renderer::utils::{GpuMatrix4, GpuVector3};

pub struct GpuGeometry {
    pub positions_buffer: wgpu::Buffer,
    pub normals_buffer: wgpu::Buffer,
    pub parts_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct MeshType {
    name: String,
    pub gpu_geometry: GpuGeometry,
    geometry: Geometry,
    pub model_matrix_buffer: wgpu::Buffer,
    pub model_matrices: Vec<GpuMatrix4>,
    free_indices: Vec<usize>,
    capacity: usize
}

impl MeshType {
    pub fn new(device: &wgpu::Device, name: &str, capacity: usize, geometry: Geometry) -> Self {

        let model_matrices = vec![GpuMatrix4::empty(); capacity];

        let model_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("ModelMatrixBuffer: {}", name)),
            size: (capacity * std::mem::size_of::<GpuMatrix4>()) as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false
        });

        let gpu_geometry = {

            let positions_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Position Buffer"),
                contents: bytemuck::cast_slice(&geometry.vertices),
                usage: wgpu::BufferUsage::VERTEX
            });

            let normals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Normal Buffer"),
                contents: bytemuck::cast_slice(&geometry.normals),
                usage: wgpu::BufferUsage::VERTEX
            });

            let parts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Parts Buffer"),
                contents: bytemuck::cast_slice(&geometry.part_ids),
                usage: wgpu::BufferUsage::VERTEX
            });

            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&geometry.indices),
                usage: wgpu::BufferUsage::INDEX
            });

            GpuGeometry {
                positions_buffer,
                normals_buffer,
                parts_buffer,
                index_buffer,
                index_count: geometry.indices.len() as u32
            }
        };

        MeshType {
            name: name.to_string(),
            free_indices: (0..capacity).rev().collect(),
            model_matrices,
            model_matrix_buffer,
            geometry,
            gpu_geometry,
            capacity
        }
    }

    pub fn create_mesh(&mut self) -> usize {
        let index = self.free_indices.pop().unwrap(); // for now, we just panic if max capacity is reached

        if let Some(matrix) = self.model_matrices.get_mut(index) {
            *matrix = GpuMatrix4::empty();
        } else {
            self.model_matrices.push(GpuMatrix4::empty());
        }

        index
    }

    pub fn prepare_instances(&self, queue: &wgpu::Queue, instance_indices: &[u32]) {
        let mut matrices_to_copy = Vec::with_capacity(instance_indices.len());

        for index in instance_indices {
            if let Some(matrix) = self.model_matrices.get(*index as usize) {
                matrices_to_copy.push(*matrix);
            }
        }

        let data = bytemuck::cast_slice(&matrices_to_copy);
        queue.write_buffer(&self.model_matrix_buffer, 0, data);
    }

    pub fn update_model_matrix(&mut self, object_index: u32, matrix: GpuMatrix4) {
        *self.model_matrices.get_mut(object_index as usize).unwrap() = matrix;
    }
}


pub struct MeshResources {
    pub mesh_types: Vec<MeshType>,
}

impl MeshResources {
    pub fn new() -> Self {

        MeshResources {
            mesh_types: Vec::new(),
        }
    }

    pub fn add_mesh_type(&mut self, mesh_type: MeshType) -> usize {
        self.mesh_types.push(mesh_type);

        self.mesh_types.len() - 1
    }

    pub fn create_mesh(&mut self, mesh_type_index: usize) -> usize {
        let mut mesh_type = self.mesh_types.get_mut(mesh_type_index).unwrap();

        mesh_type.create_mesh()
    }
}
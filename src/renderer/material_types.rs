use super::static_mesh::StaticMeshMaterial;
use wgpu::RenderPipeline;

pub trait Material {
    fn get_pipeline(&self) -> &wgpu::RenderPipeline;
    fn get_uniform_layout(&self) -> &wgpu::BindGroupLayout;
}

pub struct MaterialTypes {
    static_mesh : StaticMeshMaterial,
}

pub enum RenderObjectType {
    StaticMesh
}

impl MaterialTypes {

    pub fn new(device : &wgpu::Device, format : wgpu::TextureFormat) -> Self {
        MaterialTypes {
            static_mesh: StaticMeshMaterial::new(device, format)
        }
    }

    pub fn get_pipeline(&self, object_type: RenderObjectType) -> &RenderPipeline {
        match object_type {
            RenderObjectType::StaticMesh => {
                self.static_mesh.get_pipeline()
            }
        }
    }

    pub fn get_uniform_layout(&self, object_type: RenderObjectType) -> &wgpu::BindGroupLayout {
        match object_type {
            RenderObjectType::StaticMesh => {
                self.static_mesh.get_uniform_layout()
            }
        }
    }
}
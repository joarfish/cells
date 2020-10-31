use super::static_mesh::{ MaterialBuilder, Material };
use wgpu::RenderPipeline;

pub trait MaterialT {
    fn get_pipeline(&self) -> &wgpu::RenderPipeline;
    fn get_uniform_layout(&self) -> &wgpu::BindGroupLayout;
}

pub struct MaterialTypes {
    static_mesh : Material,
}

pub enum RenderObjectType {
    StaticMesh
}

impl MaterialTypes {

    pub fn new(device : &wgpu::Device, format : wgpu::TextureFormat) -> Self {
        MaterialTypes {
            static_mesh: MaterialBuilder::new().build(device, format)
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
use super::static_mesh::StaticMesh;

pub trait RenderObjectType {
    fn get_pipeline(&self) -> &wgpu::RenderPipeline;
}

pub enum RenderObject {
    StaticMesh(StaticMesh),
}

impl RenderObject {
    pub fn get_pipeline(&self) -> &wgpu::RenderPipeline {
        match self {
            RenderObject::StaticMesh(o) => {
                o.get_pipeline()
            }
        }
    }
}
use handled_vec::{HandledStdVec, MarkedHandle};

pub type BindGroupHandle = MarkedHandle<wgpu::BindGroup>;
pub type BindGroupLayoutHandle = MarkedHandle<wgpu::BindGroupLayout>;
pub type BindGroupBufferHandle = MarkedHandle<wgpu::Buffer>;
pub type VertexBufferHandle = MarkedHandle<wgpu::Buffer>;
pub type IndexBufferHandle = MarkedHandle<wgpu::Buffer>;
pub type BufferHandle = MarkedHandle<wgpu::Buffer>;
pub type PipelineHandle = MarkedHandle<wgpu::RenderPipeline>;

pub struct BufferSlice {
    pub buffer: BufferHandle,
    pub range: std::ops::Range<u64>
}

pub struct RenderObject {
    pub bind_groups: Vec<BindGroupHandle>, 
    pub pipeline: PipelineHandle,
    pub vertex_buffers: Vec<BufferSlice>,
    pub indices: BufferSlice,
    pub index_count: u32
}


pub type RenderObjectHandle = MarkedHandle<RenderObject>;

pub struct RendererResources {
    pub bind_groups: HandledStdVec<wgpu::BindGroup>,
    pub bind_group_layouts: HandledStdVec<wgpu::BindGroupLayout>,
    pub bind_group_buffers: HandledStdVec<wgpu::Buffer>,
    pub vertex_buffers: HandledStdVec<wgpu::Buffer>,
    pub index_buffers: HandledStdVec<wgpu::Buffer>,
    pub render_pipelines: HandledStdVec<wgpu::RenderPipeline>,
    pub render_objects: HandledStdVec<RenderObject>
}

impl RendererResources {
    pub fn new() -> Self {
        RendererResources {
            bind_groups: HandledStdVec::new(),
            bind_group_layouts: HandledStdVec::new(),
            bind_group_buffers: HandledStdVec::new(),
            vertex_buffers: HandledStdVec::new(),
            index_buffers: HandledStdVec::new(),
            render_pipelines: HandledStdVec::new(),
            render_objects: HandledStdVec::new()
        }
    }
}

impl Default for RendererResources {
    fn default() -> Self {
        Self::new()
    }
}
use std::time::{ Instant, Duration };

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Vertex {
            position,
            color
        }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3
                }
            ],
            step_mode: wgpu::InputStepMode::Vertex,
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress
        }
    }
}


pub struct DeltaTimer {
    d: Duration,
    last_render: Instant,
}

impl DeltaTimer {

    pub fn new(d : Duration, last_render : Instant) -> Self {
        DeltaTimer {
            d,
            last_render
        }
    }

    pub fn get_duration_f32(&self) -> f32 {
        self.d.as_secs_f32()
    }

    pub fn get_last_render(&self) -> Instant {
        self.last_render
    }
}
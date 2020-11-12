use cgmath::Zero;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpuMatrix4 {
    pub matrix: cgmath::Matrix4<f32>
}

impl GpuMatrix4 {
    pub fn new(matrix: cgmath::Matrix4<f32>) -> Self {
        GpuMatrix4 {
            matrix
        }
    }

    pub fn empty() -> Self {
        GpuMatrix4 {
            matrix: cgmath::Matrix4::zero()
        }
    }
}

unsafe impl bytemuck::Pod for GpuMatrix4 {}
unsafe impl bytemuck::Zeroable for GpuMatrix4 {}

#[repr(C, align(256))]
#[derive(Debug, Copy, Clone)]
pub struct GpuMatrix4BGA {
    pub matrix: cgmath::Matrix4<f32>
}

impl GpuMatrix4BGA {
    pub fn new(matrix: cgmath::Matrix4<f32>) -> Self {
        GpuMatrix4BGA {
            matrix
        }
    }

    pub fn empty() -> Self {
        GpuMatrix4BGA {
            matrix: cgmath::Matrix4::zero()
        }
    }
}

unsafe impl bytemuck::Pod for GpuMatrix4BGA {}
unsafe impl bytemuck::Zeroable for GpuMatrix4BGA {}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpuVector3 {
    pub vector: cgmath::Vector3<f32>
}

impl GpuVector3 {
    pub fn empty() -> Self {
        GpuVector3 {
            vector: cgmath::Vector3::new(0.0, 0.0, 0.0)
        }
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        GpuVector3 {
            vector: cgmath::Vector3::new(x, y, z)
        }
    }
}

unsafe impl bytemuck::Pod for GpuVector3 {}
unsafe impl bytemuck::Zeroable for GpuVector3 {} 


#[repr(C, align(256))]
#[derive(Debug, Copy, Clone)]
pub struct GpuVector3BGA {
    pub vector: cgmath::Vector3<f32>
}

impl GpuVector3BGA {
    pub fn empty() -> Self {
        GpuVector3BGA {
            vector: cgmath::Vector3::new(0.0, 0.0, 0.0)
        }
    }

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        GpuVector3BGA {
            vector: cgmath::Vector3::new(x, y, z)
        }
    }
}

unsafe impl bytemuck::Pod for GpuVector3BGA {}
unsafe impl bytemuck::Zeroable for GpuVector3BGA {} 
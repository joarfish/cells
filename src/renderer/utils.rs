use cgmath::{Zero, ElementWise};

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

fn max3(a: f32, b: f32, c: f32) -> f32 {
    if a > b {
        if a > c {
            a
        } else {
            c
        }
    } else {
        if b > c {
            b
        } else {
            c
        }
    }
}

fn min3(a: f32, b: f32, c: f32) -> f32 {
    if a < b {
        if a < c {
            a
        } else {
            c
        }
    } else {
        if b < c {
            b
        } else {
            c
        }
    }
}

pub struct AABB {
    pub min: cgmath::Point3<f32>,
    pub max: cgmath::Point3<f32>,
}

impl AABB {
    pub fn new(min: cgmath::Point3<f32>, max: cgmath::Point3<f32>) -> Self {
        AABB {
            min,
            max
        }
    }

    pub fn shortest_distance(&self, point: cgmath::Point3<f32>) -> f32 {
        let dx = max3(self.min.x - point.x, 0.0, point.x - self.max.x);
        let dy = max3(self.min.y - point.y, 0.0, point.y - self.max.y);
        let dz = max3(self.min.z - point.z, 0.0, point.z - self.max.z);

        (dx*dx + dy*dy + dz*dz).sqrt()
    }

    pub fn farthest_distance(&self, point: cgmath::Point3<f32>) -> f32 {
        let dx = min3(self.min.x - point.x, 0.0, point.x - self.max.x);
        let dy = min3(self.min.y - point.y, 0.0, point.y - self.max.y);
        let dz = min3(self.min.z - point.z, 0.0, point.z - self.max.z);

        (dx*dx + dy*dy + dz*dz).sqrt()
    }
}
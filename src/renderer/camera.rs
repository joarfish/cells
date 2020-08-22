use crate::DeltaTimer;
use cgmath::prelude::*;
use specs::prelude::*;
use specs::{Component, ReadStorage, System, VecStorage, WriteStorage};
use winit::dpi::PhysicalSize;

#[derive(Component)]
#[storage(VecStorage)]
pub struct Camera {
    position: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        Camera {
            position: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: aspect_ratio,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(self.position, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        super::utils::OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

pub struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        WriteStorage<'a, Camera>,
        ReadStorage<'a, ActiveCamera>,
        ReadExpect<'a, DeltaTimer>,
    );

    fn run(&mut self, (mut cameras, active_cameras, delta_timer): Self::SystemData) {
        for (camera, _) in (&mut cameras, &active_cameras).join() {
            camera.position = camera.position.add_element_wise(cgmath::Point3::new(
                0.0,
                0.0,
                0.1 * delta_timer.get_duration_f32(),
            ));
        }
    }
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct ActiveCamera;

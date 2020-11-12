use super::DeltaTimer;
use crate::input::{ KeyState, InputMap };
use cgmath::prelude::*;
use specs::prelude::*;
use specs::{Component, System, VecStorage, WriteStorage};
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
            position: (0.0, 0.0, 3.0).into(),
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
        ReadExpect<'a, ActiveCamera>,
        ReadExpect<'a, DeltaTimer>,
        ReadExpect<'a, InputMap>
    );

    fn run(&mut self, (mut cameras, active_camera, delta_timer, input_map): Self::SystemData) {

        let d = delta_timer.get_duration_f32();
        let speed = 0.75;

        let d_position = cgmath::Point3::new(
            speed * d * match input_map.key_d { KeyState::Pressed => 1.0, _ => 0.0 } + 
            speed * d * match input_map.key_a { KeyState::Pressed => -1.0, _ => 0.0 },
            speed * d * match input_map.key_w { KeyState::Pressed => 1.0, _ => 0.0 } + 
            speed * d * match input_map.key_s { KeyState::Pressed => -1.0, _ => 0.0 },
            0.0
        );

        if let Some(camera) = cameras.get_mut((*active_camera).0) {
            camera.position = camera.position.add_element_wise(d_position);
            camera.target = camera.target.add_element_wise(d_position);
        }
    }
}

pub struct ActiveCamera(pub Entity);



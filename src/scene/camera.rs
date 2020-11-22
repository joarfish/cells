use crate::renderer::{DeltaTimer, utils::GpuMatrix4, scene_base::SceneBaseResources};
use crate::input::{ KeyState, InputMap };
use cgmath::prelude::*;
use specs::prelude::*;
use specs::{Component, System, VecStorage, WriteStorage};
use winit::dpi::PhysicalSize;

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

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
            position: (0.0, 10.0, 8.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_z(),
            aspect: aspect_ratio,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect = size.width as f32 / size.height as f32;
    }

    pub fn build_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        //let view = cgmath::Matrix4::look_at(self.position, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * proj
    }

    pub fn build_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(self.position, self.target, self.up);

        view
    }
}

pub struct CameraSystem;

impl<'a> System<'a> for CameraSystem {
    type SystemData = (
        WriteStorage<'a, Camera>,
        ReadExpect<'a, ActiveCamera>,
        ReadExpect<'a, DeltaTimer>,
        ReadExpect<'a, InputMap>,
        ReadExpect<'a, SceneBaseResources>,
        ReadExpect<'a, wgpu::Device>,
        ReadExpect<'a, wgpu::Queue>
    );

    fn run(&mut self, data: Self::SystemData) {

        let (
            mut cameras,
            active_camera,
            delta_timer,
            input_map,
            scene_base_resources,
            device,
            queue
        ) = data;

        let d = delta_timer.get_duration_f32();
        let speed = 1.25;
        let zoom_speed = 10.0;

        let d_position = cgmath::Point3::new(
            speed * d * match input_map.key_d { KeyState::Pressed => -1.0, _ => 0.0 } + 
            speed * d * match input_map.key_a { KeyState::Pressed => 1.0, _ => 0.0 },
            input_map.wheel * zoom_speed * d,
            speed * d * match input_map.key_w { KeyState::Pressed => -1.0, _ => 0.0 } + 
            speed * d * match input_map.key_s { KeyState::Pressed => 1.0, _ => 0.0 }
        );

        if let Some(camera) = cameras.get_mut((*active_camera).0) {
            camera.position = camera.position.add_element_wise(d_position);
            camera.target = camera.target.add_element_wise(d_position);

            let updated_view_matrix = camera.build_view_matrix();
            let updated_projection_matrix = camera.build_projection_matrix();

            scene_base_resources.update_view_matrix(&device, &queue, updated_view_matrix);
            scene_base_resources.update_projection_matrix(&device, &queue, updated_projection_matrix);
        }
    }
}

pub struct ActiveCamera(pub Entity);



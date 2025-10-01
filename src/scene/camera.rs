use crate::renderer::{DeltaTimer, utils::GpuMatrix4, scene_base::SceneBaseResources};
use crate::input::{ KeyState, InputMap };
use cgmath::prelude::*;
use specs::prelude::*;
use specs::{Component, System, VecStorage, WriteStorage};
use winit::dpi::PhysicalSize;
use crate::renderer::scene_base::GpuSceneBase;
use crate::scene::scene_graph::SceneResources;

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
            position: (-8.0, 10.0, 8.0).into(),
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

    pub fn build_projection_matrix(&self) -> cgmath::Matrix4<f32> {
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
        ReadExpect<'a, SceneResources>,
        ReadExpect<'a, wgpu::Queue>
    );

    fn run(&mut self, data: Self::SystemData) {

        let (
            mut cameras,
            active_camera,
            delta_timer,
            input_map,
            scene_base_resources,
            scene_resources,
            queue
        ) = data;

        let d = delta_timer.get_duration_f32();
        let speed = 4.5;
        let zoom_speed = 10.0;

        if let Some(camera) = cameras.get_mut((*active_camera).0) {

            let d_left = match input_map.key_d { KeyState::Pressed => -1.0, _ => 0.0 } +
                speed * d * match input_map.key_a { KeyState::Pressed => 1.0, _ => 0.0 };
            let d_front = match input_map.key_w { KeyState::Pressed => 1.0, _ => 0.0 } +
                speed * d * match input_map.key_s { KeyState::Pressed => -1.0, _ => 0.0 };

            let front = camera.target.sub_element_wise(camera.position)
                .to_vec()
                .mul_element_wise(
                    cgmath::Vector3::new(1.0, 0.0, 1.0) // zero y axis because we want to stay on one plane
                )
                .normalize();
            let left = front.cross(camera.up).normalize();

            let d_position = front
                .mul_element_wise(d_front)
                .add_element_wise(
                    left.mul_element_wise(d_left)
                )
                .add_element_wise(
                    cgmath::Vector3::new(0.0, input_map.wheel * zoom_speed * d, 0.0)
                )
                .normalize()
                .mul_element_wise(speed * d);

            if d_position.is_finite() {
                camera.position = camera.position.add_element_wise(cgmath::Point3::new(d_position.x, d_position.y, d_position.z));
                camera.target = camera.target.add_element_wise(cgmath::Point3::new(d_position.x, d_position.y, d_position.z));
            }

            let near = scene_resources.extend.shortest_distance(camera.position);
            let far = scene_resources.extend.farthest_distance(camera.position);
            camera.znear = near;
                camera.zfar = far;

                let updated_view_matrix = camera.build_view_matrix();
                let updated_projection_matrix = camera.build_projection_matrix();

                scene_base_resources.update_scene_base(&queue, GpuSceneBase {
                    view_matrix: updated_view_matrix,
                    projection_matrix: updated_projection_matrix,
                    window_size: cgmath::Vector2::new(1024.0, 768.0),
                    padding: cgmath::Vector2::new(0.0, 0.0)
                });
        }
    }
}

pub struct ActiveCamera(pub Entity);



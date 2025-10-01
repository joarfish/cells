use crate::renderer::DeltaTimer;
use specs::prelude::*;
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(PartialEq)]
pub enum KeyState {
    Pressed,
    NotPressed,
}

pub struct InputMap {
    pub key_w: KeyState,
    pub key_s: KeyState,
    pub key_a: KeyState,
    pub key_d: KeyState,
    pub key_p: KeyState,
    pub wheel: f32,
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            key_w: KeyState::NotPressed,
            key_s: KeyState::NotPressed,
            key_a: KeyState::NotPressed,
            key_d: KeyState::NotPressed,
            key_p: KeyState::NotPressed,
            wheel: 0.0,
        }
    }

    pub fn update(&mut self, key_code: PhysicalKey, key_state: ElementState) {
        match key_code {
            PhysicalKey::Code(KeyCode::KeyW) => {
                self.key_w = match key_state {
                    ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed,
                }
            }
            PhysicalKey::Code(KeyCode::KeyS) => {
                self.key_s = match key_state {
                    ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed,
                }
            }
            PhysicalKey::Code(KeyCode::KeyA) => {
                self.key_a = match key_state {
                    ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed,
                }
            }
            PhysicalKey::Code(KeyCode::KeyD) => {
                self.key_d = match key_state {
                    ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed,
                }
            }
            PhysicalKey::Code(KeyCode::KeyP) => {
                self.key_p = match key_state {
                    ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed,
                }
            }
            _ => (),
        }
    }

    pub fn update_mouse_wheel(&mut self, delta: f32) {
        self.wheel = delta;
    }
}

pub struct InputSystem;

impl<'a> System<'a> for InputSystem {
    type SystemData = (WriteExpect<'a, InputMap>, ReadExpect<'a, DeltaTimer>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut input_map, timer) = data;
        let d = timer.get_duration_f32();
        let abs_wheel = input_map.wheel.abs();
        let wheel_dampening = cubic_bezier(0.43, 0.75, 0.59, 1.0, 1.0 - abs_wheel) * d * 15.0; // per sec?
        input_map.wheel *= wheel_dampening;
    }
}

fn cubic_bezier(b0: f32, b1: f32, b2: f32, b3: f32, t: f32) -> f32 {
    ((-1.0) * b0 + 3.0 * b1 - 3.0 * b2 + b3) * t * t * t
        + (3.0 * b0 - 6.0 * b1 + 3.0 * b2) * t * t
        + (-3.0 * b0 + 3.0 * b1) * t
        + b0
}

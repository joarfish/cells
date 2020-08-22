use specs::prelude::*;

pub enum KeyState {
    Pressed,
    NotPressed,
}

pub struct InputMap {
    key_w: KeyState,
    key_s: KeyState,
    key_a: KeyState,
    key_d: KeyState,
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            key_w: KeyState::Pressed,
            key_s: KeyState::Pressed,
            key_a: KeyState::Pressed,
            key_d: KeyState::Pressed,
        }
    }
}

pub struct InputSystem {}

impl<'a> System<'a> for InputSystem {}

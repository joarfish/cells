use specs::prelude::*;

pub enum KeyState {
    Pressed,
    NotPressed,
}

pub struct InputMap {
    pub key_w: KeyState,
    pub key_s: KeyState,
    pub key_a: KeyState,
    pub key_d: KeyState,
}

impl InputMap {
    pub fn new() -> Self {
        InputMap {
            key_w: KeyState::NotPressed,
            key_s: KeyState::NotPressed,
            key_a: KeyState::NotPressed,
            key_d: KeyState::NotPressed,
        }
    }

    pub fn update(&mut self, key_code: winit::event::VirtualKeyCode, key_state: winit::event::ElementState) {
        match key_code {
            winit::event::VirtualKeyCode::W => {
                self.key_w = match key_state {
                    winit::event::ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed
                }
            },
            winit::event::VirtualKeyCode::S => {
                self.key_s = match key_state {
                    winit::event::ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed
                }
            },
            winit::event::VirtualKeyCode::A => {
                self.key_a = match key_state {
                    winit::event::ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed
                }
            },
            winit::event::VirtualKeyCode::D => {
                self.key_d = match key_state {
                    winit::event::ElementState::Pressed => KeyState::Pressed,
                    _ => KeyState::NotPressed
                }
            },
            _ => ()
        }
    }
}

pub struct InputSystem {}

impl<'a> System<'a> for InputSystem {
    type SystemData = ReadExpect<'a, InputMap>;

    fn run(&mut self, _input_map: Self::SystemData) {

    }
}

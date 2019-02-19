use std::collections::HashMap;

use glium;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Action {
    RotateLeft,
    RotateRight,
    Up,
    Down,
    Left,
    Right,
    Space,
    Enter,
    Quit,
    Back,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeyState {
    Pressed,
    Released,
}

pub struct Inputs {
    pub keys: HashMap<Action, KeyState>,
}

impl Inputs {
    pub fn new() -> Inputs {
        let mut keys = HashMap::new();
        keys.insert(Action::Up, KeyState::Released);
        keys.insert(Action::Down, KeyState::Released);
        keys.insert(Action::Left, KeyState::Released);
        keys.insert(Action::Right, KeyState::Released);
        keys.insert(Action::RotateLeft, KeyState::Released);
        keys.insert(Action::RotateRight, KeyState::Released);
        keys.insert(Action::Enter, KeyState::Released);
        keys.insert(Action::Space, KeyState::Released);
        keys.insert(Action::Back, KeyState::Released);
        keys.insert(Action::Quit, KeyState::Released);

        Inputs {
            keys: keys,
        }
    }

    pub fn get(&mut self, key: glium::glutin::VirtualKeyCode) -> Option<(Action, &mut KeyState)> {
        match key {
            glium::glutin::VirtualKeyCode::W => Some((Action::Up, self.keys.get_mut(&Action::Up).unwrap())),
            glium::glutin::VirtualKeyCode::S => Some((Action::Down, self.keys.get_mut(&Action::Down).unwrap())),
            glium::glutin::VirtualKeyCode::A => Some((Action::Left, self.keys.get_mut(&Action::Left).unwrap())),
            glium::glutin::VirtualKeyCode::D => Some((Action::Right, self.keys.get_mut(&Action::Right).unwrap())),
            glium::glutin::VirtualKeyCode::Q => Some((Action::RotateLeft, self.keys.get_mut(&Action::RotateLeft).unwrap())),
            glium::glutin::VirtualKeyCode::E => Some((Action::RotateRight, self.keys.get_mut(&Action::RotateRight).unwrap())),
            glium::glutin::VirtualKeyCode::Space => Some((Action::Space, self.keys.get_mut(&Action::Space).unwrap())),
            glium::glutin::VirtualKeyCode::Escape => Some((Action::Quit, self.keys.get_mut(&Action::Quit).unwrap())),
            glium::glutin::VirtualKeyCode::X => Some((Action::Back, self.keys.get_mut(&Action::Back).unwrap())),
            glium::glutin::VirtualKeyCode::Return => Some((Action::Enter, self.keys.get_mut(&Action::Enter).unwrap())),
            _ => None,
        }
    }

    pub fn release_keys(&mut self) {
        for (_, val) in self.keys.iter_mut() {
            *val = KeyState::Released;
        }
    }

    pub fn update(&mut self, key: glium::glutin::VirtualKeyCode, new_state: glium::glutin::ElementState) {
        let key = self.get(key);
        if key.is_some() {
            let key = key.unwrap();

            if *key.1 == KeyState::Pressed {
                if new_state == glium::glutin::ElementState::Released {
                    *key.1 = KeyState::Released;
                }
            } else {
                if new_state == glium::glutin::ElementState::Pressed {
                    *key.1 = KeyState::Pressed;
                }
            }
        }
    }

    pub fn has_update(&self) -> bool {
        for key in self.keys.iter() {
            if *key.1 == KeyState::Pressed {
                return true;
            }
        }
        return false;
    }
}

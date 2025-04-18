use emath::{Pos2, Vec2};

#[derive(Clone)]
pub enum InputEvent<Key, MouseButton> {
    Key { key: Key, pressed: bool },
    MouseMotion { position: Pos2 },
    RawMouseMotion { delta: Vec2 },
    MouseButton { button: MouseButton, pressed: bool },
}

#[derive(Default, Clone)]
pub struct Trigger {
    pressed: bool,
    just_pressed: bool,
    just_released: bool,
}

impl Trigger {
    pub fn new() -> Self {
        Trigger::default()
    }
    pub fn pressed(&self) -> bool {
        self.pressed
    }
    pub fn just_pressed(&self) -> bool {
        self.just_pressed
    }
    pub fn just_released(&self) -> bool {
        self.just_released
    }
    pub fn set_pressed(&mut self, pressed: bool) {
        if pressed != self.pressed {
            self.pressed = pressed;
            if pressed {
                self.just_pressed = true;
            } else {
                self.just_released = true;
            }
        }
    }
    pub fn update(&mut self) {
        self.just_pressed = false;
        self.just_released = false;
    }
}

//! # Mouse Input

use glam::Vec2;

/// Mouse button
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

/// Mouse state
#[derive(Default)]
pub struct MouseState {
    pub pressed: [bool; 5],
    pub just_pressed: [bool; 5],
    pub just_released: [bool; 5],
}

/// Mouse tracker
pub struct Mouse {
    pub position: Vec2,
    pub delta: Vec2,
    pub scroll: f32,
    buttons: MouseState,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
            scroll: 0.0,
            buttons: MouseState::default(),
        }
    }

    pub fn update(&mut self) {
        self.delta = Vec2::ZERO;
        self.scroll = 0.0;
        self.buttons.just_pressed = [false; 5];
        self.buttons.just_released = [false; 5];
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        let new_pos = Vec2::new(x, y);
        self.delta = new_pos - self.position;
        self.position = new_pos;
    }

    pub fn button_down(&mut self, button: MouseButton) {
        let idx = button_index(button);
        if !self.buttons.pressed[idx] {
            self.buttons.just_pressed[idx] = true;
        }
        self.buttons.pressed[idx] = true;
    }

    pub fn button_up(&mut self, button: MouseButton) {
        let idx = button_index(button);
        self.buttons.pressed[idx] = false;
        self.buttons.just_released[idx] = true;
    }

    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.buttons.pressed[button_index(button)]
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.buttons.just_pressed[button_index(button)]
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        self.buttons.just_released[button_index(button)]
    }
}

fn button_index(button: MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Other(n) => (3 + n as usize).min(4),
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Self::new()
    }
}

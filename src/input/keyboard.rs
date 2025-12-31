//! # Keyboard Input

use std::collections::HashSet;

/// Key state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Released,
    Pressed,
    Held,
    JustReleased,
}

/// Virtual key codes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Space,
    Enter,
    Escape,
    Tab,
    Backspace,
    Left,
    Right,
    Up,
    Down,
    LShift,
    RShift,
    LControl,
    RControl,
    LAlt,
    RAlt,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

/// Keyboard state tracker
pub struct Keyboard {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
        }
    }

    /// Call at start of frame to clear transient state
    pub fn update(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Process key press event
    pub fn key_down(&mut self, key: KeyCode) {
        if !self.pressed.contains(&key) {
            self.just_pressed.insert(key);
        }
        self.pressed.insert(key);
    }

    /// Process key release event
    pub fn key_up(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    /// Check if key is currently pressed
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Check if key was just pressed this frame
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// Check if key was just released this frame
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// Get key state
    pub fn state(&self, key: KeyCode) -> KeyState {
        if self.just_pressed.contains(&key) {
            KeyState::Pressed
        } else if self.just_released.contains(&key) {
            KeyState::JustReleased
        } else if self.pressed.contains(&key) {
            KeyState::Held
        } else {
            KeyState::Released
        }
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

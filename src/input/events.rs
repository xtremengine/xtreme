//! # Input Events

use super::keyboard::KeyCode;
use super::mouse::MouseButton;

/// Input event types
#[derive(Clone, Debug)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseMoved { x: f32, y: f32 },
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseScroll(f32),
    WindowResized { width: u32, height: u32 },
    WindowClosed,
}

/// Event queue for frame-based processing
pub struct InputEventQueue {
    events: Vec<InputEvent>,
}

impl InputEventQueue {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn push(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = InputEvent> + '_ {
        self.events.drain(..)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for InputEventQueue {
    fn default() -> Self {
        Self::new()
    }
}

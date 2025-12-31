//! # Input Module
//!
//! Keyboard, mouse, and gamepad input handling.
//!
//! ## Features
//!
//! - Key state tracking (pressed, held, released)
//! - Mouse position and button state
//! - Event queue for frame-based processing
//! - Screen-to-world coordinate conversion

mod keyboard;
mod mouse;
mod events;

pub use keyboard::{KeyState, KeyCode, Keyboard};
pub use mouse::{MouseState, MouseButton, Mouse};
pub use events::{InputEvent, InputEventQueue};

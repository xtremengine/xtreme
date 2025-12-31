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

mod events;
mod keyboard;
mod mouse;

pub use events::{InputEvent, InputEventQueue};
pub use keyboard::{KeyCode, KeyState, Keyboard};
pub use mouse::{Mouse, MouseButton, MouseState};

//! # Editor Application
//!
//! Main editor application with egui UI and 3D viewport.
//!
//! This module is split into:
//! - `state.rs` - EditorApp struct and initialization
//! - `actions/` - Object manipulation, undo/redo, save/load, etc.
//! - `input.rs` - Viewport input handling (ray casting, gizmo interaction)
//! - `ui.rs` - UI drawing coordination
//! - `menu.rs` - Menu bar drawing
//! - `dialogs.rs` - Dialog windows
//! - `splash.rs` - Splash screen
//! - `lifecycle.rs` - App trait implementation
//! - `shortcuts_handler.rs` - Keyboard shortcut action handling
//! - `game_events.rs` - Game window event handling

mod actions;
mod audio_preview;
mod clipboard;
mod dialogs;
mod game_events;
mod history;
mod input;
mod lifecycle;
mod menu;
mod menu_file;
mod object_actions;
mod play_mode;
mod prefab_actions;
mod scene_io;
mod shortcuts_handler;
mod splash;
mod state;
pub mod templates;
mod ui;

pub use state::EditorApp;

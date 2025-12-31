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

mod state;
mod actions;
mod object_actions;
mod history;
mod scene_io;
mod clipboard;
mod prefab_actions;
mod play_mode;
mod input;
mod ui;
mod menu;
mod dialogs;
mod splash;
mod lifecycle;

pub use state::EditorApp;

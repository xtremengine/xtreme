//! # Scripting System
//!
//! Python scripting integration via pyo3 for game logic.
//!
//! ## Features
//! - Python script execution
//! - Godot-like lifecycle callbacks (_ready, _update, _physics_update)
//! - Scene object manipulation API
//! - Input handling API

#[cfg(feature = "scripting")]
pub mod api;
#[cfg(feature = "scripting")]
mod runtime;
#[cfg(feature = "scripting")]
mod script;

#[cfg(feature = "scripting")]
pub use api::{AnimationChange, AnimatorData, ObjectTransform, ScriptContext};
#[cfg(feature = "scripting")]
pub use runtime::ScriptRuntime;
#[cfg(feature = "scripting")]
pub use script::{Script, ScriptError, ScriptId};

/// Check if scripting feature is enabled
pub fn is_scripting_enabled() -> bool {
    cfg!(feature = "scripting")
}

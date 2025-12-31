//! # Editor Module
//!
//! Visual editor for Xtreme Engine games.
//!
//! ## Features
//!
//! - 3D viewport with isometric camera
//! - Entity hierarchy panel
//! - Inspector for editing components
//! - Toolbar for creating and manipulating objects
//! - Undo/Redo with command pattern
//! - Customizable keyboard shortcuts
//! - Scene save/load (RON/JSON)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use xtreme::editor::EditorApp;
//! use xtreme::render::{WindowConfig, run};
//!
//! fn main() {
//!     let app = EditorApp::new();
//!     let config = WindowConfig::new("Xtreme Editor");
//!     run(app, config).unwrap();
//! }
//! ```

mod app;
mod viewport;
mod selection;
mod panels;
mod gizmos;
mod commands;
mod shortcuts;
mod scene;
mod snap;
mod prefab;
mod game_window;
pub mod mesh;
pub mod project;
pub mod hierarchy;

pub use app::EditorApp;
pub use project::{Project, ProjectConfig};
pub use viewport::Viewport;
pub use selection::{SceneObject, Selection, ObjectId, Ray, pick_object};
pub use panels::{HierarchyPanel, ToolbarPanel, InspectorPanel};
pub use gizmos::{Gizmo, GizmoAxis, GizmoMode, GizmoDelta};
pub use commands::{Command, CommandHistory};
pub use shortcuts::{Shortcut, ShortcutManager, EditorAction, KeyCode, Modifiers};
pub use scene::{SceneData, SceneObjectData, SceneManager, SceneError};
pub use snap::SnapSettings;
pub use prefab::{Prefab, PrefabObject, PrefabError};

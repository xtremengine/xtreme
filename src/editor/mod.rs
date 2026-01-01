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
mod commands;
pub mod components;
mod game_window;
mod gizmos;
pub mod hierarchy;
pub mod mesh;
mod panels;
mod prefab;
pub mod project;
mod scene;
mod scene_object;
mod selection;
mod shortcuts;
mod snap;
mod viewport;

pub use app::EditorApp;
pub use commands::{Command, CommandHistory};
pub use components::{CameraComponent, CameraProjection};
pub use gizmos::{Gizmo, GizmoAxis, GizmoDelta, GizmoMode};
pub use panels::{HierarchyPanel, InspectorPanel, ToolbarPanel};
pub use prefab::{Prefab, PrefabError, PrefabObject};
pub use project::{Project, ProjectConfig};
pub use scene::{SceneData, SceneError, SceneManager, SceneObjectData};
pub use selection::{pick_object, ObjectId, Ray, SceneObject, Selection};
pub use shortcuts::{EditorAction, KeyCode, Modifiers, Shortcut, ShortcutManager};
pub use snap::SnapSettings;
pub use viewport::Viewport;

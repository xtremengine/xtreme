//! # Editor Panels
//!
//! UI panels for the editor.

pub mod asset_browser;
pub mod camera_settings;
pub mod hierarchy;
pub mod inspector;
pub mod scripts;
pub mod snap_settings;
pub mod toolbar;

#[allow(unused_imports)]
pub use asset_browser::{AssetAction, AssetBrowser, AssetEntry, AssetType};
pub use camera_settings::draw_camera_settings;
pub use hierarchy::{HierarchyAction, HierarchyPanel};
pub use inspector::InspectorPanel;
#[cfg(feature = "scripting")]
pub use scripts::draw_scripts_section;
#[cfg(feature = "scripting")]
pub use scripts::ScriptsAction;
pub use snap_settings::draw_snap_settings;
pub use toolbar::{Tool, ToolbarAction, ToolbarPanel};

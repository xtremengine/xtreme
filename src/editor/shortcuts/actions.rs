//! Editor actions that can be bound to shortcuts.

use serde::{Deserialize, Serialize};

/// Editor actions that can be bound to shortcuts
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorAction {
    // File
    NewScene,
    OpenScene,
    SaveScene,
    SaveSceneAs,

    // Edit
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Delete,
    Duplicate,
    SelectAll,

    // Tools
    SelectTool,
    MoveTool,
    RotateTool,
    ScaleTool,

    // View
    FocusSelected,
    FrameAll,
    ResetCamera,
    TopView,
    FrontView,
    SideView,

    // Object
    CreateCube,
    CreateEmpty,
    ToggleVisibility,

    // Play
    TogglePlay,
}

impl EditorAction {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            EditorAction::NewScene => "New Scene",
            EditorAction::OpenScene => "Open Scene",
            EditorAction::SaveScene => "Save Scene",
            EditorAction::SaveSceneAs => "Save Scene As",
            EditorAction::Undo => "Undo",
            EditorAction::Redo => "Redo",
            EditorAction::Cut => "Cut",
            EditorAction::Copy => "Copy",
            EditorAction::Paste => "Paste",
            EditorAction::Delete => "Delete",
            EditorAction::Duplicate => "Duplicate",
            EditorAction::SelectAll => "Select All",
            EditorAction::SelectTool => "Select Tool",
            EditorAction::MoveTool => "Move Tool",
            EditorAction::RotateTool => "Rotate Tool",
            EditorAction::ScaleTool => "Scale Tool",
            EditorAction::FocusSelected => "Focus Selected",
            EditorAction::FrameAll => "Frame All",
            EditorAction::ResetCamera => "Reset Camera",
            EditorAction::TopView => "Top View",
            EditorAction::FrontView => "Front View",
            EditorAction::SideView => "Side View",
            EditorAction::CreateCube => "Create Cube",
            EditorAction::CreateEmpty => "Create Empty",
            EditorAction::ToggleVisibility => "Toggle Visibility",
            EditorAction::TogglePlay => "Toggle Play Mode",
        }
    }
}

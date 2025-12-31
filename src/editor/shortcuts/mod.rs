//! # Keyboard Shortcuts
//!
//! Customizable keyboard shortcuts for editor actions.

mod actions;
mod keys;
mod types;

pub use actions::EditorAction;
pub use keys::KeyCode;
pub use types::{Modifiers, Shortcut};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Shortcut manager
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortcutManager {
    /// Map from shortcut to action
    bindings: HashMap<Shortcut, EditorAction>,
    /// Reverse map for lookup
    #[serde(skip)]
    action_shortcuts: HashMap<EditorAction, Shortcut>,
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutManager {
    /// Create with default shortcuts
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            action_shortcuts: HashMap::new(),
        };

        // Default bindings
        manager.bind(Shortcut::ctrl(KeyCode::N), EditorAction::NewScene);
        manager.bind(Shortcut::ctrl(KeyCode::O), EditorAction::OpenScene);
        manager.bind(Shortcut::ctrl(KeyCode::S), EditorAction::SaveScene);
        manager.bind(Shortcut::ctrl_shift(KeyCode::S), EditorAction::SaveSceneAs);

        manager.bind(Shortcut::ctrl(KeyCode::Z), EditorAction::Undo);
        manager.bind(Shortcut::ctrl_shift(KeyCode::Z), EditorAction::Redo);
        manager.bind(Shortcut::ctrl(KeyCode::Y), EditorAction::Redo);
        manager.bind(Shortcut::ctrl(KeyCode::X), EditorAction::Cut);
        manager.bind(Shortcut::ctrl(KeyCode::C), EditorAction::Copy);
        manager.bind(Shortcut::ctrl(KeyCode::V), EditorAction::Paste);
        manager.bind(Shortcut::key(KeyCode::Delete), EditorAction::Delete);
        manager.bind(Shortcut::ctrl(KeyCode::D), EditorAction::Duplicate);
        manager.bind(Shortcut::ctrl(KeyCode::A), EditorAction::SelectAll);

        manager.bind(Shortcut::key(KeyCode::Q), EditorAction::SelectTool);
        manager.bind(Shortcut::key(KeyCode::W), EditorAction::MoveTool);
        manager.bind(Shortcut::key(KeyCode::E), EditorAction::RotateTool);
        manager.bind(Shortcut::key(KeyCode::R), EditorAction::ScaleTool);

        manager.bind(Shortcut::key(KeyCode::F), EditorAction::FocusSelected);
        manager.bind(Shortcut::key(KeyCode::Home), EditorAction::FrameAll);
        manager.bind(Shortcut::ctrl(KeyCode::Num1), EditorAction::FrontView);
        manager.bind(Shortcut::ctrl(KeyCode::Num7), EditorAction::TopView);
        manager.bind(Shortcut::ctrl(KeyCode::Num3), EditorAction::SideView);

        manager.bind(Shortcut::key(KeyCode::H), EditorAction::ToggleVisibility);

        manager.bind(Shortcut::key(KeyCode::F5), EditorAction::TogglePlay);

        manager
    }

    /// Bind a shortcut to an action
    pub fn bind(&mut self, shortcut: Shortcut, action: EditorAction) {
        // Remove old binding for this action
        if let Some(old_shortcut) = self.action_shortcuts.remove(&action) {
            self.bindings.remove(&old_shortcut);
        }
        // Remove old action for this shortcut
        if let Some(old_action) = self.bindings.remove(&shortcut) {
            self.action_shortcuts.remove(&old_action);
        }
        // Add new binding
        self.bindings.insert(shortcut, action);
        self.action_shortcuts.insert(action, shortcut);
    }

    /// Unbind a shortcut
    pub fn unbind(&mut self, shortcut: &Shortcut) {
        if let Some(action) = self.bindings.remove(shortcut) {
            self.action_shortcuts.remove(&action);
        }
    }

    /// Get action for a shortcut
    pub fn get_action(&self, shortcut: &Shortcut) -> Option<EditorAction> {
        self.bindings.get(shortcut).copied()
    }

    /// Get shortcut for an action
    pub fn get_shortcut(&self, action: EditorAction) -> Option<&Shortcut> {
        self.action_shortcuts.get(&action)
    }

    /// Get shortcut display string for an action
    pub fn get_shortcut_text(&self, action: EditorAction) -> String {
        self.action_shortcuts
            .get(&action)
            .map(|s| s.display())
            .unwrap_or_default()
    }

    /// Process egui input and return triggered actions
    pub fn process_input(&self, ctx: &egui::Context) -> Vec<EditorAction> {
        // Don't process shortcuts when typing in a text field
        if ctx.wants_keyboard_input() {
            return Vec::new();
        }

        let mut actions = Vec::new();

        ctx.input(|input| {
            let mods = &input.modifiers;

            for (shortcut, action) in &self.bindings {
                if shortcut.modifiers.matches(mods) {
                    // Check if key was pressed this frame
                    let egui_key = shortcut.key.to_egui();
                    if input.key_pressed(egui_key) {
                        actions.push(*action);
                    }
                }
            }
        });

        actions
    }

    /// Rebuild the reverse lookup map (call after deserialization)
    pub fn rebuild_lookup(&mut self) {
        self.action_shortcuts.clear();
        for (shortcut, action) in &self.bindings {
            self.action_shortcuts.insert(*action, *shortcut);
        }
    }

    /// Get all bindings for display/editing
    pub fn all_bindings(&self) -> &HashMap<Shortcut, EditorAction> {
        &self.bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_display() {
        let shortcut = Shortcut::ctrl(KeyCode::S);
        assert_eq!(shortcut.display(), "Ctrl+S");

        let shortcut = Shortcut::ctrl_shift(KeyCode::Z);
        assert_eq!(shortcut.display(), "Ctrl+Shift+Z");
    }

    #[test]
    fn test_default_bindings() {
        let manager = ShortcutManager::new();

        assert_eq!(
            manager.get_action(&Shortcut::ctrl(KeyCode::Z)),
            Some(EditorAction::Undo)
        );

        assert_eq!(
            manager.get_action(&Shortcut::key(KeyCode::W)),
            Some(EditorAction::MoveTool)
        );
    }
}

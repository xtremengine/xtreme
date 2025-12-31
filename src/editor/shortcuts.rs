//! # Keyboard Shortcuts
//!
//! Customizable keyboard shortcuts for editor actions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Modifier keys
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::none()
    }
}

impl Modifiers {
    /// No modifiers
    pub fn none() -> Self {
        Self { ctrl: false, shift: false, alt: false }
    }

    /// Ctrl modifier
    pub fn ctrl() -> Self {
        Self { ctrl: true, shift: false, alt: false }
    }

    /// Shift modifier
    pub fn shift() -> Self {
        Self { ctrl: false, shift: true, alt: false }
    }

    /// Alt modifier
    pub fn alt() -> Self {
        Self { ctrl: false, shift: false, alt: true }
    }

    /// Ctrl+Shift modifier
    pub fn ctrl_shift() -> Self {
        Self { ctrl: true, shift: true, alt: false }
    }

    /// Check if matches egui modifiers
    pub fn matches(&self, mods: &egui::Modifiers) -> bool {
        self.ctrl == (mods.ctrl || mods.command) &&
        self.shift == mods.shift &&
        self.alt == mods.alt
    }
}

/// A keyboard shortcut (key + modifiers)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    pub key: KeyCode,
    pub modifiers: Modifiers,
}

impl Shortcut {
    /// Create a new shortcut
    pub fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    /// Simple key without modifiers
    pub fn key(key: KeyCode) -> Self {
        Self { key, modifiers: Modifiers::none() }
    }

    /// Ctrl + key
    pub fn ctrl(key: KeyCode) -> Self {
        Self { key, modifiers: Modifiers::ctrl() }
    }

    /// Shift + key
    pub fn shift(key: KeyCode) -> Self {
        Self { key, modifiers: Modifiers::shift() }
    }

    /// Ctrl+Shift + key
    pub fn ctrl_shift(key: KeyCode) -> Self {
        Self { key, modifiers: Modifiers::ctrl_shift() }
    }

    /// Format as human-readable string
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.ctrl { parts.push("Ctrl"); }
        if self.modifiers.shift { parts.push("Shift"); }
        if self.modifiers.alt { parts.push("Alt"); }
        parts.push(self.key.name());
        parts.join("+")
    }
}

/// Key codes (subset of common keys)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Escape, Tab, Space, Enter, Backspace, Delete,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Home, End, PageUp, PageDown,
}

impl KeyCode {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            KeyCode::A => "A", KeyCode::B => "B", KeyCode::C => "C",
            KeyCode::D => "D", KeyCode::E => "E", KeyCode::F => "F",
            KeyCode::G => "G", KeyCode::H => "H", KeyCode::I => "I",
            KeyCode::J => "J", KeyCode::K => "K", KeyCode::L => "L",
            KeyCode::M => "M", KeyCode::N => "N", KeyCode::O => "O",
            KeyCode::P => "P", KeyCode::Q => "Q", KeyCode::R => "R",
            KeyCode::S => "S", KeyCode::T => "T", KeyCode::U => "U",
            KeyCode::V => "V", KeyCode::W => "W", KeyCode::X => "X",
            KeyCode::Y => "Y", KeyCode::Z => "Z",
            KeyCode::Num0 => "0", KeyCode::Num1 => "1", KeyCode::Num2 => "2",
            KeyCode::Num3 => "3", KeyCode::Num4 => "4", KeyCode::Num5 => "5",
            KeyCode::Num6 => "6", KeyCode::Num7 => "7", KeyCode::Num8 => "8",
            KeyCode::Num9 => "9",
            KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3",
            KeyCode::F4 => "F4", KeyCode::F5 => "F5", KeyCode::F6 => "F6",
            KeyCode::F7 => "F7", KeyCode::F8 => "F8", KeyCode::F9 => "F9",
            KeyCode::F10 => "F10", KeyCode::F11 => "F11", KeyCode::F12 => "F12",
            KeyCode::Escape => "Esc", KeyCode::Tab => "Tab",
            KeyCode::Space => "Space", KeyCode::Enter => "Enter",
            KeyCode::Backspace => "Backspace", KeyCode::Delete => "Del",
            KeyCode::ArrowUp => "Up", KeyCode::ArrowDown => "Down",
            KeyCode::ArrowLeft => "Left", KeyCode::ArrowRight => "Right",
            KeyCode::Home => "Home", KeyCode::End => "End",
            KeyCode::PageUp => "PgUp", KeyCode::PageDown => "PgDn",
        }
    }

    /// Convert from egui Key
    pub fn from_egui(key: egui::Key) -> Option<Self> {
        Some(match key {
            egui::Key::A => KeyCode::A, egui::Key::B => KeyCode::B,
            egui::Key::C => KeyCode::C, egui::Key::D => KeyCode::D,
            egui::Key::E => KeyCode::E, egui::Key::F => KeyCode::F,
            egui::Key::G => KeyCode::G, egui::Key::H => KeyCode::H,
            egui::Key::I => KeyCode::I, egui::Key::J => KeyCode::J,
            egui::Key::K => KeyCode::K, egui::Key::L => KeyCode::L,
            egui::Key::M => KeyCode::M, egui::Key::N => KeyCode::N,
            egui::Key::O => KeyCode::O, egui::Key::P => KeyCode::P,
            egui::Key::Q => KeyCode::Q, egui::Key::R => KeyCode::R,
            egui::Key::S => KeyCode::S, egui::Key::T => KeyCode::T,
            egui::Key::U => KeyCode::U, egui::Key::V => KeyCode::V,
            egui::Key::W => KeyCode::W, egui::Key::X => KeyCode::X,
            egui::Key::Y => KeyCode::Y, egui::Key::Z => KeyCode::Z,
            egui::Key::Num0 => KeyCode::Num0, egui::Key::Num1 => KeyCode::Num1,
            egui::Key::Num2 => KeyCode::Num2, egui::Key::Num3 => KeyCode::Num3,
            egui::Key::Num4 => KeyCode::Num4, egui::Key::Num5 => KeyCode::Num5,
            egui::Key::Num6 => KeyCode::Num6, egui::Key::Num7 => KeyCode::Num7,
            egui::Key::Num8 => KeyCode::Num8, egui::Key::Num9 => KeyCode::Num9,
            egui::Key::F1 => KeyCode::F1, egui::Key::F2 => KeyCode::F2,
            egui::Key::F3 => KeyCode::F3, egui::Key::F4 => KeyCode::F4,
            egui::Key::F5 => KeyCode::F5, egui::Key::F6 => KeyCode::F6,
            egui::Key::F7 => KeyCode::F7, egui::Key::F8 => KeyCode::F8,
            egui::Key::F9 => KeyCode::F9, egui::Key::F10 => KeyCode::F10,
            egui::Key::F11 => KeyCode::F11, egui::Key::F12 => KeyCode::F12,
            egui::Key::Escape => KeyCode::Escape, egui::Key::Tab => KeyCode::Tab,
            egui::Key::Space => KeyCode::Space, egui::Key::Enter => KeyCode::Enter,
            egui::Key::Backspace => KeyCode::Backspace, egui::Key::Delete => KeyCode::Delete,
            egui::Key::ArrowUp => KeyCode::ArrowUp, egui::Key::ArrowDown => KeyCode::ArrowDown,
            egui::Key::ArrowLeft => KeyCode::ArrowLeft, egui::Key::ArrowRight => KeyCode::ArrowRight,
            egui::Key::Home => KeyCode::Home, egui::Key::End => KeyCode::End,
            egui::Key::PageUp => KeyCode::PageUp, egui::Key::PageDown => KeyCode::PageDown,
            _ => return None,
        })
    }
}

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
        self.action_shortcuts.get(&action)
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
                    if let Some(egui_key) = self.keycode_to_egui(shortcut.key) {
                        if input.key_pressed(egui_key) {
                            actions.push(*action);
                        }
                    }
                }
            }
        });

        actions
    }

    /// Convert our KeyCode to egui::Key
    fn keycode_to_egui(&self, key: KeyCode) -> Option<egui::Key> {
        Some(match key {
            KeyCode::A => egui::Key::A, KeyCode::B => egui::Key::B,
            KeyCode::C => egui::Key::C, KeyCode::D => egui::Key::D,
            KeyCode::E => egui::Key::E, KeyCode::F => egui::Key::F,
            KeyCode::G => egui::Key::G, KeyCode::H => egui::Key::H,
            KeyCode::I => egui::Key::I, KeyCode::J => egui::Key::J,
            KeyCode::K => egui::Key::K, KeyCode::L => egui::Key::L,
            KeyCode::M => egui::Key::M, KeyCode::N => egui::Key::N,
            KeyCode::O => egui::Key::O, KeyCode::P => egui::Key::P,
            KeyCode::Q => egui::Key::Q, KeyCode::R => egui::Key::R,
            KeyCode::S => egui::Key::S, KeyCode::T => egui::Key::T,
            KeyCode::U => egui::Key::U, KeyCode::V => egui::Key::V,
            KeyCode::W => egui::Key::W, KeyCode::X => egui::Key::X,
            KeyCode::Y => egui::Key::Y, KeyCode::Z => egui::Key::Z,
            KeyCode::Num0 => egui::Key::Num0, KeyCode::Num1 => egui::Key::Num1,
            KeyCode::Num2 => egui::Key::Num2, KeyCode::Num3 => egui::Key::Num3,
            KeyCode::Num4 => egui::Key::Num4, KeyCode::Num5 => egui::Key::Num5,
            KeyCode::Num6 => egui::Key::Num6, KeyCode::Num7 => egui::Key::Num7,
            KeyCode::Num8 => egui::Key::Num8, KeyCode::Num9 => egui::Key::Num9,
            KeyCode::F1 => egui::Key::F1, KeyCode::F2 => egui::Key::F2,
            KeyCode::F3 => egui::Key::F3, KeyCode::F4 => egui::Key::F4,
            KeyCode::F5 => egui::Key::F5, KeyCode::F6 => egui::Key::F6,
            KeyCode::F7 => egui::Key::F7, KeyCode::F8 => egui::Key::F8,
            KeyCode::F9 => egui::Key::F9, KeyCode::F10 => egui::Key::F10,
            KeyCode::F11 => egui::Key::F11, KeyCode::F12 => egui::Key::F12,
            KeyCode::Escape => egui::Key::Escape, KeyCode::Tab => egui::Key::Tab,
            KeyCode::Space => egui::Key::Space, KeyCode::Enter => egui::Key::Enter,
            KeyCode::Backspace => egui::Key::Backspace, KeyCode::Delete => egui::Key::Delete,
            KeyCode::ArrowUp => egui::Key::ArrowUp, KeyCode::ArrowDown => egui::Key::ArrowDown,
            KeyCode::ArrowLeft => egui::Key::ArrowLeft, KeyCode::ArrowRight => egui::Key::ArrowRight,
            KeyCode::Home => egui::Key::Home, KeyCode::End => egui::Key::End,
            KeyCode::PageUp => egui::Key::PageUp, KeyCode::PageDown => egui::Key::PageDown,
        })
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

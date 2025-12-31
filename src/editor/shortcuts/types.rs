//! Shortcut types: Modifiers and Shortcut structs.

use serde::{Deserialize, Serialize};

use super::keys::KeyCode;

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
        Self {
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// Ctrl modifier
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            shift: false,
            alt: false,
        }
    }

    /// Shift modifier
    pub fn shift() -> Self {
        Self {
            ctrl: false,
            shift: true,
            alt: false,
        }
    }

    /// Alt modifier
    pub fn alt() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: true,
        }
    }

    /// Ctrl+Shift modifier
    pub fn ctrl_shift() -> Self {
        Self {
            ctrl: true,
            shift: true,
            alt: false,
        }
    }

    /// Check if matches egui modifiers
    pub fn matches(&self, mods: &egui::Modifiers) -> bool {
        self.ctrl == (mods.ctrl || mods.command) && self.shift == mods.shift && self.alt == mods.alt
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
        Self {
            key,
            modifiers: Modifiers::none(),
        }
    }

    /// Ctrl + key
    pub fn ctrl(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: Modifiers::ctrl(),
        }
    }

    /// Shift + key
    pub fn shift(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: Modifiers::shift(),
        }
    }

    /// Ctrl+Shift + key
    pub fn ctrl_shift(key: KeyCode) -> Self {
        Self {
            key,
            modifiers: Modifiers::ctrl_shift(),
        }
    }

    /// Format as human-readable string
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers.ctrl {
            parts.push("Ctrl");
        }
        if self.modifiers.shift {
            parts.push("Shift");
        }
        if self.modifiers.alt {
            parts.push("Alt");
        }
        parts.push(self.key.name());
        parts.join("+")
    }
}

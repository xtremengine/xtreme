//! Animator component for the editor.

use serde::{Deserialize, Serialize};

/// Animator component - handles skeletal animation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimatorComponent {
    /// Path to GLTF file with skeleton/animations
    pub skeleton_path: Option<String>,
    /// Current animation state name
    pub current_state: String,
    /// Animation playback speed multiplier
    pub speed: f32,
    /// Whether animation is currently playing
    pub playing: bool,
}

impl Default for AnimatorComponent {
    fn default() -> Self {
        Self {
            skeleton_path: None,
            current_state: "Idle".to_string(),
            speed: 1.0,
            playing: false,
        }
    }
}

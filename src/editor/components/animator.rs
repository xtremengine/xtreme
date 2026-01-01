//! Animator component for the editor.

use serde::{Deserialize, Serialize};

/// A single animation clip (can be from skeleton file or separate file)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimationClip {
    /// Animation name (e.g., "Idle", "Walk", "Run")
    pub name: String,
    /// Path to animation file (if separate file) or None (if from skeleton)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Animation index within the GLTF file (for files with multiple animations)
    #[serde(default)]
    pub animation_index: usize,
    /// Playback speed for this animation
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether this animation should loop
    #[serde(default = "default_looping")]
    pub looping: bool,
}

fn default_speed() -> f32 {
    1.0
}

fn default_looping() -> bool {
    true
}

impl Default for AnimationClip {
    fn default() -> Self {
        Self {
            name: "New Animation".to_string(),
            file_path: None,
            animation_index: 0,
            speed: 1.0,
            looping: true,
        }
    }
}

impl AnimationClip {
    /// Create a new animation clip with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create from a file path
    pub fn from_file(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            file_path: Some(path.into()),
            ..Default::default()
        }
    }
}

/// Animator component - handles skeletal animation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimatorComponent {
    /// Path to GLTF file with skeleton/animations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skeleton_path: Option<String>,
    /// List of available animations
    #[serde(default)]
    pub animations: Vec<AnimationClip>,
    /// Current animation name (must match one in animations list)
    #[serde(default = "default_current_animation")]
    pub current_animation: String,
    /// Global animation playback speed multiplier
    #[serde(default = "default_speed")]
    pub speed: f32,
    /// Whether animation is currently playing
    #[serde(default)]
    pub playing: bool,
    /// Current playback time (0.0 - 1.0 normalized)
    #[serde(default)]
    pub current_time: f32,
}

fn default_current_animation() -> String {
    "Idle".to_string()
}

impl Default for AnimatorComponent {
    fn default() -> Self {
        Self {
            skeleton_path: None,
            animations: vec![AnimationClip::new("Idle")],
            current_animation: "Idle".to_string(),
            speed: 1.0,
            playing: false,
            current_time: 0.0,
        }
    }
}

impl AnimatorComponent {
    /// Get the current animation clip
    pub fn current_clip(&self) -> Option<&AnimationClip> {
        self.animations
            .iter()
            .find(|a| a.name == self.current_animation)
    }

    /// Get mutable reference to current animation clip
    pub fn current_clip_mut(&mut self) -> Option<&mut AnimationClip> {
        let name = self.current_animation.clone();
        self.animations.iter_mut().find(|a| a.name == name)
    }

    /// Add a new animation
    pub fn add_animation(&mut self, clip: AnimationClip) {
        self.animations.push(clip);
    }

    /// Remove animation by name
    pub fn remove_animation(&mut self, name: &str) {
        self.animations.retain(|a| a.name != name);
        // If current animation was removed, switch to first available
        if self.current_animation == name {
            self.current_animation = self
                .animations
                .first()
                .map(|a| a.name.clone())
                .unwrap_or_default();
        }
    }

    /// Get list of animation names
    pub fn animation_names(&self) -> Vec<&str> {
        self.animations.iter().map(|a| a.name.as_str()).collect()
    }

    /// Set current animation by name
    pub fn set_animation(&mut self, name: &str) -> bool {
        if self.animations.iter().any(|a| a.name == name) {
            self.current_animation = name.to_string();
            self.current_time = 0.0;
            true
        } else {
            false
        }
    }
}

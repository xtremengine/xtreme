//! Audio components for the editor.

use serde::{Deserialize, Serialize};

/// Audio source component - plays sounds in the scene
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioSourceComponent {
    /// Path to the audio clip file
    pub clip_path: Option<String>,
    /// Volume level (0.0 - 1.0)
    pub volume: f32,
    /// Pitch multiplier (0.5 - 2.0)
    pub pitch: f32,
    /// Whether the audio loops
    pub looping: bool,
    /// Whether to use 3D spatial audio
    pub spatial: bool,
    /// Minimum distance for spatial audio (full volume)
    pub min_distance: f32,
    /// Maximum distance for spatial audio (silent)
    pub max_distance: f32,
    /// Whether to play automatically when scene starts
    pub autoplay: bool,
}

impl Default for AudioSourceComponent {
    fn default() -> Self {
        Self {
            clip_path: None,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: true,
            min_distance: 1.0,
            max_distance: 50.0,
            autoplay: false,
        }
    }
}

/// Audio listener component - receives audio in the scene (usually on camera/player)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioListenerComponent {
    /// Whether this listener is active
    pub is_active: bool,
    /// Master volume for all audio received by this listener
    pub master_volume: f32,
}

impl Default for AudioListenerComponent {
    fn default() -> Self {
        Self {
            is_active: true,
            master_volume: 1.0,
        }
    }
}

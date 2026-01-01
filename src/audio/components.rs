//! Audio Components
//!
//! Components for spatial audio in the ECS system.

use crate::core::Component;
use glam::Vec3;

/// Unique identifier for loaded audio clips
pub type AudioClipId = u32;

/// Playback state of an audio source
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

/// Internal command for audio system
#[derive(Clone, Copy, Debug)]
pub enum AudioCommand {
    Play,
    Pause,
    Resume,
    Stop,
}

/// Audio listener component - the "ears" in the scene.
///
/// Attach this to the camera or player entity. Only one listener
/// should be active at a time.
#[derive(Clone, Copy, Debug)]
pub struct AudioListener {
    /// Master volume multiplier (0.0 - 1.0)
    pub master_volume: f32,
    /// Forward direction vector (for stereo panning)
    pub forward: Vec3,
    /// Up direction vector
    pub up: Vec3,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
        }
    }
}

impl Component for AudioListener {}

impl AudioListener {
    /// Create a new audio listener with default orientation
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the master volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.master_volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set the listener orientation
    pub fn with_orientation(mut self, forward: Vec3, up: Vec3) -> Self {
        self.forward = forward.normalize();
        self.up = up.normalize();
        self
    }

    /// Calculate the right vector from forward and up
    pub fn right(&self) -> Vec3 {
        self.forward.cross(self.up).normalize()
    }
}

/// Audio source component - emits sound from an entity's position.
///
/// Can be configured for 2D or 3D spatial audio.
#[derive(Clone, Debug)]
pub struct AudioSource {
    /// The audio clip to play
    pub clip: Option<AudioClipId>,
    /// Current playback state
    pub state: PlaybackState,
    /// Volume multiplier (0.0 - 1.0)
    pub volume: f32,
    /// Pitch multiplier (0.5 - 2.0 typical range)
    pub pitch: f32,
    /// Whether to loop the audio
    pub looping: bool,
    /// Whether this source uses 3D spatial audio
    pub spatial: bool,
    /// Minimum distance for attenuation (full volume within this)
    pub min_distance: f32,
    /// Maximum distance (sound inaudible beyond this)
    pub max_distance: f32,
    /// Attenuation rolloff factor (1.0 = linear, 2.0 = quadratic)
    pub rolloff: f32,
    /// Internal handle to playback sink
    pub(crate) sink_id: Option<u64>,
    /// Pending command for the audio system
    pub(crate) pending_command: Option<AudioCommand>,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            clip: None,
            state: PlaybackState::Stopped,
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            spatial: true,
            min_distance: 1.0,
            max_distance: 100.0,
            rolloff: 1.0,
            sink_id: None,
            pending_command: None,
        }
    }
}

impl Component for AudioSource {}

impl AudioSource {
    /// Create a new audio source with a clip
    pub fn new(clip: AudioClipId) -> Self {
        Self {
            clip: Some(clip),
            ..Default::default()
        }
    }

    /// Create a 2D (non-spatial) audio source
    pub fn new_2d(clip: AudioClipId) -> Self {
        Self {
            clip: Some(clip),
            spatial: false,
            ..Default::default()
        }
    }

    /// Start playing the audio
    pub fn play(&mut self) {
        self.pending_command = Some(AudioCommand::Play);
    }

    /// Pause the audio
    pub fn pause(&mut self) {
        self.pending_command = Some(AudioCommand::Pause);
    }

    /// Resume paused audio
    pub fn resume(&mut self) {
        self.pending_command = Some(AudioCommand::Resume);
    }

    /// Stop the audio
    pub fn stop(&mut self) {
        self.pending_command = Some(AudioCommand::Stop);
    }

    /// Check if audio is currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Set the volume (0.0 - 1.0)
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set the pitch (0.1 - 4.0)
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 4.0);
        self
    }

    /// Enable looping
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Enable/disable spatial audio
    pub fn with_spatial(mut self, spatial: bool) -> Self {
        self.spatial = spatial;
        self
    }

    /// Set distance range for attenuation
    pub fn with_distance(mut self, min: f32, max: f32) -> Self {
        self.min_distance = min.max(0.1);
        self.max_distance = max.max(min + 0.1);
        self
    }

    /// Set rolloff factor for attenuation curve
    pub fn with_rolloff(mut self, rolloff: f32) -> Self {
        self.rolloff = rolloff.max(0.0);
        self
    }
}

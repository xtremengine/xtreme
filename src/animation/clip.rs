//! Animation Clips and Keyframes
//!
//! Defines animation data structures for keyframe-based animation.

use super::bone::BoneIndex;
use super::easing::EasingFunction;
use glam::{Quat, Vec3};
use std::collections::HashMap;

/// Unique identifier for animation clips
pub type ClipId = u32;

/// A single keyframe for a property
#[derive(Clone, Debug)]
pub struct Keyframe<T: Clone> {
    /// Time in seconds from clip start
    pub time: f32,
    /// Value at this keyframe
    pub value: T,
    /// Easing function to next keyframe
    pub easing: EasingFunction,
}

impl<T: Clone> Keyframe<T> {
    /// Create a new keyframe
    pub fn new(time: f32, value: T) -> Self {
        Self {
            time,
            value,
            easing: EasingFunction::Linear,
        }
    }

    /// Create with easing function
    pub fn with_easing(time: f32, value: T, easing: EasingFunction) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// Animation channel targeting a specific property
#[derive(Clone, Debug)]
pub enum AnimationChannel {
    Translation(Vec<Keyframe<Vec3>>),
    Rotation(Vec<Keyframe<Quat>>),
    Scale(Vec<Keyframe<Vec3>>),
}

impl AnimationChannel {
    /// Get the duration of this channel (time of last keyframe)
    pub fn duration(&self) -> f32 {
        match self {
            Self::Translation(kf) => kf.last().map(|k| k.time).unwrap_or(0.0),
            Self::Rotation(kf) => kf.last().map(|k| k.time).unwrap_or(0.0),
            Self::Scale(kf) => kf.last().map(|k| k.time).unwrap_or(0.0),
        }
    }

    /// Check if channel is empty
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Translation(kf) => kf.is_empty(),
            Self::Rotation(kf) => kf.is_empty(),
            Self::Scale(kf) => kf.is_empty(),
        }
    }
}

/// Animation track for a single bone
#[derive(Clone, Debug, Default)]
pub struct BoneTrack {
    /// Target bone index
    pub bone_index: BoneIndex,
    /// Translation keyframes
    pub translation: Option<Vec<Keyframe<Vec3>>>,
    /// Rotation keyframes
    pub rotation: Option<Vec<Keyframe<Quat>>>,
    /// Scale keyframes
    pub scale: Option<Vec<Keyframe<Vec3>>>,
}

impl BoneTrack {
    /// Create a new empty track for a bone
    pub fn new(bone_index: BoneIndex) -> Self {
        Self {
            bone_index,
            translation: None,
            rotation: None,
            scale: None,
        }
    }

    /// Set translation keyframes
    pub fn with_translation(mut self, keyframes: Vec<Keyframe<Vec3>>) -> Self {
        self.translation = Some(keyframes);
        self
    }

    /// Set rotation keyframes
    pub fn with_rotation(mut self, keyframes: Vec<Keyframe<Quat>>) -> Self {
        self.rotation = Some(keyframes);
        self
    }

    /// Set scale keyframes
    pub fn with_scale(mut self, keyframes: Vec<Keyframe<Vec3>>) -> Self {
        self.scale = Some(keyframes);
        self
    }

    /// Get the duration of this track
    pub fn duration(&self) -> f32 {
        let t_dur = self
            .translation
            .as_ref()
            .and_then(|kf| kf.last())
            .map(|k| k.time)
            .unwrap_or(0.0);
        let r_dur = self
            .rotation
            .as_ref()
            .and_then(|kf| kf.last())
            .map(|k| k.time)
            .unwrap_or(0.0);
        let s_dur = self
            .scale
            .as_ref()
            .and_then(|kf| kf.last())
            .map(|k| k.time)
            .unwrap_or(0.0);

        t_dur.max(r_dur).max(s_dur)
    }
}

/// Animation clip containing all tracks for an animation
#[derive(Clone, Debug)]
pub struct AnimationClip {
    /// Clip name
    pub name: String,
    /// Total duration in seconds
    pub duration: f32,
    /// Bone tracks
    pub tracks: Vec<BoneTrack>,
    /// Whether this animation loops
    pub looping: bool,
    /// Playback speed multiplier
    pub speed: f32,
}

impl AnimationClip {
    /// Create a new empty animation clip
    pub fn new(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            tracks: Vec::new(),
            looping: false,
            speed: 1.0,
        }
    }

    /// Add a bone track
    pub fn with_track(mut self, track: BoneTrack) -> Self {
        self.tracks.push(track);
        self
    }

    /// Set looping
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Set playback speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Calculate duration from tracks
    pub fn calculate_duration(&mut self) {
        self.duration = self
            .tracks
            .iter()
            .map(|t| t.duration())
            .fold(0.0f32, |a, b| a.max(b));
    }

    /// Get number of tracks
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Get track by bone index
    pub fn track_for_bone(&self, bone_index: BoneIndex) -> Option<&BoneTrack> {
        self.tracks.iter().find(|t| t.bone_index == bone_index)
    }
}

/// Library of animation clips
pub struct AnimationLibrary {
    clips: HashMap<ClipId, AnimationClip>,
    name_to_id: HashMap<String, ClipId>,
    next_id: ClipId,
}

impl Default for AnimationLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationLibrary {
    /// Create a new empty library
    pub fn new() -> Self {
        Self {
            clips: HashMap::new(),
            name_to_id: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a clip to the library
    pub fn add_clip(&mut self, clip: AnimationClip) -> ClipId {
        let id = self.next_id;
        self.next_id += 1;

        self.name_to_id.insert(clip.name.clone(), id);
        self.clips.insert(id, clip);

        id
    }

    /// Get a clip by ID
    pub fn get_clip(&self, id: ClipId) -> Option<&AnimationClip> {
        self.clips.get(&id)
    }

    /// Get a clip by name
    pub fn get_clip_by_name(&self, name: &str) -> Option<&AnimationClip> {
        self.name_to_id.get(name).and_then(|id| self.clips.get(id))
    }

    /// Find clip ID by name
    pub fn find_clip(&self, name: &str) -> Option<ClipId> {
        self.name_to_id.get(name).copied()
    }

    /// Remove a clip
    pub fn remove_clip(&mut self, id: ClipId) -> Option<AnimationClip> {
        if let Some(clip) = self.clips.remove(&id) {
            self.name_to_id.remove(&clip.name);
            Some(clip)
        } else {
            None
        }
    }

    /// Number of clips
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Iterate all clips
    pub fn clips(&self) -> impl Iterator<Item = (&ClipId, &AnimationClip)> {
        self.clips.iter()
    }
}

/// Animation event that can be triggered at specific times
#[derive(Clone, Debug)]
pub struct AnimationEvent {
    /// Time in the clip when this event triggers
    pub time: f32,
    /// Event name/identifier
    pub name: String,
    /// Optional event data
    pub data: Option<String>,
}

impl AnimationEvent {
    /// Create a new event
    pub fn new(time: f32, name: impl Into<String>) -> Self {
        Self {
            time,
            name: name.into(),
            data: None,
        }
    }

    /// Create with data
    pub fn with_data(time: f32, name: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            time,
            name: name.into(),
            data: Some(data.into()),
        }
    }
}

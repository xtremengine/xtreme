//! # Timeline Sequence
//!
//! Core data structures for timeline sequences, tracks, and keyframes.

use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Easing function for keyframe interpolation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Easing {
    /// Linear interpolation
    #[default]
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out (slow start and end)
    EaseInOut,
    /// Step function (no interpolation)
    Step,
    /// Cubic bezier (custom control points)
    CubicBezier,
}

impl Easing {
    /// Apply easing function to a 0-1 value
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::Step => {
                if t < 1.0 {
                    0.0
                } else {
                    1.0
                }
            }
            Easing::CubicBezier => t, // TODO: implement cubic bezier
        }
    }
}

/// Type of track
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    // Skeletal animation
    /// Bone translation track
    BoneTranslation,
    /// Bone rotation track
    BoneRotation,
    /// Bone scale track
    BoneScale,

    // Object animation
    /// Object position track
    ObjectPosition,
    /// Object rotation track
    ObjectRotation,
    /// Object scale track
    ObjectScale,
    /// Object color track
    ObjectColor,
    /// Object visibility track
    ObjectVisibility,

    // Camera
    /// Camera position track
    CameraPosition,
    /// Camera target/look-at track
    CameraTarget,
    /// Camera field of view track
    CameraFov,

    // Events
    /// Generic event track
    Event,
    /// Dialogue/subtitle track
    Dialogue,
    /// Audio playback track
    Audio,
}

impl TrackType {
    /// Get display name for this track type
    pub fn display_name(&self) -> &'static str {
        match self {
            TrackType::BoneTranslation => "Bone Position",
            TrackType::BoneRotation => "Bone Rotation",
            TrackType::BoneScale => "Bone Scale",
            TrackType::ObjectPosition => "Position",
            TrackType::ObjectRotation => "Rotation",
            TrackType::ObjectScale => "Scale",
            TrackType::ObjectColor => "Color",
            TrackType::ObjectVisibility => "Visibility",
            TrackType::CameraPosition => "Camera Position",
            TrackType::CameraTarget => "Camera Target",
            TrackType::CameraFov => "Camera FOV",
            TrackType::Event => "Event",
            TrackType::Dialogue => "Dialogue",
            TrackType::Audio => "Audio",
        }
    }

    /// Check if this is a bone track
    pub fn is_bone_track(&self) -> bool {
        matches!(
            self,
            TrackType::BoneTranslation | TrackType::BoneRotation | TrackType::BoneScale
        )
    }

    /// Check if this is an object track
    pub fn is_object_track(&self) -> bool {
        matches!(
            self,
            TrackType::ObjectPosition
                | TrackType::ObjectRotation
                | TrackType::ObjectScale
                | TrackType::ObjectColor
                | TrackType::ObjectVisibility
        )
    }

    /// Check if this is an event track
    pub fn is_event_track(&self) -> bool {
        matches!(
            self,
            TrackType::Event | TrackType::Dialogue | TrackType::Audio
        )
    }
}

/// Value stored in a keyframe
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum KeyframeValue {
    /// Single float value
    Float(f32),
    /// 3D vector (position, scale, euler rotation)
    Vec3(Vec3),
    /// Quaternion rotation
    Quat(Quat),
    /// Boolean value (visibility)
    Bool(bool),
    /// Color value (RGBA)
    Color([f32; 4]),
    /// Generic event with name and optional data
    Event {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data: Option<String>,
    },
    /// Dialogue entry
    Dialogue {
        speaker: String,
        text: String,
        duration: f32,
    },
    /// Audio clip reference
    Audio { clip_path: String, volume: f32 },
}

impl KeyframeValue {
    /// Create a new Vec3 keyframe value
    pub fn vec3(x: f32, y: f32, z: f32) -> Self {
        KeyframeValue::Vec3(Vec3::new(x, y, z))
    }

    /// Create a new event keyframe
    pub fn event(name: impl Into<String>) -> Self {
        KeyframeValue::Event {
            name: name.into(),
            data: None,
        }
    }

    /// Create a dialogue keyframe
    pub fn dialogue(speaker: impl Into<String>, text: impl Into<String>, duration: f32) -> Self {
        KeyframeValue::Dialogue {
            speaker: speaker.into(),
            text: text.into(),
            duration,
        }
    }

    /// Interpolate between two values
    pub fn lerp(&self, other: &KeyframeValue, t: f32) -> Option<KeyframeValue> {
        match (self, other) {
            (KeyframeValue::Float(a), KeyframeValue::Float(b)) => {
                Some(KeyframeValue::Float(a + (b - a) * t))
            }
            (KeyframeValue::Vec3(a), KeyframeValue::Vec3(b)) => {
                Some(KeyframeValue::Vec3(a.lerp(*b, t)))
            }
            (KeyframeValue::Quat(a), KeyframeValue::Quat(b)) => {
                Some(KeyframeValue::Quat(a.slerp(*b, t)))
            }
            (KeyframeValue::Color(a), KeyframeValue::Color(b)) => {
                let color = [
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                    a[3] + (b[3] - a[3]) * t,
                ];
                Some(KeyframeValue::Color(color))
            }
            // Non-interpolatable types return first value
            _ => None,
        }
    }
}

/// A single keyframe in a track
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time of this keyframe in seconds
    pub time: f32,
    /// Value at this keyframe
    pub value: KeyframeValue,
    /// Easing function to next keyframe
    #[serde(default)]
    pub easing: Easing,
}

impl Keyframe {
    /// Create a new keyframe
    pub fn new(time: f32, value: KeyframeValue) -> Self {
        Self {
            time,
            value,
            easing: Easing::default(),
        }
    }

    /// Create with easing
    pub fn with_easing(time: f32, value: KeyframeValue, easing: Easing) -> Self {
        Self {
            time,
            value,
            easing,
        }
    }
}

/// A track containing keyframes for a specific property
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    /// Unique ID for this track
    pub id: u32,
    /// Display name
    pub name: String,
    /// Type of track
    pub track_type: TrackType,
    /// Target entity ID (for object/camera tracks)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_entity: Option<u32>,
    /// Target bone index (for skeletal tracks)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_bone: Option<u16>,
    /// Keyframes in this track (sorted by time)
    pub keyframes: Vec<Keyframe>,
    /// Whether track is muted
    #[serde(default)]
    pub muted: bool,
    /// Whether track is locked for editing
    #[serde(default)]
    pub locked: bool,
    /// Track color for UI
    #[serde(default = "default_track_color")]
    pub color: [f32; 4],
}

fn default_track_color() -> [f32; 4] {
    [0.4, 0.6, 0.8, 1.0]
}

impl Track {
    /// Create a new track
    pub fn new(id: u32, name: impl Into<String>, track_type: TrackType) -> Self {
        Self {
            id,
            name: name.into(),
            track_type,
            target_entity: None,
            target_bone: None,
            keyframes: Vec::new(),
            muted: false,
            locked: false,
            color: default_track_color(),
        }
    }

    /// Add a keyframe, maintaining sorted order
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        let pos = self
            .keyframes
            .binary_search_by(|k| k.time.partial_cmp(&keyframe.time).unwrap())
            .unwrap_or_else(|e| e);
        self.keyframes.insert(pos, keyframe);
    }

    /// Remove keyframe at time
    pub fn remove_keyframe_at(&mut self, time: f32) -> Option<Keyframe> {
        if let Some(pos) = self
            .keyframes
            .iter()
            .position(|k| (k.time - time).abs() < 0.001)
        {
            Some(self.keyframes.remove(pos))
        } else {
            None
        }
    }

    /// Get value at time
    pub fn sample(&self, time: f32) -> Option<KeyframeValue> {
        if self.keyframes.is_empty() {
            return None;
        }

        // Find surrounding keyframes
        let mut prev = &self.keyframes[0];
        let mut next = &self.keyframes[0];

        for kf in &self.keyframes {
            if kf.time <= time {
                prev = kf;
            }
            if kf.time >= time {
                next = kf;
                break;
            }
        }

        // If same keyframe or no interpolation possible
        if prev.time >= next.time {
            return Some(prev.value.clone());
        }

        // Calculate interpolation factor
        let t = (time - prev.time) / (next.time - prev.time);
        let eased_t = prev.easing.apply(t);

        // Interpolate
        prev.value
            .lerp(&next.value, eased_t)
            .or_else(|| Some(prev.value.clone()))
    }

    /// Get duration (time of last keyframe)
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }
}

/// A group of related tracks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackGroup {
    /// Group name
    pub name: String,
    /// Track IDs in this group
    pub track_ids: Vec<u32>,
    /// Whether group is collapsed in UI
    #[serde(default)]
    pub collapsed: bool,
}

impl TrackGroup {
    /// Create a new track group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            track_ids: Vec::new(),
            collapsed: false,
        }
    }
}

/// A complete timeline sequence
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineSequence {
    /// Sequence name
    pub name: String,
    /// Duration in seconds
    pub duration: f32,
    /// Frame rate for display/snapping
    pub frame_rate: f32,
    /// All tracks in this sequence
    pub tracks: Vec<Track>,
    /// Track groups for organization
    #[serde(default)]
    pub groups: Vec<TrackGroup>,
    /// Whether sequence should loop
    #[serde(default)]
    pub looping: bool,
    /// Sequence version
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    1
}

impl Default for TimelineSequence {
    fn default() -> Self {
        Self {
            name: "New Sequence".to_string(),
            duration: 5.0,
            frame_rate: 30.0,
            tracks: Vec::new(),
            groups: Vec::new(),
            looping: false,
            version: 1,
        }
    }
}

impl TimelineSequence {
    /// Create a new sequence
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create with duration
    pub fn with_duration(name: impl Into<String>, duration: f32) -> Self {
        Self {
            name: name.into(),
            duration,
            ..Default::default()
        }
    }

    /// Add a track
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    /// Get track by ID
    pub fn get_track(&self, id: u32) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == id)
    }

    /// Get mutable track by ID
    pub fn get_track_mut(&mut self, id: u32) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id == id)
    }

    /// Remove track by ID
    pub fn remove_track(&mut self, id: u32) -> Option<Track> {
        if let Some(pos) = self.tracks.iter().position(|t| t.id == id) {
            // Also remove from groups
            for group in &mut self.groups {
                group.track_ids.retain(|&tid| tid != id);
            }
            Some(self.tracks.remove(pos))
        } else {
            None
        }
    }

    /// Calculate actual duration from tracks
    pub fn calculate_duration(&self) -> f32 {
        self.tracks
            .iter()
            .map(|t| t.duration())
            .fold(0.0_f32, |a, b| a.max(b))
    }

    /// Generate next track ID
    pub fn next_track_id(&self) -> u32 {
        self.tracks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    /// Convert time to frame number
    pub fn time_to_frame(&self, time: f32) -> i32 {
        (time * self.frame_rate) as i32
    }

    /// Convert frame number to time
    pub fn frame_to_time(&self, frame: i32) -> f32 {
        frame as f32 / self.frame_rate
    }

    /// Snap time to nearest frame
    pub fn snap_to_frame(&self, time: f32) -> f32 {
        let frame = self.time_to_frame(time);
        self.frame_to_time(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing() {
        assert_eq!(Easing::Linear.apply(0.5), 0.5);
        assert!(Easing::EaseIn.apply(0.5) < 0.5);
        assert!(Easing::EaseOut.apply(0.5) > 0.5);
    }

    #[test]
    fn test_keyframe_lerp() {
        let a = KeyframeValue::Float(0.0);
        let b = KeyframeValue::Float(10.0);
        let lerped = a.lerp(&b, 0.5);
        assert_eq!(lerped, Some(KeyframeValue::Float(5.0)));
    }

    #[test]
    fn test_track_sample() {
        let mut track = Track::new(1, "Test", TrackType::ObjectPosition);
        track.add_keyframe(Keyframe::new(0.0, KeyframeValue::vec3(0.0, 0.0, 0.0)));
        track.add_keyframe(Keyframe::new(1.0, KeyframeValue::vec3(10.0, 0.0, 0.0)));

        let sample = track.sample(0.5);
        assert!(sample.is_some());
        if let Some(KeyframeValue::Vec3(v)) = sample {
            assert!((v.x - 5.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_sequence() {
        let mut seq = TimelineSequence::new("Test");
        let mut track = Track::new(1, "Position", TrackType::ObjectPosition);
        track.add_keyframe(Keyframe::new(2.0, KeyframeValue::vec3(0.0, 0.0, 0.0)));

        seq.add_track(track);
        assert_eq!(seq.calculate_duration(), 2.0);
    }
}

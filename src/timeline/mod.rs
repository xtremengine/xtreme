//! # Timeline Module
//!
//! Timeline and sequencer system for animations and cutscenes.
//!
//! ## Features
//!
//! - **Timeline Sequences**: Multi-track animation with keyframes
//! - **Track Types**: Object, bone, camera, event, dialogue, audio
//! - **Keyframe Interpolation**: Linear, ease in/out, step, cubic bezier
//! - **Playback Control**: Play, pause, stop, speed, looping
//! - **Cutscene Support**: Dialogue, events, camera movements
//!
//! ## Example
//!
//! ```rust,ignore
//! use xtreme::timeline::{TimelineSequence, Track, TrackType, Keyframe, KeyframeValue};
//! use xtreme::timeline::TimelinePlayer;
//!
//! // Create a sequence
//! let mut seq = TimelineSequence::new("Walk Cycle");
//! seq.duration = 2.0;
//!
//! // Add a position track
//! let mut track = Track::new(1, "Position", TrackType::ObjectPosition);
//! track.target_entity = Some(player_id);
//! track.add_keyframe(Keyframe::new(0.0, KeyframeValue::vec3(0.0, 0.0, 0.0)));
//! track.add_keyframe(Keyframe::new(2.0, KeyframeValue::vec3(10.0, 0.0, 0.0)));
//! seq.add_track(track);
//!
//! // Playback
//! let mut player = TimelinePlayer::new();
//! player.play();
//!
//! // In update loop:
//! player.update(delta_time, &seq);
//! let result = player.sample(&seq);
//! ```

pub mod cutscene;
pub mod playback;
pub mod sequence;

// Re-exports
pub use cutscene::{CutsceneBuilder, CutsceneState, DialogueEntry};
pub use playback::{
    BoneTransform, CameraValues, ObjectTransform, PlaybackEvent, PlaybackState, SampleResult,
    TimelinePlayer,
};
pub use sequence::{
    Easing, Keyframe, KeyframeValue, TimelineSequence, Track, TrackGroup, TrackType,
};

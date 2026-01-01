//! # Timeline Playback
//!
//! Playback engine for timeline sequences.

use std::collections::HashMap;

use super::sequence::{KeyframeValue, TimelineSequence, TrackType};

/// Playback state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackState {
    /// Stopped at beginning
    #[default]
    Stopped,
    /// Currently playing
    Playing,
    /// Paused at current time
    Paused,
}

/// Event fired during playback
#[derive(Clone, Debug)]
pub struct PlaybackEvent {
    /// Track ID that generated the event
    pub track_id: u32,
    /// Time of the event
    pub time: f32,
    /// Event data
    pub data: KeyframeValue,
}

/// Result of sampling a timeline at a specific time
#[derive(Clone, Debug, Default)]
pub struct SampleResult {
    /// Object transforms (object_id -> (position, rotation, scale))
    pub object_transforms: HashMap<u32, ObjectTransform>,
    /// Bone transforms (bone_index -> (position, rotation, scale))
    pub bone_transforms: HashMap<u16, BoneTransform>,
    /// Camera values
    pub camera: Option<CameraValues>,
    /// Events that occurred at this time
    pub events: Vec<PlaybackEvent>,
}

/// Object transform from timeline
#[derive(Clone, Debug, Default)]
pub struct ObjectTransform {
    pub position: Option<glam::Vec3>,
    pub rotation: Option<glam::Vec3>,
    pub scale: Option<glam::Vec3>,
    pub color: Option<[f32; 4]>,
    pub visible: Option<bool>,
}

/// Bone transform from timeline
#[derive(Clone, Debug, Default)]
pub struct BoneTransform {
    pub translation: Option<glam::Vec3>,
    pub rotation: Option<glam::Quat>,
    pub scale: Option<glam::Vec3>,
}

/// Camera values from timeline
#[derive(Clone, Debug, Default)]
pub struct CameraValues {
    pub position: Option<glam::Vec3>,
    pub target: Option<glam::Vec3>,
    pub fov: Option<f32>,
}

/// Timeline player for controlling playback
pub struct TimelinePlayer {
    /// Current playback time in seconds
    current_time: f32,
    /// Playback state
    state: PlaybackState,
    /// Playback speed multiplier
    speed: f32,
    /// Events that have been fired this frame
    pending_events: Vec<PlaybackEvent>,
    /// Last sampled time for event detection
    last_sample_time: f32,
}

impl Default for TimelinePlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelinePlayer {
    /// Create a new player
    pub fn new() -> Self {
        Self {
            current_time: 0.0,
            state: PlaybackState::Stopped,
            speed: 1.0,
            pending_events: Vec::new(),
            last_sample_time: 0.0,
        }
    }

    /// Get current time
    pub fn current_time(&self) -> f32 {
        self.current_time
    }

    /// Set current time
    pub fn set_time(&mut self, time: f32) {
        self.current_time = time.max(0.0);
        self.last_sample_time = self.current_time;
    }

    /// Get playback state
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Is currently playing
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Get playback speed
    pub fn speed(&self) -> f32 {
        self.speed
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// Start playback
    pub fn play(&mut self) {
        self.state = PlaybackState::Playing;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.state = PlaybackState::Paused;
    }

    /// Stop and reset to beginning
    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped;
        self.current_time = 0.0;
        self.last_sample_time = 0.0;
    }

    /// Toggle play/pause
    pub fn toggle(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            _ => self.play(),
        }
    }

    /// Go to start
    pub fn go_to_start(&mut self) {
        self.current_time = 0.0;
        self.last_sample_time = 0.0;
    }

    /// Go to end
    pub fn go_to_end(&mut self, sequence: &TimelineSequence) {
        self.current_time = sequence.duration;
        self.last_sample_time = self.current_time;
    }

    /// Update playback (call each frame)
    pub fn update(&mut self, delta_time: f32, sequence: &TimelineSequence) {
        if self.state != PlaybackState::Playing {
            return;
        }

        self.last_sample_time = self.current_time;
        self.current_time += delta_time * self.speed;

        // Handle looping or end of sequence
        if self.current_time >= sequence.duration {
            if sequence.looping {
                self.current_time %= sequence.duration;
                self.last_sample_time = 0.0;
            } else {
                self.current_time = sequence.duration;
                self.state = PlaybackState::Paused;
            }
        }

        // Handle negative time
        if self.current_time < 0.0 {
            self.current_time = 0.0;
        }
    }

    /// Sample the sequence at current time
    pub fn sample(&mut self, sequence: &TimelineSequence) -> SampleResult {
        let mut result = SampleResult::default();

        for track in &sequence.tracks {
            if track.muted {
                continue;
            }

            if let Some(value) = track.sample(self.current_time) {
                self.apply_sample_to_result(
                    &mut result,
                    track.id,
                    &track.track_type,
                    &value,
                    track.target_entity,
                    track.target_bone,
                );

                // Check for events between last and current time
                if track.track_type.is_event_track() {
                    self.collect_events(track, &mut result);
                }
            }
        }

        result
    }

    /// Apply a sampled value to the result
    fn apply_sample_to_result(
        &self,
        result: &mut SampleResult,
        _track_id: u32,
        track_type: &TrackType,
        value: &KeyframeValue,
        target_entity: Option<u32>,
        target_bone: Option<u16>,
    ) {
        match track_type {
            TrackType::ObjectPosition => {
                if let (Some(entity), KeyframeValue::Vec3(v)) = (target_entity, value) {
                    result.object_transforms.entry(entity).or_default().position = Some(*v);
                }
            }
            TrackType::ObjectRotation => {
                if let (Some(entity), KeyframeValue::Vec3(v)) = (target_entity, value) {
                    result.object_transforms.entry(entity).or_default().rotation = Some(*v);
                }
            }
            TrackType::ObjectScale => {
                if let (Some(entity), KeyframeValue::Vec3(v)) = (target_entity, value) {
                    result.object_transforms.entry(entity).or_default().scale = Some(*v);
                }
            }
            TrackType::ObjectColor => {
                if let (Some(entity), KeyframeValue::Color(c)) = (target_entity, value) {
                    result.object_transforms.entry(entity).or_default().color = Some(*c);
                }
            }
            TrackType::ObjectVisibility => {
                if let (Some(entity), KeyframeValue::Bool(v)) = (target_entity, value) {
                    result.object_transforms.entry(entity).or_default().visible = Some(*v);
                }
            }
            TrackType::BoneTranslation => {
                if let (Some(bone), KeyframeValue::Vec3(v)) = (target_bone, value) {
                    result.bone_transforms.entry(bone).or_default().translation = Some(*v);
                }
            }
            TrackType::BoneRotation => {
                if let (Some(bone), KeyframeValue::Quat(q)) = (target_bone, value) {
                    result.bone_transforms.entry(bone).or_default().rotation = Some(*q);
                }
            }
            TrackType::BoneScale => {
                if let (Some(bone), KeyframeValue::Vec3(v)) = (target_bone, value) {
                    result.bone_transforms.entry(bone).or_default().scale = Some(*v);
                }
            }
            TrackType::CameraPosition => {
                if let KeyframeValue::Vec3(v) = value {
                    result.camera.get_or_insert_with(Default::default).position = Some(*v);
                }
            }
            TrackType::CameraTarget => {
                if let KeyframeValue::Vec3(v) = value {
                    result.camera.get_or_insert_with(Default::default).target = Some(*v);
                }
            }
            TrackType::CameraFov => {
                if let KeyframeValue::Float(f) = value {
                    result.camera.get_or_insert_with(Default::default).fov = Some(*f);
                }
            }
            _ => {}
        }
    }

    /// Collect events that occurred between last and current time
    fn collect_events(&self, track: &super::sequence::Track, result: &mut SampleResult) {
        let (start, end) = if self.last_sample_time <= self.current_time {
            (self.last_sample_time, self.current_time)
        } else {
            // Wrapped around (looping)
            (self.last_sample_time, self.current_time)
        };

        for kf in &track.keyframes {
            if kf.time > start && kf.time <= end {
                result.events.push(PlaybackEvent {
                    track_id: track.id,
                    time: kf.time,
                    data: kf.value.clone(),
                });
            }
        }
    }

    /// Take pending events (clears the list)
    pub fn take_events(&mut self) -> Vec<PlaybackEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::sequence::{Keyframe, Track};

    #[test]
    fn test_player_basic() {
        let mut player = TimelinePlayer::new();
        assert_eq!(player.state(), PlaybackState::Stopped);

        player.play();
        assert_eq!(player.state(), PlaybackState::Playing);

        player.pause();
        assert_eq!(player.state(), PlaybackState::Paused);

        player.stop();
        assert_eq!(player.state(), PlaybackState::Stopped);
        assert_eq!(player.current_time(), 0.0);
    }

    #[test]
    fn test_player_update() {
        let mut player = TimelinePlayer::new();
        let seq = TimelineSequence::with_duration("Test", 2.0);

        player.play();
        player.update(0.5, &seq);
        assert!((player.current_time() - 0.5).abs() < 0.001);

        player.update(1.0, &seq);
        assert!((player.current_time() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_player_sample() {
        let mut player = TimelinePlayer::new();
        let mut seq = TimelineSequence::with_duration("Test", 2.0);

        let mut track = Track::new(1, "Position", TrackType::ObjectPosition);
        track.target_entity = Some(1);
        track.add_keyframe(Keyframe::new(0.0, KeyframeValue::Vec3(glam::Vec3::ZERO)));
        track.add_keyframe(Keyframe::new(
            1.0,
            KeyframeValue::Vec3(glam::Vec3::new(10.0, 0.0, 0.0)),
        ));
        seq.add_track(track);

        player.set_time(0.5);
        let result = player.sample(&seq);

        assert!(result.object_transforms.contains_key(&1));
        if let Some(pos) = result.object_transforms[&1].position {
            assert!((pos.x - 5.0).abs() < 0.1);
        }
    }
}

//! # Cutscene Helpers
//!
//! Utilities for creating and managing cutscenes.

use super::playback::PlaybackEvent;
use super::sequence::{Easing, Keyframe, KeyframeValue, TimelineSequence, Track, TrackType};

/// Cutscene builder for easy cutscene creation
pub struct CutsceneBuilder {
    sequence: TimelineSequence,
    next_track_id: u32,
}

impl CutsceneBuilder {
    /// Create a new cutscene builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            sequence: TimelineSequence::new(name),
            next_track_id: 1,
        }
    }

    /// Set duration
    pub fn duration(mut self, duration: f32) -> Self {
        self.sequence.duration = duration;
        self
    }

    /// Set looping
    pub fn looping(mut self, looping: bool) -> Self {
        self.sequence.looping = looping;
        self
    }

    /// Add a camera movement from one position to another
    pub fn camera_move(
        mut self,
        start_time: f32,
        end_time: f32,
        from: glam::Vec3,
        to: glam::Vec3,
        easing: Easing,
    ) -> Self {
        let mut track = Track::new(
            self.next_track_id,
            "Camera Position",
            TrackType::CameraPosition,
        );
        self.next_track_id += 1;

        track.add_keyframe(Keyframe::with_easing(
            start_time,
            KeyframeValue::Vec3(from),
            easing,
        ));
        track.add_keyframe(Keyframe::new(end_time, KeyframeValue::Vec3(to)));

        self.sequence.add_track(track);
        self
    }

    /// Add a camera target movement
    pub fn camera_look_at(
        mut self,
        start_time: f32,
        end_time: f32,
        from: glam::Vec3,
        to: glam::Vec3,
        easing: Easing,
    ) -> Self {
        let mut track = Track::new(self.next_track_id, "Camera Target", TrackType::CameraTarget);
        self.next_track_id += 1;

        track.add_keyframe(Keyframe::with_easing(
            start_time,
            KeyframeValue::Vec3(from),
            easing,
        ));
        track.add_keyframe(Keyframe::new(end_time, KeyframeValue::Vec3(to)));

        self.sequence.add_track(track);
        self
    }

    /// Add object position animation
    pub fn move_object(
        mut self,
        entity_id: u32,
        start_time: f32,
        end_time: f32,
        from: glam::Vec3,
        to: glam::Vec3,
        easing: Easing,
    ) -> Self {
        let mut track = Track::new(
            self.next_track_id,
            format!("Object {} Position", entity_id),
            TrackType::ObjectPosition,
        );
        track.target_entity = Some(entity_id);
        self.next_track_id += 1;

        track.add_keyframe(Keyframe::with_easing(
            start_time,
            KeyframeValue::Vec3(from),
            easing,
        ));
        track.add_keyframe(Keyframe::new(end_time, KeyframeValue::Vec3(to)));

        self.sequence.add_track(track);
        self
    }

    /// Add a dialogue line
    pub fn dialogue(
        mut self,
        time: f32,
        speaker: impl Into<String>,
        text: impl Into<String>,
        duration: f32,
    ) -> Self {
        // Find or create dialogue track
        let track_id = self
            .sequence
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Dialogue)
            .map(|t| t.id);

        if let Some(id) = track_id {
            if let Some(track) = self.sequence.get_track_mut(id) {
                track.add_keyframe(Keyframe::new(
                    time,
                    KeyframeValue::dialogue(speaker, text, duration),
                ));
            }
        } else {
            let mut track = Track::new(self.next_track_id, "Dialogue", TrackType::Dialogue);
            self.next_track_id += 1;
            track.add_keyframe(Keyframe::new(
                time,
                KeyframeValue::dialogue(speaker, text, duration),
            ));
            self.sequence.add_track(track);
        }

        self
    }

    /// Add an event trigger
    pub fn event(mut self, time: f32, event_name: impl Into<String>) -> Self {
        // Find or create events track
        let track_id = self
            .sequence
            .tracks
            .iter()
            .find(|t| t.track_type == TrackType::Event)
            .map(|t| t.id);

        if let Some(id) = track_id {
            if let Some(track) = self.sequence.get_track_mut(id) {
                track.add_keyframe(Keyframe::new(time, KeyframeValue::event(event_name)));
            }
        } else {
            let mut track = Track::new(self.next_track_id, "Events", TrackType::Event);
            self.next_track_id += 1;
            track.add_keyframe(Keyframe::new(time, KeyframeValue::event(event_name)));
            self.sequence.add_track(track);
        }

        self
    }

    /// Add a custom track
    pub fn with_track(mut self, track: Track) -> Self {
        self.sequence.add_track(track);
        self.next_track_id = self.sequence.next_track_id();
        self
    }

    /// Build the cutscene
    pub fn build(self) -> TimelineSequence {
        self.sequence
    }
}

/// Dialogue entry for subtitle display
#[derive(Clone, Debug)]
pub struct DialogueEntry {
    /// Speaker name
    pub speaker: String,
    /// Dialogue text
    pub text: String,
    /// Duration to display
    pub duration: f32,
    /// Time the dialogue started
    pub start_time: f32,
}

/// Cutscene state manager
pub struct CutsceneState {
    /// Currently active dialogue
    pub active_dialogue: Option<DialogueEntry>,
    /// Time remaining on current dialogue
    pub dialogue_time_remaining: f32,
    /// Events that were triggered this frame
    pub triggered_events: Vec<String>,
    /// Whether cutscene UI should be shown
    pub show_ui: bool,
    /// Skip requested by player
    pub skip_requested: bool,
}

impl Default for CutsceneState {
    fn default() -> Self {
        Self::new()
    }
}

impl CutsceneState {
    /// Create new cutscene state
    pub fn new() -> Self {
        Self {
            active_dialogue: None,
            dialogue_time_remaining: 0.0,
            triggered_events: Vec::new(),
            show_ui: true,
            skip_requested: false,
        }
    }

    /// Process playback events
    pub fn process_events(&mut self, events: &[PlaybackEvent], current_time: f32) {
        self.triggered_events.clear();

        for event in events {
            match &event.data {
                KeyframeValue::Dialogue {
                    speaker,
                    text,
                    duration,
                } => {
                    self.active_dialogue = Some(DialogueEntry {
                        speaker: speaker.clone(),
                        text: text.clone(),
                        duration: *duration,
                        start_time: current_time,
                    });
                    self.dialogue_time_remaining = *duration;
                }
                KeyframeValue::Event { name, .. } => {
                    self.triggered_events.push(name.clone());
                }
                _ => {}
            }
        }
    }

    /// Update dialogue timing
    pub fn update(&mut self, delta_time: f32) {
        if self.active_dialogue.is_some() {
            self.dialogue_time_remaining -= delta_time;
            if self.dialogue_time_remaining <= 0.0 {
                self.active_dialogue = None;
            }
        }
    }

    /// Skip current dialogue
    pub fn skip_dialogue(&mut self) {
        self.active_dialogue = None;
        self.dialogue_time_remaining = 0.0;
    }

    /// Request skip of entire cutscene
    pub fn request_skip(&mut self) {
        self.skip_requested = true;
    }

    /// Clear state
    pub fn clear(&mut self) {
        self.active_dialogue = None;
        self.dialogue_time_remaining = 0.0;
        self.triggered_events.clear();
        self.skip_requested = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cutscene_builder() {
        let cutscene = CutsceneBuilder::new("Intro")
            .duration(10.0)
            .camera_move(
                0.0,
                5.0,
                glam::Vec3::new(0.0, 5.0, -10.0),
                glam::Vec3::new(0.0, 2.0, -5.0),
                Easing::EaseInOut,
            )
            .dialogue(1.0, "Hero", "Hello, world!", 2.0)
            .event(3.0, "start_music")
            .build();

        assert_eq!(cutscene.name, "Intro");
        assert_eq!(cutscene.duration, 10.0);
        assert_eq!(cutscene.tracks.len(), 3);
    }
}

//! Animator Component
//!
//! Manages animation playback and state transitions.

use super::clip::ClipId;
pub use super::state_machine::{
    AnimationState, AnimationStateMachine, Comparison, StateMachineBuilder, StateTransition,
    TransitionCondition,
};
use crate::core::Component;
use std::collections::HashMap;

/// Current playback state
#[derive(Clone, Debug)]
pub struct PlaybackInfo {
    /// Current state name
    pub state: String,
    /// Current time in the animation
    pub time: f32,
    /// Current speed
    pub speed: f32,
    /// Whether the current animation has finished
    pub finished: bool,
}

impl Default for PlaybackInfo {
    fn default() -> Self {
        Self {
            state: String::new(),
            time: 0.0,
            speed: 1.0,
            finished: false,
        }
    }
}

/// Active blend between states
#[derive(Clone, Debug)]
pub struct AnimationBlend {
    /// Source state
    pub from_state: String,
    /// Source time
    pub from_time: f32,
    /// Target state
    pub to_state: String,
    /// Target time
    pub to_time: f32,
    /// Blend duration
    pub duration: f32,
    /// Elapsed blend time
    pub elapsed: f32,
}

impl AnimationBlend {
    /// Calculate blend factor (0 = from, 1 = to)
    pub fn factor(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Check if blend is complete
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// Animator component - manages animation playback for an entity
#[derive(Clone, Debug)]
pub struct Animator {
    /// Skeleton ID (for shared skeletons)
    pub skeleton_id: u32,
    /// State machine definition
    pub state_machine: AnimationStateMachine,
    /// Current playback info
    pub playback: PlaybackInfo,
    /// Active blend (if transitioning)
    pub blend: Option<AnimationBlend>,
    /// Animation parameters (floats)
    pub parameters: HashMap<String, f32>,
    /// Active triggers (cleared after processing)
    pub triggers: Vec<String>,
    /// Whether playback is paused
    pub paused: bool,
    /// Time since last state change
    pub time_in_state: f32,
}

impl Component for Animator {}

impl Animator {
    /// Create a new animator
    pub fn new(skeleton_id: u32, state_machine: AnimationStateMachine) -> Self {
        let default_state = state_machine.default_state.clone();
        Self {
            skeleton_id,
            state_machine,
            playback: PlaybackInfo {
                state: default_state,
                ..Default::default()
            },
            blend: None,
            parameters: HashMap::new(),
            triggers: Vec::new(),
            paused: false,
            time_in_state: 0.0,
        }
    }

    /// Set a parameter value
    pub fn set_parameter(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }

    /// Get a parameter value
    pub fn get_parameter(&self, name: &str) -> f32 {
        self.parameters.get(name).copied().unwrap_or(0.0)
    }

    /// Set a trigger
    pub fn set_trigger(&mut self, name: &str) {
        if !self.triggers.contains(&name.to_string()) {
            self.triggers.push(name.to_string());
        }
    }

    /// Check if trigger is set
    pub fn has_trigger(&self, name: &str) -> bool {
        self.triggers.contains(&name.to_string())
    }

    /// Clear a trigger
    pub fn clear_trigger(&mut self, name: &str) {
        self.triggers.retain(|t| t != name);
    }

    /// Clear all triggers
    pub fn clear_triggers(&mut self) {
        self.triggers.clear();
    }

    /// Force transition to a state
    pub fn play(&mut self, state_name: &str, blend_duration: f32) {
        if self.state_machine.get_state(state_name).is_some() {
            if blend_duration > 0.0 && !self.playback.state.is_empty() {
                // Start blend
                self.blend = Some(AnimationBlend {
                    from_state: self.playback.state.clone(),
                    from_time: self.playback.time,
                    to_state: state_name.to_string(),
                    to_time: 0.0,
                    duration: blend_duration,
                    elapsed: 0.0,
                });
            }
            self.playback.state = state_name.to_string();
            self.playback.time = 0.0;
            self.playback.finished = false;
            self.time_in_state = 0.0;
        }
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume playback
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Get current state
    pub fn current_state(&self) -> Option<&AnimationState> {
        self.state_machine.get_state(&self.playback.state)
    }

    /// Get current clip ID
    pub fn current_clip_id(&self) -> Option<ClipId> {
        self.current_state().map(|s| s.clip_id)
    }

    /// Check if a transition condition is met
    pub fn check_condition(
        &self,
        condition: &TransitionCondition,
        animation_duration: f32,
    ) -> bool {
        match condition {
            TransitionCondition::Always => true,
            TransitionCondition::OnAnimationEnd => {
                self.playback.finished || self.playback.time >= animation_duration
            }
            TransitionCondition::Parameter {
                name,
                comparison,
                value,
            } => {
                let param_value = self.get_parameter(name);
                comparison.evaluate(param_value, *value)
            }
            TransitionCondition::Trigger(name) => self.has_trigger(name),
            TransitionCondition::AfterTime(time) => self.time_in_state >= *time,
        }
    }

    /// Update the animator (call each frame with delta time)
    pub fn update(&mut self, dt: f32, get_clip_duration: impl Fn(ClipId) -> f32) {
        if self.paused {
            return;
        }

        self.time_in_state += dt;

        // Update blend if active
        if let Some(ref mut blend) = self.blend {
            blend.elapsed += dt;
            blend.from_time += dt;
            blend.to_time += dt;

            if blend.is_complete() {
                self.blend = None;
            }
        }

        // Update playback time
        if let Some(state) = self.state_machine.get_state(&self.playback.state) {
            let clip_duration = get_clip_duration(state.clip_id);
            self.playback.time += dt * state.speed * self.playback.speed;

            if state.looping && clip_duration > 0.0 {
                self.playback.time %= clip_duration;
            } else if self.playback.time >= clip_duration {
                self.playback.finished = true;
                self.playback.time = clip_duration;
            }
        }

        // Check for automatic transitions
        if let Some(state) = self.state_machine.get_state(&self.playback.state).cloned() {
            let transitions: Vec<_> = self
                .state_machine
                .get_transitions_from(&self.playback.state)
                .into_iter()
                .cloned()
                .collect();

            let clip_duration = get_clip_duration(state.clip_id);

            for transition in transitions {
                if self.check_condition(&transition.condition, clip_duration) {
                    // Consume trigger if that was the condition
                    if let TransitionCondition::Trigger(name) = &transition.condition {
                        self.clear_trigger(name);
                    }

                    self.play(&transition.to, transition.duration);
                    break;
                }
            }
        }
    }
}

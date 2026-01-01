//! Animation State Machine
//!
//! Defines states, transitions, and conditions for animation playback.

use super::clip::ClipId;
use std::collections::HashMap;

/// Animation state in the state machine
#[derive(Clone, Debug)]
pub struct AnimationState {
    /// State name
    pub name: String,
    /// Clip to play in this state
    pub clip_id: ClipId,
    /// Playback speed multiplier
    pub speed: f32,
    /// Whether to loop
    pub looping: bool,
    /// Time offset to start at
    pub start_time: f32,
}

impl AnimationState {
    /// Create a new state
    pub fn new(name: impl Into<String>, clip_id: ClipId) -> Self {
        Self {
            name: name.into(),
            clip_id,
            speed: 1.0,
            looping: true,
            start_time: 0.0,
        }
    }

    /// Set speed
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set looping
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Set start time
    pub fn with_start_time(mut self, start_time: f32) -> Self {
        self.start_time = start_time;
        self
    }
}

/// Condition for automatic state transitions
#[derive(Clone, Debug)]
pub enum TransitionCondition {
    /// Always transition (immediate)
    Always,
    /// Transition when animation ends
    OnAnimationEnd,
    /// Transition when parameter meets condition
    Parameter {
        name: String,
        comparison: Comparison,
        value: f32,
    },
    /// Transition when trigger is set
    Trigger(String),
    /// Transition after time
    AfterTime(f32),
}

/// Comparison operator for parameter conditions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl Comparison {
    /// Evaluate the comparison
    pub fn evaluate(&self, a: f32, b: f32) -> bool {
        const EPSILON: f32 = 0.0001;
        match self {
            Self::Equal => (a - b).abs() < EPSILON,
            Self::NotEqual => (a - b).abs() >= EPSILON,
            Self::LessThan => a < b,
            Self::LessOrEqual => a <= b,
            Self::GreaterThan => a > b,
            Self::GreaterOrEqual => a >= b,
        }
    }
}

/// Transition between states
#[derive(Clone, Debug)]
pub struct StateTransition {
    /// Source state name
    pub from: String,
    /// Target state name
    pub to: String,
    /// Transition blend duration
    pub duration: f32,
    /// Condition for the transition
    pub condition: TransitionCondition,
    /// Priority (higher = checked first)
    pub priority: i32,
}

impl StateTransition {
    /// Create a new transition
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            duration: 0.25,
            condition: TransitionCondition::OnAnimationEnd,
            priority: 0,
        }
    }

    /// Set blend duration
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Set condition
    pub fn with_condition(mut self, condition: TransitionCondition) -> Self {
        self.condition = condition;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

/// Animation state machine definition
#[derive(Clone, Debug, Default)]
pub struct AnimationStateMachine {
    /// All states
    pub states: HashMap<String, AnimationState>,
    /// All transitions
    pub transitions: Vec<StateTransition>,
    /// Default state name
    pub default_state: String,
}

impl AnimationStateMachine {
    /// Create a new empty state machine
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a state
    pub fn add_state(&mut self, state: AnimationState) {
        if self.default_state.is_empty() {
            self.default_state = state.name.clone();
        }
        self.states.insert(state.name.clone(), state);
    }

    /// Add a transition
    pub fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
        // Sort by priority (descending)
        self.transitions.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Set the default state
    pub fn set_default_state(&mut self, name: impl Into<String>) {
        self.default_state = name.into();
    }

    /// Get a state by name
    pub fn get_state(&self, name: &str) -> Option<&AnimationState> {
        self.states.get(name)
    }

    /// Get transitions from a state
    pub fn get_transitions_from(&self, state: &str) -> Vec<&StateTransition> {
        self.transitions
            .iter()
            .filter(|t| t.from == state)
            .collect()
    }
}

/// Builder for AnimationStateMachine
pub struct StateMachineBuilder {
    machine: AnimationStateMachine,
}

impl StateMachineBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            machine: AnimationStateMachine::new(),
        }
    }

    /// Add a state
    pub fn state(mut self, state: AnimationState) -> Self {
        self.machine.add_state(state);
        self
    }

    /// Add a transition
    pub fn transition(mut self, transition: StateTransition) -> Self {
        self.machine.add_transition(transition);
        self
    }

    /// Set default state
    pub fn default_state(mut self, name: impl Into<String>) -> Self {
        self.machine.set_default_state(name);
        self
    }

    /// Build the state machine
    pub fn build(self) -> AnimationStateMachine {
        self.machine
    }
}

impl Default for StateMachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

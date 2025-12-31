//! # AI Brain Component

use crate::core::Component;

/// AI state machine states
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AIState {
    #[default]
    Idle,
    Patrol,
    Chase,
    Attack,
    Flee,
    Search,
    Custom(u32),
}

/// Decision from AI system
#[derive(Clone, Debug, Default)]
pub enum Decision {
    #[default]
    None,
    MoveTo {
        target: glam::Vec3,
    },
    Attack {
        target_entity: crate::core::Entity,
    },
    Flee {
        from: glam::Vec3,
    },
    Patrol {
        waypoints: Vec<glam::Vec3>,
    },
    Custom(String),
}

/// AI brain component for intelligent agents
#[derive(Clone, Debug, Default)]
pub struct AIBrain {
    pub state: AIState,
    pub decision: Decision,
    pub think_interval: f32,
    pub time_since_think: f32,
    pub threat_level: f32,
}

impl AIBrain {
    pub fn new() -> Self {
        Self {
            think_interval: 0.1, // Think every 100ms
            ..Default::default()
        }
    }

    /// Check if it's time to make a new decision
    pub fn should_think(&self) -> bool {
        self.time_since_think >= self.think_interval
    }

    /// Reset think timer after making decision
    pub fn did_think(&mut self) {
        self.time_since_think = 0.0;
    }

    /// Update think timer
    pub fn update(&mut self, dt: f32) {
        self.time_since_think += dt;
    }

    /// Transition to new state
    pub fn set_state(&mut self, state: AIState) {
        if self.state != state {
            self.state = state;
            self.decision = Decision::None;
        }
    }
}

impl Component for AIBrain {}

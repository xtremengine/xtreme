//! # AI Perception System

use crate::core::{Component, Entity};
use glam::Vec3;

/// Sensory information about a perceived entity
#[derive(Clone, Debug)]
pub struct PerceivedEntity {
    pub entity: Entity,
    pub position: Vec3,
    pub velocity: Vec3,
    pub time_since_seen: f32,
    pub is_visible: bool,
}

/// Field of view sensor
#[derive(Clone, Debug)]
pub struct FieldOfView {
    pub range: f32,
    pub half_angle: f32, // radians
    pub height_tolerance: f32,
}

impl FieldOfView {
    pub fn new(range: f32, angle_degrees: f32) -> Self {
        Self {
            range,
            half_angle: (angle_degrees / 2.0).to_radians(),
            height_tolerance: 2.0,
        }
    }

    /// Check if a point is within the field of view
    pub fn can_see(&self, origin: Vec3, forward: Vec3, target: Vec3) -> bool {
        let to_target = target - origin;
        let distance = to_target.length();

        // Range check
        if distance > self.range {
            return false;
        }

        // Height check
        if to_target.y.abs() > self.height_tolerance {
            return false;
        }

        // Angle check (ignore Y for horizontal FOV)
        let forward_2d = Vec3::new(forward.x, 0.0, forward.z).normalize();
        let to_target_2d = Vec3::new(to_target.x, 0.0, to_target.z).normalize();

        let dot = forward_2d.dot(to_target_2d);
        let angle = dot.acos();

        angle <= self.half_angle
    }
}

impl Default for FieldOfView {
    fn default() -> Self {
        Self::new(20.0, 90.0)
    }
}

/// Sensor component for AI perception
#[derive(Clone, Debug, Default)]
pub struct Sensor {
    pub fov: FieldOfView,
    pub hearing_range: f32,
}

impl Sensor {
    pub fn new() -> Self {
        Self {
            fov: FieldOfView::default(),
            hearing_range: 10.0,
        }
    }

    pub fn with_vision(mut self, range: f32, angle: f32) -> Self {
        self.fov = FieldOfView::new(range, angle);
        self
    }

    pub fn with_hearing(mut self, range: f32) -> Self {
        self.hearing_range = range;
        self
    }
}

impl Component for Sensor {}

/// Memory of perceived entities
#[derive(Clone, Debug, Default)]
pub struct PerceptionMemory {
    pub perceived: Vec<PerceivedEntity>,
    pub memory_duration: f32,
}

impl PerceptionMemory {
    pub fn new() -> Self {
        Self {
            perceived: Vec::new(),
            memory_duration: 5.0, // Remember for 5 seconds
        }
    }

    /// Update memory (forget old perceptions)
    pub fn update(&mut self, dt: f32) {
        for p in &mut self.perceived {
            p.time_since_seen += dt;
        }
        self.perceived
            .retain(|p| p.time_since_seen < self.memory_duration);
    }

    /// Update or add a perception
    pub fn see(&mut self, entity: Entity, position: Vec3, velocity: Vec3) {
        if let Some(existing) = self.perceived.iter_mut().find(|p| p.entity == entity) {
            existing.position = position;
            existing.velocity = velocity;
            existing.time_since_seen = 0.0;
            existing.is_visible = true;
        } else {
            self.perceived.push(PerceivedEntity {
                entity,
                position,
                velocity,
                time_since_seen: 0.0,
                is_visible: true,
            });
        }
    }

    /// Mark entity as no longer visible
    pub fn lose_sight(&mut self, entity: Entity) {
        if let Some(p) = self.perceived.iter_mut().find(|p| p.entity == entity) {
            p.is_visible = false;
        }
    }

    /// Get the nearest visible entity
    pub fn nearest_visible(&self, from: Vec3) -> Option<&PerceivedEntity> {
        self.perceived
            .iter()
            .filter(|p| p.is_visible)
            .min_by(|a, b| {
                let dist_a = (a.position - from).length_squared();
                let dist_b = (b.position - from).length_squared();
                dist_a.partial_cmp(&dist_b).unwrap()
            })
    }
}

impl Component for PerceptionMemory {}

//! # Movement and Physics Integration

use glam::Vec3;
use crate::core::Component;

/// Velocity component
#[derive(Clone, Copy, Debug, Default)]
pub struct Velocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

impl Velocity {
    pub fn new(linear: Vec3) -> Self {
        Self {
            linear,
            angular: Vec3::ZERO,
        }
    }

    pub fn zero() -> Self {
        Self::default()
    }
}

impl Component for Velocity {}

/// Simple rigid body (no rotation physics yet)
#[derive(Clone, Copy, Debug)]
pub struct RigidBody {
    pub mass: f32,
    pub drag: f32,
    pub gravity_scale: f32,
    pub is_kinematic: bool,
}

impl RigidBody {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            drag: 0.0,
            gravity_scale: 1.0,
            is_kinematic: false,
        }
    }

    pub fn kinematic() -> Self {
        Self {
            mass: 1.0,
            drag: 0.0,
            gravity_scale: 0.0,
            is_kinematic: true,
        }
    }

    pub fn inverse_mass(&self) -> f32 {
        if self.is_kinematic || self.mass <= 0.0 {
            0.0
        } else {
            1.0 / self.mass
        }
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl Component for RigidBody {}

/// Integrate velocity to update position
pub fn integrate(position: &mut Vec3, velocity: &Velocity, dt: f32) {
    *position += velocity.linear * dt;
}

/// Apply gravity to velocity
pub fn apply_gravity(velocity: &mut Velocity, body: &RigidBody, gravity: Vec3, dt: f32) {
    if !body.is_kinematic {
        velocity.linear += gravity * body.gravity_scale * dt;
    }
}

/// Apply drag to velocity
pub fn apply_drag(velocity: &mut Velocity, body: &RigidBody, dt: f32) {
    if body.drag > 0.0 {
        let drag_factor = (1.0 - body.drag * dt).max(0.0);
        velocity.linear *= drag_factor;
    }
}

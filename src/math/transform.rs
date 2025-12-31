//! # Transform Component
//!
//! Represents position, rotation, and scale of an entity in 3D space.

use glam::{Mat4, Quat, Vec3};

use crate::core::Component;

/// Position in 3D space
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn from_vec3(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl Component for Position {}

/// Rotation as a quaternion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rotation {
    pub quat: Quat,
}

impl Rotation {
    pub fn identity() -> Self {
        Self { quat: Quat::IDENTITY }
    }

    pub fn from_euler(pitch: f32, yaw: f32, roll: f32) -> Self {
        Self {
            quat: Quat::from_euler(glam::EulerRot::XYZ, pitch, yaw, roll),
        }
    }

    pub fn from_axis_angle(axis: Vec3, angle: f32) -> Self {
        Self {
            quat: Quat::from_axis_angle(axis, angle),
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::identity()
    }
}

impl Component for Rotation {}

/// Scale in 3D space
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Scale {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn uniform(s: f32) -> Self {
        Self { x: s, y: s, z: s }
    }

    pub fn one() -> Self {
        Self::uniform(1.0)
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self::one()
    }
}

impl Component for Scale {}

/// Complete transform (position + rotation + scale)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    pub position: Position,
    pub rotation: Rotation,
    pub scale: Scale,
}

impl Transform {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_position(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: Position::new(x, y, z),
            ..Default::default()
        }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_position(x, y, z)
    }

    /// Build the 4x4 transformation matrix
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.to_vec3(),
            self.rotation.quat,
            self.position.to_vec3(),
        )
    }

    /// Get the forward direction vector
    pub fn forward(&self) -> Vec3 {
        self.rotation.quat * Vec3::NEG_Z
    }

    /// Get the right direction vector
    pub fn right(&self) -> Vec3 {
        self.rotation.quat * Vec3::X
    }

    /// Get the up direction vector
    pub fn up(&self) -> Vec3 {
        self.rotation.quat * Vec3::Y
    }

    /// Translate by a vector
    pub fn translate(&mut self, delta: Vec3) {
        self.position.x += delta.x;
        self.position.y += delta.y;
        self.position.z += delta.z;
    }

    /// Rotate by euler angles (radians)
    pub fn rotate_euler(&mut self, pitch: f32, yaw: f32, roll: f32) {
        let delta = Quat::from_euler(glam::EulerRot::XYZ, pitch, yaw, roll);
        self.rotation.quat = delta * self.rotation.quat;
    }

    /// Look at a target position
    pub fn look_at(&mut self, target: Vec3, _up: Vec3) {
        let forward = (target - self.position.to_vec3()).normalize();
        self.rotation.quat = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
    }
}

impl Component for Transform {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_matrix() {
        let t = Transform::from_xyz(1.0, 2.0, 3.0);
        let m = t.to_matrix();

        // Translation should be in the last column
        assert_eq!(m.w_axis.x, 1.0);
        assert_eq!(m.w_axis.y, 2.0);
        assert_eq!(m.w_axis.z, 3.0);
    }

    #[test]
    fn test_transform_directions() {
        let t = Transform::new();

        // Default transform should have standard directions
        assert!((t.forward() - Vec3::NEG_Z).length() < 0.001);
        assert!((t.right() - Vec3::X).length() < 0.001);
        assert!((t.up() - Vec3::Y).length() < 0.001);
    }
}

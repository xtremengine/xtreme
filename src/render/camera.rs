//! # Camera System
//!
//! Camera types including isometric camera.

use glam::{Mat4, Vec3};

/// Camera trait for different camera types
pub trait Camera {
    fn view_matrix(&self) -> Mat4;
    fn projection_matrix(&self) -> Mat4;
    fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }
}

/// Isometric camera with orthographic projection
#[derive(Clone)]
pub struct IsometricCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub aspect_ratio: f32,
}

impl IsometricCamera {
    pub fn new() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: 20.0,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: 30.0_f32.to_radians(),
            zoom: 10.0,
            aspect_ratio: 16.0 / 9.0,
        }
    }

    pub fn eye_position(&self) -> Vec3 {
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        self.target + Vec3::new(
            self.distance * cos_pitch * sin_yaw,
            self.distance * sin_pitch,
            self.distance * cos_pitch * cos_yaw,
        )
    }

    /// Get camera position (alias for eye_position)
    pub fn position(&self) -> Vec3 {
        self.eye_position()
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self) -> Mat4 {
        self.view_projection()
    }
}

impl Default for IsometricCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera for IsometricCamera {
    fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye_position(), self.target, Vec3::Y)
    }

    fn projection_matrix(&self) -> Mat4 {
        let half_width = self.zoom * self.aspect_ratio;
        let half_height = self.zoom;
        Mat4::orthographic_rh(-half_width, half_width, -half_height, half_height, 0.1, 1000.0)
    }
}

/// Camera controller for user input
pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.01,
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new()
    }
}

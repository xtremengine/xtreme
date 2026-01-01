//! Camera component for game objects.

use glam::{EulerRot, Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Camera projection type
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CameraProjection {
    /// Perspective projection (3D games)
    Perspective {
        /// Field of view in degrees
        fov: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },
    /// Orthographic projection (2D/isometric games)
    Orthographic {
        /// Half-height of the view
        size: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },
}

impl Default for CameraProjection {
    fn default() -> Self {
        CameraProjection::Perspective {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
        }
    }
}

/// Camera component that can be attached to any SceneObject
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraComponent {
    /// Whether this camera is the main/active camera
    pub is_main: bool,
    /// Camera projection settings
    pub projection: CameraProjection,
    /// Priority (higher = more important, for multiple cameras)
    pub priority: i32,
    /// Clear color (background)
    pub clear_color: [f32; 4],
}

impl Default for CameraComponent {
    fn default() -> Self {
        Self {
            is_main: true,
            projection: CameraProjection::default(),
            priority: 0,
            clear_color: [0.1, 0.1, 0.15, 1.0],
        }
    }
}

impl CameraComponent {
    /// Create a new perspective camera
    pub fn perspective(fov_degrees: f32) -> Self {
        Self {
            projection: CameraProjection::Perspective {
                fov: fov_degrees,
                near: 0.1,
                far: 1000.0,
            },
            ..Default::default()
        }
    }

    /// Create a new orthographic camera
    pub fn orthographic(size: f32) -> Self {
        Self {
            projection: CameraProjection::Orthographic {
                size,
                near: 0.1,
                far: 1000.0,
            },
            ..Default::default()
        }
    }

    /// Get projection matrix
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self.projection {
            CameraProjection::Perspective { fov, near, far } => {
                Mat4::perspective_rh(fov.to_radians(), aspect_ratio, near, far)
            }
            CameraProjection::Orthographic { size, near, far } => {
                let half_width = size * aspect_ratio;
                Mat4::orthographic_rh(-half_width, half_width, -size, size, near, far)
            }
        }
    }

    /// Get view matrix from object's world transform
    pub fn view_matrix(world_position: Vec3, world_rotation: Vec3) -> Mat4 {
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            world_rotation.x,
            world_rotation.y,
            world_rotation.z,
        );

        // Camera looks down -Z in its local space
        let forward = rotation * Vec3::NEG_Z;
        let up = rotation * Vec3::Y;
        let target = world_position + forward;

        Mat4::look_at_rh(world_position, target, up)
    }
}

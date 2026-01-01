//! Camera view projection calculation for game window.

use glam::{EulerRot, Mat4, Quat, Vec3};

use super::GameWindow;

/// Camera information for rendering
pub(crate) struct CameraInfo {
    pub view_proj: Mat4,
    pub position: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub clear_color: [f32; 4],
}

impl GameWindow {
    /// Get full camera information for rendering (particles, etc.)
    pub(crate) fn get_camera_info(&self) -> CameraInfo {
        // Find main camera entity (highest priority is_main camera)
        let main_camera = self
            .scene_objects
            .iter()
            .filter(|obj| obj.camera.as_ref().map(|c| c.is_main).unwrap_or(false))
            .max_by_key(|obj| obj.camera.as_ref().map(|c| c.priority).unwrap_or(0));

        if let Some(cam_obj) = main_camera {
            if let Some(ref camera) = cam_obj.camera {
                // Get world position and rotation
                let world_pos = cam_obj.world_position(&self.scene_objects);
                let world_rot = cam_obj.rotation;

                // Calculate view matrix
                let rotation =
                    Quat::from_euler(EulerRot::XYZ, world_rot.x, world_rot.y, world_rot.z);
                let forward = rotation * Vec3::NEG_Z;
                let up = rotation * Vec3::Y;
                let right = rotation * Vec3::X;
                let target = world_pos + forward;
                let view = Mat4::look_at_rh(world_pos, target, up);

                // Calculate projection matrix
                let proj = camera.projection_matrix(self.aspect_ratio);

                return CameraInfo {
                    view_proj: proj * view,
                    position: world_pos,
                    right,
                    up,
                    clear_color: camera.clear_color,
                };
            }
        }

        // Fallback: default camera looking at origin
        let position = Vec3::new(0.0, 10.0, -20.0);
        let view = Mat4::look_at_rh(position, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), self.aspect_ratio, 0.1, 1000.0);
        CameraInfo {
            view_proj: proj * view,
            position,
            right: Vec3::X,
            up: Vec3::Y,
            clear_color: [0.1, 0.1, 0.15, 1.0],
        }
    }
}

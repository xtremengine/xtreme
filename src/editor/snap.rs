//! # Transform Snapping
//!
//! Grid snapping for position, rotation, and scale transforms.

use glam::Vec3;

/// Snap settings for transform operations
#[derive(Clone, Debug)]
pub struct SnapSettings {
    /// Whether snapping is enabled
    pub enabled: bool,
    /// Grid size for position snapping (e.g., 0.5, 1.0, 2.0)
    pub grid_size: f32,
    /// Rotation snap in degrees (e.g., 15, 45, 90)
    pub rotation_snap: f32,
    /// Scale snap increment (e.g., 0.1, 0.25, 0.5)
    pub scale_snap: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            grid_size: 1.0,
            rotation_snap: 15.0,
            scale_snap: 0.25,
        }
    }
}

impl SnapSettings {
    /// Create new snap settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Snap a position to the grid
    pub fn snap_position(&self, pos: Vec3) -> Vec3 {
        if !self.enabled || self.grid_size <= 0.0 {
            return pos;
        }
        Vec3::new(
            snap_value(pos.x, self.grid_size),
            snap_value(pos.y, self.grid_size),
            snap_value(pos.z, self.grid_size),
        )
    }

    /// Snap rotation angles (in radians) to the rotation snap (in degrees)
    pub fn snap_rotation(&self, rot: Vec3) -> Vec3 {
        if !self.enabled || self.rotation_snap <= 0.0 {
            return rot;
        }
        let snap_rad = self.rotation_snap.to_radians();
        Vec3::new(
            snap_value(rot.x, snap_rad),
            snap_value(rot.y, snap_rad),
            snap_value(rot.z, snap_rad),
        )
    }

    /// Snap scale values to the scale snap increment
    pub fn snap_scale(&self, scale: Vec3) -> Vec3 {
        if !self.enabled || self.scale_snap <= 0.0 {
            return scale;
        }
        Vec3::new(
            snap_value(scale.x, self.scale_snap).max(self.scale_snap),
            snap_value(scale.y, self.scale_snap).max(self.scale_snap),
            snap_value(scale.z, self.scale_snap).max(self.scale_snap),
        )
    }

    /// Apply snap to a position delta (for relative movement)
    pub fn snap_delta(&self, delta: Vec3) -> Vec3 {
        if !self.enabled || self.grid_size <= 0.0 {
            return delta;
        }
        Vec3::new(
            snap_value(delta.x, self.grid_size),
            snap_value(delta.y, self.grid_size),
            snap_value(delta.z, self.grid_size),
        )
    }
}

/// Snap a single value to the nearest multiple of snap_size
fn snap_value(value: f32, snap_size: f32) -> f32 {
    (value / snap_size).round() * snap_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_position() {
        let settings = SnapSettings {
            enabled: true,
            grid_size: 1.0,
            ..Default::default()
        };

        let pos = Vec3::new(1.3, 2.7, -0.4);
        let snapped = settings.snap_position(pos);
        assert_eq!(snapped, Vec3::new(1.0, 3.0, 0.0));
    }

    #[test]
    fn test_snap_disabled() {
        let settings = SnapSettings {
            enabled: false,
            grid_size: 1.0,
            ..Default::default()
        };

        let pos = Vec3::new(1.3, 2.7, -0.4);
        let snapped = settings.snap_position(pos);
        assert_eq!(snapped, pos);
    }

    #[test]
    fn test_snap_rotation() {
        let settings = SnapSettings {
            enabled: true,
            rotation_snap: 45.0,
            ..Default::default()
        };

        let rot = Vec3::new(20.0_f32.to_radians(), 50.0_f32.to_radians(), 0.0);
        let snapped = settings.snap_rotation(rot);
        let expected = Vec3::new(0.0, 45.0_f32.to_radians(), 0.0);
        assert!((snapped.x - expected.x).abs() < 0.01);
        assert!((snapped.y - expected.y).abs() < 0.01);
    }

    #[test]
    fn test_snap_scale() {
        let settings = SnapSettings {
            enabled: true,
            scale_snap: 0.5,
            ..Default::default()
        };

        let scale = Vec3::new(1.3, 2.7, 0.1);
        let snapped = settings.snap_scale(scale);
        assert_eq!(snapped, Vec3::new(1.5, 2.5, 0.5)); // Min is 0.5
    }
}

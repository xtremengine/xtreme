//! Gizmo types, enums, and constants.

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// Gizmo axis/handle being interacted with
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GizmoAxis {
    /// No axis selected
    #[default]
    None,
    /// X axis (red)
    X,
    /// Y axis (green)
    Y,
    /// Z axis (blue)
    Z,
    /// XY plane
    XY,
    /// XZ plane
    XZ,
    /// YZ plane
    YZ,
    /// All axes / center (white)
    XYZ,
}

/// Gizmo operation mode
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GizmoMode {
    /// Translation gizmo (arrows)
    #[default]
    Translate,
    /// Rotation gizmo (rings)
    Rotate,
    /// Scale gizmo (cubes)
    Scale,
}

/// Vertex for gizmo rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GizmoVertex {
    /// Position
    pub position: [f32; 3],
    /// Color (RGBA)
    pub color: [f32; 4],
}

impl GizmoVertex {
    /// Create a new gizmo vertex
    pub fn new(position: Vec3, color: [f32; 4]) -> Self {
        Self {
            position: position.to_array(),
            color,
        }
    }
}

/// Delta from a gizmo drag operation
#[derive(Clone, Copy, Debug, Default)]
pub struct GizmoDelta {
    /// Translation delta
    pub translation: Vec3,
    /// Rotation delta (euler angles in radians)
    pub rotation: Vec3,
    /// Scale delta (multiplicative for scale mode)
    pub scale: Vec3,
}

/// Standard gizmo colors
pub mod colors {
    /// X axis (red)
    pub const X: [f32; 4] = [0.9, 0.2, 0.2, 1.0];
    /// Y axis (green)
    pub const Y: [f32; 4] = [0.2, 0.9, 0.2, 1.0];
    /// Z axis (blue)
    pub const Z: [f32; 4] = [0.2, 0.2, 0.9, 1.0];
    /// Hover highlight (yellow)
    pub const HOVER: [f32; 4] = [1.0, 0.9, 0.2, 1.0];
    /// Center/all axes (white)
    pub const CENTER: [f32; 4] = [0.9, 0.9, 0.9, 0.8];
    /// Plane XY (semi-transparent)
    pub const PLANE_XY: [f32; 4] = [0.9, 0.9, 0.2, 0.3];
    /// Plane XZ
    pub const PLANE_XZ: [f32; 4] = [0.9, 0.2, 0.9, 0.3];
    /// Plane YZ
    pub const PLANE_YZ: [f32; 4] = [0.2, 0.9, 0.9, 0.3];
}

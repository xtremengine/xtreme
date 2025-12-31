//! # Gizmo System
//!
//! Visual manipulation gizmos for translate, rotate, and scale operations.
//!
//! This module is split into:
//! - `types.rs` - Enums, structs, and color constants
//! - `geometry.rs` - Vertex generation for rendering
//! - `hit_test.rs` - Ray intersection tests
//! - `drag.rs` - Drag handling and delta calculation
//! - `math.rs` - Helper math functions

mod types;
mod geometry;
mod hit_test;
mod drag;
mod math;

pub use types::{GizmoAxis, GizmoMode, GizmoVertex, GizmoDelta, colors};

use glam::Vec3;

/// 3D Gizmo for object manipulation
pub struct Gizmo {
    /// Current mode
    pub(crate) mode: GizmoMode,
    /// Currently hovered axis
    pub(crate) hovered_axis: GizmoAxis,
    /// Currently active (dragging) axis
    pub(crate) active_axis: GizmoAxis,
    /// Is currently dragging
    pub(crate) is_dragging: bool,
    /// World position where drag started
    pub(crate) drag_start_world: Vec3,
    /// Object transform when drag started (position, rotation, scale)
    pub(crate) drag_start_transform: (Vec3, Vec3, Vec3),
    /// Accumulated drag delta
    pub(crate) drag_accumulated: Vec3,
}

impl Default for Gizmo {
    fn default() -> Self {
        Self::new()
    }
}

impl Gizmo {
    /// Create a new gizmo
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::Translate,
            hovered_axis: GizmoAxis::None,
            active_axis: GizmoAxis::None,
            is_dragging: false,
            drag_start_world: Vec3::ZERO,
            drag_start_transform: (Vec3::ZERO, Vec3::ZERO, Vec3::ONE),
            drag_accumulated: Vec3::ZERO,
        }
    }

    /// Get current mode
    pub fn mode(&self) -> GizmoMode {
        self.mode
    }

    /// Set gizmo mode
    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
    }

    /// Get hovered axis
    pub fn hovered_axis(&self) -> GizmoAxis {
        self.hovered_axis
    }

    /// Set hovered axis
    pub fn set_hovered(&mut self, axis: GizmoAxis) {
        self.hovered_axis = axis;
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Get active axis
    pub fn active_axis(&self) -> GizmoAxis {
        self.active_axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gizmo_creation() {
        let gizmo = Gizmo::new();
        assert_eq!(gizmo.mode(), GizmoMode::Translate);
        assert!(!gizmo.is_dragging());
    }

    #[test]
    fn test_vertex_generation() {
        let gizmo = Gizmo::new();
        let (lines, triangles) = gizmo.generate_vertices(Vec3::ZERO, 1.0);
        assert!(!lines.is_empty());
        assert!(!triangles.is_empty());
    }
}

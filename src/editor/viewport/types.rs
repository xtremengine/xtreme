//! Viewport types and constants.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// Grid uniforms
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GridUniforms {
    /// View-projection matrix
    pub view_proj: [[f32; 4]; 4],
    /// Camera position
    pub camera_pos: [f32; 3],
    /// Grid scale
    pub grid_scale: f32,
}

impl Default for GridUniforms {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0, 10.0, 10.0],
            grid_scale: 1.0,
        }
    }
}

/// Gizmo uniforms
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GizmoUniforms {
    /// View-projection matrix
    pub view_proj: [[f32; 4]; 4],
    /// Model matrix (translation to gizmo position)
    pub model: [[f32; 4]; 4],
}

impl Default for GizmoUniforms {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

/// Maximum gizmo vertices
pub const MAX_GIZMO_VERTICES: usize = 4096;

/// Maximum number of objects we can render
pub const MAX_OBJECTS: usize = 256;

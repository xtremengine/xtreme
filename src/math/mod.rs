//! # Math Module
//!
//! Transform components and coordinate utilities.
//!
//! ## Features
//!
//! - Transform (position, rotation, scale)
//! - Isometric coordinate conversion
//! - Screen-to-world picking

pub mod isometric;
pub mod transform;

pub use isometric::{screen_to_world, world_to_screen, IsometricConfig, IsometricCoord};
pub use transform::{Position, Rotation, Scale, Transform};

// Re-export glam types for convenience
pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};

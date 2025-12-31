//! # Math Module
//!
//! Transform components and coordinate utilities.
//!
//! ## Features
//!
//! - Transform (position, rotation, scale)
//! - Isometric coordinate conversion
//! - Screen-to-world picking

pub mod transform;
pub mod isometric;

pub use transform::{Transform, Position, Rotation, Scale};
pub use isometric::{IsometricCoord, IsometricConfig, world_to_screen, screen_to_world};

// Re-export glam types for convenience
pub use glam::{Vec2, Vec3, Vec4, Mat4, Quat};

//! # Xtreme Engine
//!
//! A modular ECS game engine with isometric 3D rendering and ML-powered AI.
//!
//! ## Modules
//!
//! - `core` - Entity Component System (ECS) implementation
//! - `render` - WGPU-based 3D rendering with isometric camera
//! - `input` - Keyboard and mouse input handling
//! - `physics` - Collision detection and movement
//! - `ai` - Machine learning inference and pathfinding
//! - `math` - Transform and coordinate utilities
//! - `utils` - Common utilities (pools, timers)
//!
//! ## Example
//!
//! ```rust,ignore
//! use xtreme::prelude::*;
//!
//! fn main() {
//!     let mut world = World::new();
//!
//!     // Spawn an entity with components
//!     world.spawn((
//!         Transform::default(),
//!         Mesh::cube(1.0),
//!         Material::default(),
//!     ));
//!
//!     // Run systems
//!     world.run();
//! }
//! ```

pub mod core;
pub mod render;
pub mod input;
pub mod physics;
pub mod ai;
pub mod math;
pub mod utils;
pub mod editor;
pub mod scripting;

/// Prelude module - commonly used types
pub mod prelude {
    pub use crate::core::{World, Entity, Component};
    pub use crate::math::Transform;
    pub use crate::render::{Mesh, Material, Camera};
}

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

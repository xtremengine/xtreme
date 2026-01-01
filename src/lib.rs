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

pub mod ai;
pub mod animation;
pub mod audio;
pub mod core;
pub mod editor;
pub mod input;
pub mod math;
pub mod particles;
pub mod physics;
pub mod render;
pub mod scripting;
pub mod utils;

/// Prelude module - commonly used types
pub mod prelude {
    // Core ECS
    pub use crate::core::{Component, Entity, World};

    // Math
    pub use crate::math::Transform;

    // Rendering
    pub use crate::render::{Camera, Material, Mesh};

    // Audio
    pub use crate::audio::{AudioListener, AudioManager, AudioSource};

    // Animation
    pub use crate::animation::{AnimationClip, AnimationLibrary, Animator, Skeleton, SkeletonPose};

    // Particles
    pub use crate::particles::{EmitterConfig, ParticleEmitter, ParticleManager};
}

/// Engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

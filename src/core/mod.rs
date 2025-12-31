#![allow(dead_code)]
//! # Core ECS Module
//!
//! Entity Component System implementation with data-oriented design.
//!
//! ## Architecture
//!
//! - **Entity**: Unique identifier (u64 = index + generation)
//! - **Component**: Pure data structs stored in SparseSet
//! - **System**: Logic that operates on component queries
//! - **World**: Container that manages entities and components
//!
//! ## Example
//!
//! ```rust,ignore
//! use xtreme::core::*;
//!
//! // Define a component
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//!
//! // Create world and spawn entity
//! let mut world = World::new();
//! let entity = world.spawn((Position { x: 0.0, y: 0.0 },));
//!
//! // Query components
//! for pos in world.query::<&mut Position>() {
//!     pos.x += 1.0;
//! }
//! ```

mod archetype;
mod component;
mod entity;
mod query;
mod system;
mod world;

pub use archetype::{Archetype, ArchetypeId};
pub use component::{Component, ComponentStorage, SparseSet};
pub use entity::{Entity, EntityId, Generation};
pub use query::{Query, QueryIter};
pub use system::{Stage, System, SystemScheduler};
pub use world::World;

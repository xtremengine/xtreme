#![allow(dead_code)]
//! # Physics Module
//!
//! Collision detection and physics simulation.
//!
//! ## Features
//!
//! - AABB and OBB collision shapes
//! - Spatial grid for broadphase optimization
//! - Raycast queries
//! - Simple movement with collision response

mod collision;
mod movement;
mod raycast;
mod shapes;
mod spatial;

pub use collision::{CollisionPair, CollisionWorld, Contact};
pub use movement::{integrate, RigidBody, Velocity};
pub use raycast::{Ray, RaycastHit, RaycastQuery};
pub use shapes::{Shape, Sphere, AABB, OBB};
pub use spatial::{GridCell, SpatialGrid};

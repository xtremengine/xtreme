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

mod shapes;
mod collision;
mod spatial;
mod raycast;
mod movement;

pub use shapes::{AABB, Sphere, OBB, Shape};
pub use collision::{CollisionWorld, Contact, CollisionPair};
pub use spatial::{SpatialGrid, GridCell};
pub use raycast::{Ray, RaycastHit, RaycastQuery};
pub use movement::{Velocity, RigidBody, integrate};

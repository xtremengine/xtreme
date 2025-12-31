//! # Utils Module
//!
//! Common utilities used across the engine.
//!
//! ## Features
//!
//! - Object pool for reducing allocations
//! - Timer and delta time tracking
//! - Fixed timestep utilities

mod pool;
mod timer;

pub use pool::{Handle, Pool};
pub use timer::{DeltaTime, FixedTimestep, Timer};

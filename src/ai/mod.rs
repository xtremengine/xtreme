//! # AI Module
//!
//! Machine learning inference and pathfinding for game AI.
//!
//! ## Features
//!
//! - ONNX model inference for decision making
//! - A* pathfinding on grids
//! - Perception system (field of view, sensors)
//! - AI brain component for agents
//!
//! ## ML Pipeline
//!
//! ```text
//! Training (Python)          Runtime (Rust)
//! ┌──────────────┐          ┌──────────────┐
//! │ PyTorch/SB3  │ ──ONNX──▶│ ort runtime  │
//! │ PPO/SAC      │          │ < 1ms infer  │
//! └──────────────┘          └──────────────┘
//! ```

mod brain;
mod perception;
mod pathfinding;
mod navmesh;
#[cfg(feature = "ml")]
mod inference;

pub use brain::{AIBrain, AIState, Decision};
pub use perception::{Sensor, FieldOfView, PerceptionMemory};
pub use pathfinding::{AStar, Grid, Path, PathNode};
pub use navmesh::{NavMesh, NavAgent};

#[cfg(feature = "ml")]
pub use inference::{OnnxModel, ModelInput, ModelOutput};

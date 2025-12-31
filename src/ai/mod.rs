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
#[cfg(feature = "ml")]
mod inference;
mod navmesh;
mod pathfinding;
mod perception;

pub use brain::{AIBrain, AIState, Decision};
pub use navmesh::{NavAgent, NavMesh};
pub use pathfinding::{AStar, Grid, Path, PathNode};
pub use perception::{FieldOfView, PerceptionMemory, Sensor};

#[cfg(feature = "ml")]
pub use inference::{ModelInput, ModelOutput, OnnxModel};

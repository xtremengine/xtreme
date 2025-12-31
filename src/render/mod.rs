//! # Render Module
//!
//! WGPU-based 3D rendering with isometric camera support.
//!
//! ## Features
//!
//! - Isometric camera with orthographic projection
//! - Mesh rendering (primitives and loaded models)
//! - Material and shader system
//! - Instanced rendering for performance
//!
//! ## Architecture
//!
//! ```text
//! RenderContext (WGPU device, queue, surface)
//!     └── RenderPipeline
//!             ├── Shaders
//!             ├── Vertex layouts
//!             └── Bind groups (uniforms, textures)
//! ```

mod context;
mod window;
mod camera;
mod vertex;
mod mesh;
mod material;
mod texture;
mod pipeline;
mod egui_integration;

pub use context::{RenderContext, RenderContextBuilder, RenderError};
pub use window::{Window, WindowConfig, App, AppEvent, run};
pub use camera::{Camera, IsometricCamera, CameraController};
pub use vertex::{Vertex, VertexLayout};
pub use mesh::{Mesh, MeshBuilder, Primitive, GpuMesh};
pub use material::{Material, Shader};
pub use texture::{Texture, Sampler};
pub use pipeline::{RenderPipeline, RenderPass, Uniforms};
pub use egui_integration::EguiIntegration;

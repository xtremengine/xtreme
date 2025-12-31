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

mod camera;
mod context;
mod egui_integration;
mod material;
mod mesh;
mod pipeline;
mod texture;
mod vertex;
mod window;

pub use camera::{Camera, CameraController, IsometricCamera};
pub use context::{RenderContext, RenderContextBuilder, RenderError};
pub use egui_integration::EguiIntegration;
pub use material::{Material, MaterialError, MaterialId, MaterialManager, MaterialUniform, Shader};
pub use mesh::{GpuMesh, Mesh, MeshBuilder, Primitive};
pub use pipeline::{RenderPass, RenderPipeline, Uniforms};
pub use texture::{SamplerConfig, Texture, TextureError, TextureId};
pub use vertex::{Vertex, VertexLayout};
pub use window::{run, App, AppEvent, Window, WindowConfig};

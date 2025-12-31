//! # Viewport
//!
//! 3D viewport with render-to-texture for egui integration.
//!
//! This module is split into:
//! - `types.rs` - Uniforms and constants
//! - `textures.rs` - Texture creation helpers
//! - `pipelines.rs` - Pipeline creation helpers
//! - `render.rs` - Rendering methods
//! - `shader_cache.rs` - Custom shader caching

mod pipelines;
mod render;
mod shader_cache;
mod textures;
mod types;

pub use shader_cache::ShaderCache;
#[allow(unused_imports)]
pub use types::{GizmoUniforms, GridUniforms};

use glam::Mat4;
use std::collections::HashMap;

use crate::render::{GpuMesh, Mesh, RenderContext, Texture, Uniforms};
use pipelines::{
    create_gizmo_pipeline, create_grid_pipeline, create_mesh_pipeline, create_textured_pipeline,
};
use textures::{create_depth_texture, create_render_texture};
use types::{MAX_GIZMO_VERTICES, MAX_OBJECTS};

/// Render data for an object
#[derive(Clone)]
pub struct ObjectRenderData {
    /// Model matrix (world space)
    pub model: Mat4,
    /// Object color (RGBA)
    pub color: [f32; 4],
    /// Optional texture path
    pub texture_path: Option<String>,
    /// Optional shader path (.wgsl file)
    pub shader_path: Option<String>,
}

/// Cached texture data with bind group
pub struct CachedTexture {
    /// The loaded texture (kept alive for bind_group reference)
    #[allow(dead_code)]
    pub texture: Texture,
    /// Bind group for this texture
    pub bind_group: wgpu::BindGroup,
}

/// 3D viewport for the editor
pub struct Viewport {
    /// Viewport size
    pub(crate) size: (u32, u32),
    /// Render texture
    pub(crate) render_texture: wgpu::Texture,
    /// Render texture view
    pub(crate) render_view: wgpu::TextureView,
    /// Depth texture
    pub(crate) depth_texture: wgpu::Texture,
    /// Depth texture view
    pub(crate) depth_view: wgpu::TextureView,
    /// Egui texture ID
    pub(crate) egui_texture_id: egui::TextureId,
    /// Grid pipeline
    pub(crate) grid_pipeline: wgpu::RenderPipeline,
    /// Grid uniform buffer
    pub(crate) grid_uniform_buffer: wgpu::Buffer,
    /// Grid bind group
    pub(crate) grid_bind_group: wgpu::BindGroup,
    /// Mesh pipeline (non-textured)
    pub(crate) mesh_pipeline: wgpu::RenderPipeline,
    /// Mesh uniform buffer (large enough for MAX_OBJECTS)
    pub(crate) mesh_uniform_buffer: wgpu::Buffer,
    /// Mesh bind group
    pub(crate) mesh_bind_group: wgpu::BindGroup,
    /// Uniform alignment requirement
    pub(crate) uniform_alignment: u32,
    /// Test cube mesh
    pub(crate) cube_mesh: GpuMesh,
    /// Gizmo line pipeline
    pub(crate) gizmo_line_pipeline: wgpu::RenderPipeline,
    /// Gizmo triangle pipeline
    pub(crate) gizmo_tri_pipeline: wgpu::RenderPipeline,
    /// Gizmo vertex buffer (lines)
    pub(crate) gizmo_line_buffer: wgpu::Buffer,
    /// Gizmo vertex buffer (triangles)
    pub(crate) gizmo_tri_buffer: wgpu::Buffer,
    /// Gizmo uniform buffer
    pub(crate) gizmo_uniform_buffer: wgpu::Buffer,
    /// Gizmo bind group
    pub(crate) gizmo_bind_group: wgpu::BindGroup,
    /// Textured mesh pipeline
    pub(crate) textured_pipeline: wgpu::RenderPipeline,
    /// Texture bind group layout
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Default white texture bind group (fallback for failed texture loads)
    #[allow(dead_code)]
    pub(crate) default_texture_bind_group: wgpu::BindGroup,
    /// Texture cache: path -> cached texture
    pub(crate) texture_cache: HashMap<String, CachedTexture>,
    /// Shader cache for custom shaders
    pub(crate) shader_cache: ShaderCache,
    /// Mesh bind group layout (for shader cache)
    #[allow(dead_code)]
    pub(crate) mesh_bind_group_layout: wgpu::BindGroupLayout,
}

impl Viewport {
    /// Create a new viewport
    pub fn new(
        ctx: &RenderContext,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) -> Self {
        let device = &ctx.device;

        // Create render and depth textures
        let (render_texture, render_view) = create_render_texture(device, width, height);
        let (depth_texture, depth_view) = create_depth_texture(device, width, height);

        // Register texture with egui
        let egui_texture_id =
            egui_renderer.register_native_texture(device, &render_view, wgpu::FilterMode::Linear);

        // Create pipelines
        let grid = create_grid_pipeline(device);
        let mesh = create_mesh_pipeline(device, MAX_OBJECTS);
        let gizmo = create_gizmo_pipeline(device, MAX_GIZMO_VERTICES);
        let textured = create_textured_pipeline(device, &ctx.queue, &mesh.bind_group_layout);

        // Create test cube
        let cube_mesh = GpuMesh::from_mesh(device, &Mesh::cube(1.0));

        // Create shader cache
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let shader_cache_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shader Cache Mesh Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(uniform_size as u64),
                    },
                    count: None,
                }],
            });

        let shader_cache = ShaderCache::new(
            device,
            shader_cache_layout,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        Self {
            size: (width, height),
            render_texture,
            render_view,
            depth_texture,
            depth_view,
            egui_texture_id,
            grid_pipeline: grid.pipeline,
            grid_uniform_buffer: grid.uniform_buffer,
            grid_bind_group: grid.bind_group,
            mesh_pipeline: mesh.pipeline,
            mesh_uniform_buffer: mesh.uniform_buffer,
            mesh_bind_group: mesh.bind_group,
            uniform_alignment: mesh.uniform_alignment,
            cube_mesh,
            gizmo_line_pipeline: gizmo.line_pipeline,
            gizmo_tri_pipeline: gizmo.tri_pipeline,
            gizmo_line_buffer: gizmo.line_buffer,
            gizmo_tri_buffer: gizmo.tri_buffer,
            gizmo_uniform_buffer: gizmo.uniform_buffer,
            gizmo_bind_group: gizmo.bind_group,
            textured_pipeline: textured.pipeline,
            texture_bind_group_layout: textured.texture_bind_group_layout,
            default_texture_bind_group: textured.default_texture_bind_group,
            texture_cache: HashMap::new(),
            shader_cache,
            mesh_bind_group_layout: mesh.bind_group_layout,
        }
    }

    /// Get or load a texture from the cache
    pub fn get_or_load_texture(
        &mut self,
        ctx: &RenderContext,
        path: &str,
    ) -> Option<&wgpu::BindGroup> {
        // Return cached if exists
        if self.texture_cache.contains_key(path) {
            return self.texture_cache.get(path).map(|c| &c.bind_group);
        }

        // Try to load texture
        match Texture::from_file(&ctx.device, &ctx.queue, path) {
            Ok(texture) => {
                let bind_group =
                    texture.create_bind_group(&ctx.device, &self.texture_bind_group_layout);
                self.texture_cache.insert(
                    path.to_string(),
                    CachedTexture {
                        texture,
                        bind_group,
                    },
                );
                log::info!("Loaded texture: {}", path);
                self.texture_cache.get(path).map(|c| &c.bind_group)
            }
            Err(e) => {
                log::error!("Failed to load texture {}: {}", path, e);
                None
            }
        }
    }

    /// Clear the texture cache
    pub fn clear_texture_cache(&mut self) {
        self.texture_cache.clear();
    }

    /// Resize the viewport
    pub fn resize(
        &mut self,
        ctx: &RenderContext,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) {
        if width == 0 || height == 0 || (width == self.size.0 && height == self.size.1) {
            return;
        }

        self.size = (width, height);

        // Recreate textures
        let (render_texture, render_view) = create_render_texture(&ctx.device, width, height);
        let (depth_texture, depth_view) = create_depth_texture(&ctx.device, width, height);

        self.render_texture = render_texture;
        self.render_view = render_view;
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        // Update egui texture
        egui_renderer.update_egui_texture_from_wgpu_texture(
            &ctx.device,
            &self.render_view,
            wgpu::FilterMode::Linear,
            self.egui_texture_id,
        );
    }

    /// Get the egui texture ID for this viewport
    pub fn texture_id(&self) -> egui::TextureId {
        self.egui_texture_id
    }

    /// Get viewport size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

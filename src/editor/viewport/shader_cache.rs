//! Shader cache for dynamic shader compilation.
//!
//! Allows objects to use custom shaders by compiling them at runtime.

use std::collections::HashMap;
use std::fs;

use crate::render::Texture;

/// Cached shader pipeline
pub struct CachedShader {
    /// The compiled render pipeline
    pub pipeline: wgpu::RenderPipeline,
    /// Whether this shader uses textures
    pub uses_texture: bool,
}

/// Cache for dynamically compiled shaders
pub struct ShaderCache {
    /// Compiled pipelines by shader path
    pipelines: HashMap<String, CachedShader>,
    /// Mesh bind group layout (shared with viewport)
    mesh_bind_group_layout: wgpu::BindGroupLayout,
    /// Texture bind group layout (shared with viewport)
    texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Surface format for render targets
    surface_format: wgpu::TextureFormat,
    /// Errors during compilation (path -> error message)
    errors: HashMap<String, String>,
}

impl ShaderCache {
    /// Create a new shader cache
    pub fn new(
        device: &wgpu::Device,
        mesh_bind_group_layout: wgpu::BindGroupLayout,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        // Create texture bind group layout
        let texture_bind_group_layout = Texture::bind_group_layout(device);

        Self {
            pipelines: HashMap::new(),
            mesh_bind_group_layout,
            texture_bind_group_layout,
            surface_format,
            errors: HashMap::new(),
        }
    }

    /// Get a cached pipeline or compile a new one
    pub fn get_or_compile(&mut self, device: &wgpu::Device, path: &str) -> Option<&CachedShader> {
        // Return cached if exists
        if self.pipelines.contains_key(path) {
            return self.pipelines.get(path);
        }

        // Check if we already failed to compile this shader
        if self.errors.contains_key(path) {
            return None;
        }

        // Try to compile the shader
        match self.compile_shader(device, path) {
            Ok(cached) => {
                log::info!("Compiled custom shader: {}", path);
                self.pipelines.insert(path.to_string(), cached);
                self.pipelines.get(path)
            }
            Err(e) => {
                log::error!("Failed to compile shader {}: {}", path, e);
                self.errors.insert(path.to_string(), e);
                None
            }
        }
    }

    /// Compile a shader from file
    fn compile_shader(&self, device: &wgpu::Device, path: &str) -> Result<CachedShader, String> {
        // Read shader source from file
        let shader_source =
            fs::read_to_string(path).map_err(|e| format!("Failed to read shader file: {}", e))?;

        // Check if shader uses textures (simple heuristic)
        let uses_texture = shader_source.contains("texture_2d")
            || shader_source.contains("t_diffuse")
            || shader_source.contains("@group(1)");

        // Create shader module
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(path),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        // Create pipeline layout based on whether shader uses textures
        let pipeline_layout = if uses_texture {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", path)),
                bind_group_layouts: &[
                    &self.mesh_bind_group_layout,
                    &self.texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            })
        } else {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", path)),
                bind_group_layouts: &[&self.mesh_bind_group_layout],
                push_constant_ranges: &[],
            })
        };

        // Create vertex buffer layout (must match Vertex struct)
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::render::Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3, // normal
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2, // uv
                },
            ],
        };

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{} Pipeline", path)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(CachedShader {
            pipeline,
            uses_texture,
        })
    }

    /// Clear the shader cache (forces recompilation)
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.errors.clear();
    }

    /// Reload a specific shader
    #[allow(dead_code)]
    pub fn reload(&mut self, path: &str) {
        self.pipelines.remove(path);
        self.errors.remove(path);
    }

    /// Check if a shader has compilation errors
    #[allow(dead_code)]
    pub fn get_error(&self, path: &str) -> Option<&str> {
        self.errors.get(path).map(|s| s.as_str())
    }

    /// Get the texture bind group layout for creating bind groups
    #[allow(dead_code)]
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }
}

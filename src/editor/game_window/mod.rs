//! # Game Window
//!
//! Separate window for running the game in play mode.

mod camera;
mod render;
mod splash;

use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;
use winit::window::WindowAttributes;

use crate::editor::selection::SceneObject;
use crate::render::{EguiIntegration, GpuMesh, Mesh, RenderContext, Texture, Uniforms};

/// Cached texture for game window
pub(crate) struct GameTexture {
    #[allow(dead_code)]
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
}

/// Maximum number of objects that can be rendered
pub(crate) const MAX_OBJECTS: usize = 1024;

/// Game window settings from project config
#[derive(Clone, Debug)]
pub struct GameSettings {
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Splash duration in seconds
    pub splash_duration: f32,
    /// Background color (RGB 0-255)
    pub background_color: [u8; 3],
    /// Show FPS counter
    pub show_fps: bool,
    /// Enable VSync
    pub vsync: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            splash_duration: 2.0,
            background_color: [25, 25, 38],
            show_fps: false,
            vsync: true,
        }
    }
}

/// Game window for play mode
pub struct GameWindow {
    /// Window handle
    pub window: Arc<Window>,
    /// Render context
    pub(crate) ctx: RenderContext,
    /// Aspect ratio
    pub(crate) aspect_ratio: f32,
    /// Mesh pipeline (no texture)
    pub(crate) mesh_pipeline: wgpu::RenderPipeline,
    /// Textured mesh pipeline
    pub(crate) textured_pipeline: wgpu::RenderPipeline,
    /// Mesh bind group layout
    #[allow(dead_code)]
    pub(crate) mesh_bind_group_layout: wgpu::BindGroupLayout,
    /// Texture bind group layout
    #[allow(dead_code)]
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Mesh uniform buffer
    pub(crate) mesh_uniform_buffer: wgpu::Buffer,
    /// Mesh bind group
    pub(crate) mesh_bind_group: wgpu::BindGroup,
    /// Uniform alignment
    pub(crate) uniform_alignment: u32,
    /// Cube mesh
    pub(crate) cube_mesh: GpuMesh,
    /// Depth texture
    pub(crate) depth_texture: wgpu::Texture,
    /// Depth view
    pub(crate) depth_view: wgpu::TextureView,
    /// Is window open
    pub is_open: bool,
    /// Scene objects snapshot
    pub(crate) scene_objects: Vec<SceneObject>,
    /// Play time
    pub(crate) play_time: f32,
    /// Last frame instant
    pub(crate) last_frame_instant: Option<std::time::Instant>,
    /// Game settings
    pub(crate) settings: GameSettings,
    /// Whether splash screen is showing
    pub(crate) in_splash: bool,
    /// Egui integration for overlays (splash, FPS)
    pub(crate) egui: EguiIntegration,
    /// Splash texture handle
    pub(crate) splash_texture: Option<egui::TextureHandle>,
    /// FPS accumulator for smoothing
    pub(crate) fps_samples: Vec<f32>,
    /// Current smoothed FPS
    pub(crate) current_fps: f32,
    /// Texture cache
    pub(crate) texture_cache: HashMap<String, GameTexture>,
}

impl GameWindow {
    /// Create a new game window
    pub async fn new(
        event_loop: &ActiveEventLoop,
        scene_objects: Vec<SceneObject>,
        settings: GameSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create window
        let window_attrs = WindowAttributes::default()
            .with_title("Xtreme Engine - Game")
            .with_inner_size(winit::dpi::LogicalSize::new(
                settings.window_width,
                settings.window_height,
            ));

        let window = Arc::new(event_loop.create_window(window_attrs)?);

        // Create render context with vsync setting
        let ctx = RenderContext::new_with_vsync(window.clone(), settings.vsync).await?;

        // Create mesh shader
        let mesh_shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Game Mesh Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/basic.wgsl").into()),
            });

        // Uniform alignment
        let uniform_alignment = ctx.device.limits().min_uniform_buffer_offset_alignment;
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = uniform_size.div_ceil(uniform_alignment) * uniform_alignment;

        // Create uniform buffer
        let mesh_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Game Mesh Uniform Buffer"),
            size: (aligned_size as usize * MAX_OBJECTS) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let mesh_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Game Mesh Bind Group Layout"),
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

        // Create bind group
        let mesh_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game Mesh Bind Group"),
            layout: &mesh_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &mesh_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(aligned_size as u64),
                }),
            }],
        });

        // Create depth texture
        let (width, height) = ctx.size();
        let depth_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create pipeline layout
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Game Mesh Pipeline Layout"),
                bind_group_layouts: &[&mesh_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Define vertex buffer layout
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<crate::render::Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        // Create mesh pipeline
        let mesh_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Game Mesh Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &mesh_shader,
                    entry_point: Some("vs_main"),
                    buffers: std::slice::from_ref(&vertex_buffer_layout),
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &mesh_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.format(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
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

        // Create textured shader
        let textured_shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Game Textured Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("../../shaders/textured_simple.wgsl").into(),
                ),
            });

        // Create texture bind group layout
        let texture_bind_group_layout = Texture::bind_group_layout(&ctx.device);

        // Create textured pipeline layout
        let textured_pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Game Textured Pipeline Layout"),
                bind_group_layouts: &[&mesh_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Create textured pipeline
        let textured_pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Game Textured Pipeline"),
                layout: Some(&textured_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &textured_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_buffer_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &textured_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.format(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
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

        // Create cube mesh
        let cube = Mesh::cube(1.0);
        let cube_mesh = GpuMesh::from_mesh(&ctx.device, &cube);

        // Calculate aspect ratio
        let (width, height) = ctx.size();
        let aspect_ratio = width as f32 / height as f32;

        let in_splash = settings.splash_duration > 0.0;

        // Initialize egui for overlays (splash screen, FPS)
        let mut egui = EguiIntegration::new(&ctx, &window);

        // Load splash screen texture if splash is enabled
        let splash_texture = if in_splash {
            splash::load_splash_texture(&mut egui)
        } else {
            None
        };

        // Preload textures for scene objects
        let mut texture_cache = HashMap::new();
        for obj in &scene_objects {
            if let Some(ref path) = obj.texture_path {
                if !texture_cache.contains_key(path) {
                    if let Ok(texture) = Texture::from_file(&ctx.device, &ctx.queue, path) {
                        let bind_group =
                            texture.create_bind_group(&ctx.device, &texture_bind_group_layout);
                        texture_cache.insert(path.clone(), GameTexture { texture, bind_group });
                        log::info!("Game: Loaded texture {}", path);
                    } else {
                        log::warn!("Game: Failed to load texture {}", path);
                    }
                }
            }
        }

        Ok(Self {
            window,
            ctx,
            aspect_ratio,
            mesh_pipeline,
            textured_pipeline,
            mesh_bind_group_layout,
            texture_bind_group_layout,
            mesh_uniform_buffer,
            mesh_bind_group,
            uniform_alignment,
            cube_mesh,
            depth_texture,
            depth_view,
            is_open: true,
            scene_objects,
            play_time: 0.0,
            last_frame_instant: Some(std::time::Instant::now()),
            settings,
            in_splash,
            egui,
            splash_texture,
            fps_samples: Vec::with_capacity(60),
            current_fps: 60.0,
            texture_cache,
        })
    }

    /// Get window ID
    pub fn id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    /// Handle resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.resize(new_size.width, new_size.height);

            // Update aspect ratio
            self.aspect_ratio = new_size.width as f32 / new_size.height as f32;

            // Recreate depth texture
            self.depth_texture = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Game Depth Texture"),
                size: wgpu::Extent3d {
                    width: new_size.width,
                    height: new_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.depth_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    /// Update game state, returns delta time for script execution
    pub fn update(&mut self) -> f32 {
        // Calculate delta time
        let now = std::time::Instant::now();
        let delta = if let Some(last) = self.last_frame_instant {
            now.duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.last_frame_instant = Some(now);
        self.play_time += delta;

        // Calculate FPS with smoothing
        if self.settings.show_fps && delta > 0.0 {
            let instant_fps = 1.0 / delta;
            self.fps_samples.push(instant_fps);
            if self.fps_samples.len() > 30 {
                self.fps_samples.remove(0);
            }
            self.current_fps = self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
        }

        delta
    }

    /// Get mutable scene objects for script updates
    #[cfg_attr(not(feature = "scripting"), allow(dead_code))]
    pub fn scene_objects_mut(&mut self) -> &mut Vec<SceneObject> {
        &mut self.scene_objects
    }

    /// Get scene objects (immutable)
    #[cfg_attr(not(feature = "scripting"), allow(dead_code))]
    pub fn scene_objects(&self) -> &Vec<SceneObject> {
        &self.scene_objects
    }

    /// Get play time
    #[cfg_attr(not(feature = "scripting"), allow(dead_code))]
    pub fn play_time(&self) -> f32 {
        self.play_time
    }

    /// Close the window
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

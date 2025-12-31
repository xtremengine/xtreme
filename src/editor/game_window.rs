//! # Game Window
//!
//! Separate window for running the game in play mode.

use std::sync::Arc;
use winit::window::Window;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use crate::render::{IsometricCamera, RenderContext, Uniforms, Mesh, GpuMesh, EguiIntegration};
use crate::editor::selection::SceneObject;

/// Maximum number of objects that can be rendered
const MAX_OBJECTS: usize = 1024;

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
    ctx: RenderContext,
    /// Camera
    camera: IsometricCamera,
    /// Mesh pipeline
    mesh_pipeline: wgpu::RenderPipeline,
    /// Mesh bind group layout
    #[allow(dead_code)]
    mesh_bind_group_layout: wgpu::BindGroupLayout,
    /// Mesh uniform buffer
    mesh_uniform_buffer: wgpu::Buffer,
    /// Mesh bind group
    mesh_bind_group: wgpu::BindGroup,
    /// Uniform alignment
    uniform_alignment: u32,
    /// Cube mesh
    cube_mesh: crate::render::GpuMesh,
    /// Depth texture
    depth_texture: wgpu::Texture,
    /// Depth view
    depth_view: wgpu::TextureView,
    /// Is window open
    pub is_open: bool,
    /// Scene objects snapshot
    scene_objects: Vec<SceneObject>,
    /// Play time
    play_time: f32,
    /// Last frame instant
    last_frame_instant: Option<std::time::Instant>,
    /// Game settings
    settings: GameSettings,
    /// Whether splash screen is showing
    in_splash: bool,
    /// Egui integration for overlays (splash, FPS)
    egui: EguiIntegration,
    /// Splash texture handle
    splash_texture: Option<egui::TextureHandle>,
    /// FPS accumulator for smoothing
    fps_samples: Vec<f32>,
    /// Current smoothed FPS
    current_fps: f32,
}

impl GameWindow {
    /// Create a new game window
    pub async fn new(
        event_loop: &ActiveEventLoop,
        scene_objects: Vec<SceneObject>,
        camera: IsometricCamera,
        settings: GameSettings,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create window
        let window_attrs = WindowAttributes::default()
            .with_title("Xtreme Engine - Game")
            .with_inner_size(winit::dpi::LogicalSize::new(settings.window_width, settings.window_height));

        let window = Arc::new(event_loop.create_window(window_attrs)?);

        // Create render context with vsync setting
        let ctx = RenderContext::new_with_vsync(window.clone(), settings.vsync).await?;

        // Create mesh shader
        let mesh_shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Game Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/basic.wgsl").into()),
        });

        // Uniform alignment
        let uniform_alignment = ctx.device.limits().min_uniform_buffer_offset_alignment;
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = ((uniform_size + uniform_alignment - 1) / uniform_alignment) * uniform_alignment;

        // Create uniform buffer
        let mesh_uniform_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Game Mesh Uniform Buffer"),
            size: (aligned_size as usize * MAX_OBJECTS) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let mesh_bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create pipeline layout
        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
        let mesh_pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Game Mesh Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.format(),
                    blend: Some(wgpu::BlendState::REPLACE),
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

        // Update camera aspect ratio for the new window
        let mut camera = camera;
        let (width, height) = ctx.size();
        camera.aspect_ratio = width as f32 / height as f32;

        let in_splash = settings.splash_duration > 0.0;

        // Initialize egui for overlays (splash screen, FPS)
        let mut egui = EguiIntegration::new(&ctx, &window);

        // Load splash screen texture if splash is enabled
        let splash_texture = if in_splash {
            Self::load_splash_texture(&mut egui)
        } else {
            None
        };

        Ok(Self {
            window,
            ctx,
            camera,
            mesh_pipeline,
            mesh_bind_group_layout,
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
        })
    }

    /// Load splash screen texture from assets/logo.png
    fn load_splash_texture(egui: &mut EguiIntegration) -> Option<egui::TextureHandle> {
        let logo_paths = ["assets/logo.png", "assets/xtreme-logo.png"];

        for path in &logo_paths {
            if let Ok(img) = image::open(path) {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();

                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture = egui.ctx().load_texture(
                    "splash_logo",
                    color_image,
                    egui::TextureOptions::LINEAR,
                );

                log::info!("Loaded splash texture from {}", path);
                return Some(texture);
            }
        }

        log::warn!("Could not load splash texture");
        None
    }

    /// Get window ID
    pub fn id(&self) -> winit::window::WindowId {
        self.window.id()
    }

    /// Handle resize
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.resize(new_size.width, new_size.height);

            // Update camera aspect ratio
            self.camera.aspect_ratio = new_size.width as f32 / new_size.height as f32;

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
            self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
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

    /// Calculate splash fade alpha (0.0 to 1.0)
    fn splash_alpha(&self) -> f32 {
        let duration = self.settings.splash_duration;
        if duration <= 0.0 {
            return 0.0;
        }

        let fade_time = 0.5; // 0.5s fade in/out
        let t = self.play_time;

        if t < fade_time {
            // Fade in
            t / fade_time
        } else if t > duration - fade_time {
            // Fade out
            ((duration - t) / fade_time).max(0.0)
        } else {
            // Full opacity
            1.0
        }
    }

    /// Render the game
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Check if splash screen is done
        if self.in_splash && self.play_time >= self.settings.splash_duration {
            self.in_splash = false;
            log::info!("Splash screen finished");
        }

        let output = self.ctx.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game Render Encoder"),
        });

        // Calculate background color (with splash fade to black)
        let bg = self.settings.background_color;
        let (r, g, b) = if self.in_splash {
            // During splash, fade from black to bg color
            let alpha = self.splash_alpha();
            let splash_r = bg[0] as f64 / 255.0 * (1.0 - alpha as f64);
            let splash_g = bg[1] as f64 / 255.0 * (1.0 - alpha as f64);
            let splash_b = bg[2] as f64 / 255.0 * (1.0 - alpha as f64);
            (splash_r, splash_g, splash_b)
        } else {
            (bg[0] as f64 / 255.0, bg[1] as f64 / 255.0, bg[2] as f64 / 255.0)
        };

        // Update uniforms only when not in splash
        let view_proj = self.camera.view_projection_matrix();
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = ((uniform_size + self.uniform_alignment - 1) / self.uniform_alignment) * self.uniform_alignment;

        let num_objects = if self.in_splash { 0 } else { self.scene_objects.len().min(MAX_OBJECTS) };
        for (i, obj) in self.scene_objects.iter().take(num_objects).enumerate() {
            if !obj.visible {
                continue;
            }

            // Use world matrix to account for parent hierarchy
            let model = obj.world_matrix(&self.scene_objects);

            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: model.to_cols_array_2d(),
                color: obj.color,
            };

            let offset = (i as u32 * aligned_size) as u64;
            self.ctx.queue.write_buffer(&self.mesh_uniform_buffer, offset, bytemuck::cast_slice(&[uniforms]));
        }

        // Clear pass with background color
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render objects
        if num_objects > 0 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Mesh Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.mesh_pipeline);

            for (i, obj) in self.scene_objects.iter().take(num_objects).enumerate() {
                if !obj.visible {
                    continue;
                }

                let dynamic_offset = i as u32 * aligned_size;
                pass.set_bind_group(0, &self.mesh_bind_group, &[dynamic_offset]);
                self.cube_mesh.draw(&mut pass);
            }
        }

        // Render egui overlays (splash screen and/or FPS)
        let needs_egui = self.in_splash || self.settings.show_fps;
        if needs_egui {
            self.egui.begin_frame(&self.window);
            let ctx = self.egui.ctx().clone();

            // Render splash screen with logo
            if self.in_splash {
                let alpha = self.splash_alpha();
                let (width, height) = self.ctx.size();

                // Full screen dark overlay
                let overlay_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 200.0) as u8);

                egui::Area::new(egui::Id::new("splash_overlay"))
                    .fixed_pos(egui::pos2(0.0, 0.0))
                    .show(&ctx, |ui| {
                        let rect = egui::Rect::from_min_size(
                            egui::pos2(0.0, 0.0),
                            egui::vec2(width as f32, height as f32),
                        );
                        ui.painter().rect_filled(rect, 0.0, overlay_color);
                    });

                // Render logo centered
                if let Some(ref texture) = self.splash_texture {
                    let tex_size = texture.size_vec2();
                    let center_x = width as f32 / 2.0 - tex_size.x / 2.0;
                    let center_y = height as f32 / 2.0 - tex_size.y / 2.0;

                    egui::Area::new(egui::Id::new("splash_logo"))
                        .fixed_pos(egui::pos2(center_x, center_y))
                        .show(&ctx, |ui| {
                            let tint = egui::Color32::from_rgba_unmultiplied(255, 255, 255, (alpha * 255.0) as u8);
                            ui.add(egui::Image::new(texture).tint(tint));
                        });
                }
            }

            // Render FPS overlay if enabled
            if self.settings.show_fps {
                let fps_text = format!("FPS: {:.0}", self.current_fps);

                egui::Area::new(egui::Id::new("fps_overlay"))
                    .fixed_pos(egui::pos2(10.0, 10.0))
                    .show(&ctx, |ui| {
                        let black = egui::Color32::BLACK;
                        let yellow = egui::Color32::from_rgb(255, 220, 0);
                        let font = egui::FontId::proportional(18.0);
                        let text_pos = ui.cursor().min;

                        // Draw black shadow/outline
                        for offset in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                            ui.painter().text(
                                egui::pos2(text_pos.x + offset.0, text_pos.y + offset.1),
                                egui::Align2::LEFT_TOP,
                                &fps_text,
                                font.clone(),
                                black,
                            );
                        }

                        // Draw yellow text on top
                        ui.painter().text(
                            text_pos,
                            egui::Align2::LEFT_TOP,
                            &fps_text,
                            font,
                            yellow,
                        );

                        ui.allocate_space(egui::vec2(100.0, 24.0));
                    });
            }

            self.egui.end_frame(&self.window);
            self.egui.render(&self.ctx, &mut encoder, &view);
        }

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Close the window
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

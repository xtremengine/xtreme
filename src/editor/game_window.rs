//! # Game Window
//!
//! Separate window for running the game in play mode.

use std::sync::Arc;
use glam::Mat4;
use winit::window::Window;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

use crate::render::{IsometricCamera, RenderContext, Uniforms, Mesh, GpuMesh};
use crate::editor::selection::SceneObject;

/// Maximum number of objects that can be rendered
const MAX_OBJECTS: usize = 1024;

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
}

impl GameWindow {
    /// Create a new game window
    pub async fn new(
        event_loop: &ActiveEventLoop,
        scene_objects: Vec<SceneObject>,
        camera: IsometricCamera,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create window
        let window_attrs = WindowAttributes::default()
            .with_title("Xtreme Engine - Game")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        let window = Arc::new(event_loop.create_window(window_attrs)?);

        // Create render context
        let ctx = RenderContext::new(window.clone()).await?;

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

    /// Render the game
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.ctx.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game Render Encoder"),
        });

        // Update uniforms
        let view_proj = self.camera.view_projection_matrix();
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = ((uniform_size + self.uniform_alignment - 1) / self.uniform_alignment) * self.uniform_alignment;

        let num_objects = self.scene_objects.len().min(MAX_OBJECTS);
        for (i, obj) in self.scene_objects.iter().take(num_objects).enumerate() {
            if !obj.visible {
                continue;
            }

            let model = Mat4::from_scale_rotation_translation(
                obj.scale,
                glam::Quat::from_euler(glam::EulerRot::XYZ, obj.rotation.x, obj.rotation.y, obj.rotation.z),
                obj.position,
            );

            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: model.to_cols_array_2d(),
                color: obj.color,
            };

            let offset = (i as u32 * aligned_size) as u64;
            self.ctx.queue.write_buffer(&self.mesh_uniform_buffer, offset, bytemuck::cast_slice(&[uniforms]));
        }

        // Clear pass
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
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

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Close the window
    pub fn close(&mut self) {
        self.is_open = false;
    }
}

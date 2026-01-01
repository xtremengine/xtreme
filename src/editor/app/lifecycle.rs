//! App trait implementation: init, events, update, render.

use glam::Mat4;
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowId};

use super::EditorApp;
use crate::editor::gizmos::GizmoMode;
use crate::editor::panels::Tool;
use crate::editor::shortcuts::EditorAction;
use crate::editor::viewport::{ObjectRenderData, Viewport};
use crate::render::{App, AppEvent, EguiIntegration, RenderContext};

impl EditorApp {
    /// Get model matrices, colors, and textures for rendering (uses world transforms)
    /// Returns (regular_objects, camera_objects) - cameras are rendered separately as wireframes
    pub fn get_render_data(&self) -> (Vec<ObjectRenderData>, Vec<(Mat4, [f32; 4])>) {
        let mut regular = Vec::new();
        let mut cameras = Vec::new();

        for o in self.scene_objects.iter().filter(|o| o.visible) {
            let is_selected = self.selection.is_selected(o.id);
            let color = if is_selected {
                [1.0, 0.8, 0.2, 1.0] // Yellow highlight
            } else {
                o.color
            };
            // Use world matrix to account for parent hierarchy
            let model = o.world_matrix(&self.scene_objects);

            if o.camera.is_some() {
                cameras.push((model, color));
            } else {
                regular.push(ObjectRenderData {
                    model,
                    color,
                    texture_path: o.texture_path.clone(),
                    shader_path: o.shader_path.clone(),
                });
            }
        }

        (regular, cameras)
    }
}

impl App for EditorApp {
    fn init(&mut self, window: Arc<WinitWindow>) {
        log::info!("Initializing editor...");

        let ctx = pollster::block_on(RenderContext::new(window.clone()))
            .expect("Failed to create render context");

        log::info!("GPU: {}", ctx.adapter_info().name);

        // Initialize egui
        let mut egui = EguiIntegration::new(&ctx, &window);

        // Initialize viewport
        let (w, h) = ctx.size();
        let viewport_width = (w as f32 * 0.6) as u32;
        let viewport_height = (h as f32 * 0.8) as u32;

        let viewport = Viewport::new(&ctx, egui.renderer_mut(), viewport_width, viewport_height);

        // Set camera aspect ratio to match initial viewport
        self.camera.aspect_ratio = viewport_width as f32 / viewport_height as f32;
        self.last_viewport_size = (viewport_width as f32, viewport_height as f32);

        // Initialize particle manager for editor preview
        // Uses Rgba8UnormSrgb format to match viewport texture
        let particle_manager = crate::particles::ParticleManager::new(
            &ctx.device,
            &ctx.queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        self.ctx = Some(ctx);
        self.egui = Some(egui);
        self.viewport = Some(viewport);
        self.window = Some(window);
        self.particle_manager = Some(particle_manager);
        self.particles_dirty = true;

        log::info!("Editor ready!");
    }

    fn raw_event(&mut self, window: &WinitWindow, event: &WindowEvent) -> bool {
        if let Some(egui) = &mut self.egui {
            egui.handle_event(window, event)
        } else {
            false
        }
    }

    fn event(&mut self, event: AppEvent) {
        if let AppEvent::Resized { width, height } = event {
            if let Some(ctx) = &mut self.ctx {
                ctx.resize(width, height);
            }
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;

        // Begin egui frame
        if let (Some(egui), Some(window)) = (&mut self.egui, &self.window) {
            egui.begin_frame(window);
        }

        // Handle keyboard shortcuts
        let actions: Vec<EditorAction> = if let Some(egui) = &self.egui {
            self.shortcuts.process_input(egui.ctx())
        } else {
            Vec::new()
        };

        // Process triggered actions
        for action in actions {
            self.handle_editor_action(action);
        }

        // Sync gizmo mode with current tool
        match self.toolbar_panel.current_tool {
            Tool::Move => self.gizmo.set_mode(GizmoMode::Translate),
            Tool::Rotate => self.gizmo.set_mode(GizmoMode::Rotate),
            Tool::Scale => self.gizmo.set_mode(GizmoMode::Scale),
            Tool::Select => {} // No gizmo in select mode
        }

        // Draw UI and get viewport size
        let mut viewport_size = self.last_viewport_size;
        self.draw_ui(&mut viewport_size);

        // Resize viewport if needed
        if viewport_size != self.last_viewport_size
            && viewport_size.0 > 0.0
            && viewport_size.1 > 0.0
        {
            if let (Some(ctx), Some(egui), Some(viewport)) =
                (&self.ctx, &mut self.egui, &mut self.viewport)
            {
                viewport.resize(
                    ctx,
                    egui.renderer_mut(),
                    viewport_size.0 as u32,
                    viewport_size.1 as u32,
                );
                self.camera.aspect_ratio = viewport_size.0 / viewport_size.1;
            }
            self.last_viewport_size = viewport_size;
        }

        // End egui frame
        if let (Some(egui), Some(window)) = (&mut self.egui, &self.window) {
            egui.end_frame(window);
        }
    }

    fn render(&mut self) {
        // Sync particles if dirty (before borrowing ctx)
        if self.particles_dirty {
            self.sync_particles();
        }

        // Update particle transforms from scene objects (for moving emitters)
        // Must be done before borrowing ctx to avoid borrow conflicts
        self.update_particle_transforms();

        let Some(ctx) = &self.ctx else { return };

        let Ok((output, view)) = ctx.begin_frame() else {
            return;
        };

        // Update editor time for shader animations
        let now = std::time::Instant::now();
        let delta = if let Some(last) = self.last_frame_instant {
            now.duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.editor_time += delta;
        self.last_frame_instant = Some(now);

        let mut encoder = ctx.create_encoder("Editor Frame");

        // Get render data (regular objects and cameras separately)
        let (regular_objects, camera_objects) = self.get_render_data();

        // Preload textures into cache (must be done before render)
        if let Some(viewport) = &mut self.viewport {
            for obj in &regular_objects {
                if let Some(ref path) = obj.texture_path {
                    // This loads the texture into the cache if not already there
                    viewport.get_or_load_texture(ctx, path);
                }
            }
        }

        // Get camera info for particles
        let view_proj = self.camera.view_projection_matrix();
        let cam_pos = self.camera.position();
        let cam_right = self.camera.right();
        let cam_up = self.camera.up();

        // Update particles
        if let Some(particle_manager) = &mut self.particle_manager {
            particle_manager.update(
                &ctx.device,
                &mut encoder,
                &ctx.queue,
                &mut self.particle_world,
                view_proj,
                cam_pos,
                cam_right,
                cam_up,
                delta,
            );
        }

        // Render viewport
        if let Some(viewport) = &mut self.viewport {
            // Render regular objects as cubes
            viewport.render_objects(
                ctx,
                &mut encoder,
                &self.camera,
                &regular_objects,
                self.editor_time,
            );

            // Render particles
            if let Some(particle_manager) = &self.particle_manager {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Viewport Particle Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &viewport.render_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &viewport.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                particle_manager.render(&mut pass, &self.particle_world);
            }

            // Render camera objects as wireframe pyramids
            viewport.render_camera_wireframes(ctx, &mut encoder, &self.camera, &camera_objects);

            // Render gizmo if we have a selection and not in Select mode
            if self.toolbar_panel.current_tool != Tool::Select {
                if let Some(id) = self.selection.first() {
                    if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
                        // Use world position for gizmo
                        let world_pos = obj.world_position(&self.scene_objects);
                        viewport.render_gizmo(
                            ctx,
                            &mut encoder,
                            &self.camera,
                            &self.gizmo,
                            world_pos,
                        );
                    }
                }
            }
        }

        // Clear the main window
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor Clear Pass"),
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
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render egui on top
        if let Some(egui) = &mut self.egui {
            egui.render(ctx, &mut encoder, &view);
        }

        ctx.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn shutdown(&mut self) {
        log::info!("Editor shutting down after {} frames", self.frame_count);
    }

    fn handle_pending_windows(&mut self, event_loop: &ActiveEventLoop) {
        self.handle_game_window_creation(event_loop);
    }

    fn secondary_window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> bool {
        self.handle_game_window_event(window_id, event)
    }

    fn main_window_id(&self) -> Option<WindowId> {
        self.window.as_ref().map(|w| w.id())
    }
}

impl EditorApp {
    /// Sync particle emitters from scene objects to particle world
    pub(crate) fn sync_particles(&mut self) {
        use crate::editor::components::ParticlePreset;
        use crate::math::Transform;
        use crate::particles::{presets, EmitterConfig, ParticleEmitter};
        use glam::Vec4;

        // Clear existing particle world and entity mapping
        self.particle_world.clear();
        self.particle_entity_map.clear();

        // Convert each scene object with particle_emitter to ECS entity
        for obj in &self.scene_objects {
            if let Some(ref emitter_comp) = obj.particle_emitter {
                if !emitter_comp.enabled {
                    continue;
                }

                // Convert editor component to runtime config
                let base_config = match emitter_comp.preset {
                    ParticlePreset::Fire => presets::fire(),
                    ParticlePreset::Smoke => presets::smoke(),
                    ParticlePreset::Sparkles => presets::sparkles(),
                    ParticlePreset::Rain => presets::rain(),
                    ParticlePreset::Explosion => presets::explosion(),
                    ParticlePreset::Custom => EmitterConfig::new("Custom")
                        .with_max_particles(emitter_comp.max_particles)
                        .with_lifetime(emitter_comp.lifetime_min, emitter_comp.lifetime_max)
                        .with_spawn_rate(emitter_comp.spawn_rate)
                        .with_gravity(glam::Vec3::new(
                            emitter_comp.gravity[0],
                            emitter_comp.gravity[1],
                            emitter_comp.gravity[2],
                        ))
                        .with_colors(
                            Vec4::new(
                                emitter_comp.start_color[0],
                                emitter_comp.start_color[1],
                                emitter_comp.start_color[2],
                                emitter_comp.start_color[3],
                            ),
                            Vec4::new(
                                emitter_comp.end_color[0],
                                emitter_comp.end_color[1],
                                emitter_comp.end_color[2],
                                emitter_comp.end_color[3],
                            ),
                        )
                        .with_sizes(emitter_comp.start_size, emitter_comp.end_size),
                };

                // Apply local_space setting from editor component
                let config = base_config.with_local_space(emitter_comp.local_space);

                // Create runtime emitter
                let emitter = ParticleEmitter::new(config);

                // Create transform from scene object (use world position)
                let world_pos = obj.world_position(&self.scene_objects);
                let transform = Transform::from_xyz(world_pos.x, world_pos.y, world_pos.z);

                // Spawn entity in particle world and store the mapping
                let entity = self
                    .particle_world
                    .spawn()
                    .with(transform)
                    .with(emitter)
                    .build();

                // Store mapping from scene object ID to ECS entity
                self.particle_entity_map.insert(obj.id, entity);
            }
        }

        self.particles_dirty = false;
    }

    /// Update particle emitter transforms from scene objects
    /// Called every frame to sync positions when objects move
    pub(crate) fn update_particle_transforms(&mut self) {
        use crate::math::Transform;

        // Update transforms for all mapped entities
        for obj in &self.scene_objects {
            if let Some(&entity) = self.particle_entity_map.get(&obj.id) {
                if let Some(transform) = self.particle_world.get_mut::<Transform>(entity) {
                    let world_pos = obj.world_position(&self.scene_objects);
                    transform.position =
                        crate::math::Position::new(world_pos.x, world_pos.y, world_pos.z);
                }
            }
        }
    }

    /// Mark particles as needing re-sync
    /// Called when particle emitters are added/modified in the scene
    #[allow(dead_code)]
    pub(crate) fn mark_particles_dirty(&mut self) {
        self.particles_dirty = true;
    }
}

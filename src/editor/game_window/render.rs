//! Rendering logic for game window.

use crate::render::Uniforms;

use super::{GameWindow, MAX_OBJECTS};

impl GameWindow {
    /// Render the game
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Check if splash screen is done
        if self.in_splash && self.play_time >= self.settings.splash_duration {
            self.in_splash = false;
            log::info!("Splash screen finished");
        }

        let output = self.ctx.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Game Render Encoder"),
            });

        // Get camera view_projection matrix and clear color
        let (view_proj, camera_clear_color) = self.get_camera_view_projection();

        // Calculate background color
        let bg = self.calculate_background_color(camera_clear_color);
        let (r, g, b) = self.apply_splash_fade(bg);

        // Update uniforms
        let num_objects = self.update_uniforms(view_proj);

        // Clear pass
        self.render_clear_pass(&mut encoder, &view, r, g, b);

        // Render objects
        if num_objects > 0 {
            self.render_objects(&mut encoder, &view, num_objects);
        }

        // Render egui overlays
        self.render_overlays(&mut encoder, &view);

        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Calculate background color from camera or settings
    fn calculate_background_color(&self, camera_clear_color: [f32; 4]) -> [u8; 3] {
        if camera_clear_color != [0.1, 0.1, 0.15, 1.0] {
            [
                (camera_clear_color[0] * 255.0) as u8,
                (camera_clear_color[1] * 255.0) as u8,
                (camera_clear_color[2] * 255.0) as u8,
            ]
        } else {
            self.settings.background_color
        }
    }

    /// Apply splash fade effect to background color
    fn apply_splash_fade(&self, bg: [u8; 3]) -> (f64, f64, f64) {
        if self.in_splash {
            let alpha = self.splash_alpha();
            let r = bg[0] as f64 / 255.0 * (1.0 - alpha as f64);
            let g = bg[1] as f64 / 255.0 * (1.0 - alpha as f64);
            let b = bg[2] as f64 / 255.0 * (1.0 - alpha as f64);
            (r, g, b)
        } else {
            (
                bg[0] as f64 / 255.0,
                bg[1] as f64 / 255.0,
                bg[2] as f64 / 255.0,
            )
        }
    }

    /// Update uniform buffers for all objects
    fn update_uniforms(&mut self, view_proj: glam::Mat4) -> usize {
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = uniform_size.div_ceil(self.uniform_alignment) * self.uniform_alignment;

        let num_objects = if self.in_splash {
            0
        } else {
            self.scene_objects.len().min(MAX_OBJECTS)
        };

        for (i, obj) in self.scene_objects.iter().take(num_objects).enumerate() {
            if !obj.visible {
                continue;
            }

            let model = obj.world_matrix(&self.scene_objects);

            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: model.to_cols_array_2d(),
                color: obj.color,
            };

            let offset = (i as u32 * aligned_size) as u64;
            self.ctx.queue.write_buffer(
                &self.mesh_uniform_buffer,
                offset,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        num_objects
    }

    /// Render clear pass with background color
    fn render_clear_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        r: f64,
        g: f64,
        b: f64,
    ) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Game Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

    /// Render scene objects
    fn render_objects(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        num_objects: usize,
    ) {
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = uniform_size.div_ceil(self.uniform_alignment) * self.uniform_alignment;

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Game Mesh Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

    /// Render egui overlays (splash screen and FPS)
    fn render_overlays(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let needs_egui = self.in_splash || self.settings.show_fps;
        if !needs_egui {
            return;
        }

        self.egui.begin_frame(&self.window);
        let ctx = self.egui.ctx().clone();

        // Render splash screen
        if self.in_splash {
            self.render_splash(&ctx);
        }

        // Render FPS overlay
        if self.settings.show_fps {
            self.render_fps_overlay(&ctx);
        }

        self.egui.end_frame(&self.window);
        self.egui.render(&self.ctx, encoder, view);
    }

    /// Render FPS counter overlay
    fn render_fps_overlay(&self, ctx: &egui::Context) {
        let fps_text = format!("FPS: {:.0}", self.current_fps);

        egui::Area::new(egui::Id::new("fps_overlay"))
            .fixed_pos(egui::pos2(10.0, 10.0))
            .show(ctx, |ui| {
                let black = egui::Color32::BLACK;
                let yellow = egui::Color32::from_rgb(255, 220, 0);
                let font = egui::FontId::proportional(18.0);
                let text_pos = ui.cursor().min;

                // Draw black shadow/outline
                for offset in [
                    (-1.0, 0.0),
                    (1.0, 0.0),
                    (0.0, -1.0),
                    (0.0, 1.0),
                    (-1.0, -1.0),
                    (1.0, -1.0),
                    (-1.0, 1.0),
                    (1.0, 1.0),
                ] {
                    ui.painter().text(
                        egui::pos2(text_pos.x + offset.0, text_pos.y + offset.1),
                        egui::Align2::LEFT_TOP,
                        &fps_text,
                        font.clone(),
                        black,
                    );
                }

                // Draw yellow text on top
                ui.painter()
                    .text(text_pos, egui::Align2::LEFT_TOP, &fps_text, font, yellow);

                ui.allocate_space(egui::vec2(100.0, 24.0));
            });
    }
}

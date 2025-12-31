//! Splash screen for play mode.

use super::EditorApp;

impl EditorApp {
    /// Draw splash screen overlay
    pub(super) fn draw_splash_screen(&mut self, ctx: &egui::Context) {
        if !self.show_splash {
            return;
        }

        // Load splash texture if not loaded
        if self.splash_texture.is_none() {
            if let Ok(img) = image::open("assets/xtreme-logo.png") {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture = ctx.load_texture("splash", color_image, egui::TextureOptions::LINEAR);
                self.splash_texture = Some(texture);
            }
        }

        let elapsed = self.splash_start_time
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        egui::Area::new(egui::Id::new("splash_overlay"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let screen = ctx.input(|i| i.viewport().inner_rect)
                    .unwrap_or(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0)));
                ui.allocate_response(screen.size(), egui::Sense::hover());

                // Dark background
                ui.painter().rect_filled(
                    screen,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(20, 20, 30, 240)
                );

                let center = screen.center();

                // Draw logo if loaded
                if let Some(texture) = &self.splash_texture {
                    let size = texture.size_vec2() * 0.5;
                    let rect = egui::Rect::from_center_size(
                        egui::pos2(center.x, center.y - 30.0),
                        size
                    );
                    ui.painter().image(
                        texture.id(),
                        rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE
                    );
                }

                // Engine name
                ui.painter().text(
                    egui::pos2(center.x, center.y + 80.0),
                    egui::Align2::CENTER_CENTER,
                    "XTREME ENGINE",
                    egui::FontId::proportional(32.0),
                    egui::Color32::WHITE
                );

                // Loading animation
                let dots = ".".repeat((elapsed * 3.0) as usize % 4);
                ui.painter().text(
                    egui::pos2(center.x, center.y + 120.0),
                    egui::Align2::CENTER_CENTER,
                    format!("Loading{}", dots),
                    egui::FontId::proportional(16.0),
                    egui::Color32::GRAY
                );

                // Progress bar
                self.draw_progress_bar(ui, center, elapsed);

                // Skip hint
                ui.painter().text(
                    egui::pos2(center.x, screen.max.y - 30.0),
                    egui::Align2::CENTER_CENTER,
                    "Press ESC to skip",
                    egui::FontId::proportional(12.0),
                    egui::Color32::DARK_GRAY
                );
            });

        // Skip splash on ESC
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Escape) {
                self.enter_play_mode_from_splash();
            }
        });
    }

    fn draw_progress_bar(&self, ui: &egui::Ui, center: egui::Pos2, elapsed: f32) {
        let bar_width = 200.0;
        let bar_height = 4.0;
        let progress = (elapsed / 2.0).min(1.0);

        let bar_rect = egui::Rect::from_center_size(
            egui::pos2(center.x, center.y + 150.0),
            egui::vec2(bar_width, bar_height)
        );
        ui.painter().rect_filled(bar_rect, 2.0, egui::Color32::DARK_GRAY);

        let filled_rect = egui::Rect::from_min_size(
            bar_rect.min,
            egui::vec2(bar_width * progress, bar_height)
        );
        ui.painter().rect_filled(filled_rect, 2.0, egui::Color32::from_rgb(100, 200, 100));
    }

    /// Enter play mode from splash screen
    pub(super) fn enter_play_mode_from_splash(&mut self) {
        self.show_splash = false;
        self.play_time = 0.0;
        self.last_frame_instant = Some(std::time::Instant::now());
        self.is_playing = true;

        #[cfg(feature = "scripting")]
        {
            self.sync_script_context();
            if let Err(e) = self.script_runtime.call_ready(&mut self.script_context) {
                log::error!("Script ready failed: {}", e);
            }
        }

        log::info!("Play mode started (splash skipped)");
    }
}

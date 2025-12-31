//! Splash screen functionality for game window.

use crate::render::EguiIntegration;

use super::GameWindow;

/// Load splash screen texture from assets/logo.png
pub fn load_splash_texture(egui: &mut EguiIntegration) -> Option<egui::TextureHandle> {
    let logo_paths = ["assets/logo.png", "assets/xtreme-logo.png"];

    for path in &logo_paths {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            let texture =
                egui.ctx()
                    .load_texture("splash_logo", color_image, egui::TextureOptions::LINEAR);

            log::info!("Loaded splash texture from {}", path);
            return Some(texture);
        }
    }

    log::warn!("Could not load splash texture");
    None
}

impl GameWindow {
    /// Calculate splash fade alpha (0.0 to 1.0)
    pub(crate) fn splash_alpha(&self) -> f32 {
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

    /// Render splash screen overlay
    pub(crate) fn render_splash(&self, ctx: &egui::Context) {
        let alpha = self.splash_alpha();
        let (width, height) = self.ctx.size();

        // Full screen dark overlay
        let overlay_color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha * 200.0) as u8);

        egui::Area::new(egui::Id::new("splash_overlay"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(0.0, 0.0),
                    egui::vec2(width as f32, height as f32),
                );
                ui.painter().rect_filled(rect, 0.0, overlay_color);
            });

        // Render logo centered at half size with "Xtreme Engine" text below
        if let Some(ref texture) = self.splash_texture {
            let tex_size = texture.size_vec2();
            let scale = 0.5; // Half size
            let scaled_size = tex_size * scale;
            let text_height = 30.0; // Space for text below logo
            let total_height = scaled_size.y + text_height;

            let center_x = width as f32 / 2.0 - scaled_size.x / 2.0;
            let center_y = height as f32 / 2.0 - total_height / 2.0;

            egui::Area::new(egui::Id::new("splash_logo"))
                .fixed_pos(egui::pos2(center_x, center_y))
                .show(ctx, |ui| {
                    let tint =
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, (alpha * 255.0) as u8);
                    ui.add(
                        egui::Image::new(texture)
                            .fit_to_exact_size(scaled_size)
                            .tint(tint),
                    );
                });

            // Render "Xtreme Engine" text below the logo (bold)
            let text_y = center_y + scaled_size.y - 150.0;

            egui::Area::new(egui::Id::new("splash_text"))
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, text_y))
                .show(ctx, |ui| {
                    let text_color = egui::Color32::from_rgba_unmultiplied(
                        0x33,
                        0x33,
                        0x33,
                        (alpha * 255.0) as u8,
                    );
                    let text = egui::RichText::new("Xtreme Engine")
                        .size(48.0)
                        .strong()
                        .color(text_color);
                    ui.label(text);
                });
        }
    }
}

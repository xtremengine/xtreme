//! Particle emitter inspector section.

use crate::editor::components::{ParticleEmitterComponent, ParticlePreset};
use egui::Ui;

/// Draw the particle emitter section
pub fn draw_particle_section(ui: &mut Ui, emitter: &mut ParticleEmitterComponent) -> bool {
    let mut changed = false;

    // Enabled checkbox
    if ui.checkbox(&mut emitter.enabled, "Enabled").changed() {
        changed = true;
    }

    // Local space checkbox
    if ui
        .checkbox(&mut emitter.local_space, "Local Space")
        .on_hover_text("If enabled, particles follow the emitter when it moves")
        .changed()
    {
        changed = true;
    }

    ui.add_space(4.0);

    // Preset selector
    ui.horizontal(|ui| {
        ui.label("Preset:");
        egui::ComboBox::from_id_salt("particle_preset")
            .selected_text(emitter.preset.name())
            .show_ui(ui, |ui| {
                for preset in ParticlePreset::all() {
                    if ui
                        .selectable_value(&mut emitter.preset, *preset, preset.name())
                        .clicked()
                    {
                        // Apply preset values
                        *emitter = ParticleEmitterComponent::with_preset(*preset);
                        changed = true;
                    }
                }
            });
    });

    ui.separator();

    // Emission settings
    ui.label("Emission");
    ui.horizontal(|ui| {
        ui.label("Max Particles:");
        if ui
            .add(egui::DragValue::new(&mut emitter.max_particles).speed(10))
            .changed()
        {
            changed = true;
        }
    });

    ui.horizontal(|ui| {
        ui.label("Spawn Rate:");
        if ui
            .add(
                egui::DragValue::new(&mut emitter.spawn_rate)
                    .speed(1.0)
                    .suffix("/s"),
            )
            .changed()
        {
            emitter.spawn_rate = emitter.spawn_rate.max(0.0);
            changed = true;
        }
    });

    ui.separator();

    // Lifetime settings
    ui.label("Lifetime");
    ui.horizontal(|ui| {
        ui.label("Min:");
        if ui
            .add(
                egui::DragValue::new(&mut emitter.lifetime_min)
                    .speed(0.1)
                    .suffix("s"),
            )
            .changed()
        {
            emitter.lifetime_min = emitter.lifetime_min.max(0.01);
            changed = true;
        }
        ui.label("Max:");
        if ui
            .add(
                egui::DragValue::new(&mut emitter.lifetime_max)
                    .speed(0.1)
                    .suffix("s"),
            )
            .changed()
        {
            emitter.lifetime_max = emitter.lifetime_max.max(emitter.lifetime_min);
            changed = true;
        }
    });

    ui.separator();

    // Appearance
    ui.label("Appearance");

    // Start color
    ui.horizontal(|ui| {
        ui.label("Start Color:");
        let mut color = egui::Color32::from_rgba_unmultiplied(
            (emitter.start_color[0] * 255.0) as u8,
            (emitter.start_color[1] * 255.0) as u8,
            (emitter.start_color[2] * 255.0) as u8,
            (emitter.start_color[3] * 255.0) as u8,
        );
        if ui.color_edit_button_srgba(&mut color).changed() {
            emitter.start_color = [
                color.r() as f32 / 255.0,
                color.g() as f32 / 255.0,
                color.b() as f32 / 255.0,
                color.a() as f32 / 255.0,
            ];
            changed = true;
        }
    });

    // End color
    ui.horizontal(|ui| {
        ui.label("End Color:");
        let mut color = egui::Color32::from_rgba_unmultiplied(
            (emitter.end_color[0] * 255.0) as u8,
            (emitter.end_color[1] * 255.0) as u8,
            (emitter.end_color[2] * 255.0) as u8,
            (emitter.end_color[3] * 255.0) as u8,
        );
        if ui.color_edit_button_srgba(&mut color).changed() {
            emitter.end_color = [
                color.r() as f32 / 255.0,
                color.g() as f32 / 255.0,
                color.b() as f32 / 255.0,
                color.a() as f32 / 255.0,
            ];
            changed = true;
        }
    });

    // Size
    ui.horizontal(|ui| {
        ui.label("Start Size:");
        if ui
            .add(egui::DragValue::new(&mut emitter.start_size).speed(0.01))
            .changed()
        {
            emitter.start_size = emitter.start_size.max(0.001);
            changed = true;
        }
        ui.label("End Size:");
        if ui
            .add(egui::DragValue::new(&mut emitter.end_size).speed(0.01))
            .changed()
        {
            emitter.end_size = emitter.end_size.max(0.001);
            changed = true;
        }
    });

    ui.separator();

    // Physics
    ui.label("Physics");
    ui.horizontal(|ui| {
        ui.label("Gravity:");
        ui.label("X");
        if ui
            .add(egui::DragValue::new(&mut emitter.gravity[0]).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label("Y");
        if ui
            .add(egui::DragValue::new(&mut emitter.gravity[1]).speed(0.1))
            .changed()
        {
            changed = true;
        }
        ui.label("Z");
        if ui
            .add(egui::DragValue::new(&mut emitter.gravity[2]).speed(0.1))
            .changed()
        {
            changed = true;
        }
    });

    changed
}

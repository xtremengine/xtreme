//! Snap settings panel for transform operations.

use crate::editor::snap::SnapSettings;
use egui::Ui;

/// Draw snap settings UI
pub fn draw_snap_settings(ui: &mut Ui, snap_settings: &mut SnapSettings) {
    ui.heading("Snap");
    ui.checkbox(&mut snap_settings.enabled, "Enable Snap");

    if snap_settings.enabled {
        ui.horizontal(|ui| {
            ui.label("Grid:");
            ui.add(
                egui::DragValue::new(&mut snap_settings.grid_size)
                    .speed(0.1)
                    .range(0.1..=10.0)
                    .suffix(" u")
                    .fixed_decimals(1),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Rotation:");
            ui.add(
                egui::DragValue::new(&mut snap_settings.rotation_snap)
                    .speed(1.0)
                    .range(1.0..=90.0)
                    .suffix("°")
                    .fixed_decimals(0),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Scale:");
            ui.add(
                egui::DragValue::new(&mut snap_settings.scale_snap)
                    .speed(0.05)
                    .range(0.01..=1.0)
                    .fixed_decimals(2),
            );
        });

        ui.horizontal(|ui| {
            if ui.small_button("0.5").clicked() {
                snap_settings.grid_size = 0.5;
            }
            if ui.small_button("1.0").clicked() {
                snap_settings.grid_size = 1.0;
            }
            if ui.small_button("2.0").clicked() {
                snap_settings.grid_size = 2.0;
            }
        });
    }
}

//! Editor camera settings panel.

use crate::render::IsometricCamera;
use egui::Ui;

/// Draw camera controls UI
pub fn draw_camera_settings(ui: &mut Ui, camera: &mut IsometricCamera) {
    ui.push_id("editor_camera_controls", |ui| {
        ui.heading("Camera");

        ui.horizontal(|ui| {
            ui.label("Distance:");
            ui.add(
                egui::DragValue::new(&mut camera.distance)
                    .speed(0.1)
                    .range(1.0..=100.0)
                    .fixed_decimals(1),
            );
        });

        let mut pitch_deg = camera.pitch.to_degrees();
        ui.horizontal(|ui| {
            ui.label("Pitch:");
            if ui
                .add(
                    egui::DragValue::new(&mut pitch_deg)
                        .speed(1.0)
                        .range(-89.0..=89.0)
                        .suffix("°")
                        .fixed_decimals(1),
                )
                .changed()
            {
                camera.pitch = pitch_deg.to_radians();
            }
        });

        let mut yaw_deg = camera.yaw.to_degrees();
        ui.horizontal(|ui| {
            ui.label("Yaw:");
            if ui
                .add(
                    egui::DragValue::new(&mut yaw_deg)
                        .speed(1.0)
                        .suffix("°")
                        .fixed_decimals(1),
                )
                .changed()
            {
                camera.yaw = yaw_deg.to_radians();
            }
        });

        ui.horizontal(|ui| {
            ui.label("Zoom:");
            ui.add(
                egui::DragValue::new(&mut camera.zoom)
                    .speed(0.1)
                    .range(1.0..=50.0)
                    .fixed_decimals(1),
            );
        });
    });
}

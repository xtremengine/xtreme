//! Camera section UI for the inspector panel.

use crate::editor::components::{CameraComponent, CameraProjection};
use egui::Ui;

/// Draw the camera section of the inspector
pub fn draw_camera_section(ui: &mut Ui, camera: &mut CameraComponent) -> bool {
    let mut changed = false;

    // Main camera checkbox
    ui.horizontal(|ui| {
        if ui.checkbox(&mut camera.is_main, "Main Camera").changed() {
            changed = true;
        }
    });

    // Priority
    ui.horizontal(|ui| {
        ui.label("Priority:");
        if ui
            .add(egui::DragValue::new(&mut camera.priority).speed(1))
            .changed()
        {
            changed = true;
        }
    });

    ui.separator();

    // Projection type
    ui.horizontal(|ui| {
        ui.label("Projection:");
        let is_perspective = matches!(camera.projection, CameraProjection::Perspective { .. });
        if ui.selectable_label(is_perspective, "Perspective").clicked() && !is_perspective {
            camera.projection = CameraProjection::Perspective {
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
            };
            changed = true;
        }
        if ui
            .selectable_label(!is_perspective, "Orthographic")
            .clicked()
            && is_perspective
        {
            camera.projection = CameraProjection::Orthographic {
                size: 10.0,
                near: 0.1,
                far: 1000.0,
            };
            changed = true;
        }
    });

    // Projection-specific settings
    match &mut camera.projection {
        CameraProjection::Perspective { fov, near, far } => {
            ui.horizontal(|ui| {
                ui.label("FOV:");
                if ui
                    .add(
                        egui::DragValue::new(fov)
                            .range(10.0..=170.0)
                            .speed(1.0)
                            .suffix("°")
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Near:");
                if ui
                    .add(
                        egui::DragValue::new(near)
                            .range(0.001..=100.0)
                            .speed(0.01)
                            .fixed_decimals(3),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Far:");
                if ui
                    .add(
                        egui::DragValue::new(far)
                            .range(1.0..=100000.0)
                            .speed(10.0)
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
        }
        CameraProjection::Orthographic { size, near, far } => {
            ui.horizontal(|ui| {
                ui.label("Size:");
                if ui
                    .add(
                        egui::DragValue::new(size)
                            .range(0.1..=1000.0)
                            .speed(0.1)
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Near:");
                if ui
                    .add(
                        egui::DragValue::new(near)
                            .range(0.001..=100.0)
                            .speed(0.01)
                            .fixed_decimals(3),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Far:");
                if ui
                    .add(
                        egui::DragValue::new(far)
                            .range(1.0..=100000.0)
                            .speed(10.0)
                            .fixed_decimals(1),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
        }
    }

    ui.separator();

    // Clear color
    ui.horizontal(|ui| {
        ui.label("Clear Color:");
        let mut color = [
            camera.clear_color[0],
            camera.clear_color[1],
            camera.clear_color[2],
        ];
        if ui.color_edit_button_rgb(&mut color).changed() {
            camera.clear_color[0] = color[0];
            camera.clear_color[1] = color[1];
            camera.clear_color[2] = color[2];
            changed = true;
        }
    });

    changed
}

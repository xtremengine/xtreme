//! Transform section UI for the inspector panel.

use crate::editor::selection::SceneObject;
use egui::Ui;
use glam::Vec3;

/// Draw the transform section of the inspector
pub fn draw_transform_section(ui: &mut Ui, obj: &mut SceneObject) -> bool {
    let mut changed = false;

    // Position (local space)
    ui.horizontal(|ui| {
        ui.label("Position (local):");
    });
    ui.horizontal(|ui| {
        ui.label("X:");
        if ui
            .add(
                egui::DragValue::new(&mut obj.position.x)
                    .speed(0.1)
                    .fixed_decimals(2),
            )
            .changed()
        {
            changed = true;
        }
        ui.label("Y:");
        if ui
            .add(
                egui::DragValue::new(&mut obj.position.y)
                    .speed(0.1)
                    .fixed_decimals(2),
            )
            .changed()
        {
            changed = true;
        }
        ui.label("Z:");
        if ui
            .add(
                egui::DragValue::new(&mut obj.position.z)
                    .speed(0.1)
                    .fixed_decimals(2),
            )
            .changed()
        {
            changed = true;
        }
    });

    // Rotation (in degrees for UI)
    ui.horizontal(|ui| {
        ui.label("Rotation:");
    });
    let mut rot_deg = Vec3::new(
        obj.rotation.x.to_degrees(),
        obj.rotation.y.to_degrees(),
        obj.rotation.z.to_degrees(),
    );
    ui.horizontal(|ui| {
        ui.label("X:");
        if ui
            .add(
                egui::DragValue::new(&mut rot_deg.x)
                    .speed(1.0)
                    .suffix("°")
                    .fixed_decimals(1),
            )
            .changed()
        {
            obj.rotation.x = rot_deg.x.to_radians();
            changed = true;
        }
        ui.label("Y:");
        if ui
            .add(
                egui::DragValue::new(&mut rot_deg.y)
                    .speed(1.0)
                    .suffix("°")
                    .fixed_decimals(1),
            )
            .changed()
        {
            obj.rotation.y = rot_deg.y.to_radians();
            changed = true;
        }
        ui.label("Z:");
        if ui
            .add(
                egui::DragValue::new(&mut rot_deg.z)
                    .speed(1.0)
                    .suffix("°")
                    .fixed_decimals(1),
            )
            .changed()
        {
            obj.rotation.z = rot_deg.z.to_radians();
            changed = true;
        }
    });

    // Scale
    ui.horizontal(|ui| {
        ui.label("Scale:");
    });
    ui.horizontal(|ui| {
        ui.label("X:");
        if ui
            .add(
                egui::DragValue::new(&mut obj.scale.x)
                    .speed(0.01)
                    .range(0.01..=100.0)
                    .fixed_decimals(2),
            )
            .changed()
        {
            changed = true;
        }
        ui.label("Y:");
        if ui
            .add(
                egui::DragValue::new(&mut obj.scale.y)
                    .speed(0.01)
                    .range(0.01..=100.0)
                    .fixed_decimals(2),
            )
            .changed()
        {
            changed = true;
        }
        ui.label("Z:");
        if ui
            .add(
                egui::DragValue::new(&mut obj.scale.z)
                    .speed(0.01)
                    .range(0.01..=100.0)
                    .fixed_decimals(2),
            )
            .changed()
        {
            changed = true;
        }
    });

    // Reset buttons
    ui.horizontal(|ui| {
        if ui.small_button("Reset Position").clicked() {
            obj.position = Vec3::ZERO;
            changed = true;
        }
        if ui.small_button("Reset Rotation").clicked() {
            obj.rotation = Vec3::ZERO;
            changed = true;
        }
        if ui.small_button("Reset Scale").clicked() {
            obj.scale = Vec3::ONE;
            changed = true;
        }
    });

    changed
}

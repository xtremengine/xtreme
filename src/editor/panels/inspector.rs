//! # Inspector Panel
//!
//! Property inspector for selected objects.

use super::super::components::CameraProjection;
use super::super::selection::{SceneObject, Selection};
use egui::Ui;
use glam::Vec3;

/// Inspector panel for editing object properties
pub struct InspectorPanel {
    /// Whether transform is expanded
    transform_expanded: bool,
    /// Whether material is expanded
    material_expanded: bool,
    /// Whether camera is expanded
    camera_expanded: bool,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InspectorPanel {
    /// Create a new inspector panel
    pub fn new() -> Self {
        Self {
            transform_expanded: true,
            material_expanded: true,
            camera_expanded: true,
        }
    }

    /// Draw the inspector panel
    pub fn show(
        &mut self,
        ui: &mut Ui,
        objects: &mut [SceneObject],
        selection: &Selection,
    ) -> bool {
        let mut changed = false;

        ui.heading("Inspector");
        ui.separator();

        if selection.is_empty() {
            ui.label("No object selected");
            return false;
        }

        let selected_ids = selection.all();

        // Multiple selection info
        if selected_ids.len() > 1 {
            ui.horizontal(|ui| {
                ui.strong(format!("{} objects selected", selected_ids.len()));
            });

            // Show list of selected objects
            egui::CollapsingHeader::new("Selected Objects")
                .default_open(false)
                .show(ui, |ui| {
                    for id in selected_ids {
                        if let Some(obj) = objects.iter().find(|o| o.id == *id) {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{}] {}", id, obj.name));
                            });
                        }
                    }
                });

            ui.separator();
            ui.label("Editing first selected:");
        }

        // Get first selected object for editing
        let Some(obj) = objects.iter_mut().find(|o| Some(o.id) == selection.first()) else {
            ui.label("Selected object not found");
            return false;
        };

        // Use object ID to ensure unique widget IDs when switching selection
        let obj_id = obj.id;
        ui.push_id(obj_id, |ui| {
            // Name field
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut obj.name).changed() {
                    changed = true;
                }
            });

            // Visibility toggle
            ui.horizontal(|ui| {
                if ui.checkbox(&mut obj.visible, "Visible").changed() {
                    changed = true;
                }
            });

            ui.separator();

            // Transform section
            let header =
                egui::CollapsingHeader::new("Transform").default_open(self.transform_expanded);

            header.show(ui, |ui| {
                self.transform_expanded = true;

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
            });

            ui.separator();

            // Material section
            let header =
                egui::CollapsingHeader::new("Material").default_open(self.material_expanded);

            header.show(ui, |ui| {
                self.material_expanded = true;

                // Color picker
                ui.horizontal(|ui| {
                    ui.label("Color:");
                    let mut color = [obj.color[0], obj.color[1], obj.color[2]];
                    if ui.color_edit_button_rgb(&mut color).changed() {
                        obj.color[0] = color[0];
                        obj.color[1] = color[1];
                        obj.color[2] = color[2];
                        changed = true;
                    }
                });

                // Alpha
                ui.horizontal(|ui| {
                    ui.label("Alpha:");
                    if ui
                        .add(egui::Slider::new(&mut obj.color[3], 0.0..=1.0).fixed_decimals(2))
                        .changed()
                    {
                        changed = true;
                    }
                });

                // Color presets
                ui.horizontal(|ui| {
                    ui.label("Presets:");
                    if ui.small_button("Red").clicked() {
                        obj.color = [0.9, 0.2, 0.2, 1.0];
                        changed = true;
                    }
                    if ui.small_button("Green").clicked() {
                        obj.color = [0.2, 0.9, 0.2, 1.0];
                        changed = true;
                    }
                    if ui.small_button("Blue").clicked() {
                        obj.color = [0.2, 0.2, 0.9, 1.0];
                        changed = true;
                    }
                    if ui.small_button("Orange").clicked() {
                        obj.color = [0.9, 0.5, 0.1, 1.0];
                        changed = true;
                    }
                });
            });

            // Camera section (only if object has camera component)
            if let Some(ref mut camera) = obj.camera {
                ui.separator();

                let header =
                    egui::CollapsingHeader::new("Camera").default_open(self.camera_expanded);

                header.show(ui, |ui| {
                    self.camera_expanded = true;

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
                        let is_perspective =
                            matches!(camera.projection, CameraProjection::Perspective { .. });
                        if ui.selectable_label(is_perspective, "Perspective").clicked()
                            && !is_perspective
                        {
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
                });
            }
        }); // end push_id

        changed
    }
}

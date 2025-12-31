//! # Inspector Panel
//!
//! Property inspector for selected objects.
//!
//! This module is split into:
//! - `transform.rs` - Transform section UI
//! - `material.rs` - Material section UI
//! - `camera.rs` - Camera section UI

mod camera;
mod material;
mod transform;

use super::super::selection::{SceneObject, Selection};
use egui::Ui;

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
                if transform::draw_transform_section(ui, obj) {
                    changed = true;
                }
            });

            ui.separator();

            // Material section
            let header =
                egui::CollapsingHeader::new("Material").default_open(self.material_expanded);
            header.show(ui, |ui| {
                self.material_expanded = true;
                if material::draw_material_section(ui, obj) {
                    changed = true;
                }
            });

            // Camera section (only if object has camera component)
            if let Some(ref mut cam) = obj.camera {
                ui.separator();

                let header =
                    egui::CollapsingHeader::new("Camera").default_open(self.camera_expanded);
                header.show(ui, |ui| {
                    self.camera_expanded = true;
                    if camera::draw_camera_section(ui, cam) {
                        changed = true;
                    }
                });
            }
        });

        changed
    }
}

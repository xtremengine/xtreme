//! Prefab section UI for the inspector panel.
//!
//! Shows prefab instance info and override indicators.

use std::collections::HashMap;

use egui::{Color32, RichText, Ui};

use crate::editor::prefab::{overrides::PropertyPath, PrefabInstance};
use crate::editor::selection::SceneObject;

/// Color for overridden properties
pub const OVERRIDE_COLOR: Color32 = Color32::from_rgb(255, 180, 80);

/// Action returned by prefab UI
#[derive(Clone, Debug, PartialEq)]
pub enum PrefabAction {
    /// No action
    None,
    /// Revert a property to prefab value
    RevertProperty { object_id: u32, path: String },
    /// Revert all overrides
    RevertAll { object_id: u32 },
    /// Apply overrides back to prefab
    ApplyToPrefab { object_id: u32 },
    /// Select prefab in browser
    SelectPrefab { path: String },
    /// Open prefab for editing
    OpenPrefab { path: String },
}

/// Draw the prefab instance section in inspector
pub fn draw_prefab_section(
    ui: &mut Ui,
    obj: &SceneObject,
    prefab_instances: &HashMap<u32, PrefabInstance>,
) -> PrefabAction {
    let mut action = PrefabAction::None;

    // Check if this object is a prefab instance
    let Some(instance) = prefab_instances.get(&obj.id) else {
        return action;
    };

    ui.separator();

    // Prefab header with colored background
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(50, 60, 80))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong(RichText::new("📦 Prefab Instance").color(OVERRIDE_COLOR));
            });

            ui.horizontal(|ui| {
                ui.label("Source:");
                ui.label(
                    RichText::new(instance.prefab_name())
                        .color(Color32::LIGHT_BLUE)
                        .strong(),
                );
            });

            // Show path
            ui.horizontal(|ui| {
                ui.label("Path:");
                ui.label(
                    RichText::new(instance.prefab_path().display().to_string())
                        .small()
                        .color(Color32::GRAY),
                );
            });

            // Instance info
            if !instance.is_root {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Object {} of prefab (Instance #{})",
                        instance.object_index + 1,
                        instance.instance_id
                    ));
                });
            }

            ui.separator();

            // Overrides section
            let override_count = instance.overrides.count();
            if override_count > 0 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Overrides: {}", override_count))
                            .color(OVERRIDE_COLOR),
                    );
                });

                // List overridden properties
                egui::CollapsingHeader::new("Overridden Properties")
                    .default_open(true)
                    .show(ui, |ui| {
                        for path in instance.overrides.paths() {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("• {}", path.as_str()))
                                        .color(OVERRIDE_COLOR),
                                );
                                if ui.small_button("Revert").clicked() {
                                    action = PrefabAction::RevertProperty {
                                        object_id: obj.id,
                                        path: path.as_str().to_string(),
                                    };
                                }
                            });
                        }
                    });

                ui.horizontal(|ui| {
                    if ui.button("Revert All").clicked() {
                        action = PrefabAction::RevertAll { object_id: obj.id };
                    }
                    if ui.button("Apply to Prefab").clicked() {
                        action = PrefabAction::ApplyToPrefab { object_id: obj.id };
                    }
                });
            } else {
                ui.label(
                    RichText::new("No overrides - using prefab values").color(Color32::DARK_GRAY),
                );
            }

            ui.separator();

            // Prefab actions
            ui.horizontal(|ui| {
                if ui.small_button("Select Prefab").clicked() {
                    action = PrefabAction::SelectPrefab {
                        path: instance.prefab_path().display().to_string(),
                    };
                }
                if ui.small_button("Open Prefab").clicked() {
                    action = PrefabAction::OpenPrefab {
                        path: instance.prefab_path().display().to_string(),
                    };
                }
            });
        });

    action
}

/// Helper to create a property label with override indicator
#[allow(dead_code)]
pub fn property_label(ui: &mut Ui, label: &str, is_overridden: bool) -> egui::Response {
    if is_overridden {
        ui.label(
            RichText::new(format!("{}*", label))
                .strong()
                .color(OVERRIDE_COLOR),
        )
    } else {
        ui.label(label)
    }
}

/// Check if a property is overridden for an object
#[allow(dead_code)]
pub fn is_property_overridden(
    obj_id: u32,
    property: &str,
    prefab_instances: &HashMap<u32, PrefabInstance>,
) -> bool {
    if let Some(instance) = prefab_instances.get(&obj_id) {
        let path = PropertyPath::new(property);
        instance.overrides.is_effectively_overridden(&path)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::prefab::instance::PrefabRef;

    #[test]
    fn test_is_property_overridden() {
        let mut instances = HashMap::new();
        let mut instance = PrefabInstance::root(PrefabRef::new("test.prefab"), 1);
        instance
            .overrides
            .set_override(PropertyPath::new("position"));

        instances.insert(42, instance);

        assert!(is_property_overridden(42, "position", &instances));
        assert!(is_property_overridden(42, "position.x", &instances));
        assert!(!is_property_overridden(42, "rotation", &instances));
        assert!(!is_property_overridden(99, "position", &instances));
    }
}

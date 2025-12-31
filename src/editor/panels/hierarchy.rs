//! # Hierarchy Panel
//!
//! Shows the scene hierarchy with all objects.

use egui::{Ui, RichText, Color32};
use super::super::selection::{SceneObject, Selection, ObjectId};

/// Hierarchy panel for displaying scene objects
pub struct HierarchyPanel {
    /// Search filter
    filter: String,
    /// Whether to show hidden objects
    show_hidden: bool,
}

impl Default for HierarchyPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchyPanel {
    /// Create a new hierarchy panel
    pub fn new() -> Self {
        Self {
            filter: String::new(),
            show_hidden: false,
        }
    }

    /// Draw the hierarchy panel
    /// Returns the ID of clicked object if any
    pub fn show(
        &mut self,
        ui: &mut Ui,
        objects: &mut Vec<SceneObject>,
        selection: &mut Selection,
    ) -> HierarchyAction {
        let mut action = HierarchyAction::None;

        ui.heading("Hierarchy");
        ui.separator();

        // Toolbar
        ui.horizontal(|ui| {
            if ui.button("+ Cube").clicked() {
                action = HierarchyAction::CreateCube;
            }
            if ui.button("+ Empty").clicked() {
                action = HierarchyAction::CreateEmpty;
            }
            ui.separator();
            if ui.button("Delete").clicked() {
                if let Some(id) = selection.first() {
                    action = HierarchyAction::Delete(id);
                }
            }
        });

        ui.separator();

        // Search
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.filter);
            if ui.small_button("x").clicked() {
                self.filter.clear();
            }
        });

        ui.checkbox(&mut self.show_hidden, "Show hidden");

        ui.separator();

        // Object count
        let visible_count = objects.iter().filter(|o| o.visible || self.show_hidden).count();
        ui.label(format!("Objects: {} / {}", visible_count, objects.len()));

        ui.separator();

        // Object list
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let filter_lower = self.filter.to_lowercase();

                for obj in objects.iter() {
                    // Filter by visibility
                    if !obj.visible && !self.show_hidden {
                        continue;
                    }

                    // Filter by search
                    if !self.filter.is_empty() && !obj.name.to_lowercase().contains(&filter_lower) {
                        continue;
                    }

                    let is_selected = selection.is_selected(obj.id);
                    let is_hovered = selection.hovered() == Some(obj.id);

                    // Build label with icon
                    let icon = if obj.visible { "◆" } else { "◇" };
                    let text = format!("{} {}", icon, obj.name);

                    let mut text = RichText::new(text);
                    if !obj.visible {
                        text = text.color(Color32::GRAY);
                    }
                    if is_hovered && !is_selected {
                        text = text.color(Color32::LIGHT_BLUE);
                    }

                    let response = ui.selectable_label(is_selected, text);

                    if response.clicked() {
                        if ui.input(|i| i.modifiers.ctrl) {
                            selection.toggle(obj.id);
                        } else {
                            selection.select(obj.id);
                        }
                    }

                    if response.double_clicked() {
                        action = HierarchyAction::Focus(obj.id);
                    }

                    // Context menu
                    response.context_menu(|ui| {
                        if ui.button("Rename").clicked() {
                            action = HierarchyAction::Rename(obj.id);
                            ui.close();
                        }
                        if ui.button("Duplicate").clicked() {
                            action = HierarchyAction::Duplicate(obj.id);
                            ui.close();
                        }
                        ui.separator();
                        let vis_text = if obj.visible { "Hide" } else { "Show" };
                        if ui.button(vis_text).clicked() {
                            action = HierarchyAction::ToggleVisibility(obj.id);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Delete").clicked() {
                            action = HierarchyAction::Delete(obj.id);
                            ui.close();
                        }
                    });
                }
            });

        action
    }
}

/// Actions from the hierarchy panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchyAction {
    /// No action
    None,
    /// Create a new cube
    CreateCube,
    /// Create an empty object
    CreateEmpty,
    /// Delete an object
    Delete(ObjectId),
    /// Duplicate an object
    Duplicate(ObjectId),
    /// Focus camera on object
    Focus(ObjectId),
    /// Rename an object
    Rename(ObjectId),
    /// Toggle visibility
    ToggleVisibility(ObjectId),
}

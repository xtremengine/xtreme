//! # Hierarchy Panel
//!
//! Shows the scene hierarchy as a tree with context menu reparenting.

use std::collections::HashSet;
use egui::{Ui, RichText, Color32};
use super::super::selection::{SceneObject, Selection, ObjectId};

/// Hierarchy panel for displaying scene objects as a tree
pub struct HierarchyPanel {
    /// Search filter
    filter: String,
    /// Whether to show hidden objects
    show_hidden: bool,
    /// Expanded nodes (collapsed by default)
    expanded: HashSet<ObjectId>,
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
            expanded: HashSet::new(),
        }
    }

    /// Toggle expansion state of an object
    pub fn toggle_expanded(&mut self, id: ObjectId) {
        if self.expanded.contains(&id) {
            self.expanded.remove(&id);
        } else {
            self.expanded.insert(id);
        }
    }

    /// Check if an object is expanded
    pub fn is_expanded(&self, id: ObjectId) -> bool {
        self.expanded.contains(&id)
    }

    /// Draw the hierarchy panel
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

        // Tree view
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // If search is active, show flat list
                if !self.filter.is_empty() {
                    action = self.show_flat_list(ui, objects, selection, action);
                } else {
                    // Show tree view starting from root objects
                    action = self.show_tree(ui, objects, selection, None, 0, action);
                }
            });

        action
    }

    /// Show flat list (for search results)
    fn show_flat_list(
        &mut self,
        ui: &mut Ui,
        objects: &[SceneObject],
        selection: &mut Selection,
        mut action: HierarchyAction,
    ) -> HierarchyAction {
        let filter_lower = self.filter.to_lowercase();

        for obj in objects.iter() {
            if !obj.visible && !self.show_hidden {
                continue;
            }
            if !obj.name.to_lowercase().contains(&filter_lower) {
                continue;
            }

            let result = self.show_object_row(ui, obj, objects, selection, 0);
            if result != HierarchyAction::None {
                action = result;
            }
        }

        action
    }

    /// Show tree recursively
    fn show_tree(
        &mut self,
        ui: &mut Ui,
        objects: &[SceneObject],
        selection: &mut Selection,
        parent_id: Option<ObjectId>,
        depth: usize,
        mut action: HierarchyAction,
    ) -> HierarchyAction {
        // Get children of this parent
        let children: Vec<_> = objects.iter()
            .filter(|o| o.hierarchy.parent == parent_id)
            .collect();

        for obj in children {
            // Filter by visibility
            if !obj.visible && !self.show_hidden {
                continue;
            }

            let result = self.show_object_row(ui, obj, objects, selection, depth);
            if result != HierarchyAction::None {
                action = result;
            }

            // Show children if expanded
            if self.is_expanded(obj.id) {
                action = self.show_tree(ui, objects, selection, Some(obj.id), depth + 1, action);
            }
        }

        action
    }

    /// Show a single object row
    fn show_object_row(
        &mut self,
        ui: &mut Ui,
        obj: &SceneObject,
        objects: &[SceneObject],
        selection: &mut Selection,
        depth: usize,
    ) -> HierarchyAction {
        let mut action = HierarchyAction::None;

        let is_selected = selection.is_selected(obj.id);
        let is_hovered = selection.hovered() == Some(obj.id);
        let has_children = !obj.hierarchy.children.is_empty();
        let is_expanded = self.is_expanded(obj.id);

        // Calculate indentation
        let indent = depth as f32 * 16.0;

        ui.horizontal(|ui| {
            // Indent
            ui.add_space(indent);

            // Expand/collapse button
            if has_children {
                let arrow = if is_expanded { "v" } else { ">" };
                if ui.small_button(arrow).clicked() {
                    self.toggle_expanded(obj.id);
                }
            } else {
                // Placeholder for alignment
                ui.add_space(20.0);
            }

            // Icon
            let icon = if obj.visible { "o" } else { "x" };

            // Build label
            let text = format!("{} {}", icon, obj.name);
            let mut rich_text = RichText::new(text);

            if !obj.visible {
                rich_text = rich_text.color(Color32::GRAY);
            }
            if is_hovered && !is_selected {
                rich_text = rich_text.color(Color32::LIGHT_BLUE);
            }

            // Simple selectable label (no drag to avoid egui bug)
            let response = ui.selectable_label(is_selected, rich_text);

            // Handle click for selection
            if response.clicked() {
                if ui.input(|i| i.modifiers.ctrl) {
                    selection.toggle(obj.id);
                } else {
                    selection.select(obj.id);
                }
            }

            // Handle double-click for focus
            if response.double_clicked() {
                action = HierarchyAction::Focus(obj.id);
            }

            // Context menu with reparenting options
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

                // Reparent options
                if obj.hierarchy.parent.is_some() {
                    if ui.button("Unparent (move to root)").clicked() {
                        action = HierarchyAction::Reparent(obj.id, None);
                        ui.close();
                    }
                    ui.separator();
                }

                // Show "Set Parent" submenu with available parents
                ui.menu_button("Set Parent...", |ui| {
                    // Option to move to root
                    if obj.hierarchy.parent.is_some() {
                        if ui.button("(Root)").clicked() {
                            action = HierarchyAction::Reparent(obj.id, None);
                            ui.close();
                        }
                        ui.separator();
                    }

                    // List all possible parents (excluding self and descendants)
                    for potential_parent in objects {
                        // Skip self
                        if potential_parent.id == obj.id {
                            continue;
                        }
                        // Skip current parent
                        if obj.hierarchy.parent == Some(potential_parent.id) {
                            continue;
                        }
                        // Skip descendants (would create cycle)
                        if obj.hierarchy.children.contains(&potential_parent.id) {
                            continue;
                        }
                        // Check if potential_parent is a descendant
                        let mut is_descendant = false;
                        let mut check_id = Some(potential_parent.id);
                        while let Some(id) = check_id {
                            if let Some(check_obj) = objects.iter().find(|o| o.id == id) {
                                if check_obj.hierarchy.parent == Some(obj.id) {
                                    is_descendant = true;
                                    break;
                                }
                                check_id = check_obj.hierarchy.parent;
                            } else {
                                break;
                            }
                        }
                        if is_descendant {
                            continue;
                        }

                        if ui.button(&potential_parent.name).clicked() {
                            action = HierarchyAction::Reparent(obj.id, Some(potential_parent.id));
                            ui.close();
                        }
                    }
                });

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
    /// Reparent an object (child_id, new_parent_id)
    Reparent(ObjectId, Option<ObjectId>),
}

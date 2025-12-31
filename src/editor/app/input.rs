//! Viewport input handling: ray casting, gizmo interaction, multi-selection.

use glam::Vec3;

use crate::render::Camera;
use crate::editor::selection::{Ray, pick_object};
use crate::editor::panels::Tool;
use crate::editor::gizmos::GizmoAxis;
use super::EditorApp;

impl EditorApp {
    /// Create a ray from screen position
    pub fn create_ray(&self, pos: egui::Pos2) -> Option<Ray> {
        let local_x = pos.x - self.viewport_rect.min.x;
        let local_y = pos.y - self.viewport_rect.min.y;
        let width = self.viewport_rect.width();
        let height = self.viewport_rect.height();

        if width <= 0.0 || height <= 0.0 {
            return None;
        }

        let view_proj = self.camera.view_projection();
        let view_proj_inverse = view_proj.inverse();

        Some(Ray::from_screen(local_x, local_y, width, height, view_proj_inverse))
    }

    /// Calculate the center of all selected objects (for gizmo positioning)
    /// Uses world positions to account for hierarchy
    fn selection_center(&self) -> Option<Vec3> {
        let selected = self.selection.all();
        if selected.is_empty() {
            return None;
        }

        let mut sum = Vec3::ZERO;
        let mut count = 0;

        for id in selected {
            if let Some(obj) = self.scene_objects.iter().find(|o| o.id == *id) {
                // Use world position to account for parent hierarchy
                sum += obj.world_position(&self.scene_objects);
                count += 1;
            }
        }

        if count > 0 {
            Some(sum / count as f32)
        } else {
            None
        }
    }

    /// Handle viewport click for selection or gizmo interaction
    pub fn handle_viewport_click(&mut self, pos: egui::Pos2) {
        let Some(ray) = self.create_ray(pos) else { return };
        let modifiers = self.input_modifiers;

        // If we have a selection and a tool that uses gizmo, check gizmo hit first
        if self.toolbar_panel.current_tool != Tool::Select && !self.selection.is_empty() {
            // Use selection center for gizmo position
            if let Some(gizmo_pos) = self.selection_center() {
                let gizmo_scale = self.camera.distance * 0.08;
                let axis = self.gizmo.hit_test(&ray, gizmo_pos, gizmo_scale);

                if axis != GizmoAxis::None {
                    // Collect transforms of all selected objects for batch tracking
                    let selected_ids: Vec<u32> = self.selection.all().to_vec();
                    let transforms: Vec<(Vec3, Vec3, Vec3)> = selected_ids.iter()
                        .filter_map(|id| {
                            self.scene_objects.iter().find(|o| o.id == *id)
                                .map(|o| (o.position, o.rotation, o.scale))
                        })
                        .collect();

                    // Start gizmo drag
                    let first_transform = transforms.first().copied().unwrap_or_default();
                    self.gizmo.begin_drag(axis, &ray, gizmo_pos, first_transform);

                    // Start batch command tracking
                    self.command_history.begin_drag_batch(&selected_ids, transforms);
                    log::info!("Started batch gizmo drag on axis {:?} ({} objects)", axis, selected_ids.len());
                    return;
                }
            }
        }

        // Normal object picking with modifier support
        if let Some(id) = pick_object(&ray, &self.scene_objects) {
            if modifiers.ctrl {
                // Ctrl+click: toggle selection
                self.selection.toggle(id);
                log::info!("Toggled object {} (now {})", id,
                    if self.selection.is_selected(id) { "selected" } else { "deselected" });
            } else if modifiers.shift {
                // Shift+click: add to selection
                self.selection.add(id);
                log::info!("Added object {} to selection ({} total)", id, self.selection.count());
            } else {
                // Normal click: replace selection
                self.selection.select(id);
                log::info!("Selected object {}", id);
            }
        } else if !modifiers.ctrl && !modifiers.shift {
            // Click on empty space: clear selection (unless holding modifier)
            self.selection.clear();
        }
    }

    /// Handle viewport drag for gizmo manipulation (multi-selection)
    pub fn handle_viewport_drag(&mut self, pos: egui::Pos2) {
        if !self.gizmo.is_dragging() {
            return;
        }

        let Some(ray) = self.create_ray(pos) else { return };

        // Get gizmo position (center of selection)
        let Some(gizmo_pos) = self.selection_center() else { return };

        // Update gizmo and get delta
        if let Some(delta) = self.gizmo.update_drag(&ray, gizmo_pos) {
            // Apply delta to ALL selected objects
            let selected_ids: Vec<u32> = self.selection.all().to_vec();
            let mut new_transforms = Vec::new();

            for id in &selected_ids {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *id) {
                    // Apply delta
                    let new_pos = obj.position + delta.translation;
                    let new_rot = obj.rotation + delta.rotation;
                    let new_scale = (obj.scale + delta.scale).max(Vec3::splat(0.1));

                    // Apply snap if enabled
                    if self.snap_settings.enabled {
                        obj.position = self.snap_settings.snap_position(new_pos);
                        obj.rotation = self.snap_settings.snap_rotation(new_rot);
                        obj.scale = self.snap_settings.snap_scale(new_scale);
                    } else {
                        obj.position = new_pos;
                        obj.rotation = new_rot;
                        obj.scale = new_scale;
                    }

                    new_transforms.push((obj.position, obj.rotation, obj.scale));
                }
            }

            // Update batch command tracking
            self.command_history.update_drag_batch(new_transforms);
        }
    }

    /// Handle viewport mouse release
    pub fn handle_viewport_release(&mut self) {
        if self.gizmo.is_dragging() {
            self.gizmo.end_drag();
            self.command_history.end_drag_batch();
            self.scene_manager.mark_dirty();
            log::info!("Ended gizmo drag");
        }
    }

    /// Handle camera controls from viewport input
    pub fn handle_camera_input(&mut self, response: &egui::Response, ui: &egui::Ui) {
        // Scroll: Zoom (orthographic)
        if response.hovered() {
            let scroll = ui.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                // Zoom in/out (affects orthographic projection size)
                let zoom_speed = 0.1;
                self.camera.zoom *= 1.0 - scroll * zoom_speed * 0.01;
                self.camera.zoom = self.camera.zoom.clamp(1.0, 50.0);
            }
        }

        // Middle mouse drag: Orbit (Shift: Pan)
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            let modifiers = self.input_modifiers;

            if modifiers.shift {
                // Pan camera
                let pan_speed = self.camera.distance * 0.005;

                // Calculate right and up vectors from camera orientation
                let yaw = self.camera.yaw;
                let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
                let up = Vec3::Y;

                self.camera.target -= right * delta.x * pan_speed;
                self.camera.target += up * delta.y * pan_speed;
            } else {
                // Orbit camera
                let orbit_speed = 0.005;
                self.camera.yaw += delta.x * orbit_speed;
                self.camera.pitch -= delta.y * orbit_speed;
                self.camera.pitch = self.camera.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
            }
        }

        // Right mouse drag: Alternative orbit
        if response.dragged_by(egui::PointerButton::Secondary) && !self.gizmo.is_dragging() {
            let delta = response.drag_delta();
            let orbit_speed = 0.005;
            self.camera.yaw += delta.x * orbit_speed;
            self.camera.pitch -= delta.y * orbit_speed;
            self.camera.pitch = self.camera.pitch.clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());
        }
    }
}

//! Editor UI: main drawing coordination and panel layout.

use glam::Vec3;

#[allow(unused_imports)]
use crate::render::IsometricCamera;
use crate::editor::selection::SceneObject;
use crate::editor::panels::{HierarchyAction, ToolbarAction, Tool, AssetAction};
use crate::editor::gizmos::GizmoAxis;
use super::state::InputModifiers;
use super::EditorApp;

/// Simple pseudo-random float [0, 1)
pub fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32 / 1_000_000_000.0).fract()
}

impl EditorApp {
    /// Draw the editor UI
    pub fn draw_ui(&mut self, viewport_size: &mut (f32, f32)) {
        let Some(egui) = &self.egui else { return };
        let egui_ctx = egui.ctx().clone();

        // Draw UI components (methods from other modules)
        self.draw_menu_bar(&egui_ctx);
        self.draw_toolbar(&egui_ctx);
        self.draw_hierarchy_panel(&egui_ctx);
        self.draw_inspector_panel(&egui_ctx);
        self.draw_asset_browser_panel(&egui_ctx);
        self.draw_status_bar(&egui_ctx, viewport_size);
        self.draw_viewport_panel(&egui_ctx, viewport_size);
        self.draw_file_dialog(&egui_ctx);
        self.draw_prefab_dialog(&egui_ctx);
        self.draw_script_dialog(&egui_ctx);
        self.draw_attach_script_dialog(&egui_ctx);
        self.draw_project_dialog(&egui_ctx);
        self.draw_splash_screen(&egui_ctx);
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Play/Stop buttons
                if self.is_playing {
                    if ui.add(egui::Button::new("⏹ Stop").fill(egui::Color32::from_rgb(180, 60, 60))).clicked() {
                        self.stop_play();
                    }
                    ui.label(format!("▶ {:.1}s", self.play_time));
                } else {
                    if ui.add(egui::Button::new("▶ Play").fill(egui::Color32::from_rgb(60, 180, 60))).clicked() {
                        self.start_play();
                    }
                }

                ui.separator();

                let action = self.toolbar_panel.show(ui);
                match action {
                    ToolbarAction::CenterView => {
                        if let Some(id) = self.selection.first() {
                            self.focus_on_object(id);
                        }
                    }
                    ToolbarAction::FrameAll => {
                        self.camera.target = Vec3::ZERO;
                        self.camera.distance = 30.0;
                    }
                    ToolbarAction::None => {}
                }
            });
        });
    }

    fn draw_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("hierarchy")
            .default_width(200.0)
            .show(ctx, |ui| {
                let action = self.hierarchy_panel.show(ui, &mut self.scene_objects, &mut self.selection);
                match action {
                    HierarchyAction::CreateCube => {
                        let obj = SceneObject::cube(self.next_id, Vec3::new(0.0, 0.5, 0.0));
                        self.next_id += 1;
                        self.create_object(obj);
                    }
                    HierarchyAction::CreateEmpty => {
                        let mut obj = SceneObject::new(self.next_id, format!("Empty {}", self.next_id));
                        self.next_id += 1;
                        obj.visible = true;
                        self.create_object(obj);
                    }
                    HierarchyAction::Delete(id) => {
                        self.delete_object(id);
                    }
                    HierarchyAction::Duplicate(id) => {
                        self.duplicate_object(id);
                    }
                    HierarchyAction::Focus(id) => {
                        self.focus_on_object(id);
                    }
                    HierarchyAction::ToggleVisibility(id) => {
                        if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                            obj.visible = !obj.visible;
                        }
                    }
                    HierarchyAction::Rename(_) => {}
                    HierarchyAction::None => {}
                }
            });
    }

    fn draw_inspector_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .default_width(280.0)
            .show(ctx, |ui| {
                self.inspector_panel.show(ui, &mut self.scene_objects, &self.selection);

                ui.separator();

                // Camera controls
                ui.heading("Camera");
                ui.horizontal(|ui| {
                    ui.label("Distance:");
                    ui.add(egui::DragValue::new(&mut self.camera.distance).speed(0.1).range(1.0..=100.0));
                });

                let mut pitch_deg = self.camera.pitch.to_degrees();
                ui.horizontal(|ui| {
                    ui.label("Pitch:");
                    if ui.add(egui::DragValue::new(&mut pitch_deg).speed(1.0).range(-89.0..=89.0).suffix("°")).changed() {
                        self.camera.pitch = pitch_deg.to_radians();
                    }
                });

                let mut yaw_deg = self.camera.yaw.to_degrees();
                ui.horizontal(|ui| {
                    ui.label("Yaw:");
                    if ui.add(egui::DragValue::new(&mut yaw_deg).speed(1.0).suffix("°")).changed() {
                        self.camera.yaw = yaw_deg.to_radians();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    ui.add(egui::DragValue::new(&mut self.camera.zoom).speed(0.1).range(1.0..=50.0));
                });

                ui.separator();

                // Snap settings
                ui.heading("Snap");
                ui.checkbox(&mut self.snap_settings.enabled, "Enable Snap");

                if self.snap_settings.enabled {
                    ui.horizontal(|ui| {
                        ui.label("Grid:");
                        ui.add(egui::DragValue::new(&mut self.snap_settings.grid_size)
                            .speed(0.1)
                            .range(0.1..=10.0)
                            .suffix(" u"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.add(egui::DragValue::new(&mut self.snap_settings.rotation_snap)
                            .speed(1.0)
                            .range(1.0..=90.0)
                            .suffix("°"));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(egui::DragValue::new(&mut self.snap_settings.scale_snap)
                            .speed(0.05)
                            .range(0.01..=1.0));
                    });

                    ui.horizontal(|ui| {
                        if ui.small_button("0.5").clicked() {
                            self.snap_settings.grid_size = 0.5;
                        }
                        if ui.small_button("1.0").clicked() {
                            self.snap_settings.grid_size = 1.0;
                        }
                        if ui.small_button("2.0").clicked() {
                            self.snap_settings.grid_size = 2.0;
                        }
                    });
                }

                // Scripts section
                self.draw_scripts_section(ui);
            });
    }

    fn draw_scripts_section(&mut self, ui: &mut egui::Ui) {
        let Some(obj_id) = self.selection.first() else { return };

        ui.separator();
        ui.heading("Scripts");

        let script_ids: Vec<u32> = self.scene_objects.iter()
            .find(|o| o.id == obj_id)
            .map(|o| o.scripts.clone())
            .unwrap_or_default();

        if script_ids.is_empty() {
            ui.label("No scripts attached");
        } else {
            let mut remove_script_id = None;
            for script_id in &script_ids {
                ui.horizontal(|ui| {
                    #[cfg(feature = "scripting")]
                    {
                        if let Some(script) = self.script_runtime.get_script(*script_id) {
                            ui.label(format!("[{}] {}", script_id, &script.name));
                        } else {
                            ui.label(format!("[{}] <unknown>", script_id));
                        }
                    }
                    #[cfg(not(feature = "scripting"))]
                    {
                        ui.label(format!("[{}]", script_id));
                    }

                    if ui.small_button("X").clicked() {
                        remove_script_id = Some(*script_id);
                    }
                });
            }

            if let Some(script_id) = remove_script_id {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == obj_id) {
                    obj.scripts.retain(|&id| id != script_id);
                }
                #[cfg(feature = "scripting")]
                self.script_runtime.detach_script(script_id);
            }
        }

        #[cfg(feature = "scripting")]
        {
            ui.horizontal(|ui| {
                if ui.button("New Script...").clicked() {
                    self.show_script_dialog = true;
                    self.script_path_input.clear();
                }
                if ui.button("Attach Script...").clicked() {
                    self.show_attach_script_dialog = true;
                }
            });
        }

        #[cfg(not(feature = "scripting"))]
        {
            ui.add_enabled(false, egui::Button::new("New Script..."));
            ui.add_enabled(false, egui::Button::new("Attach Script..."));
            ui.label("(Enable 'scripting' feature)");
        }
    }

    fn draw_asset_browser_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("asset_browser_panel")
            .resizable(true)
            .default_height(180.0)
            .min_height(100.0)
            .show(ctx, |ui| {
                let action = self.asset_browser.show(ui);
                match action {
                    AssetAction::OpenScene(path) => {
                        self.load_scene(path);
                    }
                    AssetAction::LoadPrefab(path) => {
                        self.load_prefab(path);
                    }
                    AssetAction::OpenDirectory(_) => {}
                    AssetAction::None => {}
                }
            });
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context, viewport_size: &(f32, f32)) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let dirty = if self.scene_manager.is_dirty() { "*" } else { "" };
                ui.label(format!("{}{}", dirty, self.scene_manager.scene_name()));
                ui.separator();

                ui.label(format!("Objects: {}", self.scene_objects.len()));
                ui.separator();
                ui.label(format!("Selected: {}", self.selection.count()));
                ui.separator();
                ui.label(format!("Tool: {}", self.toolbar_panel.current_tool.name()));
                ui.separator();

                let undo_count = self.command_history.undo_count();
                let redo_count = self.command_history.redo_count();
                if undo_count > 0 || redo_count > 0 {
                    ui.label(format!("History: {} undo, {} redo", undo_count, redo_count));
                    ui.separator();
                }

                ui.label(format!("Viewport: {:.0}x{:.0}", viewport_size.0, viewport_size.1));
            });
        });
    }

    fn draw_viewport_panel(&mut self, ctx: &egui::Context, viewport_size: &mut (f32, f32)) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            *viewport_size = (available.x, available.y);

            if let Some(viewport) = &self.viewport {
                let texture_id = viewport.texture_id();
                let size = egui::vec2(available.x, available.y);

                let response = ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture_id, size))
                        .sense(egui::Sense::click_and_drag())
                );

                self.viewport_rect = response.rect;

                ui.input(|i| {
                    self.input_modifiers = InputModifiers {
                        ctrl: i.modifiers.ctrl,
                        shift: i.modifiers.shift,
                        alt: i.modifiers.alt,
                    };
                });

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_viewport_click(pos);
                    }
                }

                if response.drag_started() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_viewport_click(pos);
                    }
                }

                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_viewport_drag(pos);
                    }
                }

                if response.drag_stopped() {
                    self.handle_viewport_release();
                }

                self.handle_camera_input(&response, ui);

                if response.hovered() && !self.gizmo.is_dragging() {
                    self.mouse_in_viewport = ui.input(|i| i.pointer.hover_pos());
                    if let Some(pos) = self.mouse_in_viewport {
                        if let Some(ray) = self.create_ray(pos) {
                            if self.toolbar_panel.current_tool != Tool::Select {
                                if let Some(id) = self.selection.first() {
                                    if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
                                        let gizmo_scale = self.camera.distance * 0.08;
                                        let axis = self.gizmo.hit_test(&ray, obj.position, gizmo_scale);
                                        self.gizmo.set_hovered(axis);
                                    }
                                }
                            }
                        }
                    }
                } else if !response.hovered() {
                    self.mouse_in_viewport = None;
                    self.gizmo.set_hovered(GizmoAxis::None);
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Viewport not initialized");
                });
            }
        });
    }
}

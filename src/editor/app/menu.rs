//! Menu bar drawing for the editor.

use glam::Vec3;

use super::ui::rand_float;
use super::EditorApp;
use crate::editor::components::ParticlePreset;
use crate::editor::selection::SceneObject;
use crate::editor::shortcuts::EditorAction;
use crate::render::IsometricCamera;

impl EditorApp {
    /// Draw the main menu bar
    pub(super) fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                self.draw_file_menu(ui);
                self.draw_edit_menu(ui);
                self.draw_create_menu(ui);
                self.draw_view_menu(ui);
            });
        });
    }

    fn draw_edit_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Edit", |ui| {
            let undo_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Undo);
            let undo_label = if let Some(cmd) = self.command_history.peek_undo() {
                format!("Undo {}", cmd.description())
            } else {
                "Undo".to_string()
            };
            if ui
                .add_enabled(
                    self.command_history.can_undo(),
                    egui::Button::new(&undo_label).shortcut_text(&undo_shortcut),
                )
                .clicked()
            {
                self.undo();
                ui.close();
            }

            let redo_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Redo);
            let redo_label = if let Some(cmd) = self.command_history.peek_redo() {
                format!("Redo {}", cmd.description())
            } else {
                "Redo".to_string()
            };
            if ui
                .add_enabled(
                    self.command_history.can_redo(),
                    egui::Button::new(&redo_label).shortcut_text(&redo_shortcut),
                )
                .clicked()
            {
                self.redo();
                ui.close();
            }

            ui.separator();

            let cut_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Cut);
            if ui
                .add_enabled(
                    !self.selection.is_empty(),
                    egui::Button::new("Cut").shortcut_text(&cut_shortcut),
                )
                .clicked()
            {
                self.cut_selected();
                ui.close();
            }

            let copy_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Copy);
            if ui
                .add_enabled(
                    !self.selection.is_empty(),
                    egui::Button::new("Copy").shortcut_text(&copy_shortcut),
                )
                .clicked()
            {
                self.copy_selected();
                ui.close();
            }

            let paste_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Paste);
            if ui
                .add_enabled(
                    self.has_clipboard(),
                    egui::Button::new("Paste").shortcut_text(&paste_shortcut),
                )
                .clicked()
            {
                self.paste();
                ui.close();
            }

            ui.separator();

            let delete_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Delete);
            if ui
                .add_enabled(
                    !self.selection.is_empty(),
                    egui::Button::new("Delete Selected").shortcut_text(&delete_shortcut),
                )
                .clicked()
            {
                let ids: Vec<u32> = self.selection.all().to_vec();
                for id in ids {
                    self.delete_object(id);
                }
                ui.close();
            }

            let duplicate_shortcut = self.shortcuts.get_shortcut_text(EditorAction::Duplicate);
            if ui
                .add_enabled(
                    self.selection.first().is_some(),
                    egui::Button::new("Duplicate Selected").shortcut_text(&duplicate_shortcut),
                )
                .clicked()
            {
                if let Some(id) = self.selection.first() {
                    self.duplicate_object(id);
                }
                ui.close();
            }
        });
    }

    fn draw_create_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Create", |ui| {
            // Basic objects
            if ui.button("Cube").clicked() {
                let obj = SceneObject::cube(self.next_id, Vec3::new(0.0, 0.5, 0.0));
                self.next_id += 1;
                self.create_object(obj);
                ui.close();
            }
            if ui.button("Camera").clicked() {
                let obj = SceneObject::camera(self.next_id, Vec3::new(0.0, 5.0, -10.0));
                self.next_id += 1;
                self.create_object(obj);
                ui.close();
            }
            if ui.button("Random Cubes (5)").clicked() {
                for _ in 0..5 {
                    let x = (rand_float() - 0.5) * 10.0;
                    let z = (rand_float() - 0.5) * 10.0;
                    let obj = SceneObject::cube(self.next_id, Vec3::new(x, 0.5, z));
                    self.next_id += 1;
                    self.scene_objects.push(obj);
                }
                log::info!("Created 5 random cubes");
                ui.close();
            }

            ui.separator();

            // Audio submenu
            self.draw_audio_submenu(ui);

            // Effects submenu (particles)
            self.draw_effects_submenu(ui);

            // Animation submenu
            self.draw_animation_submenu(ui);

            ui.separator();

            // Import 3D model
            if ui.button("Import 3D Model...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("3D Models", &["obj"])
                    .add_filter("All files", &["*"])
                    .set_title("Import 3D Model")
                    .pick_file()
                {
                    match crate::editor::mesh::load_obj(&path) {
                        Ok(meshes) => {
                            let mesh_path_str = path.to_string_lossy().to_string();
                            for mesh in meshes {
                                log::info!(
                                    "Imported mesh '{}': {} vertices, {} triangles",
                                    mesh.name,
                                    mesh.vertex_count(),
                                    mesh.triangle_count()
                                );

                                let center = mesh.center();
                                let mut obj = SceneObject::new(self.next_id, &mesh.name);
                                obj.position = center;
                                obj.mesh_path = Some(mesh_path_str.clone());
                                self.next_id += 1;
                                self.create_object(obj);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to import model: {}", e);
                        }
                    }
                }
                ui.close();
            }

            ui.separator();

            // Prefab submenu
            self.draw_prefab_submenu(ui);
        });
    }

    fn draw_audio_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Audio", |ui| {
            if ui.button("Audio Source").clicked() {
                let obj = SceneObject::audio_source(self.next_id, Vec3::new(0.0, 1.0, 0.0));
                self.next_id += 1;
                self.create_object(obj);
                log::info!("Created Audio Source");
                ui.close();
            }
            if ui.button("Audio Listener").clicked() {
                let obj = SceneObject::audio_listener(self.next_id, Vec3::new(0.0, 1.5, 0.0));
                self.next_id += 1;
                self.create_object(obj);
                log::info!("Created Audio Listener");
                ui.close();
            }
        });
    }

    fn draw_effects_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Effects", |ui| {
            ui.menu_button("Particle Emitter", |ui| {
                for preset in ParticlePreset::all() {
                    if ui.button(preset.name()).clicked() {
                        let obj = SceneObject::particle_emitter(
                            self.next_id,
                            Vec3::new(0.0, 1.0, 0.0),
                            *preset,
                        );
                        self.next_id += 1;
                        self.create_object(obj);
                        log::info!("Created Particle Emitter with {} preset", preset.name());
                        ui.close();
                    }
                }
            });
        });
    }

    fn draw_animation_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Animation", |ui| {
            if ui.button("Animated Object").clicked() {
                let obj = SceneObject::animated(self.next_id, Vec3::new(0.0, 0.5, 0.0));
                self.next_id += 1;
                self.create_object(obj);
                log::info!("Created Animated Object");
                ui.close();
            }

            ui.separator();

            if ui.button("Create Cutscene...").clicked() {
                self.show_cutscene_dialog = true;
                ui.close();
            }

            if ui.button("New Timeline Sequence").clicked() {
                use crate::timeline::TimelineSequence;
                let seq = TimelineSequence::new("New Sequence");
                self.timeline_panel.set_sequence(seq);
                self.show_timeline = true;
                log::info!("Created new timeline sequence");
                ui.close();
            }
        });
    }

    fn draw_prefab_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Prefab", |ui| {
            if ui
                .add_enabled(
                    !self.selection.is_empty(),
                    egui::Button::new("Create from Selection..."),
                )
                .clicked()
            {
                self.create_prefab_with_file_dialog();
                ui.close();
            }

            if ui.button("Load Prefab...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Xtreme Prefab", &["xpfb"])
                    .set_title("Load Prefab")
                    .pick_file()
                {
                    self.load_prefab(path);
                }
                ui.close();
            }

            ui.separator();

            // Prefab Registry controls
            let registry_count = self.prefab_registry.len();
            let modified_count = self.prefab_registry.get_modified_prefabs().len();

            ui.label(format!(
                "Registry: {} loaded{}",
                registry_count,
                if modified_count > 0 {
                    format!(" ({} modified)", modified_count)
                } else {
                    String::new()
                }
            ));

            if ui.button("Reload All Prefabs").clicked() {
                self.reload_all_prefabs();
                ui.close();
            }

            if modified_count > 0 && ui.button("Reload Modified Only").clicked() {
                self.reload_modified_prefabs();
                ui.close();
            }

            ui.separator();

            // Replace selected with prefab (for nested prefabs)
            if !self.prefabs.is_empty() && !self.selection.is_empty() {
                ui.menu_button("Replace Selected with Prefab", |ui| {
                    let prefab_names: Vec<(usize, String)> = self
                        .prefabs
                        .iter()
                        .enumerate()
                        .map(|(i, p)| (i, p.name.clone()))
                        .collect();

                    for (i, name) in prefab_names {
                        if ui.button(&name).clicked() {
                            self.replace_selected_with_prefab(i);
                            ui.close();
                        }
                    }
                });

                ui.separator();
            }

            // Instantiate prefab list
            if self.prefabs.is_empty() {
                ui.label("(No prefabs)");
            } else {
                ui.label("Instantiate:");
                let prefab_info: Vec<(usize, String, usize)> = self
                    .prefabs
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (i, p.name.clone(), p.object_count()))
                    .collect();

                let mut instantiate_idx = None;
                for (i, name, count) in prefab_info {
                    let label = format!("{} ({} objs)", name, count);
                    if ui.button(&label).clicked() {
                        instantiate_idx = Some(i);
                        ui.close();
                    }
                }

                if let Some(idx) = instantiate_idx {
                    let pos = self.camera.target;
                    self.instantiate_prefab(idx, pos);
                }
            }
        });
    }

    fn draw_view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            // Panel toggles
            let console_label = if self.console_panel.visible {
                "✓ Console"
            } else {
                "  Console"
            };
            if ui.button(console_label).clicked() {
                self.console_panel.visible = !self.console_panel.visible;
                ui.close();
            }

            let timeline_label = if self.show_timeline {
                "✓ Timeline"
            } else {
                "  Timeline"
            };
            if ui.button(timeline_label).clicked() {
                self.show_timeline = !self.show_timeline;
                ui.close();
            }

            ui.separator();

            // Camera views
            if ui.button("Reset Camera").clicked() {
                self.camera = IsometricCamera::default();
                ui.close();
            }
            if ui.button("Top View").clicked() {
                self.camera.pitch = -89.0_f32.to_radians();
                self.camera.yaw = 0.0;
                ui.close();
            }
            if ui.button("Front View").clicked() {
                self.camera.pitch = 0.0;
                self.camera.yaw = 0.0;
                ui.close();
            }
            if ui.button("Focus Selected").clicked() {
                if let Some(id) = self.selection.first() {
                    self.focus_on_object(id);
                }
                ui.close();
            }
        });
    }
}

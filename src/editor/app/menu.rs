//! Menu bar drawing for the editor.

use glam::Vec3;

use super::state::FileDialogAction;
use super::ui::rand_float;
use super::EditorApp;
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

    fn draw_file_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            let new_shortcut = self.shortcuts.get_shortcut_text(EditorAction::NewScene);
            if ui
                .add(egui::Button::new("New Scene").shortcut_text(&new_shortcut))
                .clicked()
            {
                self.new_scene();
                ui.close();
            }

            let open_shortcut = self.shortcuts.get_shortcut_text(EditorAction::OpenScene);
            if ui
                .add(egui::Button::new("Open...").shortcut_text(&open_shortcut))
                .clicked()
            {
                self.file_dialog_action = Some(FileDialogAction::Open);
                ui.close();
            }

            ui.separator();

            let save_shortcut = self.shortcuts.get_shortcut_text(EditorAction::SaveScene);
            let can_save = self.scene_manager.current_path().is_some();
            if ui
                .add_enabled(
                    can_save,
                    egui::Button::new("Save").shortcut_text(&save_shortcut),
                )
                .clicked()
            {
                if let Some(path) = self.scene_manager.current_path() {
                    self.save_scene(path.to_path_buf());
                }
                ui.close();
            }

            let save_as_shortcut = self.shortcuts.get_shortcut_text(EditorAction::SaveSceneAs);
            if ui
                .add(egui::Button::new("Save As...").shortcut_text(&save_as_shortcut))
                .clicked()
            {
                self.file_dialog_action = Some(FileDialogAction::SaveAs);
                ui.close();
            }

            ui.separator();

            // Project submenu
            self.draw_project_submenu(ui);

            ui.separator();
            if ui.button("Exit").clicked() {
                std::process::exit(0);
            }
        });
    }

    fn draw_project_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Project", |ui| {
            if ui.button("New Project...").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Select Project Location")
                    .pick_folder()
                {
                    let name = folder
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("New Project")
                        .to_string();

                    match crate::editor::project::Project::create(folder, &name) {
                        Ok(project) => {
                            self.asset_browser.set_root(project.root.clone());

                            // Set scripts directory for Python runtime
                            #[cfg(feature = "scripting")]
                            {
                                let scripts_dir = project.scripts_dir();
                                self.script_runtime.set_scripts_dir(scripts_dir.clone());
                                // Ensure xtreme module exists in scripts directory
                                self.ensure_xtreme_module(&scripts_dir);
                            }

                            self.current_project = Some(project);
                            log::info!("Created new project");
                        }
                        Err(e) => log::error!("Failed to create project: {}", e),
                    }
                }
                ui.close();
            }

            if ui.button("Open Project...").clicked() {
                if let Some(folder) = rfd::FileDialog::new()
                    .set_title("Open Project")
                    .pick_folder()
                {
                    match crate::editor::project::Project::open(folder) {
                        Ok(project) => {
                            self.asset_browser.set_root(project.root.clone());

                            // Set scripts directory for Python runtime
                            #[cfg(feature = "scripting")]
                            {
                                let scripts_dir = project.scripts_dir();
                                self.script_runtime.set_scripts_dir(scripts_dir.clone());
                                // Ensure xtreme module exists in scripts directory
                                self.ensure_xtreme_module(&scripts_dir);
                            }

                            let main_scene = project.main_scene_path();
                            if main_scene.exists() {
                                self.load_scene(main_scene);
                            }
                            self.current_project = Some(project);
                            log::info!("Opened project");
                        }
                        Err(e) => log::error!("Failed to open project: {}", e),
                    }
                }
                ui.close();
            }

            ui.separator();

            let has_project = self.current_project.is_some();
            let mut save_project = false;
            if ui
                .add_enabled(has_project, egui::Button::new("Save Project"))
                .clicked()
            {
                save_project = true;
                ui.close();
            }

            if save_project {
                let main_scene = self.current_project.as_ref().map(|p| p.main_scene_path());
                if let Some(path) = main_scene {
                    self.save_scene(path);
                }
                if let Some(ref project) = self.current_project {
                    if let Err(e) = project.save() {
                        log::error!("Failed to save project: {}", e);
                    } else {
                        log::info!("Project saved");
                    }
                }
            }

            if ui
                .add_enabled(has_project, egui::Button::new("Properties..."))
                .clicked()
            {
                self.show_project_dialog = true;
                ui.close();
            }
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
                            for mesh in meshes {
                                log::info!(
                                    "Imported mesh '{}': {} vertices, {} triangles",
                                    mesh.name,
                                    mesh.vertex_count(),
                                    mesh.triangle_count()
                                );

                                let center = mesh.center();
                                let mut obj = SceneObject::cube(self.next_id, center);
                                obj.name = mesh.name;
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

    fn draw_prefab_submenu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Prefab", |ui| {
            if ui
                .add_enabled(
                    !self.selection.is_empty(),
                    egui::Button::new("Create from Selection..."),
                )
                .clicked()
            {
                self.prefab_name_input = format!("Prefab {}", self.prefabs.len() + 1);
                self.show_prefab_dialog = true;
                ui.close();
            }

            ui.separator();

            if self.prefabs.is_empty() {
                ui.label("(No prefabs)");
            } else {
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

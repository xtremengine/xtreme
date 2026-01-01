//! File and Project menu drawing.

use super::state::FileDialogAction;
use super::EditorApp;
use crate::editor::shortcuts::EditorAction;

impl EditorApp {
    pub(super) fn draw_file_menu(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn draw_project_submenu(&mut self, ui: &mut egui::Ui) {
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

                            #[cfg(feature = "scripting")]
                            {
                                let scripts_dir = project.scripts_dir();
                                self.script_runtime.set_scripts_dir(scripts_dir.clone());
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

                            #[cfg(feature = "scripting")]
                            {
                                let scripts_dir = project.scripts_dir();
                                self.script_runtime.set_scripts_dir(scripts_dir.clone());
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
}

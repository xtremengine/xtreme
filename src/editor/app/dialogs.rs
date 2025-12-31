//! Dialog windows for the editor.

use std::path::PathBuf;
use super::state::FileDialogAction;
use super::EditorApp;

impl EditorApp {
    /// Draw file open/save dialog
    pub(super) fn draw_file_dialog(&mut self, _ctx: &egui::Context) {
        if let Some(action) = self.file_dialog_action.take() {
            match action {
                FileDialogAction::Open => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Scene files", &["ron", "json"])
                        .add_filter("All files", &["*"])
                        .set_title("Open Scene")
                        .pick_file()
                    {
                        self.load_scene(path);
                    }
                }
                FileDialogAction::Save | FileDialogAction::SaveAs => {
                    let default_name = self.scene_manager.current_path()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("scene.ron")
                        .to_string();

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("RON scene", &["ron"])
                        .add_filter("JSON scene", &["json"])
                        .set_title("Save Scene")
                        .set_file_name(&default_name)
                        .save_file()
                    {
                        let path = if path.extension().is_none() {
                            path.with_extension("ron")
                        } else {
                            path
                        };
                        self.save_scene(path);
                    }
                }
            }
            self.file_path_input.clear();
        }
    }

    /// Draw prefab creation dialog
    pub(super) fn draw_prefab_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_prefab_dialog {
            return;
        }

        let mut open = true;
        let mut create = false;

        egui::Window::new("Create Prefab")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.prefab_name_input);
                });

                ui.label(format!("Objects: {}", self.selection.count()));

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_prefab_dialog = false;
                        self.prefab_name_input.clear();
                    }

                    if ui.add_enabled(
                        !self.prefab_name_input.is_empty(),
                        egui::Button::new("Create")
                    ).clicked() {
                        create = true;
                    }
                });
            });

        if create {
            let name = self.prefab_name_input.clone();
            self.create_prefab_from_selection(&name);
            self.show_prefab_dialog = false;
            self.prefab_name_input.clear();
        }

        if !open {
            self.show_prefab_dialog = false;
            self.prefab_name_input.clear();
        }
    }

    /// Draw script attachment dialog
    pub(super) fn draw_script_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_script_dialog {
            return;
        }

        let mut open = true;
        let mut attach = false;
        let mut create = false;
        let mut open_in_editor = false;

        let file_exists = if !self.script_path_input.is_empty() && self.script_path_input.ends_with(".py") {
            std::path::Path::new(&self.script_path_input).exists()
        } else {
            false
        };

        egui::Window::new("Attach Script")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Enter path to Python script (.py):");
                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("Path:");
                    ui.text_edit_singleline(&mut self.script_path_input);
                });

                ui.add_space(5.0);

                let valid_path = !self.script_path_input.is_empty() && self.script_path_input.ends_with(".py");

                if valid_path {
                    if file_exists {
                        ui.colored_label(egui::Color32::GREEN, "✓ File exists");
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "⚠ File not found");
                    }
                } else {
                    ui.label("Example: scripts/player.py");
                }

                ui.add_space(5.0);

                // Checkbox to open in editor
                ui.checkbox(&mut self.open_script_in_editor, "Open in default editor");

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_script_dialog = false;
                        self.script_path_input.clear();
                    }

                    if valid_path && !file_exists {
                        if ui.button("Create & Attach").clicked() {
                            create = true;
                            attach = true;
                            open_in_editor = self.open_script_in_editor;
                        }
                    }

                    if ui.add_enabled(valid_path && file_exists, egui::Button::new("Attach")).clicked() {
                        attach = true;
                        open_in_editor = self.open_script_in_editor;
                    }
                });
            });

        if create {
            self.create_script_template();
            if !PathBuf::from(&self.script_path_input).exists() {
                attach = false;
                open_in_editor = false;
            }
        }

        // Open in editor if requested
        if open_in_editor {
            let path = PathBuf::from(&self.script_path_input);
            self.open_file_in_editor(&path);
        }

        #[cfg(feature = "scripting")]
        if attach {
            self.attach_script_to_selected();
        }

        #[cfg(not(feature = "scripting"))]
        if attach {
            log::warn!("Scripting feature not enabled");
            self.show_script_dialog = false;
        }

        if !open {
            self.show_script_dialog = false;
            self.script_path_input.clear();
        }
    }

    /// Open a file in the system's default editor
    fn open_file_in_editor(&self, path: &PathBuf) {
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = std::process::Command::new("cmd")
                .args(["/C", "start", "", path.to_str().unwrap_or("")])
                .spawn()
            {
                log::error!("Failed to open file in editor: {}", e);
            } else {
                log::info!("Opened {:?} in default editor", path);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = std::process::Command::new("open")
                .arg(path)
                .spawn()
            {
                log::error!("Failed to open file in editor: {}", e);
            } else {
                log::info!("Opened {:?} in default editor", path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(e) = std::process::Command::new("xdg-open")
                .arg(path)
                .spawn()
            {
                log::error!("Failed to open file in editor: {}", e);
            } else {
                log::info!("Opened {:?} in default editor", path);
            }
        }
    }

    fn create_script_template(&self) {
        let path = PathBuf::from(&self.script_path_input);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let script_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("script");

        let template = format!(r#"# {}
# Xtreme Engine Script

import xtreme

def _ready(ctx):
    """Called once when the script is attached"""
    xtreme.log_info("{} ready!")

def _update(ctx, delta):
    """Called every frame"""
    pass

def _physics_update(ctx, delta):
    """Called at fixed physics rate (optional)"""
    pass
"#, script_name, script_name);

        match std::fs::write(&path, template) {
            Ok(_) => log::info!("Created script template: {:?}", path),
            Err(e) => log::error!("Failed to create script: {}", e),
        }
    }

    #[cfg(feature = "scripting")]
    fn attach_script_to_selected(&mut self) {
        if let Some(obj_id) = self.selection.first() {
            let path = PathBuf::from(&self.script_path_input);
            match self.script_runtime.attach_script(path, obj_id) {
                Ok(script_id) => {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == obj_id) {
                        obj.scripts.push(script_id);
                    }
                    log::info!("Attached script {} to object {}", script_id, obj_id);
                }
                Err(e) => {
                    log::error!("Failed to attach script: {}", e);
                }
            }
        }
        self.show_script_dialog = false;
        self.script_path_input.clear();
    }

    /// Draw project properties dialog
    pub(super) fn draw_project_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_project_dialog {
            return;
        }

        let Some(ref mut project) = self.current_project else {
            self.show_project_dialog = false;
            return;
        };

        let mut open = true;
        let mut save = false;

        egui::Window::new("Project Properties")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(400.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading("General");
                ui.add_space(5.0);

                egui::Grid::new("project_props_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut project.config.name);
                        ui.end_row();

                        ui.label("Version:");
                        ui.text_edit_singleline(&mut project.config.version);
                        ui.end_row();

                        ui.label("Author:");
                        ui.text_edit_singleline(&mut project.config.author);
                        ui.end_row();

                        ui.label("Description:");
                        ui.text_edit_multiline(&mut project.config.description);
                        ui.end_row();
                    });

                ui.add_space(15.0);
                ui.heading("Build Settings");
                ui.add_space(5.0);

                egui::Grid::new("project_build_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Main Scene:");
                        ui.text_edit_singleline(&mut project.config.main_scene);
                        ui.end_row();

                        ui.label("Window Width:");
                        ui.add(egui::DragValue::new(&mut project.config.window_width).range(320..=3840));
                        ui.end_row();

                        ui.label("Window Height:");
                        ui.add(egui::DragValue::new(&mut project.config.window_height).range(240..=2160));
                        ui.end_row();

                        ui.label("Splash Duration:");
                        ui.add(egui::DragValue::new(&mut project.config.splash_duration)
                            .range(0.0..=10.0)
                            .speed(0.1)
                            .suffix(" s"));
                        ui.end_row();
                    });

                ui.add_space(15.0);
                ui.heading("Project Path");
                ui.label(project.root.display().to_string());

                ui.add_space(15.0);
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_project_dialog = false;
                    }

                    if ui.button("Save").clicked() {
                        save = true;
                    }
                });
            });

        if save {
            if let Some(ref project) = self.current_project {
                match project.save() {
                    Ok(_) => {
                        log::info!("Project properties saved");
                        self.show_project_dialog = false;
                    }
                    Err(e) => log::error!("Failed to save project: {}", e),
                }
            }
        }

        if !open {
            self.show_project_dialog = false;
        }
    }
}

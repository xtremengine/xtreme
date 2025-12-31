//! Project properties dialog.

use crate::editor::app::EditorApp;

impl EditorApp {
    /// Draw project properties dialog
    pub(crate) fn draw_project_dialog(&mut self, ctx: &egui::Context) {
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
            .default_width(420.0)
            .default_height(320.0)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                // Isolate dialog widgets from main window
                ui.push_id("project_dialog_content", |ui| {
                    // Tab bar
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.project_dialog_tab == 0, "General").clicked() {
                            self.project_dialog_tab = 0;
                        }
                        if ui.selectable_label(self.project_dialog_tab == 1, "Build Settings").clicked() {
                            self.project_dialog_tab = 1;
                        }
                        if ui.selectable_label(self.project_dialog_tab == 2, "Window").clicked() {
                            self.project_dialog_tab = 2;
                        }
                    });

                    ui.separator();
                    ui.add_space(10.0);

                    match self.project_dialog_tab {
                        0 => {
                            // General tab
                            ui.push_id("general_tab", |ui| {
                                egui::Grid::new("project_general_grid")
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
                                        ui.add(egui::TextEdit::multiline(&mut project.config.description)
                                            .desired_rows(4));
                                        ui.end_row();
                                    });
                            });
                        }
                        1 => {
                            // Build Settings tab
                            ui.push_id("build_tab", |ui| {
                                egui::Grid::new("project_build_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Main Scene:");
                                        ui.text_edit_singleline(&mut project.config.main_scene);
                                        ui.end_row();

                                        ui.label("Project Path:");
                                        ui.label(project.root.display().to_string());
                                        ui.end_row();
                                    });
                            });
                        }
                        2 => {
                            // Window tab
                            ui.push_id("window_tab", |ui| {
                                egui::Grid::new("project_window_grid")
                                    .num_columns(2)
                                    .spacing([10.0, 8.0])
                                    .show(ui, |ui| {
                                        ui.label("Width:");
                                        ui.add(egui::DragValue::new(&mut project.config.window_width)
                                            .range(320..=3840)
                                            .suffix(" px")
                                            .fixed_decimals(0));
                                        ui.end_row();

                                        ui.label("Height:");
                                        ui.add(egui::DragValue::new(&mut project.config.window_height)
                                            .range(240..=2160)
                                            .suffix(" px")
                                            .fixed_decimals(0));
                                        ui.end_row();

                                        ui.label("Target FPS:");
                                        ui.horizontal(|ui| {
                                            ui.add(egui::DragValue::new(&mut project.config.target_fps)
                                                .range(-1..=240)
                                                .fixed_decimals(0));
                                            if project.config.target_fps == -1 {
                                                ui.label("(unlimited)");
                                            }
                                        });
                                        ui.end_row();

                                        ui.label("Show FPS:");
                                        ui.checkbox(&mut project.config.show_fps, "");
                                        ui.end_row();

                                        ui.label("VSync:");
                                        ui.checkbox(&mut project.config.vsync, "");
                                        ui.end_row();

                                        ui.label("Background:");
                                        let mut color = egui::Color32::from_rgb(
                                            project.config.background_color[0],
                                            project.config.background_color[1],
                                            project.config.background_color[2],
                                        );
                                        if ui.color_edit_button_srgba(&mut color).changed() {
                                            project.config.background_color = [color.r(), color.g(), color.b()];
                                        }
                                        ui.end_row();

                                        ui.label("Splash Duration:");
                                        ui.add(egui::DragValue::new(&mut project.config.splash_duration)
                                            .range(0.0..=10.0)
                                            .speed(0.1)
                                            .suffix(" s")
                                            .fixed_decimals(1));
                                        ui.end_row();
                                    });

                                ui.add_space(10.0);
                                ui.label("Set Target FPS to -1 for unlimited framerate.");
                            });
                        }
                        _ => {}
                    }

                    ui.add_space(15.0);
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Close").clicked() {
                            save = true;
                        }
                    });
                });
            });

        // Auto-save when dialog is closed (either by button or X)
        if save || !open {
            if let Some(ref project) = self.current_project {
                match project.save() {
                    Ok(_) => {
                        log::info!("Project properties saved");
                    }
                    Err(e) => log::error!("Failed to save project: {}", e),
                }
            }
            self.show_project_dialog = false;
        }
    }
}

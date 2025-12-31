//! Prefab creation dialog.

use crate::editor::app::EditorApp;

impl EditorApp {
    /// Draw prefab creation dialog
    pub(crate) fn draw_prefab_dialog(&mut self, ctx: &egui::Context) {
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
                // Isolate dialog widgets from main window
                ui.push_id("prefab_dialog_content", |ui| {
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

                        if ui
                            .add_enabled(
                                !self.prefab_name_input.is_empty(),
                                egui::Button::new("Create"),
                            )
                            .clicked()
                        {
                            create = true;
                        }
                    });
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
}

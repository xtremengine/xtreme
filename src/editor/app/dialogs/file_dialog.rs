//! File open/save dialog.

use crate::editor::app::state::FileDialogAction;
use crate::editor::app::EditorApp;

impl EditorApp {
    /// Draw file open/save dialog
    pub(crate) fn draw_file_dialog(&mut self, _ctx: &egui::Context) {
        if let Some(action) = self.file_dialog_action.take() {
            match action {
                FileDialogAction::Open => {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Xtreme Scene", &["xtrm"])
                        .add_filter("All files", &["*"])
                        .set_title("Open Scene")
                        .pick_file()
                    {
                        self.load_scene(path);
                    }
                }
                FileDialogAction::Save | FileDialogAction::SaveAs => {
                    let default_name = self
                        .scene_manager
                        .current_path()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("scene.xtrm")
                        .to_string();

                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Xtreme Scene", &["xtrm"])
                        .set_title("Save Scene")
                        .set_file_name(&default_name)
                        .save_file()
                    {
                        let path = if path.extension().is_none() {
                            path.with_extension("xtrm")
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
}

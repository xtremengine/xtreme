//! Scripts section panel for attaching and managing object scripts.

use egui::Ui;

#[cfg(feature = "scripting")]
use crate::editor::selection::SceneObject;
#[cfg(feature = "scripting")]
use crate::scripting::ScriptRuntime;

/// Result of scripts section interaction
#[cfg_attr(not(feature = "scripting"), allow(dead_code))]
pub enum ScriptsAction {
    /// No action
    None,
    /// Show new script dialog
    ShowNewScriptDialog,
    /// Show attach script dialog
    ShowAttachScriptDialog,
    /// Remove script from object
    RemoveScript { object_id: u32, script_id: u32 },
}

/// Draw scripts section UI
#[cfg(feature = "scripting")]
pub fn draw_scripts_section(
    ui: &mut Ui,
    selected_object_id: Option<u32>,
    objects: &[SceneObject],
    script_runtime: &ScriptRuntime,
) -> ScriptsAction {
    let Some(obj_id) = selected_object_id else {
        return ScriptsAction::None;
    };

    ui.separator();
    ui.heading("Scripts");

    let script_ids: Vec<u32> = objects
        .iter()
        .find(|o| o.id == obj_id)
        .map(|o| o.scripts.clone())
        .unwrap_or_default();

    let mut action = ScriptsAction::None;

    if script_ids.is_empty() {
        ui.label("No scripts attached");
    } else {
        for script_id in &script_ids {
            ui.horizontal(|ui| {
                if let Some(script) = script_runtime.get_script(*script_id) {
                    ui.label(format!("[{}] {}", script_id, &script.name));
                } else {
                    ui.label(format!("[{}] <unknown>", script_id));
                }

                if ui.small_button("X").clicked() {
                    action = ScriptsAction::RemoveScript {
                        object_id: obj_id,
                        script_id: *script_id,
                    };
                }
            });
        }
    }

    ui.horizontal(|ui| {
        if ui.button("New Script...").clicked() {
            action = ScriptsAction::ShowNewScriptDialog;
        }
        if ui.button("Attach Script...").clicked() {
            action = ScriptsAction::ShowAttachScriptDialog;
        }
    });

    action
}

/// Draw scripts section UI (no scripting feature)
#[cfg(not(feature = "scripting"))]
pub fn draw_scripts_section_disabled(ui: &mut Ui, selected_object_id: Option<u32>) {
    if selected_object_id.is_none() {
        return;
    }

    ui.separator();
    ui.heading("Scripts");
    ui.add_enabled(false, egui::Button::new("New Script..."));
    ui.add_enabled(false, egui::Button::new("Attach Script..."));
    ui.label("(Enable 'scripting' feature)");
}

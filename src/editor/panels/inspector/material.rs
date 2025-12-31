//! Material section UI for the inspector panel.

use crate::editor::selection::SceneObject;
use egui::Ui;

/// Draw the material section of the inspector
pub fn draw_material_section(ui: &mut Ui, obj: &mut SceneObject) -> bool {
    let mut changed = false;

    // Color picker
    ui.horizontal(|ui| {
        ui.label("Color:");
        let mut color = [obj.color[0], obj.color[1], obj.color[2]];
        if ui.color_edit_button_rgb(&mut color).changed() {
            obj.color[0] = color[0];
            obj.color[1] = color[1];
            obj.color[2] = color[2];
            changed = true;
        }
    });

    // Alpha
    ui.horizontal(|ui| {
        ui.label("Alpha:");
        if ui
            .add(egui::Slider::new(&mut obj.color[3], 0.0..=1.0).fixed_decimals(2))
            .changed()
        {
            changed = true;
        }
    });

    // Color presets
    ui.horizontal(|ui| {
        ui.label("Presets:");
        if ui.small_button("Red").clicked() {
            obj.color = [0.9, 0.2, 0.2, 1.0];
            changed = true;
        }
        if ui.small_button("Green").clicked() {
            obj.color = [0.2, 0.9, 0.2, 1.0];
            changed = true;
        }
        if ui.small_button("Blue").clicked() {
            obj.color = [0.2, 0.2, 0.9, 1.0];
            changed = true;
        }
        if ui.small_button("Orange").clicked() {
            obj.color = [0.9, 0.5, 0.1, 1.0];
            changed = true;
        }
    });

    ui.separator();

    // Texture
    ui.horizontal(|ui| {
        ui.label("Texture:");
        if let Some(ref path) = obj.texture_path {
            // Show filename only
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            ui.label(&filename);
            if ui.small_button("X").clicked() {
                obj.texture_path = None;
                changed = true;
            }
        } else {
            ui.label("None");
        }
    });

    // Browse button for texture
    if ui.button("Browse Texture...").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "tga"])
            .set_title("Select Texture")
            .pick_file()
        {
            obj.texture_path = Some(path.to_string_lossy().to_string());
            changed = true;
        }
    }

    ui.separator();

    // Shader
    ui.horizontal(|ui| {
        ui.label("Shader:");
        if let Some(ref path) = obj.shader_path {
            // Show filename only
            let filename = std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            ui.label(&filename);
            if ui.small_button("X").clicked() {
                obj.shader_path = None;
                changed = true;
            }
        } else {
            ui.label("Default");
        }
    });

    // Browse button for shader
    if ui.button("Browse Shader...").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("WGSL Shader", &["wgsl"])
            .add_filter("All Shaders", &["wgsl", "glsl", "hlsl"])
            .set_title("Select Shader")
            .pick_file()
        {
            obj.shader_path = Some(path.to_string_lossy().to_string());
            changed = true;
        }
    }

    changed
}

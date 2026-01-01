//! Audio component inspector sections.

use crate::editor::components::{AudioListenerComponent, AudioSourceComponent};
use egui::Ui;

/// Actions that can be triggered from audio inspector
#[derive(Clone, Debug)]
pub enum AudioAction {
    /// No action
    None,
    /// Play preview of audio clip
    PlayPreview(String),
    /// Stop current preview
    StopPreview,
}

/// Draw the audio source section
pub fn draw_audio_source_section(
    ui: &mut Ui,
    audio: &mut AudioSourceComponent,
) -> (bool, AudioAction) {
    let mut changed = false;
    let mut action = AudioAction::None;

    // Clip path
    ui.horizontal(|ui| {
        ui.label("Clip:");
        let path_str = audio.clip_path.clone().unwrap_or_default();
        let mut path_edit = path_str.clone();
        if ui.text_edit_singleline(&mut path_edit).changed() {
            audio.clip_path = if path_edit.is_empty() {
                None
            } else {
                Some(path_edit)
            };
            changed = true;
        }
        if ui.button("...").clicked() {
            // Open file dialog for audio files
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio Files", &["wav", "mp3", "ogg", "flac"])
                .add_filter("All Files", &["*"])
                .pick_file()
            {
                audio.clip_path = Some(path.display().to_string());
                changed = true;
            }
        }
        if audio.clip_path.is_some() && ui.button("X").clicked() {
            audio.clip_path = None;
            changed = true;
        }
    });

    ui.add_space(4.0);

    // Volume slider
    ui.horizontal(|ui| {
        ui.label("Volume:");
        if ui
            .add(egui::Slider::new(&mut audio.volume, 0.0..=1.0).show_value(true))
            .changed()
        {
            changed = true;
        }
    });

    // Pitch slider
    ui.horizontal(|ui| {
        ui.label("Pitch:");
        if ui
            .add(egui::Slider::new(&mut audio.pitch, 0.5..=2.0).show_value(true))
            .changed()
        {
            changed = true;
        }
    });

    ui.add_space(4.0);

    // Checkboxes
    if ui.checkbox(&mut audio.looping, "Looping").changed() {
        changed = true;
    }

    if ui.checkbox(&mut audio.spatial, "Spatial Audio").changed() {
        changed = true;
    }

    // Spatial settings (only if spatial is enabled)
    if audio.spatial {
        ui.indent("spatial_settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Min Distance:");
                if ui
                    .add(egui::DragValue::new(&mut audio.min_distance).speed(0.1))
                    .changed()
                {
                    audio.min_distance = audio.min_distance.max(0.0);
                    changed = true;
                }
            });

            ui.horizontal(|ui| {
                ui.label("Max Distance:");
                if ui
                    .add(egui::DragValue::new(&mut audio.max_distance).speed(0.1))
                    .changed()
                {
                    audio.max_distance = audio.max_distance.max(audio.min_distance);
                    changed = true;
                }
            });
        });
    }

    if ui.checkbox(&mut audio.autoplay, "Autoplay").changed() {
        changed = true;
    }

    ui.add_space(4.0);

    // Preview controls
    ui.horizontal(|ui| {
        let has_clip = audio.clip_path.is_some();
        if ui
            .add_enabled(has_clip, egui::Button::new("▶ Play"))
            .clicked()
        {
            if let Some(ref path) = audio.clip_path {
                action = AudioAction::PlayPreview(path.clone());
            }
        }
        if ui.button("■ Stop").clicked() {
            action = AudioAction::StopPreview;
        }
    });

    (changed, action)
}

/// Draw the audio listener section
pub fn draw_audio_listener_section(ui: &mut Ui, listener: &mut AudioListenerComponent) -> bool {
    let mut changed = false;

    if ui.checkbox(&mut listener.is_active, "Active").changed() {
        changed = true;
    }

    ui.horizontal(|ui| {
        ui.label("Master Volume:");
        if ui
            .add(egui::Slider::new(&mut listener.master_volume, 0.0..=1.0).show_value(true))
            .changed()
        {
            changed = true;
        }
    });

    changed
}

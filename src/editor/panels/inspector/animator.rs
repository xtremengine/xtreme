//! Animator component inspector section.

use crate::editor::components::AnimatorComponent;
use egui::Ui;

/// Draw the animator section
pub fn draw_animator_section(ui: &mut Ui, animator: &mut AnimatorComponent) -> bool {
    let mut changed = false;

    // Skeleton path
    ui.horizontal(|ui| {
        ui.label("Skeleton:");
        let path_str = animator.skeleton_path.clone().unwrap_or_default();
        let mut path_edit = path_str.clone();
        if ui.text_edit_singleline(&mut path_edit).changed() {
            animator.skeleton_path = if path_edit.is_empty() {
                None
            } else {
                Some(path_edit)
            };
            changed = true;
        }
        if ui.button("...").clicked() {
            // TODO: Open file dialog for GLTF files
        }
        if animator.skeleton_path.is_some() && ui.button("X").clicked() {
            animator.skeleton_path = None;
            changed = true;
        }
    });

    ui.add_space(4.0);

    // Current state (animation name)
    ui.horizontal(|ui| {
        ui.label("State:");
        if ui
            .text_edit_singleline(&mut animator.current_state)
            .changed()
        {
            changed = true;
        }
    });

    // Speed slider
    ui.horizontal(|ui| {
        ui.label("Speed:");
        if ui
            .add(egui::Slider::new(&mut animator.speed, 0.0..=3.0).show_value(true))
            .changed()
        {
            changed = true;
        }
    });

    ui.add_space(4.0);

    // Playing checkbox
    if ui.checkbox(&mut animator.playing, "Playing").changed() {
        changed = true;
    }

    ui.add_space(4.0);

    // Playback controls (non-functional for now)
    ui.horizontal(|ui| {
        if ui.button("▶ Play").clicked() {
            animator.playing = true;
            changed = true;
        }
        if ui.button("⏸ Pause").clicked() {
            animator.playing = false;
            changed = true;
        }
        if ui.button("⏹ Stop").clicked() {
            animator.playing = false;
            // TODO: Reset animation to beginning
            changed = true;
        }
    });

    changed
}

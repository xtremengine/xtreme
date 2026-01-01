//! Animator component inspector section.

use crate::editor::components::{AnimationClip, AnimatorComponent};
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
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("GLTF Animation", &["gltf", "glb"])
                .add_filter("All Files", &["*"])
                .set_title("Select Animation File")
                .pick_file()
            {
                animator.skeleton_path = Some(path.display().to_string());
                changed = true;
            }
        }
        if animator.skeleton_path.is_some() && ui.button("X").clicked() {
            animator.skeleton_path = None;
            changed = true;
        }
    });

    ui.add_space(4.0);

    // Current animation dropdown
    ui.horizontal(|ui| {
        ui.label("Current:");
        let names: Vec<String> = animator.animations.iter().map(|a| a.name.clone()).collect();
        let current_idx = names
            .iter()
            .position(|n| n == &animator.current_animation)
            .unwrap_or(0);

        egui::ComboBox::from_id_salt("current_animation")
            .selected_text(&animator.current_animation)
            .show_ui(ui, |ui| {
                for (i, name) in names.iter().enumerate() {
                    if ui.selectable_label(i == current_idx, name).clicked() {
                        animator.current_animation = name.clone();
                        animator.current_time = 0.0;
                        changed = true;
                    }
                }
            });
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

    // Playback controls
    ui.horizontal(|ui| {
        if ui.checkbox(&mut animator.playing, "Playing").changed() {
            changed = true;
        }

        if ui.button("▶").on_hover_text("Play").clicked() {
            animator.playing = true;
            changed = true;
        }
        if ui.button("⏸").on_hover_text("Pause").clicked() {
            animator.playing = false;
            changed = true;
        }
        if ui.button("⏹").on_hover_text("Stop & Reset").clicked() {
            animator.playing = false;
            animator.current_time = 0.0;
            changed = true;
        }
    });

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // Animations list header
    ui.horizontal(|ui| {
        ui.strong("Animations");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+").on_hover_text("Add animation").clicked() {
                // Create new animation with unique name
                let base_name = "Animation";
                let mut name = base_name.to_string();
                let mut counter = 1;
                while animator.animations.iter().any(|a| a.name == name) {
                    name = format!("{} {}", base_name, counter);
                    counter += 1;
                }
                animator.add_animation(AnimationClip::new(name));
                changed = true;
            }
        });
    });

    ui.add_space(4.0);

    // Animation list
    let mut to_remove: Option<String> = None;
    let mut set_current: Option<String> = None;
    let anim_count = animator.animations.len();
    let current_anim = animator.current_animation.clone();

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for idx in 0..anim_count {
                let is_current = animator.animations[idx].name == current_anim;
                let anim_name = animator.animations[idx].name.clone();

                ui.push_id(idx, |ui| {
                    egui::Frame::new()
                        .fill(if is_current {
                            egui::Color32::from_rgba_unmultiplied(100, 150, 200, 50)
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            // Animation header
                            ui.horizontal(|ui| {
                                // Name field
                                let mut name = animator.animations[idx].name.clone();
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut name)
                                        .desired_width(100.0)
                                        .hint_text("Name"),
                                );
                                if response.changed() {
                                    animator.animations[idx].name = name;
                                    changed = true;
                                }

                                // Select button
                                if !is_current
                                    && ui.button("▶").on_hover_text("Set as current").clicked()
                                {
                                    set_current = Some(anim_name.clone());
                                    changed = true;
                                }

                                // Remove button (can't remove last animation)
                                if anim_count > 1
                                    && ui.button("🗑").on_hover_text("Remove").clicked()
                                {
                                    to_remove = Some(anim_name.clone());
                                }
                            });

                            // Animation settings
                            ui.horizontal(|ui| {
                                ui.label("File:");
                                let mut file_path = animator.animations[idx]
                                    .file_path
                                    .clone()
                                    .unwrap_or_default();
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(&mut file_path)
                                            .desired_width(120.0)
                                            .hint_text("(from skeleton)"),
                                    )
                                    .changed()
                                {
                                    animator.animations[idx].file_path = if file_path.is_empty() {
                                        None
                                    } else {
                                        Some(file_path)
                                    };
                                    changed = true;
                                }

                                if ui.button("...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("GLTF Animation", &["gltf", "glb"])
                                        .add_filter("All Files", &["*"])
                                        .set_title("Select Animation File")
                                        .pick_file()
                                    {
                                        animator.animations[idx].file_path =
                                            Some(path.display().to_string());
                                        changed = true;
                                    }
                                }
                            });

                            ui.horizontal(|ui| {
                                ui.label("Index:");
                                if ui
                                    .add(
                                        egui::DragValue::new(
                                            &mut animator.animations[idx].animation_index,
                                        )
                                        .range(0..=99),
                                    )
                                    .on_hover_text("Animation index in GLTF file")
                                    .changed()
                                {
                                    changed = true;
                                }

                                ui.label("Speed:");
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut animator.animations[idx].speed)
                                            .speed(0.1)
                                            .range(0.0..=10.0),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }

                                if ui
                                    .checkbox(&mut animator.animations[idx].looping, "Loop")
                                    .changed()
                                {
                                    changed = true;
                                }
                            });
                        });

                    ui.add_space(2.0);
                });
            }
        });

    // Handle set current after iteration
    if let Some(name) = set_current {
        animator.set_animation(&name);
        changed = true;
    }

    // Handle removal after iteration
    if let Some(name) = to_remove {
        animator.remove_animation(&name);
        changed = true;
    }

    changed
}

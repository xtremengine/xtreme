//! # Inspector Panel
//!
//! Property inspector for selected objects.
//!
//! This module is split into:
//! - `transform.rs` - Transform section UI
//! - `material.rs` - Material section UI
//! - `camera.rs` - Camera section UI
//! - `audio.rs` - Audio component UI
//! - `particles.rs` - Particle emitter UI
//! - `animator.rs` - Animator component UI

mod animator;
mod audio;
mod camera;
mod material;
mod particles;
mod transform;

pub use audio::AudioAction;

use super::super::components::{
    AnimatorComponent, AudioListenerComponent, AudioSourceComponent, ParticleEmitterComponent,
    ParticlePreset,
};
use super::super::selection::{SceneObject, Selection};
use egui::Ui;

/// Inspector panel for editing object properties
pub struct InspectorPanel {
    /// Whether transform is expanded
    transform_expanded: bool,
    /// Whether material is expanded
    material_expanded: bool,
    /// Whether camera is expanded
    camera_expanded: bool,
    /// Whether audio source is expanded
    audio_source_expanded: bool,
    /// Whether audio listener is expanded
    audio_listener_expanded: bool,
    /// Whether particle emitter is expanded
    particle_emitter_expanded: bool,
    /// Whether animator is expanded
    animator_expanded: bool,
}

impl Default for InspectorPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl InspectorPanel {
    /// Create a new inspector panel
    pub fn new() -> Self {
        Self {
            transform_expanded: true,
            material_expanded: true,
            camera_expanded: true,
            audio_source_expanded: true,
            audio_listener_expanded: true,
            particle_emitter_expanded: true,
            animator_expanded: true,
        }
    }

    /// Draw the inspector panel
    /// Returns (changed, audio_action)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        objects: &mut [SceneObject],
        selection: &Selection,
    ) -> (bool, AudioAction) {
        let mut changed = false;
        let mut audio_action = AudioAction::None;

        ui.heading("Inspector");
        ui.separator();

        if selection.is_empty() {
            ui.label("No object selected");
            return (false, AudioAction::None);
        }

        let selected_ids = selection.all();

        // Multiple selection info
        if selected_ids.len() > 1 {
            ui.horizontal(|ui| {
                ui.strong(format!("{} objects selected", selected_ids.len()));
            });

            egui::CollapsingHeader::new("Selected Objects")
                .default_open(false)
                .show(ui, |ui| {
                    for id in selected_ids {
                        if let Some(obj) = objects.iter().find(|o| o.id == *id) {
                            ui.horizontal(|ui| {
                                ui.label(format!("[{}] {}", id, obj.name));
                            });
                        }
                    }
                });

            ui.separator();
            ui.label("Editing first selected:");
        }

        // Get first selected object for editing
        let Some(obj) = objects.iter_mut().find(|o| Some(o.id) == selection.first()) else {
            ui.label("Selected object not found");
            return (false, AudioAction::None);
        };

        let obj_id = obj.id;
        ui.push_id(obj_id, |ui| {
            // Name field
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut obj.name).changed() {
                    changed = true;
                }
            });

            // Visibility toggle
            ui.horizontal(|ui| {
                if ui.checkbox(&mut obj.visible, "Visible").changed() {
                    changed = true;
                }
            });

            ui.separator();

            // Transform section
            let header =
                egui::CollapsingHeader::new("Transform").default_open(self.transform_expanded);
            header.show(ui, |ui| {
                self.transform_expanded = true;
                if transform::draw_transform_section(ui, obj) {
                    changed = true;
                }
            });

            ui.separator();

            // Material section
            let header =
                egui::CollapsingHeader::new("Material").default_open(self.material_expanded);
            header.show(ui, |ui| {
                self.material_expanded = true;
                if material::draw_material_section(ui, obj) {
                    changed = true;
                }
            });

            // Camera section (only if object has camera component)
            if let Some(ref mut cam) = obj.camera {
                ui.separator();

                let header =
                    egui::CollapsingHeader::new("Camera").default_open(self.camera_expanded);
                header.show(ui, |ui| {
                    self.camera_expanded = true;
                    if camera::draw_camera_section(ui, cam) {
                        changed = true;
                    }
                });
            }

            // Audio Source section
            if let Some(ref mut audio_src) = obj.audio_source {
                ui.separator();

                ui.horizontal(|ui| {
                    let header = egui::CollapsingHeader::new("Audio Source")
                        .default_open(self.audio_source_expanded);
                    header.show(ui, |ui| {
                        self.audio_source_expanded = true;
                        let (audio_changed, action) =
                            audio::draw_audio_source_section(ui, audio_src);
                        if audio_changed {
                            changed = true;
                        }
                        if !matches!(action, AudioAction::None) {
                            audio_action = action;
                        }
                    });
                });
            }

            // Audio Listener section
            if let Some(ref mut listener) = obj.audio_listener {
                ui.separator();

                let header = egui::CollapsingHeader::new("Audio Listener")
                    .default_open(self.audio_listener_expanded);
                header.show(ui, |ui| {
                    self.audio_listener_expanded = true;
                    if audio::draw_audio_listener_section(ui, listener) {
                        changed = true;
                    }
                });
            }

            // Particle Emitter section
            if let Some(ref mut emitter) = obj.particle_emitter {
                ui.separator();

                let header = egui::CollapsingHeader::new("Particle Emitter")
                    .default_open(self.particle_emitter_expanded);
                header.show(ui, |ui| {
                    self.particle_emitter_expanded = true;
                    if particles::draw_particle_section(ui, emitter) {
                        changed = true;
                    }
                });
            }

            // Animator section
            if let Some(ref mut anim) = obj.animator {
                ui.separator();

                let header =
                    egui::CollapsingHeader::new("Animator").default_open(self.animator_expanded);
                header.show(ui, |ui| {
                    self.animator_expanded = true;
                    if animator::draw_animator_section(ui, anim) {
                        changed = true;
                    }
                });
            }

            // Add Component section
            ui.separator();
            self.draw_add_component(ui, obj, &mut changed);
        });

        (changed, audio_action)
    }

    /// Draw the "Add Component" dropdown
    fn draw_add_component(&mut self, ui: &mut Ui, obj: &mut SceneObject, changed: &mut bool) {
        egui::ComboBox::from_id_salt("add_component")
            .selected_text("+ Add Component")
            .show_ui(ui, |ui| {
                // Audio Source
                if obj.audio_source.is_none()
                    && ui.selectable_label(false, "Audio Source").clicked()
                {
                    obj.audio_source = Some(AudioSourceComponent::default());
                    *changed = true;
                }

                // Audio Listener
                if obj.audio_listener.is_none()
                    && ui.selectable_label(false, "Audio Listener").clicked()
                {
                    obj.audio_listener = Some(AudioListenerComponent::default());
                    *changed = true;
                }

                // Particle Emitter submenu
                if obj.particle_emitter.is_none() {
                    ui.menu_button("Particle Emitter", |ui| {
                        for preset in ParticlePreset::all() {
                            if ui.button(preset.name()).clicked() {
                                obj.particle_emitter =
                                    Some(ParticleEmitterComponent::with_preset(*preset));
                                *changed = true;
                                ui.close();
                            }
                        }
                    });
                }

                // Animator
                if obj.animator.is_none() && ui.selectable_label(false, "Animator").clicked() {
                    obj.animator = Some(AnimatorComponent::default());
                    *changed = true;
                }
            });
    }
}

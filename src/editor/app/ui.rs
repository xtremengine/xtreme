//! Editor UI: main drawing coordination and panel layout.

use std::path::Path;

use glam::Vec3;

use super::state::InputModifiers;
use super::templates::{SCENE_TEMPLATE, SCRIPT_TEMPLATE, SHADER_TEMPLATE};
use super::EditorApp;
use crate::editor::gizmos::GizmoAxis;
#[cfg(feature = "scripting")]
use crate::editor::panels::ScriptsAction;
use crate::editor::panels::{
    draw_camera_settings, draw_snap_settings, AssetAction, ConsoleAction, HierarchyAction,
    PrefabAction, TimelineAction, Tool, ToolbarAction,
};
use crate::editor::selection::SceneObject;

/// Simple pseudo-random float [0, 1)
pub fn rand_float() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f32 / 1_000_000_000.0).fract()
}

impl EditorApp {
    /// Draw the editor UI
    pub fn draw_ui(&mut self, viewport_size: &mut (f32, f32)) {
        let Some(egui) = &self.egui else { return };
        let egui_ctx = egui.ctx().clone();

        // Draw UI components (methods from other modules)
        self.draw_menu_bar(&egui_ctx);
        self.draw_toolbar(&egui_ctx);
        self.draw_hierarchy_panel(&egui_ctx);
        self.draw_inspector_panel(&egui_ctx);
        self.draw_status_bar(&egui_ctx, viewport_size);
        self.draw_console_panel(&egui_ctx);
        self.draw_timeline_panel(&egui_ctx);
        self.draw_viewport_panel(&egui_ctx, viewport_size);
        self.draw_cutscene_dialog(&egui_ctx);
        self.draw_file_dialog(&egui_ctx);
        self.draw_prefab_dialog(&egui_ctx);
        self.draw_script_dialog(&egui_ctx);
        self.draw_attach_script_dialog(&egui_ctx);
        self.draw_project_dialog(&egui_ctx);
        self.draw_splash_screen(&egui_ctx);
    }

    fn draw_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Play/Stop buttons
                if self.is_playing {
                    if ui
                        .add(egui::Button::new("⏹ Stop").fill(egui::Color32::from_rgb(180, 60, 60)))
                        .clicked()
                    {
                        self.stop_play();
                    }
                    ui.label(format!("▶ {:.1}s", self.play_time));
                } else if ui
                    .add(egui::Button::new("▶ Play").fill(egui::Color32::from_rgb(60, 180, 60)))
                    .clicked()
                {
                    self.start_play();
                }

                ui.separator();

                let action = self.toolbar_panel.show(ui);
                match action {
                    ToolbarAction::CenterView => {
                        if let Some(id) = self.selection.first() {
                            self.focus_on_object(id);
                        }
                    }
                    ToolbarAction::FrameAll => {
                        self.camera.target = Vec3::ZERO;
                        self.camera.distance = 30.0;
                    }
                    ToolbarAction::None => {}
                }
            });
        });
    }

    fn draw_hierarchy_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .default_width(220.0)
            .show(ctx, |ui| {
                // Hierarchy (top) - resizable
                egui::TopBottomPanel::top("hierarchy_inner")
                    .resizable(true)
                    .default_height(self.hierarchy_height)
                    .min_height(100.0)
                    .show_inside(ui, |ui| {
                        self.draw_hierarchy_content(ui);
                    });

                // File Explorer (bottom) - fills remaining space
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    self.draw_file_explorer_content(ui);
                });
            });
    }

    fn draw_hierarchy_content(&mut self, ui: &mut egui::Ui) {
        let action = self
            .hierarchy_panel
            .show(ui, &mut self.scene_objects, &mut self.selection);
        match action {
            HierarchyAction::CreateCube => {
                let obj = SceneObject::cube(self.next_id, Vec3::new(0.0, 0.5, 0.0));
                self.next_id += 1;
                self.create_object(obj);
            }
            HierarchyAction::CreateEmpty => {
                let mut obj = SceneObject::new(self.next_id, format!("Empty {}", self.next_id));
                self.next_id += 1;
                obj.visible = true;
                self.create_object(obj);
            }
            HierarchyAction::CreateCamera => {
                let obj = SceneObject::camera(self.next_id, Vec3::new(0.0, 5.0, -10.0));
                self.next_id += 1;
                self.create_object(obj);
            }
            HierarchyAction::Delete(id) => {
                self.delete_object(id);
            }
            HierarchyAction::Duplicate(id) => {
                self.duplicate_object(id);
            }
            HierarchyAction::Focus(id) => {
                self.focus_on_object(id);
            }
            HierarchyAction::ToggleVisibility(id) => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                    obj.visible = !obj.visible;
                }
            }
            HierarchyAction::Rename(_) => {}
            HierarchyAction::Reparent(child_id, new_parent_id) => {
                self.reparent_object(child_id, new_parent_id);
            }
            HierarchyAction::None => {}
        }
    }

    fn draw_file_explorer_content(&mut self, ui: &mut egui::Ui) {
        let action = self.asset_browser.show(ui);
        match action {
            AssetAction::OpenScene(path) => {
                self.load_scene(path);
            }
            AssetAction::LoadPrefab(path) => {
                self.load_prefab(path);
            }
            AssetAction::AssignTexture(path) => {
                // Assign texture to selected object(s)
                let path_str = path.to_string_lossy().to_string();
                for &id in self.selection.all() {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                        obj.texture_path = Some(path_str.clone());
                    }
                }
                if !self.selection.is_empty() {
                    log::info!("Assigned texture {} to selected objects", path_str);
                    self.scene_manager.mark_dirty();
                }
            }
            AssetAction::OpenScript(path) => {
                // Open script in system editor
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &path.to_string_lossy()])
                        .spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                }
            }
            AssetAction::ShowInExplorer(path) => {
                // Show file in system explorer
                let folder = if path.is_dir() {
                    path.clone()
                } else {
                    path.parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or(path.clone())
                };
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer").arg(&folder).spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("xdg-open").arg(&folder).spawn();
                }
            }
            AssetAction::Delete(path) => {
                // Delete file
                if let Err(e) = std::fs::remove_file(&path) {
                    log::error!("Failed to delete {}: {}", path.display(), e);
                } else {
                    log::info!("Deleted {}", path.display());
                    self.asset_browser.refresh();
                }
            }
            AssetAction::CreateShader(dir) => {
                self.create_new_asset(&dir, "new_shader.wgsl", SHADER_TEMPLATE);
            }
            AssetAction::CreateScript(dir) => {
                self.create_new_asset(&dir, "new_script.py", SCRIPT_TEMPLATE);
            }
            AssetAction::CreateScene(dir) => {
                self.create_new_asset(&dir, "new_scene.ron", SCENE_TEMPLATE);
            }
            AssetAction::CreateTexture(_) => {
                // Textures should be imported, not created
                log::info!("Import textures using external tools");
            }
            AssetAction::AssignShader(path) => {
                // Assign shader to selected object(s)
                let path_str = path.to_string_lossy().to_string();
                for &id in self.selection.all() {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                        obj.shader_path = Some(path_str.clone());
                    }
                }
                if !self.selection.is_empty() {
                    log::info!("Assigned shader {} to selected objects", path_str);
                    self.scene_manager.mark_dirty();
                }
            }
            AssetAction::OpenDirectory(_) => {}
            AssetAction::None => {}
        }
    }

    /// Create a new asset file with the given template
    fn create_new_asset(&mut self, dir: &Path, default_name: &str, template: &str) {
        // Find a unique filename
        let mut counter = 0;
        let mut file_path = dir.join(default_name);

        while file_path.exists() {
            counter += 1;
            let ext = Path::new(default_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            let stem = Path::new(default_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("new_file");
            let new_name = if ext.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, ext)
            };
            file_path = dir.join(new_name);
        }

        // Write the template to the file
        match std::fs::write(&file_path, template) {
            Ok(()) => {
                log::info!("Created new asset: {}", file_path.display());
                self.asset_browser.refresh();

                // Open the file in system editor for immediate editing
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(["/C", "start", "", &file_path.to_string_lossy()])
                        .spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&file_path)
                        .spawn();
                }
            }
            Err(e) => {
                log::error!("Failed to create asset {}: {}", file_path.display(), e);
            }
        }
    }

    fn draw_inspector_panel(&mut self, ctx: &egui::Context) {
        use crate::editor::panels::AudioAction;

        egui::SidePanel::right("inspector")
            .default_width(280.0)
            .show(ctx, |ui| {
                let (changed, audio_action, prefab_action) = self.inspector_panel.show(
                    ui,
                    &mut self.scene_objects,
                    &self.selection,
                    &self.prefab_instances,
                );

                // Handle audio actions
                match audio_action {
                    AudioAction::PlayPreview(path) => {
                        self.play_audio_preview(&path);
                    }
                    AudioAction::StopPreview => {
                        self.stop_audio_preview();
                    }
                    AudioAction::None => {}
                }

                // Handle prefab actions
                match prefab_action {
                    PrefabAction::RevertProperty { object_id, path } => {
                        if let Some(instance) = self.prefab_instances.get_mut(&object_id) {
                            use crate::editor::prefab::overrides::PropertyPath;
                            instance
                                .overrides
                                .remove_override(&PropertyPath::new(&path));
                            log::info!("Reverted property '{}' on object {}", path, object_id);
                        }
                    }
                    PrefabAction::RevertAll { object_id } => {
                        if let Some(instance) = self.prefab_instances.get_mut(&object_id) {
                            instance.overrides.clear();
                            log::info!("Reverted all overrides on object {}", object_id);
                        }
                    }
                    PrefabAction::ApplyToPrefab { object_id } => {
                        log::info!(
                            "Apply to prefab requested for object {} (not yet implemented)",
                            object_id
                        );
                    }
                    PrefabAction::SelectPrefab { path } => {
                        log::info!("Select prefab in browser: {}", path);
                    }
                    PrefabAction::OpenPrefab { path } => {
                        log::info!("Open prefab: {}", path);
                        // Open prefab file in system editor
                        #[cfg(target_os = "windows")]
                        {
                            let _ = std::process::Command::new("cmd")
                                .args(["/C", "start", "", &path])
                                .spawn();
                        }
                        #[cfg(not(target_os = "windows"))]
                        {
                            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                        }
                    }
                    PrefabAction::None => {}
                }

                // Mark particles dirty if inspector changed anything related to particles
                if changed {
                    // Check if the selected object has a particle emitter
                    if let Some(id) = self.selection.first() {
                        if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
                            if obj.particle_emitter.is_some() {
                                self.particles_dirty = true;
                            }
                        }
                    }
                    self.scene_manager.mark_dirty();
                }

                ui.separator();

                // Camera controls
                draw_camera_settings(ui, &mut self.camera);

                ui.separator();

                // Snap settings
                draw_snap_settings(ui, &mut self.snap_settings);

                // Scripts section
                self.handle_scripts_section(ui);
            });
    }

    fn handle_scripts_section(&mut self, ui: &mut egui::Ui) {
        #[cfg(feature = "scripting")]
        {
            use crate::editor::panels::draw_scripts_section;
            let action = draw_scripts_section(
                ui,
                self.selection.first(),
                &self.scene_objects,
                &self.script_runtime,
            );
            match action {
                ScriptsAction::ShowNewScriptDialog => {
                    self.show_script_dialog = true;
                    self.script_path_input.clear();
                }
                ScriptsAction::ShowAttachScriptDialog => {
                    self.show_attach_script_dialog = true;
                }
                ScriptsAction::RemoveScript {
                    object_id,
                    script_id,
                } => {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == object_id) {
                        obj.scripts.retain(|&id| id != script_id);
                    }
                    self.script_runtime.detach_script(script_id);
                }
                ScriptsAction::None => {}
            }
        }
        #[cfg(not(feature = "scripting"))]
        {
            use crate::editor::panels::scripts::draw_scripts_section_disabled;
            draw_scripts_section_disabled(ui, self.selection.first());
        }
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context, viewport_size: &(f32, f32)) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let dirty = if self.scene_manager.is_dirty() {
                    "*"
                } else {
                    ""
                };
                ui.label(format!("{}{}", dirty, self.scene_manager.scene_name()));
                ui.separator();

                ui.label(format!("Objects: {}", self.scene_objects.len()));
                ui.separator();
                ui.label(format!("Selected: {}", self.selection.count()));
                ui.separator();
                ui.label(format!("Tool: {}", self.toolbar_panel.current_tool.name()));
                ui.separator();

                let undo_count = self.command_history.undo_count();
                let redo_count = self.command_history.redo_count();
                if undo_count > 0 || redo_count > 0 {
                    ui.label(format!("History: {} undo, {} redo", undo_count, redo_count));
                    ui.separator();
                }

                ui.label(format!(
                    "Viewport: {:.0}x{:.0}",
                    viewport_size.0, viewport_size.1
                ));
            });
        });
    }

    fn draw_console_panel(&mut self, ctx: &egui::Context) {
        if !self.console_panel.visible {
            return;
        }

        egui::TopBottomPanel::bottom("console")
            .resizable(true)
            .default_height(150.0)
            .min_height(80.0)
            .max_height(400.0)
            .show(ctx, |ui| {
                let action = self.console_panel.show(ui);
                match action {
                    ConsoleAction::CopyToClipboard(text) => {
                        ctx.copy_text(text);
                    }
                    ConsoleAction::None => {}
                }
            });
    }

    fn draw_viewport_panel(&mut self, ctx: &egui::Context, viewport_size: &mut (f32, f32)) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            *viewport_size = (available.x, available.y);

            if let Some(viewport) = &self.viewport {
                let texture_id = viewport.texture_id();
                let size = egui::vec2(available.x, available.y);

                let response = ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(texture_id, size))
                        .sense(egui::Sense::click_and_drag()),
                );

                self.viewport_rect = response.rect;

                ui.input(|i| {
                    self.input_modifiers = InputModifiers {
                        ctrl: i.modifiers.ctrl,
                        shift: i.modifiers.shift,
                        alt: i.modifiers.alt,
                    };
                });

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_viewport_click(pos);
                    }
                }

                if response.drag_started() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_viewport_click(pos);
                    }
                }

                if response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        self.handle_viewport_drag(pos);
                    }
                }

                if response.drag_stopped() {
                    self.handle_viewport_release();
                }

                self.handle_camera_input(&response, ui);

                if response.hovered() && !self.gizmo.is_dragging() {
                    self.mouse_in_viewport = ui.input(|i| i.pointer.hover_pos());
                    if let Some(pos) = self.mouse_in_viewport {
                        if let Some(ray) = self.create_ray(pos) {
                            if self.toolbar_panel.current_tool != Tool::Select {
                                if let Some(id) = self.selection.first() {
                                    if let Some(obj) =
                                        self.scene_objects.iter().find(|o| o.id == id)
                                    {
                                        let gizmo_scale = self.camera.distance * 0.08;
                                        // Calculate world position separately to avoid borrow issues
                                        let world_pos =
                                            if let Some(parent_id) = obj.hierarchy.parent {
                                                if let Some(parent) = self
                                                    .scene_objects
                                                    .iter()
                                                    .find(|p| p.id == parent_id)
                                                {
                                                    let parent_matrix = parent.local_matrix();
                                                    let local_pos = obj.position;
                                                    (parent_matrix
                                                        * glam::Vec4::new(
                                                            local_pos.x,
                                                            local_pos.y,
                                                            local_pos.z,
                                                            1.0,
                                                        ))
                                                    .truncate()
                                                } else {
                                                    obj.position
                                                }
                                            } else {
                                                obj.position
                                            };
                                        let axis =
                                            self.gizmo.hit_test(&ray, world_pos, gizmo_scale);
                                        self.gizmo.set_hovered(axis);
                                    }
                                }
                            }
                        }
                    }
                } else if !response.hovered() {
                    self.mouse_in_viewport = None;
                    self.gizmo.set_hovered(GizmoAxis::None);
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Viewport not initialized");
                });
            }
        });
    }

    fn draw_timeline_panel(&mut self, ctx: &egui::Context) {
        if !self.show_timeline {
            return;
        }

        // Calculate delta time for timeline update
        let delta = self
            .last_frame_instant
            .map(|i| i.elapsed().as_secs_f32())
            .unwrap_or(0.016);

        // Update timeline
        self.timeline_panel.update(delta);

        // Apply timeline values to scene objects during playback
        if self.timeline_panel.player().is_playing() {
            if let Some(sequence) = self.timeline_panel.sequence().cloned() {
                let sample = self.timeline_panel.player_mut().sample(&sequence);

                // Apply object transforms from timeline
                for (entity_id, transform) in &sample.object_transforms {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *entity_id) {
                        if let Some(pos) = transform.position {
                            obj.position = pos;
                        }
                        if let Some(rot) = transform.rotation {
                            obj.rotation = rot;
                        }
                        if let Some(scale) = transform.scale {
                            obj.scale = scale;
                        }
                        if let Some(visible) = transform.visible {
                            obj.visible = visible;
                        }
                        if let Some(color) = transform.color {
                            obj.color = color;
                        }
                    }
                }

                // Apply camera values
                if let Some(camera) = &sample.camera {
                    if let Some(pos) = camera.position {
                        self.camera.target = pos;
                    }
                    if let Some(target) = camera.target {
                        // Adjust camera to look at target
                        let dir = (target - self.camera.target).normalize_or_zero();
                        if dir.length() > 0.01 {
                            self.camera.yaw = dir.x.atan2(dir.z);
                            self.camera.pitch = (-dir.y).asin();
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::bottom("timeline")
            .resizable(true)
            .default_height(200.0)
            .min_height(100.0)
            .max_height(400.0)
            .show(ctx, |ui| {
                let action = self.timeline_panel.show(ui, &self.scene_objects);
                match action {
                    TimelineAction::TogglePlayback => {
                        log::info!("Timeline playback toggled");
                    }
                    TimelineAction::Stop => {
                        log::info!("Timeline stopped");
                    }
                    TimelineAction::Seek(time) => {
                        log::debug!("Timeline seek to {:.2}s", time);
                    }
                    TimelineAction::AddKeyframe { track_id, time } => {
                        use crate::timeline::{Keyframe, KeyframeValue, TrackType};

                        // Get track info first to avoid borrow conflict
                        let track_info = self.timeline_panel.sequence().and_then(|seq| {
                            seq.get_track(track_id)
                                .map(|t| (t.track_type, t.target_entity))
                        });

                        if let Some((track_type, target_entity)) = track_info {
                            // Get value from target object
                            let value = match track_type {
                                TrackType::ObjectPosition => target_entity
                                    .and_then(|id| self.scene_objects.iter().find(|o| o.id == id))
                                    .map(|o| KeyframeValue::Vec3(o.position))
                                    .unwrap_or(KeyframeValue::Vec3(Vec3::ZERO)),
                                TrackType::ObjectRotation => target_entity
                                    .and_then(|id| self.scene_objects.iter().find(|o| o.id == id))
                                    .map(|o| KeyframeValue::Vec3(o.rotation))
                                    .unwrap_or(KeyframeValue::Vec3(Vec3::ZERO)),
                                TrackType::ObjectScale => target_entity
                                    .and_then(|id| self.scene_objects.iter().find(|o| o.id == id))
                                    .map(|o| KeyframeValue::Vec3(o.scale))
                                    .unwrap_or(KeyframeValue::Vec3(Vec3::ONE)),
                                TrackType::ObjectColor => target_entity
                                    .and_then(|id| self.scene_objects.iter().find(|o| o.id == id))
                                    .map(|o| KeyframeValue::Color(o.color))
                                    .unwrap_or(KeyframeValue::Color([1.0, 1.0, 1.0, 1.0])),
                                TrackType::ObjectVisibility => target_entity
                                    .and_then(|id| self.scene_objects.iter().find(|o| o.id == id))
                                    .map(|o| KeyframeValue::Bool(o.visible))
                                    .unwrap_or(KeyframeValue::Bool(true)),
                                _ => KeyframeValue::Vec3(Vec3::ZERO),
                            };

                            // Now add the keyframe
                            if let Some(seq) = self.timeline_panel.sequence_mut() {
                                if let Some(track) = seq.get_track_mut(track_id) {
                                    let kf = Keyframe::new(time, value);
                                    track.add_keyframe(kf);
                                    log::info!(
                                        "Added keyframe at {:.2}s on track {}",
                                        time,
                                        track_id
                                    );
                                }
                            }
                        }
                    }
                    TimelineAction::DeleteKeyframe {
                        track_id,
                        keyframe_index,
                    } => {
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            if let Some(track) = seq.get_track_mut(track_id) {
                                if keyframe_index < track.keyframes.len() {
                                    track.keyframes.remove(keyframe_index);
                                    log::info!(
                                        "Deleted keyframe {} from track {}",
                                        keyframe_index,
                                        track_id
                                    );
                                }
                            }
                        }
                    }
                    TimelineAction::SelectTrack(track_id) => {
                        log::debug!("Selected track {}", track_id);
                    }
                    TimelineAction::AddTrack(track_type) => {
                        use crate::timeline::{Track, TrackType};
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            let id = seq.next_track_id();
                            let name = match track_type {
                                TrackType::ObjectPosition => "Position",
                                TrackType::ObjectRotation => "Rotation",
                                TrackType::ObjectScale => "Scale",
                                TrackType::Event => "Events",
                                _ => "Track",
                            };
                            let track = Track::new(id, name, track_type);
                            seq.add_track(track);
                            log::info!("Added {} track", name);
                        }
                    }
                    TimelineAction::DeleteTrack(track_id) => {
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            seq.remove_track(track_id);
                            log::info!("Deleted track {}", track_id);
                        }
                    }
                    TimelineAction::AddTrackWithTarget {
                        track_type,
                        target_entity,
                        name,
                    } => {
                        use crate::timeline::Track;
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            let id = seq.next_track_id();
                            let mut track = Track::new(id, &name, track_type);
                            track.target_entity = target_entity;
                            seq.add_track(track);
                            log::info!(
                                "Added track '{}' ({:?}) targeting {:?}",
                                name,
                                track_type,
                                target_entity
                            );
                        }
                    }
                    TimelineAction::SetTrackTarget {
                        track_id,
                        entity_id,
                    } => {
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            if let Some(track) = seq.get_track_mut(track_id) {
                                track.target_entity = entity_id;
                                log::info!("Set track {} target to {:?}", track_id, entity_id);
                            }
                        }
                    }
                    TimelineAction::UpdateKeyframe {
                        track_id,
                        keyframe_index,
                        time,
                        easing,
                    } => {
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            if let Some(track) = seq.get_track_mut(track_id) {
                                if let Some(kf) = track.keyframes.get_mut(keyframe_index) {
                                    if let Some(t) = time {
                                        kf.time = t;
                                    }
                                    if let Some(e) = easing {
                                        kf.easing = e;
                                    }
                                    log::debug!(
                                        "Updated keyframe {} on track {}",
                                        keyframe_index,
                                        track_id
                                    );
                                }
                            }
                        }
                    }
                    TimelineAction::CaptureKeyframe {
                        track_id,
                        time,
                        value,
                    } => {
                        use crate::timeline::Keyframe;
                        if let Some(seq) = self.timeline_panel.sequence_mut() {
                            if let Some(track) = seq.get_track_mut(track_id) {
                                let kf = Keyframe::new(time, value);
                                track.add_keyframe(kf);
                                log::info!("Added keyframe at {:.2}s on track {}", time, track_id);
                            }
                        }
                    }
                    TimelineAction::None => {}
                }
            });
    }

    fn draw_cutscene_dialog(&mut self, ctx: &egui::Context) {
        use crate::timeline::{CutsceneBuilder, Easing};

        if !self.show_cutscene_dialog {
            return;
        }

        let mut close = false;
        let mut create = false;

        egui::Window::new("Create Cutscene")
            .collapsible(false)
            .resizable(false)
            .default_width(350.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.cutscene_name_input);
                });

                ui.horizontal(|ui| {
                    ui.label("Duration:");
                    ui.add(
                        egui::DragValue::new(&mut self.cutscene_duration_input)
                            .range(1.0..=300.0)
                            .speed(0.5)
                            .suffix("s"),
                    );
                });

                ui.separator();

                ui.label("Quick Setup:");
                ui.horizontal(|ui| {
                    if ui.button("Camera Pan").clicked() {
                        let name = if self.cutscene_name_input.is_empty() {
                            "Camera Pan"
                        } else {
                            &self.cutscene_name_input
                        };
                        let cutscene = CutsceneBuilder::new(name)
                            .duration(self.cutscene_duration_input)
                            .camera_move(
                                0.0,
                                self.cutscene_duration_input,
                                glam::Vec3::new(0.0, 10.0, -20.0),
                                glam::Vec3::new(0.0, 5.0, 0.0),
                                Easing::EaseInOut,
                            )
                            .build();
                        self.timeline_panel.set_sequence(cutscene);
                        self.show_timeline = true;
                        create = true;
                    }
                    if ui.button("Dialogue Scene").clicked() {
                        let name = if self.cutscene_name_input.is_empty() {
                            "Dialogue"
                        } else {
                            &self.cutscene_name_input
                        };
                        let cutscene = CutsceneBuilder::new(name)
                            .duration(self.cutscene_duration_input)
                            .dialogue(0.5, "Character", "Hello, world!", 2.0)
                            .dialogue(3.0, "Character", "This is a test dialogue.", 2.5)
                            .event(0.0, "start_cutscene")
                            .event(self.cutscene_duration_input, "end_cutscene")
                            .build();
                        self.timeline_panel.set_sequence(cutscene);
                        self.show_timeline = true;
                        create = true;
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Create Empty").clicked() {
                        let name = if self.cutscene_name_input.is_empty() {
                            "New Cutscene"
                        } else {
                            &self.cutscene_name_input
                        };
                        let cutscene = CutsceneBuilder::new(name)
                            .duration(self.cutscene_duration_input)
                            .build();
                        self.timeline_panel.set_sequence(cutscene);
                        self.show_timeline = true;
                        create = true;
                    }
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });

        if create {
            log::info!(
                "Created cutscene '{}' ({:.1}s)",
                self.cutscene_name_input,
                self.cutscene_duration_input
            );
            self.show_cutscene_dialog = false;
            self.cutscene_name_input.clear();
        }

        if close {
            self.show_cutscene_dialog = false;
        }
    }
}

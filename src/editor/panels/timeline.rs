//! # Timeline Panel
//!
//! UI panel for editing timeline sequences.

use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::editor::components::CameraProjection;
use crate::editor::selection::SceneObject;
use crate::timeline::{Easing, KeyframeValue, TimelinePlayer, TimelineSequence, Track, TrackType};

/// Actions returned by the timeline panel
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineAction {
    /// No action
    None,
    /// Play/pause toggle
    TogglePlayback,
    /// Stop and reset
    Stop,
    /// Seek to time
    Seek(f32),
    /// Add keyframe at time
    AddKeyframe { track_id: u32, time: f32 },
    /// Delete keyframe
    DeleteKeyframe {
        track_id: u32,
        keyframe_index: usize,
    },
    /// Select track
    SelectTrack(u32),
    /// Add new track
    AddTrack(TrackType),
    /// Delete track
    DeleteTrack(u32),
    /// Add track with specific target entity
    AddTrackWithTarget {
        track_type: TrackType,
        target_entity: Option<u32>,
        name: String,
    },
    /// Set track target entity
    SetTrackTarget {
        track_id: u32,
        entity_id: Option<u32>,
    },
    /// Update keyframe properties
    UpdateKeyframe {
        track_id: u32,
        keyframe_index: usize,
        time: Option<f32>,
        easing: Option<Easing>,
    },
    /// Capture current object value as keyframe
    CaptureKeyframe {
        track_id: u32,
        time: f32,
        value: KeyframeValue,
    },
}

/// Timeline panel settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineSettings {
    /// Pixels per second for timeline zoom
    pub pixels_per_second: f32,
    /// Snap to frames
    pub snap_to_frame: bool,
    /// Show track colors
    pub show_track_colors: bool,
    /// Track height in pixels
    pub track_height: f32,
}

impl Default for TimelineSettings {
    fn default() -> Self {
        Self {
            pixels_per_second: 100.0,
            snap_to_frame: true,
            show_track_colors: true,
            track_height: 24.0,
        }
    }
}

/// Timeline panel for animation editing
pub struct TimelinePanel {
    /// Current sequence being edited
    sequence: Option<TimelineSequence>,
    /// Timeline player
    player: TimelinePlayer,
    /// Selected track ID
    selected_track: Option<u32>,
    /// Selected keyframe (track_id, keyframe_index)
    selected_keyframe: Option<(u32, usize)>,
    /// Panel settings
    settings: TimelineSettings,
    /// Horizontal scroll offset
    scroll_offset: f32,
    /// Whether panel is visible
    pub visible: bool,
    /// Track list width
    track_list_width: f32,
    /// Whether dragging playhead
    #[allow(dead_code)]
    dragging_playhead: bool,
    // Add track dialog state
    /// Whether add track dialog is shown
    show_add_track_dialog: bool,
    /// New track type selection
    new_track_type: TrackType,
    /// New track target entity
    new_track_target: Option<u32>,
    /// New track name
    new_track_name: String,
    // Keyframe editing state
    /// Keyframe time input string
    keyframe_time_input: String,
    // Context menu state
    /// Keyframe that was right-clicked (for context menu)
    context_menu_keyframe: Option<(u32, usize)>,
    /// Position where context menu was opened
    context_menu_pos: egui::Pos2,
}

impl Default for TimelinePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelinePanel {
    /// Create a new timeline panel
    pub fn new() -> Self {
        Self {
            sequence: None,
            player: TimelinePlayer::new(),
            selected_track: None,
            selected_keyframe: None,
            settings: TimelineSettings::default(),
            scroll_offset: 0.0,
            visible: false,
            track_list_width: 180.0,
            dragging_playhead: false,
            show_add_track_dialog: false,
            new_track_type: TrackType::ObjectPosition,
            new_track_target: None,
            new_track_name: String::new(),
            keyframe_time_input: String::new(),
            context_menu_keyframe: None,
            context_menu_pos: egui::Pos2::ZERO,
        }
    }

    /// Set the sequence to edit
    pub fn set_sequence(&mut self, sequence: TimelineSequence) {
        self.sequence = Some(sequence);
        self.player.stop();
        self.selected_track = None;
        self.selected_keyframe = None;
    }

    /// Get the current sequence
    pub fn sequence(&self) -> Option<&TimelineSequence> {
        self.sequence.as_ref()
    }

    /// Get mutable reference to sequence
    pub fn sequence_mut(&mut self) -> Option<&mut TimelineSequence> {
        self.sequence.as_mut()
    }

    /// Take the sequence out
    pub fn take_sequence(&mut self) -> Option<TimelineSequence> {
        self.sequence.take()
    }

    /// Clear the current sequence
    pub fn clear_sequence(&mut self) {
        self.sequence = None;
        self.player.stop();
        self.selected_track = None;
        self.selected_keyframe = None;
    }

    /// Get player reference
    pub fn player(&self) -> &TimelinePlayer {
        &self.player
    }

    /// Get mutable player reference
    pub fn player_mut(&mut self) -> &mut TimelinePlayer {
        &mut self.player
    }

    /// Update the timeline (call each frame)
    pub fn update(&mut self, delta_time: f32) {
        if let Some(ref sequence) = self.sequence {
            self.player.update(delta_time, sequence);
        }
    }

    /// Draw the timeline panel
    pub fn show(&mut self, ui: &mut Ui, scene_objects: &[SceneObject]) -> TimelineAction {
        let mut action = TimelineAction::None;

        // Check if we have a sequence
        if self.sequence.is_none() {
            ui.centered_and_justified(|ui| {
                ui.label("No sequence loaded");
            });
            return action;
        }

        // Extract values needed for the toolbar
        let seq_name = self.sequence.as_ref().unwrap().name.clone();
        let frame_rate = self.sequence.as_ref().unwrap().frame_rate;
        let current_time = self.player.current_time();
        let frame = self.sequence.as_ref().unwrap().time_to_frame(current_time);
        let duration = self.sequence.as_ref().unwrap().duration;
        let has_selected_track = self.selected_track.is_some();

        // Toolbar
        ui.horizontal(|ui| {
            ui.strong(format!("Timeline: {}", seq_name));
            ui.separator();

            // Playback controls - inline to avoid borrow issues
            if ui.small_button("|<").on_hover_text("Go to start").clicked() {
                self.player.go_to_start();
            }
            let play_text = if self.player.is_playing() { "||" } else { ">" };
            if ui
                .small_button(play_text)
                .on_hover_text("Play/Pause")
                .clicked()
            {
                self.player.toggle();
                action = TimelineAction::TogglePlayback;
            }
            if ui.small_button("[]").on_hover_text("Stop").clicked() {
                self.player.stop();
                action = TimelineAction::Stop;
            }
            if ui.small_button(">|").on_hover_text("Go to end").clicked() {
                self.player.set_time(duration);
            }

            ui.separator();

            // Add Keyframe button
            if ui
                .add_enabled(has_selected_track, egui::Button::new("+ Key").small())
                .on_hover_text("Add keyframe at current time (K)")
                .clicked()
            {
                if let Some(track_id) = self.selected_track {
                    // Get default value based on track type and target
                    let value = self.get_keyframe_value_for_track(track_id, scene_objects);
                    action = TimelineAction::CaptureKeyframe {
                        track_id,
                        time: current_time,
                        value,
                    };
                }
            }

            ui.separator();

            // Time display
            ui.label(format!(
                "{:02}:{:05.2} (Frame {})",
                (current_time / 60.0) as i32,
                current_time % 60.0,
                frame
            ));

            ui.separator();
            ui.label(format!("{} FPS", frame_rate as i32));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Zoom controls
                if ui.small_button("+").clicked() {
                    self.settings.pixels_per_second *= 1.2;
                }
                if ui.small_button("-").clicked() {
                    self.settings.pixels_per_second /= 1.2;
                }
                ui.label("Zoom:");

                ui.checkbox(&mut self.settings.snap_to_frame, "Snap");
            });
        });

        ui.separator();

        // Reserve space for keyframe editor if a keyframe is selected
        let keyframe_editor_height = if self.selected_keyframe.is_some() {
            90.0
        } else {
            0.0
        };

        // Main timeline area (reduced height to leave room for keyframe editor)
        let available = ui.available_size();
        let timeline_height = (available.y - keyframe_editor_height).max(80.0);
        let timeline_rect =
            Rect::from_min_size(ui.cursor().min, Vec2::new(available.x, timeline_height));

        // Split into track list and timeline
        let track_list_rect = Rect::from_min_size(
            timeline_rect.min,
            Vec2::new(self.track_list_width, timeline_rect.height()),
        );

        let timeline_area_rect = Rect::from_min_max(
            Pos2::new(track_list_rect.max.x, timeline_rect.min.y),
            timeline_rect.max,
        );

        // Draw track list
        let track_action = self.draw_track_list(ui, track_list_rect, scene_objects);
        if track_action != TimelineAction::None {
            action = track_action;
        }

        // Draw timeline area with keyframe click detection
        let timeline_action = self.draw_timeline_area(ui, timeline_area_rect);
        if timeline_action != TimelineAction::None {
            action = timeline_action;
        }

        // Advance UI cursor past the timeline area so keyframe editor has space
        ui.allocate_space(Vec2::new(available.x, timeline_height));

        // Draw keyframe editor at bottom if a keyframe is selected
        if self.selected_keyframe.is_some() {
            ui.separator();
            let kf_action = self.draw_keyframe_editor(ui);
            if kf_action != TimelineAction::None {
                action = kf_action;
            }
        }

        // Draw add track dialog
        if self.show_add_track_dialog {
            let dialog_action = self.draw_add_track_dialog(ui.ctx(), scene_objects);
            if dialog_action != TimelineAction::None {
                action = dialog_action;
            }
        }

        action
    }

    /// Draw track list
    fn draw_track_list(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        scene_objects: &[SceneObject],
    ) -> TimelineAction {
        let mut action = TimelineAction::None;

        // Collect track info to avoid borrow issues
        let tracks_info: Vec<(u32, String, bool, Option<u32>)> = self
            .sequence
            .as_ref()
            .map(|s| {
                s.tracks
                    .iter()
                    .map(|t| (t.id, t.name.clone(), t.muted, t.target_entity))
                    .collect()
            })
            .unwrap_or_default();

        let selected_track = self.selected_track;

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.set_clip_rect(rect);

            // Header
            ui.horizontal(|ui| {
                ui.strong("Tracks");
                if ui.small_button("+").on_hover_text("Add track").clicked() {
                    self.show_add_track_dialog = true;
                    self.new_track_name = "New Track".to_string();
                    self.new_track_type = TrackType::ObjectPosition;
                    self.new_track_target = None;
                }
            });

            ui.separator();

            // Track entries
            for (track_id, name, muted, target_entity) in tracks_info {
                let selected = selected_track == Some(track_id);

                // Get target name for display
                let target_name = target_entity
                    .and_then(|id| scene_objects.iter().find(|o| o.id == id))
                    .map(|o| o.name.as_str())
                    .unwrap_or("None");

                let display_name = if target_entity.is_some() {
                    format!("{} {}: {}", if muted { "M" } else { "" }, name, target_name)
                } else {
                    format!("{} {}", if muted { "M" } else { "" }, name)
                };

                let track_response = ui.selectable_label(selected, display_name);

                if track_response.clicked() {
                    self.selected_track = Some(track_id);
                    self.selected_keyframe = None;
                    action = TimelineAction::SelectTrack(track_id);
                }

                track_response.context_menu(|ui| {
                    // Set Target submenu
                    ui.menu_button("Set Target", |ui| {
                        if ui.button("None").clicked() {
                            action = TimelineAction::SetTrackTarget {
                                track_id,
                                entity_id: None,
                            };
                            ui.close();
                        }
                        ui.separator();
                        for obj in scene_objects {
                            if ui.button(&obj.name).clicked() {
                                action = TimelineAction::SetTrackTarget {
                                    track_id,
                                    entity_id: Some(obj.id),
                                };
                                ui.close();
                            }
                        }
                    });

                    ui.separator();

                    if ui.button("Delete").clicked() {
                        action = TimelineAction::DeleteTrack(track_id);
                        ui.close();
                    }
                });
            }
        });

        action
    }

    /// Draw timeline area with tracks and keyframes
    fn draw_timeline_area(&mut self, ui: &mut Ui, rect: Rect) -> TimelineAction {
        let mut action = TimelineAction::None;

        // Extract values we need before any mutable operations
        let (duration, frame_rate) = match self.sequence.as_ref() {
            Some(seq) => (seq.duration, seq.frame_rate),
            None => return action,
        };

        let painter = ui.painter_at(rect);

        // Background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(30, 30, 35));

        // Time ruler at top
        let ruler_height = 20.0;
        let ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), ruler_height));
        self.draw_time_ruler(&painter, ruler_rect);

        // Tracks area
        let tracks_rect =
            Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + ruler_height), rect.max);

        // Draw track lanes - need to borrow sequence here
        if let Some(ref sequence) = self.sequence {
            let mut y = tracks_rect.min.y;
            for track in &sequence.tracks {
                let track_rect = Rect::from_min_size(
                    Pos2::new(tracks_rect.min.x, y),
                    Vec2::new(tracks_rect.width(), self.settings.track_height),
                );

                self.draw_track_lane(&painter, track_rect, track);
                y += self.settings.track_height;
            }
        }

        // Playhead
        let playhead_x = rect.min.x
            + (self.player.current_time() - self.scroll_offset) * self.settings.pixels_per_second;

        if playhead_x >= rect.min.x && playhead_x <= rect.max.x {
            painter.line_segment(
                [
                    Pos2::new(playhead_x, rect.min.y),
                    Pos2::new(playhead_x, rect.max.y),
                ],
                Stroke::new(2.0, Color32::from_rgb(255, 100, 100)),
            );

            // Playhead head
            let head_points = vec![
                Pos2::new(playhead_x - 6.0, rect.min.y),
                Pos2::new(playhead_x + 6.0, rect.min.y),
                Pos2::new(playhead_x, rect.min.y + 10.0),
            ];
            painter.add(egui::Shape::convex_polygon(
                head_points,
                Color32::from_rgb(255, 100, 100),
                Stroke::NONE,
            ));
        }

        // Handle clicks on timeline
        let response = ui.allocate_rect(rect, Sense::click_and_drag());

        // Check for keyframe click first (single click only)
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some((track_id, kf_idx)) = self.detect_keyframe_click(pos, rect) {
                    self.selected_keyframe = Some((track_id, kf_idx));
                    self.selected_track = Some(track_id);
                    self.keyframe_time_input.clear();
                    return action; // Don't seek when clicking keyframe
                }
            }
        }

        // Check for right-click on keyframe (context menu)
        if response.secondary_clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some((track_id, kf_idx)) = self.detect_keyframe_click(pos, rect) {
                    self.context_menu_keyframe = Some((track_id, kf_idx));
                    self.context_menu_pos = pos; // Save position where menu was opened
                    self.selected_keyframe = Some((track_id, kf_idx));
                    self.selected_track = Some(track_id);
                }
            }
        }

        // Show keyframe context menu at fixed position
        if let Some((track_id, kf_idx)) = self.context_menu_keyframe {
            let popup_id = ui.make_persistent_id("keyframe_context_menu");
            let menu_pos = self.context_menu_pos;

            let area_response = egui::Area::new(popup_id)
                .order(egui::Order::Foreground)
                .fixed_pos(menu_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        if ui.button("🗑 Delete Keyframe").clicked() {
                            action = TimelineAction::DeleteKeyframe {
                                track_id,
                                keyframe_index: kf_idx,
                            };
                            self.context_menu_keyframe = None;
                            self.selected_keyframe = None;
                        }
                        if ui.button("✏ Edit Time...").clicked() {
                            // Select keyframe for editing in the keyframe editor panel
                            self.selected_keyframe = Some((track_id, kf_idx));
                            self.keyframe_time_input.clear();
                            self.context_menu_keyframe = None;
                        }
                    });
                });

            // Close menu when clicking outside
            if ui.input(|i| i.pointer.any_pressed()) {
                let menu_rect = area_response.response.rect;
                if let Some(click_pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if !menu_rect.contains(click_pos) {
                        self.context_menu_keyframe = None;
                    }
                }
            }
        }

        // Helper closure for snapping
        let snap_to_frame = |time: f32| -> f32 {
            let frame = (time * frame_rate) as i32;
            frame as f32 / frame_rate
        };

        // Handle timeline seek (click or drag)
        if response.clicked() || response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let time =
                    self.scroll_offset + (pos.x - rect.min.x) / self.settings.pixels_per_second;
                let time = time.max(0.0).min(duration);

                let time = if self.settings.snap_to_frame {
                    snap_to_frame(time)
                } else {
                    time
                };

                self.player.set_time(time);
                action = TimelineAction::Seek(time);
            }
        }

        // Handle double-click to add keyframe
        if response.double_clicked() {
            if let (Some(pos), Some(track_id)) =
                (response.interact_pointer_pos(), self.selected_track)
            {
                let time =
                    self.scroll_offset + (pos.x - rect.min.x) / self.settings.pixels_per_second;
                let time = time.max(0.0).min(duration);
                let time = if self.settings.snap_to_frame {
                    snap_to_frame(time)
                } else {
                    time
                };

                action = TimelineAction::AddKeyframe { track_id, time };
            }
        }

        action
    }

    /// Draw time ruler
    fn draw_time_ruler(&self, painter: &egui::Painter, rect: Rect) {
        let duration = self.sequence.as_ref().map(|s| s.duration).unwrap_or(10.0);

        // Background
        painter.rect_filled(rect, 0.0, Color32::from_rgb(40, 40, 45));

        // Draw time marks
        let seconds_per_mark = (50.0 / self.settings.pixels_per_second).max(0.1);
        let major_interval = if seconds_per_mark < 0.5 {
            0.5
        } else if seconds_per_mark < 1.0 {
            1.0
        } else if seconds_per_mark < 5.0 {
            5.0
        } else {
            10.0
        };

        let start_time = self.scroll_offset;
        let end_time = start_time + rect.width() / self.settings.pixels_per_second;
        let mut time = (start_time / major_interval).floor() * major_interval;

        while time <= end_time && time <= duration {
            let x = rect.min.x + (time - start_time) * self.settings.pixels_per_second;

            if x >= rect.min.x && x <= rect.max.x {
                // Major tick
                painter.line_segment(
                    [Pos2::new(x, rect.max.y - 10.0), Pos2::new(x, rect.max.y)],
                    Stroke::new(1.0, Color32::GRAY),
                );

                // Time label
                let label = format!("{:.1}s", time);
                painter.text(
                    Pos2::new(x + 2.0, rect.min.y + 4.0),
                    egui::Align2::LEFT_TOP,
                    label,
                    egui::FontId::proportional(10.0),
                    Color32::LIGHT_GRAY,
                );
            }

            time += major_interval;
        }
    }

    /// Draw a track lane with keyframes
    fn draw_track_lane(&self, painter: &egui::Painter, rect: Rect, track: &Track) {
        // Background
        let bg_color = if self.selected_track == Some(track.id) {
            Color32::from_rgb(50, 50, 60)
        } else {
            Color32::from_rgb(35, 35, 40)
        };
        painter.rect_filled(rect, 0.0, bg_color);

        // Track color bar
        if self.settings.show_track_colors {
            let color_bar = Rect::from_min_size(rect.min, Vec2::new(3.0, rect.height()));
            let color = Color32::from_rgba_unmultiplied(
                (track.color[0] * 255.0) as u8,
                (track.color[1] * 255.0) as u8,
                (track.color[2] * 255.0) as u8,
                255,
            );
            painter.rect_filled(color_bar, 0.0, color);
        }

        // Draw keyframes
        for (idx, keyframe) in track.keyframes.iter().enumerate() {
            let x =
                rect.min.x + (keyframe.time - self.scroll_offset) * self.settings.pixels_per_second;

            if x < rect.min.x || x > rect.max.x {
                continue;
            }

            let selected = self.selected_keyframe == Some((track.id, idx));

            let kf_color = if selected {
                Color32::from_rgb(255, 200, 100)
            } else {
                Color32::from_rgb(200, 200, 200)
            };

            // Keyframe diamond
            let center = Pos2::new(x, rect.center().y);
            let size = 5.0;
            let points = vec![
                Pos2::new(center.x, center.y - size),
                Pos2::new(center.x + size, center.y),
                Pos2::new(center.x, center.y + size),
                Pos2::new(center.x - size, center.y),
            ];
            painter.add(egui::Shape::convex_polygon(
                points,
                kf_color,
                Stroke::new(1.0, Color32::WHITE),
            ));
        }

        // Bottom border
        painter.line_segment(
            [
                Pos2::new(rect.min.x, rect.max.y),
                Pos2::new(rect.max.x, rect.max.y),
            ],
            Stroke::new(1.0, Color32::from_rgb(60, 60, 65)),
        );
    }

    /// Draw the add track dialog
    fn draw_add_track_dialog(
        &mut self,
        ctx: &egui::Context,
        scene_objects: &[SceneObject],
    ) -> TimelineAction {
        let mut action = TimelineAction::None;
        let mut close = false;

        egui::Window::new("Add Track")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("track_type")
                        .selected_text(self.new_track_type.display_name())
                        .show_ui(ui, |ui| {
                            let types = [
                                TrackType::ObjectPosition,
                                TrackType::ObjectRotation,
                                TrackType::ObjectScale,
                                TrackType::ObjectColor,
                                TrackType::ObjectVisibility,
                                TrackType::CameraPosition,
                                TrackType::CameraTarget,
                                TrackType::CameraFov,
                                TrackType::Event,
                                TrackType::Dialogue,
                                TrackType::Audio,
                            ];
                            for t in types {
                                ui.selectable_value(&mut self.new_track_type, t, t.display_name());
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Target:");
                    let target_name = self
                        .new_track_target
                        .and_then(|id| scene_objects.iter().find(|o| o.id == id))
                        .map(|o| o.name.as_str())
                        .unwrap_or("None");

                    egui::ComboBox::from_id_salt("track_target")
                        .selected_text(target_name)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(self.new_track_target.is_none(), "None")
                                .clicked()
                            {
                                self.new_track_target = None;
                            }
                            ui.separator();
                            for obj in scene_objects {
                                if ui
                                    .selectable_label(
                                        self.new_track_target == Some(obj.id),
                                        &obj.name,
                                    )
                                    .clicked()
                                {
                                    self.new_track_target = Some(obj.id);
                                }
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.new_track_name);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                    if ui.button("Create").clicked() {
                        action = TimelineAction::AddTrackWithTarget {
                            track_type: self.new_track_type,
                            target_entity: self.new_track_target,
                            name: self.new_track_name.clone(),
                        };
                        close = true;
                    }
                });
            });

        if close {
            self.show_add_track_dialog = false;
        }

        action
    }

    /// Draw the keyframe editor panel
    fn draw_keyframe_editor(&mut self, ui: &mut Ui) -> TimelineAction {
        let mut action = TimelineAction::None;

        let Some((track_id, kf_index)) = self.selected_keyframe else {
            return action;
        };

        // Get keyframe info
        let kf_info = self.sequence.as_ref().and_then(|s| {
            s.get_track(track_id).and_then(|t| {
                t.keyframes
                    .get(kf_index)
                    .map(|kf| (t.name.clone(), kf.time, kf.easing, kf.value.clone()))
            })
        });

        let Some((track_name, time, easing, value)) = kf_info else {
            self.selected_keyframe = None;
            return action;
        };

        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(40, 45, 50))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.strong(format!("Keyframe: {} @ {:.2}s", track_name, time));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("Delete").clicked() {
                            action = TimelineAction::DeleteKeyframe {
                                track_id,
                                keyframe_index: kf_index,
                            };
                            self.selected_keyframe = None;
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("Time:");
                    if self.keyframe_time_input.is_empty() {
                        self.keyframe_time_input = format!("{:.3}", time);
                    }
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.keyframe_time_input)
                            .desired_width(60.0),
                    );
                    if response.lost_focus() {
                        if let Ok(new_time) = self.keyframe_time_input.parse::<f32>() {
                            if (new_time - time).abs() > 0.001 {
                                action = TimelineAction::UpdateKeyframe {
                                    track_id,
                                    keyframe_index: kf_index,
                                    time: Some(new_time),
                                    easing: None,
                                };
                            }
                        }
                        self.keyframe_time_input.clear();
                    }
                    ui.label("sec");

                    ui.separator();

                    ui.label("Easing:");
                    let mut current_easing = easing;
                    egui::ComboBox::from_id_salt("keyframe_easing")
                        .selected_text(format!("{:?}", current_easing))
                        .width(100.0)
                        .show_ui(ui, |ui| {
                            let easings = [
                                Easing::Linear,
                                Easing::EaseIn,
                                Easing::EaseOut,
                                Easing::EaseInOut,
                                Easing::Step,
                            ];
                            for e in easings {
                                if ui
                                    .selectable_value(&mut current_easing, e, format!("{:?}", e))
                                    .changed()
                                {
                                    action = TimelineAction::UpdateKeyframe {
                                        track_id,
                                        keyframe_index: kf_index,
                                        time: None,
                                        easing: Some(e),
                                    };
                                }
                            }
                        });
                });

                // Show value (read-only for now)
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    match &value {
                        KeyframeValue::Float(f) => {
                            ui.label(format!("{:.3}", f));
                        }
                        KeyframeValue::Vec3(v) => {
                            ui.label(format!("X: {:.2}  Y: {:.2}  Z: {:.2}", v.x, v.y, v.z));
                        }
                        KeyframeValue::Quat(q) => {
                            let euler = q.to_euler(glam::EulerRot::XYZ);
                            ui.label(format!(
                                "X: {:.1}°  Y: {:.1}°  Z: {:.1}°",
                                euler.0.to_degrees(),
                                euler.1.to_degrees(),
                                euler.2.to_degrees()
                            ));
                        }
                        KeyframeValue::Bool(b) => {
                            ui.label(if *b { "true" } else { "false" });
                        }
                        KeyframeValue::Color(c) => {
                            let color = Color32::from_rgba_unmultiplied(
                                (c[0] * 255.0) as u8,
                                (c[1] * 255.0) as u8,
                                (c[2] * 255.0) as u8,
                                (c[3] * 255.0) as u8,
                            );
                            ui.color_edit_button_srgba(&mut color.clone());
                        }
                        KeyframeValue::Event { name, data } => {
                            ui.label(format!("Event: {} {:?}", name, data));
                        }
                        KeyframeValue::Dialogue {
                            speaker,
                            text,
                            duration,
                        } => {
                            ui.label(format!("[{}] \"{}\" ({:.1}s)", speaker, text, duration));
                        }
                        KeyframeValue::Audio { clip_path, volume } => {
                            ui.label(format!("Audio: {} @ {:.0}%", clip_path, volume * 100.0));
                        }
                    }
                });
            });

        action
    }

    /// Get the keyframe value for a track based on target entity
    fn get_keyframe_value_for_track(
        &self,
        track_id: u32,
        scene_objects: &[SceneObject],
    ) -> KeyframeValue {
        let Some(ref sequence) = self.sequence else {
            return KeyframeValue::Vec3(Vec3::ZERO);
        };

        let Some(track) = sequence.get_track(track_id) else {
            return KeyframeValue::Vec3(Vec3::ZERO);
        };

        // Get target object if available
        let target_obj = track
            .target_entity
            .and_then(|id| scene_objects.iter().find(|o| o.id == id));

        match track.track_type {
            TrackType::ObjectPosition | TrackType::CameraPosition => target_obj
                .map(|o| KeyframeValue::Vec3(o.position))
                .unwrap_or(KeyframeValue::Vec3(Vec3::ZERO)),
            TrackType::ObjectRotation => target_obj
                .map(|o| KeyframeValue::Vec3(o.rotation))
                .unwrap_or(KeyframeValue::Vec3(Vec3::ZERO)),
            TrackType::ObjectScale => target_obj
                .map(|o| KeyframeValue::Vec3(o.scale))
                .unwrap_or(KeyframeValue::Vec3(Vec3::ONE)),
            TrackType::ObjectColor => target_obj
                .map(|o| KeyframeValue::Color(o.color))
                .unwrap_or(KeyframeValue::Color([1.0, 1.0, 1.0, 1.0])),
            TrackType::ObjectVisibility => target_obj
                .map(|o| KeyframeValue::Bool(o.visible))
                .unwrap_or(KeyframeValue::Bool(true)),
            TrackType::CameraTarget => KeyframeValue::Vec3(Vec3::ZERO),
            TrackType::CameraFov => target_obj
                .and_then(|o| o.camera.as_ref())
                .map(|c| match c.projection {
                    CameraProjection::Perspective { fov, .. } => KeyframeValue::Float(fov),
                    CameraProjection::Orthographic { size, .. } => KeyframeValue::Float(size),
                })
                .unwrap_or(KeyframeValue::Float(60.0)),
            TrackType::BoneTranslation | TrackType::BoneRotation | TrackType::BoneScale => {
                KeyframeValue::Vec3(Vec3::ZERO)
            }
            TrackType::Event => KeyframeValue::Event {
                name: "event".to_string(),
                data: None,
            },
            TrackType::Dialogue => KeyframeValue::Dialogue {
                speaker: "Speaker".to_string(),
                text: "Dialogue text".to_string(),
                duration: 2.0,
            },
            TrackType::Audio => KeyframeValue::Audio {
                clip_path: String::new(),
                volume: 1.0,
            },
        }
    }

    /// Handle keyframe click in track lane - returns clicked keyframe info
    pub fn detect_keyframe_click(
        &mut self,
        pointer_pos: Pos2,
        timeline_rect: Rect,
    ) -> Option<(u32, usize)> {
        let sequence = self.sequence.as_ref()?;

        let ruler_height = 20.0;
        let tracks_start_y = timeline_rect.min.y + ruler_height;

        for (track_idx, track) in sequence.tracks.iter().enumerate() {
            let track_y = tracks_start_y + track_idx as f32 * self.settings.track_height;
            let track_center_y = track_y + self.settings.track_height / 2.0;

            for (kf_idx, keyframe) in track.keyframes.iter().enumerate() {
                let x = timeline_rect.min.x
                    + (keyframe.time - self.scroll_offset) * self.settings.pixels_per_second;

                // Check if click is within keyframe diamond bounds
                let size = 7.0; // Slightly larger than visual for easier clicking
                if (pointer_pos.x - x).abs() < size && (pointer_pos.y - track_center_y).abs() < size
                {
                    return Some((track.id, kf_idx));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_panel() {
        let mut panel = TimelinePanel::new();
        assert!(!panel.visible);
        assert!(panel.sequence().is_none());

        let seq = TimelineSequence::new("Test");
        panel.set_sequence(seq);
        assert!(panel.sequence().is_some());
    }
}

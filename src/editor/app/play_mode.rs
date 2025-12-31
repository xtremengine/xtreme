//! Play mode actions: start, stop, script updates.

use super::EditorApp;
#[cfg(feature = "scripting")]
use crate::scripting::ObjectTransform;

impl EditorApp {
    /// Start play mode (opens game window)
    pub fn start_play(&mut self) {
        if self.is_playing || self.pending_game_start || self.game_window.is_some() {
            return;
        }

        // Set flag to create game window in next frame
        // (window creation needs event loop access)
        self.pending_game_start = true;
        log::info!("Requesting game window creation...");
    }

    /// Stop play mode and close game window
    pub fn stop_play(&mut self) {
        if !self.is_playing && self.game_window.is_none() {
            return;
        }

        self.stop_play_and_close_game_window();
    }

    /// Internal method to stop play and close game window
    pub(crate) fn stop_play_and_close_game_window(&mut self) {
        // Close game window
        if let Some(mut game_window) = self.game_window.take() {
            game_window.close();
        }

        // Restore scene state
        self.scene_objects = self.saved_scene_state.clone();
        self.play_time = 0.0;
        self.last_frame_instant = None;
        self.is_playing = false;
        self.pending_game_start = false;

        log::info!("Play mode stopped, scene reset");
    }

    /// Toggle play mode
    pub fn toggle_play(&mut self) {
        if self.is_playing || self.game_window.is_some() {
            self.stop_play();
        } else {
            self.start_play();
        }
    }

    /// Update scripts (called each frame during play)
    #[cfg(feature = "scripting")]
    pub fn update_scripts(&mut self) {
        if !self.is_playing {
            return;
        }

        // Calculate delta time
        let now = std::time::Instant::now();
        let delta = if let Some(last) = self.last_frame_instant {
            now.duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.last_frame_instant = Some(now);
        self.play_time += delta;

        // Sync context with current scene state
        self.sync_script_context();

        // Call _update on all scripts
        if let Err(e) = self
            .script_runtime
            .call_update(&mut self.script_context, delta)
        {
            log::error!("Script update failed: {}", e);
        }

        // Apply changes from scripts back to scene objects
        self.apply_script_changes();
    }

    #[cfg(feature = "scripting")]
    pub(super) fn sync_script_context(&mut self) {
        self.script_context.clear_objects();
        self.script_context.set_time(self.play_time);

        for obj in &self.scene_objects {
            let transform = ObjectTransform {
                position: obj.position.to_array(),
                rotation: obj.rotation.to_array(),
                scale: obj.scale.to_array(),
            };
            self.script_context
                .register_object(obj.id, obj.name.clone(), transform, obj.visible);
        }
    }

    #[cfg(feature = "scripting")]
    fn apply_script_changes(&mut self) {
        // Apply position changes
        for (id, new_pos) in self.script_context.drain_position_changes() {
            if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                obj.position = glam::Vec3::from_array(new_pos);
            }
        }

        // Apply rotation changes
        for (id, new_rot) in self.script_context.drain_rotation_changes() {
            if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                obj.rotation = glam::Vec3::from_array(new_rot);
            }
        }

        // Apply scale changes
        for (id, new_scale) in self.script_context.drain_scale_changes() {
            if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                obj.scale = glam::Vec3::from_array(new_scale);
            }
        }
    }
}

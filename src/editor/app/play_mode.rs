//! Play mode actions: start, stop, script updates.

#[cfg(feature = "scripting")]
use crate::scripting::ObjectTransform;
use super::EditorApp;

/// Splash screen duration in seconds
const SPLASH_DURATION: f32 = 2.0;

impl EditorApp {
    /// Start play mode (shows splash first)
    pub fn start_play(&mut self) {
        if self.is_playing || self.show_splash {
            return;
        }

        // Show splash screen first
        self.show_splash = true;
        self.splash_start_time = Some(std::time::Instant::now());

        // Save current scene state
        self.saved_scene_state = self.scene_objects.clone();

        log::info!("Showing splash screen...");
    }

    /// Actually start play mode (called after splash)
    fn enter_play_mode(&mut self) {
        self.show_splash = false;
        self.play_time = 0.0;
        self.last_frame_instant = Some(std::time::Instant::now());
        self.is_playing = true;

        // Call _ready on all scripts
        #[cfg(feature = "scripting")]
        {
            self.sync_script_context();
            if let Err(e) = self.script_runtime.call_ready(&mut self.script_context) {
                log::error!("Script ready failed: {}", e);
            }
        }

        log::info!("Play mode started");
    }

    /// Update splash screen and transition to play mode
    pub fn update_splash(&mut self) {
        if !self.show_splash {
            return;
        }

        if let Some(start) = self.splash_start_time {
            let elapsed = start.elapsed().as_secs_f32();
            if elapsed >= SPLASH_DURATION {
                self.enter_play_mode();
            }
        }
    }

    /// Stop play mode and reset scene
    pub fn stop_play(&mut self) {
        if !self.is_playing && !self.show_splash {
            return;
        }

        // Cancel splash if showing
        if self.show_splash {
            self.show_splash = false;
            self.splash_start_time = None;
        }

        // Restore scene state
        self.scene_objects = self.saved_scene_state.clone();
        self.play_time = 0.0;
        self.last_frame_instant = None;
        self.is_playing = false;

        log::info!("Play mode stopped, scene reset");
    }

    /// Toggle play mode
    pub fn toggle_play(&mut self) {
        if self.is_playing {
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
        if let Err(e) = self.script_runtime.call_update(&mut self.script_context, delta) {
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
            self.script_context.register_object(obj.id, obj.name.clone(), transform, obj.visible);
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

//! Game window event handling for play mode.

use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::EditorApp;
use crate::editor::game_window::{GameSettings, GameWindow};

impl EditorApp {
    /// Handle pending window creation (game window)
    pub(super) fn handle_game_window_creation(&mut self, event_loop: &ActiveEventLoop) {
        if !self.pending_game_start {
            return;
        }

        self.pending_game_start = false;

        // Save current scene state
        self.saved_scene_state = self.scene_objects.clone();

        // Build game settings from project config
        let settings = if let Some(project) = &self.current_project {
            GameSettings {
                window_width: project.config.window_width,
                window_height: project.config.window_height,
                splash_duration: project.config.splash_duration,
                background_color: project.config.background_color,
                show_fps: project.config.show_fps,
                vsync: project.config.vsync,
            }
        } else {
            GameSettings::default()
        };

        // Create game window
        match pollster::block_on(GameWindow::new(
            event_loop,
            self.scene_objects.clone(),
            settings,
        )) {
            Ok(game_window) => {
                game_window.window.request_redraw();
                self.is_playing = true;
                self.play_time = 0.0;
                self.last_frame_instant = Some(std::time::Instant::now());
                log::info!("Game window created - play mode started");

                // Initialize scripts with game window objects
                #[cfg(feature = "scripting")]
                self.init_scripts_for_game(&game_window);

                self.game_window = Some(game_window);

                // Reset timeline to beginning (will auto-start after splash screen)
                if self.timeline_panel.sequence().is_some() {
                    self.timeline_panel.player_mut().stop(); // Reset to start
                    self.timeline_game_started = false; // Will start after splash
                }
            }
            Err(e) => {
                log::error!("Failed to create game window: {}", e);
            }
        }
    }

    /// Initialize scripts when game starts
    #[cfg(feature = "scripting")]
    fn init_scripts_for_game(&mut self, game_window: &GameWindow) {
        use crate::scripting::{AnimatorData, ObjectTransform};

        // Sync script context with game window objects
        self.script_context.clear_objects();
        self.script_context.set_time(0.0);

        for obj in game_window.scene_objects() {
            let transform = ObjectTransform {
                position: obj.position.to_array(),
                rotation: obj.rotation.to_array(),
                scale: obj.scale.to_array(),
            };
            self.script_context
                .register_object(obj.id, obj.name.clone(), transform, obj.visible);

            // Register animator data if object has animator component
            if let Some(animator) = &obj.animator {
                let animator_data = AnimatorData {
                    animations: animator
                        .animation_names()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                    current_animation: animator.current_animation.clone(),
                    playing: animator.playing,
                    speed: animator.speed,
                };
                self.script_context.register_animator(obj.id, animator_data);
            }
        }

        // Call _ready on all scripts
        if let Err(e) = self.script_runtime.call_ready(&mut self.script_context) {
            log::error!("Script ready failed: {}", e);
        }
    }

    /// Handle game window events
    pub(super) fn handle_game_window_event(
        &mut self,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> bool {
        let Some(game_window) = &mut self.game_window else {
            return false;
        };

        if game_window.id() != window_id {
            return false;
        }

        match event {
            WindowEvent::CloseRequested => {
                self.stop_play_and_close_game_window();
                true
            }
            WindowEvent::Resized(size) => {
                game_window.resize(*size);
                true
            }
            WindowEvent::RedrawRequested => {
                self.handle_game_redraw();
                true
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // ESC closes game window
                if event.state == winit::event::ElementState::Pressed {
                    if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) =
                        event.logical_key
                    {
                        self.stop_play_and_close_game_window();
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Handle game window redraw
    fn handle_game_redraw(&mut self) {
        // Update game state and get delta/play_time
        #[allow(unused_variables)]
        let (delta, play_time) = {
            let Some(game_window) = &mut self.game_window else {
                return;
            };
            let delta = game_window.update();
            let play_time = game_window.play_time();
            (delta, play_time)
        };

        // Run scripts (needs separate borrow of self)
        #[cfg(feature = "scripting")]
        {
            self.run_game_scripts(delta, play_time);
        }

        // Update and apply timeline animation during Play Game
        self.update_timeline_for_game(delta);

        // Render and request next frame
        let Some(game_window) = &mut self.game_window else {
            return;
        };

        // Check if audio should be initialized (after splash screen)
        game_window.check_and_init_audio();

        // Update particle transforms to follow moved objects
        game_window.update_particle_transforms();

        if let Err(e) = game_window.render(delta) {
            log::error!("Game window render failed: {:?}", e);
        }

        game_window.window.request_redraw();
    }

    /// Run scripts during game play
    #[cfg(feature = "scripting")]
    fn run_game_scripts(&mut self, delta: f32, play_time: f32) {
        use crate::scripting::{AnimationChange, AnimatorData, ObjectTransform};

        let Some(game_window) = &mut self.game_window else {
            return;
        };

        // Sync script context with game window objects
        self.script_context.clear_objects();
        self.script_context.set_time(play_time);

        for obj in game_window.scene_objects() {
            let transform = ObjectTransform {
                position: obj.position.to_array(),
                rotation: obj.rotation.to_array(),
                scale: obj.scale.to_array(),
            };
            self.script_context
                .register_object(obj.id, obj.name.clone(), transform, obj.visible);

            // Register animator data if object has animator component
            if let Some(animator) = &obj.animator {
                let animator_data = AnimatorData {
                    animations: animator
                        .animation_names()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                    current_animation: animator.current_animation.clone(),
                    playing: animator.playing,
                    speed: animator.speed,
                };
                self.script_context.register_animator(obj.id, animator_data);
            }
        }

        // Call _update on all scripts
        if let Err(e) = self
            .script_runtime
            .call_update(&mut self.script_context, delta)
        {
            log::error!("Script update failed: {}", e);
        }

        // Apply script changes back to game window objects
        for (id, new_pos) in self.script_context.drain_position_changes() {
            if let Some(obj) = game_window
                .scene_objects_mut()
                .iter_mut()
                .find(|o| o.id == id)
            {
                obj.position = glam::Vec3::from_array(new_pos);
            }
        }
        for (id, new_rot) in self.script_context.drain_rotation_changes() {
            if let Some(obj) = game_window
                .scene_objects_mut()
                .iter_mut()
                .find(|o| o.id == id)
            {
                obj.rotation = glam::Vec3::from_array(new_rot);
            }
        }
        for (id, new_scale) in self.script_context.drain_scale_changes() {
            if let Some(obj) = game_window
                .scene_objects_mut()
                .iter_mut()
                .find(|o| o.id == id)
            {
                obj.scale = glam::Vec3::from_array(new_scale);
            }
        }

        // Apply animation changes
        for (id, change) in self.script_context.drain_animation_changes() {
            if let Some(obj) = game_window
                .scene_objects_mut()
                .iter_mut()
                .find(|o| o.id == id)
            {
                if let Some(animator) = &mut obj.animator {
                    match change {
                        AnimationChange::Play(name) => {
                            animator.set_animation(&name);
                            animator.playing = true;
                            animator.current_time = 0.0;
                        }
                        AnimationChange::Stop => {
                            animator.playing = false;
                        }
                        AnimationChange::SetSpeed(speed) => {
                            animator.speed = speed;
                        }
                    }
                }
            }
        }
    }

    /// Update and apply timeline animation during Play Game
    fn update_timeline_for_game(&mut self, delta: f32) {
        // Check if splash screen is done before starting timeline
        let splash_done = self
            .game_window
            .as_ref()
            .map(|gw| !gw.in_splash)
            .unwrap_or(false);

        // Auto-start timeline after splash screen ends
        if splash_done
            && !self.timeline_game_started
            && self.timeline_panel.sequence().is_some()
        {
            self.timeline_panel.player_mut().play();
            self.timeline_game_started = true;
            log::info!("Timeline started after splash screen");
        }

        // Update timeline player
        self.timeline_panel.update(delta);

        // Check if timeline is playing
        if !self.timeline_panel.player().is_playing() {
            return;
        }

        // Sample timeline and apply to game window objects
        if let Some(sequence) = self.timeline_panel.sequence().cloned() {
            let sample = self.timeline_panel.player_mut().sample(&sequence);

            if let Some(game_window) = &mut self.game_window {
                // Apply object transforms
                for (entity_id, transform) in &sample.object_transforms {
                    if let Some(obj) = game_window
                        .scene_objects_mut()
                        .iter_mut()
                        .find(|o| o.id == *entity_id)
                    {
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

                // Log when applying transforms (debug)
                if !sample.object_transforms.is_empty() {
                    log::trace!(
                        "Timeline applied {} transforms during play",
                        sample.object_transforms.len()
                    );
                }
            }
        }
    }
}

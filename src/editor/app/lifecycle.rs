//! App trait implementation: init, events, update, render.

use glam::{Mat4, Vec3};
use std::sync::Arc;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window as WinitWindow, WindowId};

use super::state::FileDialogAction;
use super::EditorApp;
use crate::editor::game_window::{GameSettings, GameWindow};
use crate::editor::gizmos::GizmoMode;
use crate::editor::panels::Tool;
use crate::editor::selection::SceneObject;
use crate::editor::shortcuts::EditorAction;
use crate::editor::viewport::{ObjectRenderData, Viewport};
use crate::render::{App, AppEvent, EguiIntegration, IsometricCamera, RenderContext};

impl EditorApp {
    /// Get model matrices, colors, and textures for rendering (uses world transforms)
    /// Returns (regular_objects, camera_objects) - cameras are rendered separately as wireframes
    pub fn get_render_data(&self) -> (Vec<ObjectRenderData>, Vec<(Mat4, [f32; 4])>) {
        let mut regular = Vec::new();
        let mut cameras = Vec::new();

        for o in self.scene_objects.iter().filter(|o| o.visible) {
            let is_selected = self.selection.is_selected(o.id);
            let color = if is_selected {
                [1.0, 0.8, 0.2, 1.0] // Yellow highlight
            } else {
                o.color
            };
            // Use world matrix to account for parent hierarchy
            let model = o.world_matrix(&self.scene_objects);

            if o.camera.is_some() {
                cameras.push((model, color));
            } else {
                regular.push(ObjectRenderData {
                    model,
                    color,
                    texture_path: o.texture_path.clone(),
                    shader_path: o.shader_path.clone(),
                });
            }
        }

        (regular, cameras)
    }
}

impl App for EditorApp {
    fn init(&mut self, window: Arc<WinitWindow>) {
        log::info!("Initializing editor...");

        let ctx = pollster::block_on(RenderContext::new(window.clone()))
            .expect("Failed to create render context");

        log::info!("GPU: {}", ctx.adapter_info().name);

        // Initialize egui
        let mut egui = EguiIntegration::new(&ctx, &window);

        // Initialize viewport
        let (w, h) = ctx.size();
        let viewport_width = (w as f32 * 0.6) as u32;
        let viewport_height = (h as f32 * 0.8) as u32;

        let viewport = Viewport::new(&ctx, egui.renderer_mut(), viewport_width, viewport_height);

        self.ctx = Some(ctx);
        self.egui = Some(egui);
        self.viewport = Some(viewport);
        self.window = Some(window);

        log::info!("Editor ready!");
    }

    fn raw_event(&mut self, window: &WinitWindow, event: &WindowEvent) -> bool {
        if let Some(egui) = &mut self.egui {
            egui.handle_event(window, event)
        } else {
            false
        }
    }

    fn event(&mut self, event: AppEvent) {
        if let AppEvent::Resized { width, height } = event {
            if let Some(ctx) = &mut self.ctx {
                ctx.resize(width, height);
            }
        }
    }

    fn update(&mut self) {
        self.frame_count += 1;

        // Begin egui frame
        if let (Some(egui), Some(window)) = (&mut self.egui, &self.window) {
            egui.begin_frame(window);
        }

        // Handle keyboard shortcuts
        let actions: Vec<EditorAction> = if let Some(egui) = &self.egui {
            self.shortcuts.process_input(egui.ctx())
        } else {
            Vec::new()
        };

        // Process triggered actions
        for action in actions {
            self.handle_editor_action(action);
        }

        // Sync gizmo mode with current tool
        match self.toolbar_panel.current_tool {
            Tool::Move => self.gizmo.set_mode(GizmoMode::Translate),
            Tool::Rotate => self.gizmo.set_mode(GizmoMode::Rotate),
            Tool::Scale => self.gizmo.set_mode(GizmoMode::Scale),
            Tool::Select => {} // No gizmo in select mode
        }

        // Draw UI and get viewport size
        let mut viewport_size = self.last_viewport_size;
        self.draw_ui(&mut viewport_size);

        // Resize viewport if needed
        if viewport_size != self.last_viewport_size
            && viewport_size.0 > 0.0
            && viewport_size.1 > 0.0
        {
            if let (Some(ctx), Some(egui), Some(viewport)) =
                (&self.ctx, &mut self.egui, &mut self.viewport)
            {
                viewport.resize(
                    ctx,
                    egui.renderer_mut(),
                    viewport_size.0 as u32,
                    viewport_size.1 as u32,
                );
                self.camera.aspect_ratio = viewport_size.0 / viewport_size.1;
            }
            self.last_viewport_size = viewport_size;
        }

        // End egui frame
        if let (Some(egui), Some(window)) = (&mut self.egui, &self.window) {
            egui.end_frame(window);
        }
    }

    fn render(&mut self) {
        let Some(ctx) = &self.ctx else { return };

        let Ok((output, view)) = ctx.begin_frame() else {
            return;
        };

        // Update editor time for shader animations
        let now = std::time::Instant::now();
        if let Some(last) = self.last_frame_instant {
            let delta = now.duration_since(last).as_secs_f32();
            self.editor_time += delta;
        }
        self.last_frame_instant = Some(now);

        let mut encoder = ctx.create_encoder("Editor Frame");

        // Get render data (regular objects and cameras separately)
        let (regular_objects, camera_objects) = self.get_render_data();

        // Preload textures into cache (must be done before render)
        if let Some(viewport) = &mut self.viewport {
            for obj in &regular_objects {
                if let Some(ref path) = obj.texture_path {
                    // This loads the texture into the cache if not already there
                    viewport.get_or_load_texture(ctx, path);
                }
            }
        }

        // Render viewport
        if let Some(viewport) = &mut self.viewport {
            // Render regular objects as cubes
            viewport.render_objects(
                ctx,
                &mut encoder,
                &self.camera,
                &regular_objects,
                self.editor_time,
            );

            // Render camera objects as wireframe pyramids
            viewport.render_camera_wireframes(ctx, &mut encoder, &self.camera, &camera_objects);

            // Render gizmo if we have a selection and not in Select mode
            if self.toolbar_panel.current_tool != Tool::Select {
                if let Some(id) = self.selection.first() {
                    if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
                        // Use world position for gizmo
                        let world_pos = obj.world_position(&self.scene_objects);
                        viewport.render_gizmo(
                            ctx,
                            &mut encoder,
                            &self.camera,
                            &self.gizmo,
                            world_pos,
                        );
                    }
                }
            }
        }

        // Clear the main window
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Editor Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Render egui on top
        if let Some(egui) = &mut self.egui {
            egui.render(ctx, &mut encoder, &view);
        }

        ctx.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn shutdown(&mut self) {
        log::info!("Editor shutting down after {} frames", self.frame_count);
    }

    fn handle_pending_windows(&mut self, event_loop: &ActiveEventLoop) {
        // Create game window if requested
        if self.pending_game_start {
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
                    {
                        use crate::scripting::ObjectTransform;

                        // Sync script context with game window objects
                        self.script_context.clear_objects();
                        self.script_context.set_time(0.0);

                        for obj in game_window.scene_objects() {
                            let transform = ObjectTransform {
                                position: obj.position.to_array(),
                                rotation: obj.rotation.to_array(),
                                scale: obj.scale.to_array(),
                            };
                            self.script_context.register_object(
                                obj.id,
                                obj.name.clone(),
                                transform,
                                obj.visible,
                            );
                        }

                        // Call _ready on all scripts
                        if let Err(e) = self.script_runtime.call_ready(&mut self.script_context) {
                            log::error!("Script ready failed: {}", e);
                        }
                    }

                    self.game_window = Some(game_window);
                }
                Err(e) => {
                    log::error!("Failed to create game window: {}", e);
                }
            }
        }
    }

    fn secondary_window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> bool {
        // Check if this is our game window
        if let Some(game_window) = &mut self.game_window {
            if game_window.id() == window_id {
                match event {
                    WindowEvent::CloseRequested => {
                        self.stop_play_and_close_game_window();
                        return true;
                    }
                    WindowEvent::Resized(size) => {
                        game_window.resize(*size);
                        return true;
                    }
                    WindowEvent::RedrawRequested => {
                        // Update game state and get delta
                        let _delta = game_window.update();
                        #[cfg(feature = "scripting")]
                        let delta = _delta;
                        #[cfg(feature = "scripting")]
                        let play_time = game_window.play_time();

                        // Run scripts
                        #[cfg(feature = "scripting")]
                        {
                            use crate::scripting::ObjectTransform;

                            // Sync script context with game window objects
                            self.script_context.clear_objects();
                            self.script_context.set_time(play_time);

                            for obj in game_window.scene_objects() {
                                let transform = ObjectTransform {
                                    position: obj.position.to_array(),
                                    rotation: obj.rotation.to_array(),
                                    scale: obj.scale.to_array(),
                                };
                                self.script_context.register_object(
                                    obj.id,
                                    obj.name.clone(),
                                    transform,
                                    obj.visible,
                                );
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
                        }

                        // Render
                        if let Err(e) = game_window.render() {
                            log::error!("Game window render failed: {:?}", e);
                        }

                        // Request next frame
                        game_window.window.request_redraw();
                        return true;
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
                    }
                    _ => {}
                }
            }
        }
        false
    }

    fn main_window_id(&self) -> Option<WindowId> {
        self.window.as_ref().map(|w| w.id())
    }
}

impl EditorApp {
    /// Handle editor action from shortcuts
    fn handle_editor_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::Undo => self.undo(),
            EditorAction::Redo => self.redo(),
            EditorAction::Delete => {
                if let Some(id) = self.selection.first() {
                    self.delete_object(id);
                }
            }
            EditorAction::Duplicate => {
                if let Some(id) = self.selection.first() {
                    self.duplicate_object(id);
                }
            }
            EditorAction::SelectTool => self.toolbar_panel.current_tool = Tool::Select,
            EditorAction::MoveTool => self.toolbar_panel.current_tool = Tool::Move,
            EditorAction::RotateTool => self.toolbar_panel.current_tool = Tool::Rotate,
            EditorAction::ScaleTool => self.toolbar_panel.current_tool = Tool::Scale,
            EditorAction::FocusSelected => {
                if let Some(id) = self.selection.first() {
                    self.focus_on_object(id);
                }
            }
            EditorAction::FrameAll => {
                self.camera.target = Vec3::ZERO;
                self.camera.distance = 30.0;
            }
            EditorAction::ResetCamera => {
                self.camera = IsometricCamera::default();
            }
            EditorAction::TopView => {
                self.camera.pitch = -89.0_f32.to_radians();
                self.camera.yaw = 0.0;
            }
            EditorAction::FrontView => {
                self.camera.pitch = 0.0;
                self.camera.yaw = 0.0;
            }
            EditorAction::SideView => {
                self.camera.pitch = 0.0;
                self.camera.yaw = 90.0_f32.to_radians();
            }
            EditorAction::NewScene => self.new_scene(),
            EditorAction::SaveScene => {
                if let Some(path) = self.scene_manager.current_path() {
                    self.save_scene(path.to_path_buf());
                }
            }
            EditorAction::SaveSceneAs => {
                self.file_dialog_action = Some(FileDialogAction::SaveAs);
            }
            EditorAction::OpenScene => {
                self.file_dialog_action = Some(FileDialogAction::Open);
            }
            EditorAction::ToggleVisibility => {
                if let Some(id) = self.selection.first() {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == id) {
                        obj.visible = !obj.visible;
                    }
                }
            }
            EditorAction::CreateCube => {
                let obj = SceneObject::cube(self.next_id, Vec3::new(0.0, 0.5, 0.0));
                self.next_id += 1;
                self.create_object(obj);
            }
            EditorAction::Cut => self.cut_selected(),
            EditorAction::Copy => self.copy_selected(),
            EditorAction::Paste => self.paste(),
            EditorAction::SelectAll => {
                // Select all visible objects
                for obj in &self.scene_objects {
                    if obj.visible {
                        self.selection.add(obj.id);
                    }
                }
            }
            EditorAction::TogglePlay => {
                self.toggle_play();
            }
            _ => {} // Other actions not yet implemented
        }
    }
}

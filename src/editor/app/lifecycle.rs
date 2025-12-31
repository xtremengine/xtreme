//! App trait implementation: init, events, update, render.

use std::sync::Arc;
use glam::{Vec3, Mat4};
use winit::event::WindowEvent;
use winit::window::Window as WinitWindow;

use crate::render::{App, AppEvent, RenderContext, IsometricCamera, EguiIntegration};
use crate::editor::viewport::Viewport;
use crate::editor::selection::SceneObject;
use crate::editor::panels::Tool;
use crate::editor::gizmos::GizmoMode;
use crate::editor::shortcuts::EditorAction;
use super::state::FileDialogAction;
use super::EditorApp;

impl EditorApp {
    /// Get model matrices and colors for rendering
    pub fn get_render_data(&self) -> Vec<(Mat4, [f32; 4])> {
        self.scene_objects
            .iter()
            .filter(|o| o.visible)
            .map(|o| {
                let is_selected = self.selection.is_selected(o.id);
                let color = if is_selected {
                    [1.0, 0.8, 0.2, 1.0] // Yellow highlight
                } else {
                    o.color
                };
                (o.model_matrix(), color)
            })
            .collect()
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

        // Update splash screen
        self.update_splash();

        // Update scripts during play mode
        #[cfg(feature = "scripting")]
        self.update_scripts();

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
        if viewport_size != self.last_viewport_size && viewport_size.0 > 0.0 && viewport_size.1 > 0.0 {
            if let (Some(ctx), Some(egui), Some(viewport)) = (&self.ctx, &mut self.egui, &mut self.viewport) {
                viewport.resize(ctx, egui.renderer_mut(), viewport_size.0 as u32, viewport_size.1 as u32);
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

        let mut encoder = ctx.create_encoder("Editor Frame");

        // Get render data
        let render_data = self.get_render_data();

        // Render viewport
        if let Some(viewport) = &self.viewport {
            viewport.render(ctx, &mut encoder, &self.camera, &render_data);

            // Render gizmo if we have a selection and not in Select mode
            if self.toolbar_panel.current_tool != Tool::Select {
                if let Some(id) = self.selection.first() {
                    if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
                        viewport.render_gizmo(ctx, &mut encoder, &self.camera, &self.gizmo, obj.position);
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

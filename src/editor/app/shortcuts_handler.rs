//! Editor shortcut action handling.

use glam::Vec3;

use super::state::FileDialogAction;
use super::EditorApp;
use crate::editor::panels::Tool;
use crate::editor::selection::SceneObject;
use crate::editor::shortcuts::EditorAction;
use crate::render::IsometricCamera;

impl EditorApp {
    /// Handle editor action from shortcuts
    pub(super) fn handle_editor_action(&mut self, action: EditorAction) {
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

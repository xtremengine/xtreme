//! Undo/Redo history actions.

use crate::editor::selection::SceneObject;
use crate::editor::commands::Command;
use super::EditorApp;

impl EditorApp {
    /// Execute undo
    pub fn undo(&mut self) {
        if let Some(cmd) = self.command_history.undo() {
            self.apply_undo_command(&cmd);
            log::info!("Undo: {}", cmd.description());
        }
    }

    /// Execute redo
    pub fn redo(&mut self) {
        if let Some(cmd) = self.command_history.redo() {
            self.apply_redo_command(&cmd);
            log::info!("Redo: {}", cmd.description());
        }
    }

    /// Apply an undo command (reverse the operation)
    pub fn apply_undo_command(&mut self, cmd: &Command) {
        match cmd {
            Command::Transform { object_id, old_position, old_rotation, old_scale, .. } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.position = *old_position;
                    obj.rotation = *old_rotation;
                    obj.scale = *old_scale;
                }
            }
            Command::BatchTransform { object_ids, old_transforms, .. } => {
                for (id, (pos, rot, scale)) in object_ids.iter().zip(old_transforms.iter()) {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *id) {
                        obj.position = *pos;
                        obj.rotation = *rot;
                        obj.scale = *scale;
                    }
                }
            }
            Command::Create { object_id, .. } => {
                self.scene_objects.retain(|o| o.id != *object_id);
                self.selection.remove(*object_id);
            }
            Command::Delete { object_id, name, position, rotation, scale, color, visible } => {
                let obj = SceneObject {
                    id: *object_id,
                    name: name.clone(),
                    position: *position,
                    rotation: *rotation,
                    scale: *scale,
                    color: *color,
                    visible: *visible,
                    scripts: Vec::new(),
                };
                self.scene_objects.push(obj);
            }
            Command::Rename { object_id, old_name, .. } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.name = old_name.clone();
                }
            }
            Command::ChangeColor { object_id, old_color, .. } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.color = *old_color;
                }
            }
            Command::ToggleVisibility { object_id, was_visible } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.visible = *was_visible;
                }
            }
            Command::Batch(cmds) => {
                for cmd in cmds.iter().rev() {
                    self.apply_undo_command(cmd);
                }
            }
        }
    }

    /// Apply a redo command (re-apply the operation)
    pub fn apply_redo_command(&mut self, cmd: &Command) {
        match cmd {
            Command::Transform { object_id, new_position, new_rotation, new_scale, .. } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.position = *new_position;
                    obj.rotation = *new_rotation;
                    obj.scale = *new_scale;
                }
            }
            Command::BatchTransform { object_ids, new_transforms, .. } => {
                for (id, (pos, rot, scale)) in object_ids.iter().zip(new_transforms.iter()) {
                    if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *id) {
                        obj.position = *pos;
                        obj.rotation = *rot;
                        obj.scale = *scale;
                    }
                }
            }
            Command::Create { object_id, name, position, rotation, scale, color, visible } => {
                let obj = SceneObject {
                    id: *object_id,
                    name: name.clone(),
                    position: *position,
                    rotation: *rotation,
                    scale: *scale,
                    color: *color,
                    visible: *visible,
                    scripts: Vec::new(),
                };
                self.scene_objects.push(obj);
                self.selection.select(*object_id);
            }
            Command::Delete { object_id, .. } => {
                self.scene_objects.retain(|o| o.id != *object_id);
                self.selection.remove(*object_id);
            }
            Command::Rename { object_id, new_name, .. } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.name = new_name.clone();
                }
            }
            Command::ChangeColor { object_id, new_color, .. } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.color = *new_color;
                }
            }
            Command::ToggleVisibility { object_id, was_visible } => {
                if let Some(obj) = self.scene_objects.iter_mut().find(|o| o.id == *object_id) {
                    obj.visible = !*was_visible;
                }
            }
            Command::Batch(cmds) => {
                for cmd in cmds {
                    self.apply_redo_command(cmd);
                }
            }
        }
    }
}

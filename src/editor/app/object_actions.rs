//! Object CRUD actions: create, delete, duplicate, focus.

use glam::Vec3;

use crate::editor::selection::SceneObject;
use crate::editor::commands::Command;
use super::EditorApp;

impl EditorApp {
    /// Create a new scene object
    pub fn create_object(&mut self, obj: SceneObject) {
        // Record command for undo
        let cmd = Command::Create {
            object_id: obj.id,
            name: obj.name.clone(),
            position: obj.position,
            rotation: obj.rotation,
            scale: obj.scale,
            color: obj.color,
            visible: obj.visible,
        };
        self.command_history.execute(cmd);
        self.scene_manager.mark_dirty();

        let id = obj.id;
        self.scene_objects.push(obj);
        self.selection.select(id);
        log::info!("Created object {}", id);
    }

    /// Delete an object by ID
    pub fn delete_object(&mut self, id: u32) {
        // Record command for undo
        if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
            let cmd = Command::Delete {
                object_id: id,
                name: obj.name.clone(),
                position: obj.position,
                rotation: obj.rotation,
                scale: obj.scale,
                color: obj.color,
                visible: obj.visible,
            };
            self.command_history.execute(cmd);
            self.scene_manager.mark_dirty();
        }

        self.scene_objects.retain(|o| o.id != id);
        self.selection.remove(id);
        log::info!("Deleted object {}", id);
    }

    /// Duplicate an object
    pub fn duplicate_object(&mut self, id: u32) {
        if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
            let mut new_obj = obj.clone();
            new_obj.id = self.next_id;
            self.next_id += 1;
            new_obj.name = format!("{} (copy)", obj.name);
            new_obj.position += Vec3::new(1.0, 0.0, 1.0);
            self.create_object(new_obj);
        }
    }

    /// Focus camera on an object
    pub fn focus_on_object(&mut self, id: u32) {
        if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
            self.camera.target = obj.position;
            log::info!("Focused on object {}", id);
        }
    }
}

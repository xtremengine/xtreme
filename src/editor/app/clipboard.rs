//! Clipboard actions: copy, cut, paste.

use glam::Vec3;

use super::EditorApp;
use crate::editor::commands::Command;

impl EditorApp {
    /// Copy selected objects to clipboard
    pub fn copy_selected(&mut self) {
        self.clipboard.clear();
        for id in self.selection.all() {
            if let Some(obj) = self.scene_objects.iter().find(|o| o.id == *id) {
                self.clipboard.push(obj.clone());
            }
        }
        log::info!("Copied {} objects to clipboard", self.clipboard.len());
    }

    /// Cut selected objects (copy + delete)
    pub fn cut_selected(&mut self) {
        self.copy_selected();
        let ids: Vec<u32> = self.selection.all().to_vec();
        for id in ids {
            self.delete_object(id);
        }
        log::info!("Cut {} objects", self.clipboard.len());
    }

    /// Paste objects from clipboard
    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }

        let offset = Vec3::new(1.0, 0.0, 1.0);
        self.selection.clear();

        for template in &self.clipboard.clone() {
            let mut obj = template.clone();
            obj.id = self.next_id;
            self.next_id += 1;
            obj.name = format!("{} (copy)", template.name);
            obj.position += offset;

            // Record create command for undo
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

            let id = obj.id;
            self.scene_objects.push(obj);
            self.selection.add(id);
        }

        self.scene_manager.mark_dirty();
        log::info!("Pasted {} objects", self.clipboard.len());
    }

    /// Check if clipboard has content
    pub fn has_clipboard(&self) -> bool {
        !self.clipboard.is_empty()
    }
}

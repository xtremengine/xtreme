//! Prefab actions: create, instantiate, save, load.

use std::path::PathBuf;
use glam::Vec3;

use crate::editor::selection::SceneObject;
use crate::editor::commands::Command;
use crate::editor::prefab::Prefab;
use super::EditorApp;

impl EditorApp {
    /// Create a prefab from current selection
    pub fn create_prefab_from_selection(&mut self, name: &str) -> bool {
        let selected_ids = self.selection.all();
        if selected_ids.is_empty() {
            log::warn!("Cannot create prefab: no objects selected");
            return false;
        }

        // Collect selected objects
        let objects: Vec<SceneObject> = selected_ids.iter()
            .filter_map(|id| self.scene_objects.iter().find(|o| o.id == *id).cloned())
            .collect();

        match Prefab::from_selection(name, &objects) {
            Ok(prefab) => {
                log::info!("Created prefab '{}' with {} objects", prefab.name, prefab.object_count());
                self.prefabs.push(prefab);
                true
            }
            Err(e) => {
                log::error!("Failed to create prefab: {}", e);
                false
            }
        }
    }

    /// Instantiate a prefab at the given position
    pub fn instantiate_prefab(&mut self, prefab_idx: usize, position: Vec3) {
        let Some(prefab) = self.prefabs.get(prefab_idx) else {
            log::error!("Prefab index {} out of range", prefab_idx);
            return;
        };

        let new_objects = prefab.instantiate(position, &mut self.next_id);
        let count = new_objects.len();

        // Clear selection and add new objects
        self.selection.clear();
        for obj in new_objects {
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

            self.selection.add(obj.id);
            self.scene_objects.push(obj);
        }

        self.scene_manager.mark_dirty();
        log::info!("Instantiated prefab with {} objects", count);
    }

    /// Delete a prefab by index
    pub fn delete_prefab(&mut self, idx: usize) {
        if idx < self.prefabs.len() {
            let name = self.prefabs[idx].name.clone();
            self.prefabs.remove(idx);
            log::info!("Deleted prefab '{}'", name);
        }
    }

    /// Save a prefab to file
    pub fn save_prefab(&self, idx: usize, path: PathBuf) {
        if let Some(prefab) = self.prefabs.get(idx) {
            match prefab.save(&path) {
                Ok(()) => log::info!("Saved prefab '{}' to {:?}", prefab.name, path),
                Err(e) => log::error!("Failed to save prefab: {}", e),
            }
        }
    }

    /// Load a prefab from file
    pub fn load_prefab(&mut self, path: PathBuf) {
        match Prefab::load(&path) {
            Ok(prefab) => {
                log::info!("Loaded prefab '{}' with {} objects", prefab.name, prefab.object_count());
                self.prefabs.push(prefab);
            }
            Err(e) => log::error!("Failed to load prefab: {}", e),
        }
    }

    /// Get prefab count
    pub fn prefab_count(&self) -> usize {
        self.prefabs.len()
    }
}

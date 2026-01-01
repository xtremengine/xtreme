//! Object CRUD actions: create, delete, duplicate, focus, reparent.

use glam::Vec3;

use super::EditorApp;
use crate::editor::commands::Command;
use crate::editor::hierarchy::helpers::{can_reparent, get_descendants};
use crate::editor::selection::SceneObject;

impl EditorApp {
    /// Create a new scene object
    pub fn create_object(&mut self, obj: SceneObject) {
        // Check if this is a particle emitter
        let has_particles = obj.particle_emitter.is_some();

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

        // Mark particles dirty if we added a particle emitter
        if has_particles {
            self.particles_dirty = true;
        }

        log::info!("Created object {}", id);
    }

    /// Delete an object by ID (cascades to children)
    pub fn delete_object(&mut self, id: u32) {
        // Get all descendants to delete
        let descendants = get_descendants(&self.scene_objects, id);

        // Delete all descendants first (children, grandchildren, etc.)
        for desc_id in descendants.iter().rev() {
            self.delete_single_object(*desc_id);
        }

        // Remove from parent's children list if has parent
        if let Some(parent_id) = self
            .scene_objects
            .iter()
            .find(|o| o.id == id)
            .and_then(|o| o.hierarchy.parent)
        {
            if let Some(parent) = self.scene_objects.iter_mut().find(|o| o.id == parent_id) {
                parent.hierarchy.remove_child(id);
            }
        }

        // Delete the object itself
        self.delete_single_object(id);
    }

    /// Delete a single object (no cascade)
    fn delete_single_object(&mut self, id: u32) {
        // Check if this object has particles before deleting
        let has_particles = self
            .scene_objects
            .iter()
            .find(|o| o.id == id)
            .map(|o| o.particle_emitter.is_some())
            .unwrap_or(false);

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

        // Mark particles dirty if we deleted a particle emitter
        if has_particles {
            self.particles_dirty = true;
        }

        log::info!("Deleted object {}", id);
    }

    /// Reparent an object to a new parent
    pub fn reparent_object(&mut self, child_id: u32, new_parent_id: Option<u32>) {
        // Validate reparenting (prevent cycles)
        if !can_reparent(&self.scene_objects, child_id, new_parent_id) {
            log::warn!("Cannot reparent: would create cycle or invalid hierarchy");
            return;
        }

        // Get old parent id
        let old_parent_id = self
            .scene_objects
            .iter()
            .find(|o| o.id == child_id)
            .and_then(|o| o.hierarchy.parent);

        // Remove from old parent's children list
        if let Some(old_parent_id) = old_parent_id {
            if let Some(old_parent) = self
                .scene_objects
                .iter_mut()
                .find(|o| o.id == old_parent_id)
            {
                old_parent.hierarchy.remove_child(child_id);
            }
        }

        // Add to new parent's children list
        if let Some(new_parent_id) = new_parent_id {
            if let Some(new_parent) = self
                .scene_objects
                .iter_mut()
                .find(|o| o.id == new_parent_id)
            {
                new_parent.hierarchy.add_child(child_id);
            }
        }

        // Update child's parent reference
        if let Some(child) = self.scene_objects.iter_mut().find(|o| o.id == child_id) {
            child.hierarchy.parent = new_parent_id;
        }

        self.scene_manager.mark_dirty();

        if let Some(parent_id) = new_parent_id {
            log::info!("Reparented object {} to parent {}", child_id, parent_id);
        } else {
            log::info!("Unparented object {} to root", child_id);
        }
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

    /// Focus camera on an object (uses world position)
    pub fn focus_on_object(&mut self, id: u32) {
        if let Some(obj) = self.scene_objects.iter().find(|o| o.id == id) {
            // Use world position to account for parent hierarchy
            self.camera.target = obj.world_position(&self.scene_objects);
            log::info!("Focused on object {}", id);
        }
    }
}

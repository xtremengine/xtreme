//! # Prefab Change Propagation
//!
//! Propagates changes from prefab to instances and vice versa.

use std::collections::HashMap;
use std::path::Path;

use super::instance::PrefabInstance;
use super::registry::PrefabRegistry;
use super::resolver::{apply_overrides, detect_changes};
use super::{Prefab, PrefabError};
use crate::editor::selection::SceneObject;

/// Result of a propagation operation
#[derive(Debug)]
pub struct PropagationResult {
    /// Objects that were updated
    pub updated_objects: Vec<u32>,
    /// Objects that failed to update
    pub failed_objects: Vec<(u32, String)>,
}

impl PropagationResult {
    /// Create empty result
    pub fn new() -> Self {
        Self {
            updated_objects: Vec::new(),
            failed_objects: Vec::new(),
        }
    }

    /// Check if propagation was fully successful
    pub fn is_success(&self) -> bool {
        self.failed_objects.is_empty()
    }
}

impl Default for PropagationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Propagate prefab changes to all instances in the scene
pub fn propagate_to_instances(
    prefab: &Prefab,
    objects: &mut [SceneObject],
    instances: &HashMap<u32, PrefabInstance>,
    prefab_path: &Path,
) -> PropagationResult {
    let mut result = PropagationResult::new();

    // Find all instances of this prefab
    for (object_id, instance) in instances {
        // Check if this instance is from the updated prefab
        if instance.prefab_ref.path != prefab_path {
            continue;
        }

        // Find the object
        let Some(object) = objects.iter_mut().find(|o| o.id == *object_id) else {
            result
                .failed_objects
                .push((*object_id, "Object not found".to_string()));
            continue;
        };

        // Get the prefab object
        let Some(prefab_object) = prefab.objects.get(instance.object_index) else {
            result.failed_objects.push((
                *object_id,
                format!("Invalid prefab index: {}", instance.object_index),
            ));
            continue;
        };

        // Apply prefab values for non-overridden properties
        apply_overrides(object, prefab_object, &instance.overrides);
        result.updated_objects.push(*object_id);
    }

    result
}

/// Apply changes from an instance back to the prefab
pub fn apply_to_prefab(
    object: &SceneObject,
    instance: &PrefabInstance,
    prefab: &mut Prefab,
) -> Result<(), PrefabError> {
    let prefab_object = prefab
        .objects
        .get_mut(instance.object_index)
        .ok_or_else(|| PrefabError::Deserialize("Invalid prefab object index".to_string()))?;

    // Update prefab with instance values
    prefab_object.name = object.name.clone();
    prefab_object.local_position = object.position.to_array();
    prefab_object.rotation = object.rotation.to_array();
    prefab_object.scale = object.scale.to_array();
    prefab_object.color = object.color;
    prefab_object.visible = object.visible;
    prefab_object.camera = object.camera.clone();

    Ok(())
}

/// Check if any instances need updating after prefab change
pub fn find_affected_instances(
    prefab_path: &Path,
    instances: &HashMap<u32, PrefabInstance>,
) -> Vec<u32> {
    instances
        .iter()
        .filter(|(_, inst)| inst.prefab_ref.path == prefab_path)
        .map(|(id, _)| *id)
        .collect()
}

/// Compare an instance to its prefab and update overrides
pub fn sync_instance_overrides(
    object: &SceneObject,
    instance: &mut PrefabInstance,
    prefab: &Prefab,
) -> Result<(), PrefabError> {
    let prefab_object = prefab
        .objects
        .get(instance.object_index)
        .ok_or_else(|| PrefabError::Deserialize("Invalid prefab object index".to_string()))?;

    // Detect what's different
    let detected = detect_changes(object, prefab_object);

    // Update overrides, preserving intentional overrides
    instance.overrides = detected;

    Ok(())
}

/// Propagation manager for handling prefab updates
pub struct PropagationManager {
    /// Track which prefabs have pending updates
    pending_updates: Vec<std::path::PathBuf>,
}

impl Default for PropagationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PropagationManager {
    /// Create new manager
    pub fn new() -> Self {
        Self {
            pending_updates: Vec::new(),
        }
    }

    /// Mark a prefab as having pending updates
    pub fn mark_pending(&mut self, path: std::path::PathBuf) {
        if !self.pending_updates.contains(&path) {
            self.pending_updates.push(path);
        }
    }

    /// Check if there are pending updates
    pub fn has_pending(&self) -> bool {
        !self.pending_updates.is_empty()
    }

    /// Get and clear pending updates
    pub fn take_pending(&mut self) -> Vec<std::path::PathBuf> {
        std::mem::take(&mut self.pending_updates)
    }

    /// Process all pending updates
    pub fn process_pending(
        &mut self,
        registry: &mut PrefabRegistry,
        objects: &mut [SceneObject],
        instances: &HashMap<u32, PrefabInstance>,
    ) -> Vec<PropagationResult> {
        let pending = self.take_pending();
        let mut results = Vec::new();

        for path in pending {
            match registry.get(&path) {
                Ok(prefab) => {
                    let prefab_clone = prefab.clone();
                    let result = propagate_to_instances(&prefab_clone, objects, instances, &path);
                    results.push(result);
                }
                Err(e) => {
                    let mut result = PropagationResult::new();
                    result.failed_objects.push((0, e.to_string()));
                    results.push(result);
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_result() {
        let mut result = PropagationResult::new();
        assert!(result.is_success());

        result.updated_objects.push(1);
        assert!(result.is_success());

        result.failed_objects.push((2, "Error".to_string()));
        assert!(!result.is_success());
    }

    #[test]
    fn test_propagation_manager() {
        let mut manager = PropagationManager::new();
        assert!(!manager.has_pending());

        manager.mark_pending("test.prefab".into());
        assert!(manager.has_pending());

        let pending = manager.take_pending();
        assert_eq!(pending.len(), 1);
        assert!(!manager.has_pending());
    }
}

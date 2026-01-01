//! # Prefab Resolver
//!
//! Resolves nested prefabs and applies overrides.

use std::collections::HashSet;

use glam::Vec3;

use super::instance::{InstanceIdGenerator, PrefabInstance, PrefabInstanceGroup, PrefabRef};
use super::overrides::PropertyOverrides;
use super::registry::PrefabRegistry;
use super::{Prefab, PrefabError, PrefabObject};
use crate::editor::selection::SceneObject;

/// Result of resolving a prefab
#[derive(Debug)]
pub struct ResolvedPrefab {
    /// Resolved objects ready to be added to scene
    pub objects: Vec<SceneObject>,
    /// Instance group for tracking
    pub instance_group: PrefabInstanceGroup,
    /// Prefab instances for each object
    pub instances: Vec<PrefabInstance>,
}

/// Prefab resolver handles nested prefabs and instantiation
pub struct PrefabResolver<'a> {
    /// Registry for loading prefabs
    registry: &'a mut PrefabRegistry,
    /// Stack for detecting circular references
    resolution_stack: HashSet<String>,
    /// Instance ID generator
    id_generator: &'a mut InstanceIdGenerator,
    /// Maximum nesting depth
    max_depth: usize,
}

impl<'a> PrefabResolver<'a> {
    /// Create a new resolver
    pub fn new(
        registry: &'a mut PrefabRegistry,
        id_generator: &'a mut InstanceIdGenerator,
    ) -> Self {
        Self {
            registry,
            resolution_stack: HashSet::new(),
            id_generator,
            max_depth: 32,
        }
    }

    /// Set maximum nesting depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Resolve and instantiate a prefab at a position
    pub fn resolve(
        &mut self,
        prefab_ref: &PrefabRef,
        position: Vec3,
        next_object_id: &mut u32,
    ) -> Result<ResolvedPrefab, PrefabError> {
        self.resolve_recursive(prefab_ref, position, next_object_id, 0)
    }

    /// Internal recursive resolution
    fn resolve_recursive(
        &mut self,
        prefab_ref: &PrefabRef,
        position: Vec3,
        next_object_id: &mut u32,
        depth: usize,
    ) -> Result<ResolvedPrefab, PrefabError> {
        // Check depth limit
        if depth >= self.max_depth {
            return Err(PrefabError::Deserialize(format!(
                "Maximum prefab nesting depth ({}) exceeded",
                self.max_depth
            )));
        }

        // Check for circular references
        let path_str = prefab_ref.path.to_string_lossy().to_string();
        if self.resolution_stack.contains(&path_str) {
            return Err(PrefabError::Deserialize(format!(
                "Circular prefab reference detected: {}",
                path_str
            )));
        }

        self.resolution_stack.insert(path_str.clone());

        // Load the prefab
        let prefab = self.registry.get_cloned(&prefab_ref.path)?;

        // Generate instance ID
        let instance_id = self.id_generator.next_id();

        // Create instance group
        let mut instance_group = PrefabInstanceGroup::new(instance_id, prefab_ref.clone());

        // Create objects and instances
        let mut objects = Vec::new();
        let mut instances = Vec::new();
        let start_id = *next_object_id;

        // First pass: create all objects
        for (idx, pobj) in prefab.objects.iter().enumerate() {
            let obj = self.create_object_from_prefab(pobj, position, next_object_id, idx == 0);

            let instance = PrefabInstance::new(prefab_ref.clone(), idx, instance_id);

            instance_group.add_object(obj.id);
            objects.push(obj);
            instances.push(instance);
        }

        // Second pass: rebuild hierarchy
        self.rebuild_hierarchy(&mut objects, &prefab.objects, start_id);

        // Remove from stack
        self.resolution_stack.remove(&path_str);

        Ok(ResolvedPrefab {
            objects,
            instance_group,
            instances,
        })
    }

    /// Create a scene object from a prefab object
    fn create_object_from_prefab(
        &self,
        pobj: &PrefabObject,
        spawn_position: Vec3,
        next_id: &mut u32,
        is_root: bool,
    ) -> SceneObject {
        let id = *next_id;
        *next_id += 1;

        let mut obj = SceneObject::new(id, pobj.name.clone());

        // Position: root objects get offset by spawn position
        if is_root && pobj.parent_index.is_none() {
            obj.position = spawn_position + pobj.position_vec();
        } else {
            obj.position = pobj.position_vec();
        }

        obj.rotation = pobj.rotation_vec();
        obj.scale = pobj.scale_vec();
        obj.color = pobj.color;
        obj.visible = pobj.visible;
        obj.camera = pobj.camera.clone();

        obj
    }

    /// Rebuild hierarchy relationships
    fn rebuild_hierarchy(
        &self,
        objects: &mut [SceneObject],
        prefab_objects: &[PrefabObject],
        start_id: u32,
    ) {
        for (idx, pobj) in prefab_objects.iter().enumerate() {
            if let Some(parent_idx) = pobj.parent_index {
                let child_id = start_id + idx as u32;
                let parent_id = start_id + parent_idx as u32;

                // Set parent on child
                if let Some(child) = objects.iter_mut().find(|o| o.id == child_id) {
                    child.hierarchy.parent = Some(parent_id);
                }

                // Add child to parent's children list
                if let Some(parent) = objects.iter_mut().find(|o| o.id == parent_id) {
                    parent.hierarchy.add_child(child_id);
                }
            }
        }
    }
}

/// Apply overrides to a scene object
pub fn apply_overrides(
    object: &mut SceneObject,
    prefab_object: &PrefabObject,
    overrides: &PropertyOverrides,
) {
    use super::overrides::paths;

    // Only apply prefab values for non-overridden properties

    if !overrides.is_effectively_overridden(&paths::name()) {
        object.name = prefab_object.name.clone();
    }

    if !overrides.is_effectively_overridden(&paths::position()) {
        object.position = prefab_object.position_vec();
    }

    if !overrides.is_effectively_overridden(&paths::rotation()) {
        object.rotation = prefab_object.rotation_vec();
    }

    if !overrides.is_effectively_overridden(&paths::scale()) {
        object.scale = prefab_object.scale_vec();
    }

    if !overrides.is_effectively_overridden(&paths::color()) {
        object.color = prefab_object.color;
    }

    if !overrides.is_effectively_overridden(&paths::visible()) {
        object.visible = prefab_object.visible;
    }

    if !overrides.is_effectively_overridden(&paths::camera()) {
        object.camera = prefab_object.camera.clone();
    }
}

/// Revert an object to its prefab values (remove overrides)
pub fn revert_to_prefab(
    object: &mut SceneObject,
    prefab: &Prefab,
    instance: &PrefabInstance,
) -> Result<(), PrefabError> {
    let prefab_object = prefab
        .objects
        .get(instance.object_index)
        .ok_or_else(|| PrefabError::Deserialize("Invalid prefab object index".to_string()))?;

    // Apply all prefab values
    object.name = prefab_object.name.clone();
    object.position = prefab_object.position_vec();
    object.rotation = prefab_object.rotation_vec();
    object.scale = prefab_object.scale_vec();
    object.color = prefab_object.color;
    object.visible = prefab_object.visible;
    object.camera = prefab_object.camera.clone();

    Ok(())
}

/// Check what properties differ between object and prefab
pub fn detect_changes(object: &SceneObject, prefab_object: &PrefabObject) -> PropertyOverrides {
    use super::overrides::paths;

    let mut overrides = PropertyOverrides::new();

    if object.name != prefab_object.name {
        overrides.set_override(paths::name());
    }

    if object.position != prefab_object.position_vec() {
        overrides.set_override(paths::position());
    }

    if object.rotation != prefab_object.rotation_vec() {
        overrides.set_override(paths::rotation());
    }

    if object.scale != prefab_object.scale_vec() {
        overrides.set_override(paths::scale());
    }

    if object.color != prefab_object.color {
        overrides.set_override(paths::color());
    }

    if object.visible != prefab_object.visible {
        overrides.set_override(paths::visible());
    }

    if object.camera != prefab_object.camera {
        overrides.set_override(paths::camera());
    }

    overrides
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_changes() {
        let mut obj = SceneObject::new(1, "Test");
        obj.position = Vec3::new(1.0, 2.0, 3.0);

        let pobj = PrefabObject {
            name: "Test".to_string(),
            local_position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: obj.color,
            visible: true,
            camera: None,
            parent_index: None,
            scripts: Vec::new(),
            nested_prefab: None,
        };

        let overrides = detect_changes(&obj, &pobj);
        assert!(overrides.is_overridden(&super::super::overrides::paths::position()));
        assert!(!overrides.is_overridden(&super::super::overrides::paths::rotation()));
    }
}

//! Hierarchy system for parent-child relationships.
//!
//! Provides ECS-like components for object hierarchy with transform propagation.

use glam::{Mat4, Quat, Vec3, EulerRot};
use serde::{Serialize, Deserialize};

/// Object ID type alias
pub type ObjectId = u32;

/// Hierarchy component - tracks parent/child relationships
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Hierarchy {
    /// Parent object ID (None = root object)
    pub parent: Option<ObjectId>,
    /// Child object IDs
    #[serde(default)]
    pub children: Vec<ObjectId>,
}

impl Hierarchy {
    /// Create a new root hierarchy (no parent)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create hierarchy with parent
    pub fn with_parent(parent: ObjectId) -> Self {
        Self {
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Check if this is a root object (no parent)
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Check if this object has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Add a child to this object
    pub fn add_child(&mut self, child_id: ObjectId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Remove a child from this object
    pub fn remove_child(&mut self, child_id: ObjectId) {
        self.children.retain(|&id| id != child_id);
    }
}

/// Helper functions for hierarchy operations on object collections
pub mod helpers {
    use super::*;

    /// Trait for objects that have hierarchy
    pub trait HasHierarchy {
        fn id(&self) -> ObjectId;
        fn hierarchy(&self) -> &Hierarchy;
        fn hierarchy_mut(&mut self) -> &mut Hierarchy;
        fn position(&self) -> Vec3;
        fn rotation(&self) -> Vec3;
        fn scale(&self) -> Vec3;
    }

    /// Get all root objects (objects without a parent)
    pub fn get_root_objects<T: HasHierarchy>(objects: &[T]) -> Vec<ObjectId> {
        objects.iter()
            .filter(|obj| obj.hierarchy().is_root())
            .map(|obj| obj.id())
            .collect()
    }

    /// Get all descendants of an object (children, grandchildren, etc.)
    pub fn get_descendants<T: HasHierarchy>(objects: &[T], parent_id: ObjectId) -> Vec<ObjectId> {
        let mut result = Vec::new();
        collect_descendants(objects, parent_id, &mut result);
        result
    }

    fn collect_descendants<T: HasHierarchy>(objects: &[T], parent_id: ObjectId, result: &mut Vec<ObjectId>) {
        if let Some(parent) = objects.iter().find(|o| o.id() == parent_id) {
            for &child_id in &parent.hierarchy().children {
                result.push(child_id);
                collect_descendants(objects, child_id, result);
            }
        }
    }

    /// Check if `potential_ancestor` is an ancestor of `object_id`
    pub fn is_ancestor_of<T: HasHierarchy>(
        objects: &[T],
        potential_ancestor: ObjectId,
        object_id: ObjectId,
    ) -> bool {
        let mut current = object_id;
        while let Some(obj) = objects.iter().find(|o| o.id() == current) {
            if let Some(parent_id) = obj.hierarchy().parent {
                if parent_id == potential_ancestor {
                    return true;
                }
                current = parent_id;
            } else {
                break;
            }
        }
        false
    }

    /// Check if `potential_descendant` is a descendant of `object_id`
    pub fn is_descendant_of<T: HasHierarchy>(
        objects: &[T],
        potential_descendant: ObjectId,
        object_id: ObjectId,
    ) -> bool {
        is_ancestor_of(objects, object_id, potential_descendant)
    }

    /// Validate if reparenting would create a cycle
    pub fn can_reparent<T: HasHierarchy>(
        objects: &[T],
        child_id: ObjectId,
        new_parent_id: Option<ObjectId>,
    ) -> bool {
        let Some(new_parent_id) = new_parent_id else {
            // Moving to root is always valid
            return true;
        };

        // Cannot be parent of self
        if child_id == new_parent_id {
            return false;
        }

        // Cannot make a descendant the parent (would create cycle)
        !is_descendant_of(objects, new_parent_id, child_id)
    }

    /// Compute the world transform matrix for an object
    pub fn compute_world_matrix<T: HasHierarchy>(objects: &[T], object_id: ObjectId) -> Mat4 {
        let Some(obj) = objects.iter().find(|o| o.id() == object_id) else {
            return Mat4::IDENTITY;
        };

        let local = Mat4::from_scale_rotation_translation(
            obj.scale(),
            Quat::from_euler(
                EulerRot::XYZ,
                obj.rotation().x,
                obj.rotation().y,
                obj.rotation().z,
            ),
            obj.position(),
        );

        if let Some(parent_id) = obj.hierarchy().parent {
            let parent_world = compute_world_matrix(objects, parent_id);
            parent_world * local
        } else {
            local
        }
    }

    /// Get the depth of an object in the hierarchy (0 = root)
    pub fn get_depth<T: HasHierarchy>(objects: &[T], object_id: ObjectId) -> usize {
        let mut depth = 0;
        let mut current = object_id;

        while let Some(obj) = objects.iter().find(|o| o.id() == current) {
            if let Some(parent_id) = obj.hierarchy().parent {
                depth += 1;
                current = parent_id;
            } else {
                break;
            }
        }

        depth
    }

    /// Perform reparenting operation (updates both old parent and new parent)
    pub fn reparent<T: HasHierarchy>(
        objects: &mut [T],
        child_id: ObjectId,
        new_parent_id: Option<ObjectId>,
    ) -> bool {
        // Validate first
        if !can_reparent(objects, child_id, new_parent_id) {
            return false;
        }

        // Get old parent id
        let old_parent_id = objects.iter()
            .find(|o| o.id() == child_id)
            .and_then(|o| o.hierarchy().parent);

        // Remove from old parent's children list
        if let Some(old_parent_id) = old_parent_id {
            if let Some(old_parent) = objects.iter_mut().find(|o| o.id() == old_parent_id) {
                old_parent.hierarchy_mut().remove_child(child_id);
            }
        }

        // Add to new parent's children list
        if let Some(new_parent_id) = new_parent_id {
            if let Some(new_parent) = objects.iter_mut().find(|o| o.id() == new_parent_id) {
                new_parent.hierarchy_mut().add_child(child_id);
            }
        }

        // Update child's parent reference
        if let Some(child) = objects.iter_mut().find(|o| o.id() == child_id) {
            child.hierarchy_mut().parent = new_parent_id;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_default() {
        let h = Hierarchy::new();
        assert!(h.is_root());
        assert!(!h.has_children());
    }

    #[test]
    fn test_hierarchy_with_parent() {
        let h = Hierarchy::with_parent(42);
        assert!(!h.is_root());
        assert_eq!(h.parent, Some(42));
    }

    #[test]
    fn test_add_remove_child() {
        let mut h = Hierarchy::new();
        h.add_child(1);
        h.add_child(2);
        assert!(h.has_children());
        assert_eq!(h.children.len(), 2);

        h.remove_child(1);
        assert_eq!(h.children.len(), 1);
        assert_eq!(h.children[0], 2);
    }
}

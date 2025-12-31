//! # Component Storage
//!
//! Components are pure data attached to entities. This module provides
//! efficient storage using SparseSet data structure.
//!
//! ## SparseSet
//!
//! A SparseSet provides O(1) insertion, removal, and lookup while keeping
//! component data contiguous in memory for cache-efficient iteration.
//!
//! ```text
//! Sparse: [_, 0, _, _, 1, _, 2]  // Maps entity index -> dense index
//! Dense:  [1, 4, 6]              // Entity indices in order
//! Data:   [C1, C4, C6]           // Component data, contiguous
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::entity::Entity;

/// Marker trait for components.
///
/// Components should be simple data structs with no behavior.
/// Use `#[derive(Component)]` macro (when available) or implement manually.
pub trait Component: 'static + Send + Sync {
    /// Type name for debugging
    fn type_name() -> &'static str
    where
        Self: Sized,
    {
        std::any::type_name::<Self>()
    }
}

// Implement Component for common types
impl Component for f32 {}
impl Component for f64 {}
impl Component for i32 {}
impl Component for u32 {}
impl Component for bool {}
impl Component for String {}

/// Type-erased component storage trait.
///
/// Allows storing different component types in a single collection.
pub trait ComponentStorage: Any + Send + Sync {
    /// Get the TypeId of the stored component type
    fn type_id(&self) -> TypeId;

    /// Remove component for an entity (type-erased)
    fn remove(&mut self, entity: Entity) -> bool;

    /// Check if entity has this component
    fn contains(&self, entity: Entity) -> bool;

    /// Number of components stored
    fn len(&self) -> usize;

    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all components
    fn clear(&mut self);

    /// As Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// As Any mut for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Sparse set storage for a specific component type.
///
/// Provides O(1) operations and cache-friendly iteration.
pub struct SparseSet<T: Component> {
    /// Sparse array: maps entity index to dense index
    /// None means entity doesn't have this component
    sparse: Vec<Option<usize>>,

    /// Dense array: entity indices in insertion order
    dense: Vec<u32>,

    /// Component data, parallel to dense array
    data: Vec<T>,
}

impl<T: Component> SparseSet<T> {
    /// Create a new empty sparse set
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Create with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sparse: Vec::with_capacity(capacity),
            dense: Vec::with_capacity(capacity),
            data: Vec::with_capacity(capacity),
        }
    }

    /// Insert a component for an entity
    ///
    /// Returns the old component if entity already had one.
    pub fn insert(&mut self, entity: Entity, component: T) -> Option<T> {
        let index = entity.index() as usize;

        // Ensure sparse array is large enough
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, None);
        }

        if let Some(dense_idx) = self.sparse[index] {
            // Entity already has component, replace it
            let old = std::mem::replace(&mut self.data[dense_idx], component);
            Some(old)
        } else {
            // New component
            let dense_idx = self.dense.len();
            self.sparse[index] = Some(dense_idx);
            self.dense.push(entity.index());
            self.data.push(component);
            None
        }
    }

    /// Remove component for an entity
    ///
    /// Returns the removed component if it existed.
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let index = entity.index() as usize;

        if index >= self.sparse.len() {
            return None;
        }

        if let Some(dense_idx) = self.sparse[index].take() {
            // Swap-remove to maintain contiguous data
            let last_entity_idx = *self.dense.last().unwrap();
            self.dense.swap_remove(dense_idx);
            let component = self.data.swap_remove(dense_idx);

            // Update sparse entry for the swapped element
            if dense_idx < self.dense.len() {
                self.sparse[last_entity_idx as usize] = Some(dense_idx);
            }

            Some(component)
        } else {
            None
        }
    }

    /// Get component for an entity
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let index = entity.index() as usize;
        self.sparse
            .get(index)
            .and_then(|opt| *opt)
            .map(|dense_idx| &self.data[dense_idx])
    }

    /// Get mutable component for an entity
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let index = entity.index() as usize;
        if let Some(Some(dense_idx)) = self.sparse.get(index) {
            Some(&mut self.data[*dense_idx])
        } else {
            None
        }
    }

    /// Check if entity has this component
    pub fn contains(&self, entity: Entity) -> bool {
        let index = entity.index() as usize;
        self.sparse
            .get(index)
            .map(|opt| opt.is_some())
            .unwrap_or(false)
    }

    /// Number of components stored
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all components
    pub fn clear(&mut self) {
        self.sparse.clear();
        self.dense.clear();
        self.data.clear();
    }

    /// Iterate over all (entity_index, component) pairs
    pub fn iter(&self) -> impl Iterator<Item = (u32, &T)> {
        self.dense.iter().copied().zip(self.data.iter())
    }

    /// Iterate mutably over all (entity_index, component) pairs
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> {
        self.dense.iter().copied().zip(self.data.iter_mut())
    }

    /// Get raw slice of component data (for cache-efficient iteration)
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Get mutable slice of component data
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Get the entity indices in storage order
    pub fn entities(&self) -> &[u32] {
        &self.dense
    }
}

impl<T: Component> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Component> ComponentStorage for SparseSet<T> {
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn remove(&mut self, entity: Entity) -> bool {
        SparseSet::remove(self, entity).is_some()
    }

    fn contains(&self, entity: Entity) -> bool {
        SparseSet::contains(self, entity)
    }

    fn len(&self) -> usize {
        SparseSet::len(self)
    }

    fn clear(&mut self) {
        SparseSet::clear(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Registry of component storages by type.
///
/// Manages all component storages for a World.
pub struct ComponentRegistry {
    /// Map from TypeId to component storage
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
        }
    }

    /// Get or create storage for a component type
    pub fn get_or_create<T: Component>(&mut self) -> &mut SparseSet<T> {
        let type_id = TypeId::of::<T>();

        self.storages
            .entry(type_id)
            .or_insert_with(|| Box::new(SparseSet::<T>::new()))
            .as_any_mut()
            .downcast_mut::<SparseSet<T>>()
            .expect("Type mismatch in component registry")
    }

    /// Get storage for a component type (immutable)
    pub fn get<T: Component>(&self) -> Option<&SparseSet<T>> {
        let type_id = TypeId::of::<T>();
        self.storages
            .get(&type_id)
            .and_then(|storage| storage.as_any().downcast_ref())
    }

    /// Get storage for a component type (mutable)
    pub fn get_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>> {
        let type_id = TypeId::of::<T>();
        self.storages
            .get_mut(&type_id)
            .and_then(|storage| storage.as_any_mut().downcast_mut())
    }

    /// Remove all components for an entity
    pub fn remove_all(&mut self, entity: Entity) {
        for storage in self.storages.values_mut() {
            storage.remove(entity);
        }
    }

    /// Clear all component storages
    pub fn clear(&mut self) {
        for storage in self.storages.values_mut() {
            storage.clear();
        }
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, PartialEq, Clone)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }
    impl Component for Velocity {}

    fn entity(index: u32) -> Entity {
        Entity::new(index, super::super::entity::Generation::new())
    }

    #[test]
    fn test_sparse_set_insert_get() {
        let mut set = SparseSet::<Position>::new();
        let e = entity(5);

        set.insert(e, Position { x: 1.0, y: 2.0 });

        assert!(set.contains(e));
        assert_eq!(set.get(e), Some(&Position { x: 1.0, y: 2.0 }));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_sparse_set_remove() {
        let mut set = SparseSet::<Position>::new();
        let e = entity(0);

        set.insert(e, Position { x: 1.0, y: 2.0 });
        let removed = set.remove(e);

        assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
        assert!(!set.contains(e));
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_sparse_set_swap_remove() {
        let mut set = SparseSet::<Position>::new();
        let e0 = entity(0);
        let e1 = entity(1);
        let e2 = entity(2);

        set.insert(e0, Position { x: 0.0, y: 0.0 });
        set.insert(e1, Position { x: 1.0, y: 1.0 });
        set.insert(e2, Position { x: 2.0, y: 2.0 });

        // Remove middle element
        set.remove(e1);

        // e2's component should still be accessible
        assert_eq!(set.get(e2), Some(&Position { x: 2.0, y: 2.0 }));
        assert!(!set.contains(e1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_sparse_set_iteration() {
        let mut set = SparseSet::<Position>::new();
        set.insert(entity(0), Position { x: 0.0, y: 0.0 });
        set.insert(entity(5), Position { x: 5.0, y: 5.0 });

        let items: Vec<_> = set.iter().collect();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_component_registry() {
        let mut registry = ComponentRegistry::new();

        let pos_storage = registry.get_or_create::<Position>();
        pos_storage.insert(entity(0), Position { x: 1.0, y: 2.0 });

        let vel_storage = registry.get_or_create::<Velocity>();
        vel_storage.insert(entity(0), Velocity { dx: 0.5, dy: 0.5 });

        assert_eq!(
            registry.get::<Position>().unwrap().get(entity(0)),
            Some(&Position { x: 1.0, y: 2.0 })
        );
    }
}

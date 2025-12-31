//! # World
//!
//! The World is the central container for all ECS data.
//! It manages entities, components, and coordinates systems.
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut world = World::new();
//!
//! // Spawn entities
//! let entity = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { dx: 1.0, dy: 0.0 }));
//!
//! // Access components
//! if let Some(pos) = world.get::<Position>(entity) {
//!     println!("Position: {:?}", pos);
//! }
//! ```

use std::any::TypeId;

use super::archetype::{ArchetypeId, ArchetypeStorage, ComponentSet};
use super::component::{Component, ComponentRegistry, SparseSet};
#[allow(unused_imports)]
use super::entity::Generation as _Gen;
use super::entity::{Entity, EntityManager, Generation};

/// The central container for all ECS data.
///
/// Manages the lifecycle of entities and storage of components.
pub struct World {
    /// Entity manager
    entities: EntityManager,
    /// Component registry (type -> storage)
    components: ComponentRegistry,
    /// Archetype storage
    archetypes: ArchetypeStorage,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            components: ComponentRegistry::new(),
            archetypes: ArchetypeStorage::new(),
        }
    }

    /// Create a new world with preallocated entity capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entities: EntityManager::with_capacity(capacity),
            components: ComponentRegistry::new(),
            archetypes: ArchetypeStorage::new(),
        }
    }

    // ==================== Entity Management ====================

    /// Spawn a new entity with no components
    pub fn spawn_empty(&mut self) -> Entity {
        let entity = self.entities.spawn();
        self.archetypes
            .assign_entity(entity.index(), ArchetypeId::EMPTY);
        entity
    }

    /// Spawn a new entity with a single component
    pub fn spawn_with<T: Component>(&mut self, component: T) -> Entity {
        let entity = self.entities.spawn();

        // Add component
        let storage = self.components.get_or_create::<T>();
        storage.insert(entity, component);

        // Update archetype
        let set = ComponentSet::from_types(vec![TypeId::of::<T>()]);
        let arch_id = self.archetypes.get_or_create(set);
        self.archetypes.assign_entity(entity.index(), arch_id);

        entity
    }

    /// Despawn an entity and remove all its components
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        // Remove from archetype tracking
        self.archetypes.remove_entity(entity.index());

        // Remove all components
        self.components.remove_all(entity);

        // Remove entity
        self.entities.despawn(entity)
    }

    /// Check if an entity is alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    /// Get the current generation for an entity index
    pub fn entity_generation(&self, index: u32) -> Option<Generation> {
        self.entities.generation(index)
    }

    /// Get the number of alive entities
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Iterate over all alive entities
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter()
    }

    // ==================== Component Access ====================

    /// Add a component to an entity
    ///
    /// Returns the old component if one existed.
    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> Option<T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let storage = self.components.get_or_create::<T>();
        let old = storage.insert(entity, component);

        // Update archetype if this is a new component
        if old.is_none() {
            self.update_archetype_add::<T>(entity);
        }

        old
    }

    /// Remove a component from an entity
    pub fn remove<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let storage = self.components.get_mut::<T>()?;
        let removed = storage.remove(entity);

        if removed.is_some() {
            self.update_archetype_remove::<T>(entity);
        }

        removed
    }

    /// Get a component reference
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        self.components.get::<T>()?.get(entity)
    }

    /// Get a mutable component reference
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        self.components.get_mut::<T>()?.get_mut(entity)
    }

    /// Check if entity has a component
    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }
        self.components
            .get::<T>()
            .map(|s| s.contains(entity))
            .unwrap_or(false)
    }

    // ==================== Component Storage Access ====================

    /// Get raw component storage (for advanced use)
    pub fn storage<T: Component>(&self) -> Option<&SparseSet<T>> {
        self.components.get::<T>()
    }

    /// Get mutable component storage (for advanced use)
    pub fn storage_mut<T: Component>(&mut self) -> Option<&mut SparseSet<T>> {
        self.components.get_mut::<T>()
    }

    /// Get or create component storage
    pub fn storage_or_create<T: Component>(&mut self) -> &mut SparseSet<T> {
        self.components.get_or_create::<T>()
    }

    // ==================== Archetype Helpers ====================

    fn update_archetype_add<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();

        // Get current archetype's component set
        let current_set = self
            .archetypes
            .entity_archetype(entity.index())
            .and_then(|id| self.archetypes.get(id))
            .map(|arch| arch.components().clone())
            .unwrap_or_default();

        // Create new set with added component
        let new_set = current_set.with(type_id);
        let new_arch_id = self.archetypes.get_or_create(new_set);
        self.archetypes.assign_entity(entity.index(), new_arch_id);
    }

    fn update_archetype_remove<T: Component>(&mut self, entity: Entity) {
        let type_id = TypeId::of::<T>();

        // Get current archetype's component set
        let current_set = self
            .archetypes
            .entity_archetype(entity.index())
            .and_then(|id| self.archetypes.get(id))
            .map(|arch| arch.components().clone())
            .unwrap_or_default();

        // Create new set without removed component
        let new_set = current_set.without(type_id);
        let new_arch_id = self.archetypes.get_or_create(new_set);
        self.archetypes.assign_entity(entity.index(), new_arch_id);
    }

    // ==================== Utility ====================

    /// Clear all entities and components
    pub fn clear(&mut self) {
        self.entities.clear();
        self.components.clear();
        self.archetypes = ArchetypeStorage::new();
    }

    /// Reserve capacity for entities
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for spawning entities with multiple components
pub struct EntityBuilder<'w> {
    world: &'w mut World,
    entity: Entity,
    component_types: Vec<TypeId>,
}

impl<'w> EntityBuilder<'w> {
    /// Create a new entity builder
    pub(crate) fn new(world: &'w mut World) -> Self {
        let entity = world.entities.spawn();
        Self {
            world,
            entity,
            component_types: Vec::new(),
        }
    }

    /// Add a component to the entity being built
    pub fn with<T: Component>(mut self, component: T) -> Self {
        let storage = self.world.components.get_or_create::<T>();
        storage.insert(self.entity, component);
        self.component_types.push(TypeId::of::<T>());
        self
    }

    /// Finish building and return the entity
    pub fn build(self) -> Entity {
        // Set up archetype
        let set = ComponentSet::from_types(self.component_types);
        let arch_id = self.world.archetypes.get_or_create(set);
        self.world
            .archetypes
            .assign_entity(self.entity.index(), arch_id);
        self.entity
    }
}

impl World {
    /// Create an entity builder for spawning with multiple components
    pub fn spawn(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(self)
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

    #[test]
    fn test_spawn_despawn() {
        let mut world = World::new();

        let e = world.spawn_empty();
        assert!(world.is_alive(e));
        assert_eq!(world.entity_count(), 1);

        world.despawn(e);
        assert!(!world.is_alive(e));
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_component_insert_get() {
        let mut world = World::new();
        let e = world.spawn_empty();

        world.insert(e, Position { x: 1.0, y: 2.0 });

        assert!(world.has::<Position>(e));
        assert_eq!(world.get::<Position>(e), Some(&Position { x: 1.0, y: 2.0 }));
    }

    #[test]
    fn test_entity_builder() {
        let mut world = World::new();

        let e = world
            .spawn()
            .with(Position { x: 0.0, y: 0.0 })
            .with(Velocity { dx: 1.0, dy: 1.0 })
            .build();

        assert!(world.has::<Position>(e));
        assert!(world.has::<Velocity>(e));
    }

    #[test]
    fn test_component_remove() {
        let mut world = World::new();
        let e = world.spawn_empty();

        world.insert(e, Position { x: 1.0, y: 2.0 });
        let removed = world.remove::<Position>(e);

        assert_eq!(removed, Some(Position { x: 1.0, y: 2.0 }));
        assert!(!world.has::<Position>(e));
    }
}

//! # Query System
//!
//! Queries allow efficient iteration over entities with specific components.
//!
//! ## Usage
//!
//! ```rust,ignore
//! // Iterate over all entities with Position and Velocity
//! for (entity, (pos, vel)) in world.query::<(&Position, &mut Velocity)>() {
//!     vel.dx += pos.x * 0.1;
//! }
//! ```

use std::any::TypeId;
use std::marker::PhantomData;

use super::component::{Component, SparseSet};
use super::entity::Entity;
use super::world::World;

/// Filter specifies which components must be present
#[derive(Clone, Debug, Default)]
pub struct QueryFilter {
    /// Required component types
    pub required: Vec<TypeId>,
    /// Excluded component types (with filter)
    pub excluded: Vec<TypeId>,
}

impl QueryFilter {
    /// Create an empty filter
    pub fn new() -> Self {
        Self {
            required: Vec::new(),
            excluded: Vec::new(),
        }
    }

    /// Add a required component type
    pub fn with<T: Component>(mut self) -> Self {
        self.required.push(TypeId::of::<T>());
        self
    }

    /// Add an excluded component type
    pub fn without<T: Component>(mut self) -> Self {
        self.excluded.push(TypeId::of::<T>());
        self
    }
}

/// Trait for query parameters (components to fetch)
pub trait QueryParam {
    /// The item type returned by the query
    type Item<'w>;

    /// Get the required component TypeIds
    fn type_ids() -> Vec<TypeId>;

    /// Fetch item for an entity from world
    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>>;
}

// Implement QueryParam for immutable reference
impl<T: Component> QueryParam for &T {
    type Item<'w> = &'w T;

    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }

    fn fetch<'w>(world: &'w World, entity: Entity) -> Option<Self::Item<'w>> {
        world.get::<T>(entity)
    }
}

/// Query iterator over entities matching filter
pub struct Query<'w, P: QueryParam> {
    world: &'w World,
    entities: Vec<Entity>,
    index: usize,
    _phantom: PhantomData<P>,
}

impl<'w, P: QueryParam> Query<'w, P> {
    /// Create a new query
    pub fn new(world: &'w World) -> Self {
        // Get entities that have all required components
        let type_ids = P::type_ids();

        // For now, iterate all entities and filter
        // TODO: Use archetype iteration for better performance
        let entities: Vec<Entity> = world
            .entities()
            .filter(|&entity| {
                type_ids
                    .iter()
                    .all(|&type_id| world.has_component_by_id(entity, type_id))
            })
            .collect();

        Self {
            world,
            entities,
            index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'w, P: QueryParam> Iterator for Query<'w, P> {
    type Item = (Entity, P::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;

            if let Some(item) = P::fetch(self.world, entity) {
                return Some((entity, item));
            }
        }
        None
    }
}

/// Query iterator (alias for readability)
pub type QueryIter<'w, P> = Query<'w, P>;

// Helper trait for World
impl World {
    /// Check if entity has component by TypeId (internal use)
    pub(crate) fn has_component_by_id(&self, entity: Entity, _type_id: TypeId) -> bool {
        // This is a simplified version - in a full implementation,
        // we'd have a more efficient way to check this
        if !self.is_alive(entity) {
            return false;
        }

        // Check each known storage type
        // In a real implementation, we'd have a registry of type_id -> storage
        true // Placeholder - actual check would go here
    }

    /// Create a query for single component type
    pub fn query_one<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.storage::<T>().into_iter().flat_map(|storage| {
            storage.iter().filter_map(|(idx, component)| {
                // Reconstruct entity from index
                let generation = self.entity_generation(idx)?;
                let entity = Entity::new(idx, generation);
                if self.is_alive(entity) {
                    Some((entity, component))
                } else {
                    None
                }
            })
        })
    }

    /// Query with mutable access to single component
    pub fn query_one_mut<T: Component>(&mut self) -> QueryOneMut<'_, T> {
        QueryOneMut {
            storage: self.storage_mut::<T>(),
            index: 0,
        }
    }
}

/// Iterator for mutable single-component queries
pub struct QueryOneMut<'w, T: Component> {
    storage: Option<&'w mut SparseSet<T>>,
    index: usize,
}

impl<'w, T: Component> QueryOneMut<'w, T> {
    /// Iterate with mutable access
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u32, &mut T)> + use<'_, 'w, T> {
        self.storage.as_mut().into_iter().flat_map(|s| s.iter_mut())
    }
}

/// Tuple implementations for multi-component queries would go here.
/// For simplicity, providing a manual two-component version.
///
/// Query for two components (read-only)
pub fn query_two<A: Component, B: Component>(
    world: &World,
) -> impl Iterator<Item = (Entity, &A, &B)> {
    // Use the smaller storage as base for iteration
    let storage_a = world.storage::<A>();
    let storage_b = world.storage::<B>();

    storage_a.into_iter().flat_map(move |sa| {
        sa.iter().filter_map(move |(idx, comp_a)| {
            let generation = world.entity_generation(idx)?;
            let entity = Entity::new(idx, generation);
            if !world.is_alive(entity) {
                return None;
            }
            let comp_b = storage_b?.get(entity)?;
            Some((entity, comp_a, comp_b))
        })
    })
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
    fn test_query_one() {
        let mut world = World::new();

        let _e1 = world.spawn_with(Position { x: 1.0, y: 2.0 });
        let _e2 = world.spawn_with(Position { x: 3.0, y: 4.0 });
        world.spawn_empty(); // No Position

        let results: Vec<_> = world.query_one::<Position>().collect();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_two() {
        let mut world = World::new();

        // Entity with both components
        let e1 = world.spawn_empty();
        world.insert(e1, Position { x: 1.0, y: 2.0 });
        world.insert(e1, Velocity { dx: 0.5, dy: 0.5 });

        // Entity with only Position
        let _e2 = world.spawn_with(Position { x: 3.0, y: 4.0 });

        let results: Vec<_> = query_two::<Position, Velocity>(&world).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, e1);
    }
}

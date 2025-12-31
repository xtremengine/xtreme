//! # Archetype System
//!
//! Archetypes group entities that share the same set of components.
//! This enables more efficient iteration and memory layout.
//!
//! ## Concept
//!
//! When you spawn an entity with components (A, B, C), it belongs to
//! archetype {A, B, C}. All entities with exactly those components
//! are stored together.
//!
//! ```text
//! Archetype {Position, Velocity}:
//!   Entities: [e1, e2, e3]
//!   Position: [p1, p2, p3]  // Contiguous
//!   Velocity: [v1, v2, v3]  // Contiguous
//! ```

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// Unique identifier for an archetype.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeId(pub(crate) u32);

impl ArchetypeId {
    /// The empty archetype (no components)
    pub const EMPTY: Self = Self(0);

    /// Create a new archetype ID
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn index(&self) -> u32 {
        self.0
    }
}

/// Represents a unique combination of component types.
///
/// Used as a key to identify archetypes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ComponentSet {
    /// Sorted list of component TypeIds
    types: Vec<TypeId>,
}

impl ComponentSet {
    /// Create an empty component set
    pub fn new() -> Self {
        Self { types: Vec::new() }
    }

    /// Create from a list of TypeIds
    pub fn from_types(mut types: Vec<TypeId>) -> Self {
        types.sort();
        types.dedup();
        Self { types }
    }

    /// Add a component type to the set
    pub fn insert(&mut self, type_id: TypeId) {
        if let Err(pos) = self.types.binary_search(&type_id) {
            self.types.insert(pos, type_id);
        }
    }

    /// Remove a component type from the set
    pub fn remove(&mut self, type_id: TypeId) {
        if let Ok(pos) = self.types.binary_search(&type_id) {
            self.types.remove(pos);
        }
    }

    /// Check if set contains a component type
    pub fn contains(&self, type_id: TypeId) -> bool {
        self.types.binary_search(&type_id).is_ok()
    }

    /// Check if this set contains all types from another set
    pub fn contains_all(&self, other: &ComponentSet) -> bool {
        other.types.iter().all(|t| self.contains(*t))
    }

    /// Get the component types
    pub fn types(&self) -> &[TypeId] {
        &self.types
    }

    /// Number of component types
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Create a new set with an additional component
    pub fn with(&self, type_id: TypeId) -> Self {
        let mut new = self.clone();
        new.insert(type_id);
        new
    }

    /// Create a new set without a component
    pub fn without(&self, type_id: TypeId) -> Self {
        let mut new = self.clone();
        new.remove(type_id);
        new
    }
}

impl Default for ComponentSet {
    fn default() -> Self {
        Self::new()
    }
}

/// An archetype stores entities with identical component sets.
///
/// For now, this is a lightweight struct that just tracks membership.
/// The actual component data is stored in SparseSet per-component.
#[derive(Debug)]
pub struct Archetype {
    /// Unique ID for this archetype
    id: ArchetypeId,
    /// The set of component types this archetype has
    components: ComponentSet,
    /// Entity indices in this archetype
    entities: HashSet<u32>,
}

impl Archetype {
    /// Create a new archetype
    pub(crate) fn new(id: ArchetypeId, components: ComponentSet) -> Self {
        Self {
            id,
            components,
            entities: HashSet::new(),
        }
    }

    /// Get the archetype ID
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Get the component set
    pub fn components(&self) -> &ComponentSet {
        &self.components
    }

    /// Add an entity to this archetype
    pub(crate) fn add_entity(&mut self, entity_index: u32) {
        self.entities.insert(entity_index);
    }

    /// Remove an entity from this archetype
    pub(crate) fn remove_entity(&mut self, entity_index: u32) -> bool {
        self.entities.remove(&entity_index)
    }

    /// Check if archetype contains entity
    pub fn contains(&self, entity_index: u32) -> bool {
        self.entities.contains(&entity_index)
    }

    /// Number of entities in this archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Iterate over entity indices
    pub fn entities(&self) -> impl Iterator<Item = u32> + '_ {
        self.entities.iter().copied()
    }

    /// Check if this archetype has a specific component type
    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.components.contains(type_id)
    }
}

/// Manages all archetypes and entity-to-archetype mapping.
pub struct ArchetypeStorage {
    /// All archetypes by ID
    archetypes: Vec<Archetype>,
    /// Map from component set to archetype ID
    lookup: HashMap<ComponentSet, ArchetypeId>,
    /// Map from entity index to archetype ID
    entity_archetype: HashMap<u32, ArchetypeId>,
}

impl ArchetypeStorage {
    /// Create new archetype storage
    pub fn new() -> Self {
        let mut storage = Self {
            archetypes: Vec::new(),
            lookup: HashMap::new(),
            entity_archetype: HashMap::new(),
        };

        // Create the empty archetype (ID 0)
        storage.get_or_create(ComponentSet::new());
        storage
    }

    /// Get or create an archetype for a component set
    pub fn get_or_create(&mut self, components: ComponentSet) -> ArchetypeId {
        if let Some(&id) = self.lookup.get(&components) {
            return id;
        }

        let id = ArchetypeId::new(self.archetypes.len() as u32);
        let archetype = Archetype::new(id, components.clone());
        self.archetypes.push(archetype);
        self.lookup.insert(components, id);
        id
    }

    /// Get archetype by ID
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(id.index() as usize)
    }

    /// Get mutable archetype by ID
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id.index() as usize)
    }

    /// Get archetype for component set (if exists)
    pub fn find(&self, components: &ComponentSet) -> Option<ArchetypeId> {
        self.lookup.get(components).copied()
    }

    /// Get archetype ID for an entity
    pub fn entity_archetype(&self, entity_index: u32) -> Option<ArchetypeId> {
        self.entity_archetype.get(&entity_index).copied()
    }

    /// Assign entity to archetype
    pub fn assign_entity(&mut self, entity_index: u32, archetype_id: ArchetypeId) {
        // Remove from old archetype if any
        if let Some(old_id) = self.entity_archetype.get(&entity_index).copied() {
            if let Some(archetype) = self.archetypes.get_mut(old_id.index() as usize) {
                archetype.remove_entity(entity_index);
            }
        }

        // Add to new archetype
        if let Some(archetype) = self.archetypes.get_mut(archetype_id.index() as usize) {
            archetype.add_entity(entity_index);
        }
        self.entity_archetype.insert(entity_index, archetype_id);
    }

    /// Remove entity from archetype tracking
    pub fn remove_entity(&mut self, entity_index: u32) {
        if let Some(id) = self.entity_archetype.remove(&entity_index) {
            if let Some(archetype) = self.archetypes.get_mut(id.index() as usize) {
                archetype.remove_entity(entity_index);
            }
        }
    }

    /// Iterate all archetypes matching a component filter
    pub fn iter_matching<'a>(
        &'a self,
        required: &'a ComponentSet,
    ) -> impl Iterator<Item = &'a Archetype> {
        self.archetypes
            .iter()
            .filter(move |arch| arch.components().contains_all(required))
    }

    /// Number of archetypes
    pub fn len(&self) -> usize {
        self.archetypes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}

impl Default for ArchetypeStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_set() {
        let mut set = ComponentSet::new();
        let t1 = TypeId::of::<i32>();
        let t2 = TypeId::of::<f32>();

        set.insert(t1);
        set.insert(t2);

        assert!(set.contains(t1));
        assert!(set.contains(t2));
        assert_eq!(set.len(), 2);

        set.remove(t1);
        assert!(!set.contains(t1));
    }

    #[test]
    fn test_archetype_storage() {
        let mut storage = ArchetypeStorage::new();

        let t1 = TypeId::of::<i32>();
        let t2 = TypeId::of::<f32>();

        let set1 = ComponentSet::from_types(vec![t1]);
        let set2 = ComponentSet::from_types(vec![t1, t2]);

        let id1 = storage.get_or_create(set1.clone());
        let id2 = storage.get_or_create(set2.clone());

        assert_ne!(id1, id2);
        assert_eq!(storage.get_or_create(set1), id1); // Same set returns same ID
    }

    #[test]
    fn test_entity_archetype_assignment() {
        let mut storage = ArchetypeStorage::new();
        let t1 = TypeId::of::<i32>();
        let set1 = ComponentSet::from_types(vec![t1]);
        let arch_id = storage.get_or_create(set1);

        storage.assign_entity(42, arch_id);

        assert_eq!(storage.entity_archetype(42), Some(arch_id));
        assert!(storage.get(arch_id).unwrap().contains(42));
    }
}

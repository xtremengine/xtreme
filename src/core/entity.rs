//! # Entity System
//!
//! Entities are unique identifiers in the ECS. They have no data or behavior -
//! they're just IDs that components can be attached to.
//!
//! ## Design
//!
//! An Entity is a 64-bit value split into:
//! - Index (32 bits): Position in the entity array
//! - Generation (32 bits): Version counter for reuse detection
//!
//! When an entity is destroyed, its index is recycled but generation increases,
//! ensuring old references become invalid.

use std::fmt;

/// Generation counter to detect stale entity references.
/// Incremented each time an entity index is reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Generation(pub u32);

impl Generation {
    /// Create a new generation (starts at 0)
    pub fn new() -> Self {
        Self(0)
    }

    /// Increment generation (wraps on overflow)
    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

/// Unique identifier for an entity.
///
/// Composed of an index and generation to handle entity recycling safely.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity {
    /// Index into the entity storage
    index: u32,
    /// Generation for detecting stale references
    generation: Generation,
}

impl Entity {
    /// Create a new entity with given index and generation
    pub(crate) fn new(index: u32, generation: Generation) -> Self {
        Self { index, generation }
    }

    /// Get the entity's index
    #[inline]
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get the entity's generation
    #[inline]
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// Convert to a u64 for serialization (index in low bits, generation in high bits)
    #[inline]
    pub fn to_bits(&self) -> u64 {
        (self.generation.0 as u64) << 32 | (self.index as u64)
    }

    /// Create from a u64
    #[inline]
    pub fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: Generation((bits >> 32) as u32),
        }
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entity({}v{})", self.index, self.generation.0)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.index, self.generation.0)
    }
}

/// Alias for Entity to match common ECS terminology
pub type EntityId = Entity;

/// Entry in the entity manager tracking alive/dead state
#[derive(Clone, Copy, Debug)]
struct EntityEntry {
    /// Current generation of this slot
    generation: Generation,
    /// True if entity is alive, false if slot is free
    alive: bool,
}

impl EntityEntry {
    fn new() -> Self {
        Self {
            generation: Generation::new(),
            alive: false,
        }
    }
}

/// Manages entity lifecycle: creation, destruction, and index recycling.
///
/// Uses a free list to efficiently reuse destroyed entity indices.
pub struct EntityManager {
    /// All entity entries (indexed by entity index)
    entries: Vec<EntityEntry>,
    /// Free list of available indices
    free_indices: Vec<u32>,
    /// Number of currently alive entities
    alive_count: usize,
}

impl EntityManager {
    /// Create a new entity manager
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_indices: Vec::new(),
            alive_count: 0,
        }
    }

    /// Create a new entity manager with preallocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free_indices: Vec::new(),
            alive_count: 0,
        }
    }

    /// Spawn a new entity, reusing indices when possible
    pub fn spawn(&mut self) -> Entity {
        self.alive_count += 1;

        if let Some(index) = self.free_indices.pop() {
            // Reuse a freed index
            let entry = &mut self.entries[index as usize];
            entry.alive = true;
            Entity::new(index, entry.generation)
        } else {
            // Allocate a new index
            let index = self.entries.len() as u32;
            let generation = Generation::new();
            self.entries.push(EntityEntry {
                generation,
                alive: true,
            });
            Entity::new(index, generation)
        }
    }

    /// Destroy an entity, making its index available for reuse
    ///
    /// Returns true if the entity was alive and is now destroyed.
    /// Returns false if the entity was already dead or invalid.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let entry = &mut self.entries[entity.index() as usize];
        entry.alive = false;
        entry.generation = entry.generation.next();
        self.free_indices.push(entity.index());
        self.alive_count -= 1;
        true
    }

    /// Check if an entity is currently alive
    pub fn is_alive(&self, entity: Entity) -> bool {
        let index = entity.index() as usize;
        if index >= self.entries.len() {
            return false;
        }
        let entry = &self.entries[index];
        entry.alive && entry.generation == entity.generation()
    }

    /// Get the current generation for an index (for validation)
    pub fn generation(&self, index: u32) -> Option<Generation> {
        self.entries.get(index as usize).map(|e| e.generation)
    }

    /// Number of currently alive entities
    #[inline]
    pub fn len(&self) -> usize {
        self.alive_count
    }

    /// Check if there are no alive entities
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.alive_count == 0
    }

    /// Total capacity (includes dead slots)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.entries.len()
    }

    /// Iterate over all alive entities
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.alive)
            .map(|(index, entry)| Entity::new(index as u32, entry.generation))
    }

    /// Clear all entities (reset to empty state)
    pub fn clear(&mut self) {
        self.entries.clear();
        self.free_indices.clear();
        self.alive_count = 0;
    }

    /// Reserve capacity for additional entities
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let mut manager = EntityManager::new();
        let e1 = manager.spawn();
        let e2 = manager.spawn();

        assert_eq!(e1.index(), 0);
        assert_eq!(e2.index(), 1);
        assert_eq!(manager.len(), 2);
    }

    #[test]
    fn test_entity_despawn() {
        let mut manager = EntityManager::new();
        let e1 = manager.spawn();

        assert!(manager.is_alive(e1));
        assert!(manager.despawn(e1));
        assert!(!manager.is_alive(e1));
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_entity_reuse() {
        let mut manager = EntityManager::new();
        let e1 = manager.spawn();
        manager.despawn(e1);

        let e2 = manager.spawn();
        // Same index, different generation
        assert_eq!(e2.index(), e1.index());
        assert_ne!(e2.generation(), e1.generation());

        // Old reference should be invalid
        assert!(!manager.is_alive(e1));
        assert!(manager.is_alive(e2));
    }

    #[test]
    fn test_entity_bits() {
        let entity = Entity::new(42, Generation(7));
        let bits = entity.to_bits();
        let restored = Entity::from_bits(bits);

        assert_eq!(entity, restored);
    }
}

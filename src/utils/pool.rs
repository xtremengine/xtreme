//! # Object Pool
//!
//! Efficient memory pool for reducing allocations.

use std::marker::PhantomData;

/// Handle to a pooled object
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    _phantom: PhantomData<T>,
}

impl<T> Handle<T> {
    fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _phantom: PhantomData,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

/// Entry in the pool
struct PoolEntry<T> {
    value: Option<T>,
    generation: u32,
}

/// Object pool with generational handles
pub struct Pool<T> {
    entries: Vec<PoolEntry<T>>,
    free_list: Vec<u32>,
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    /// Insert a value and return a handle
    pub fn insert(&mut self, value: T) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let entry = &mut self.entries[index as usize];
            entry.value = Some(value);
            Handle::new(index, entry.generation)
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(PoolEntry {
                value: Some(value),
                generation: 0,
            });
            Handle::new(index, 0)
        }
    }

    /// Remove and return a value
    pub fn remove(&mut self, handle: Handle<T>) -> Option<T> {
        let entry = self.entries.get_mut(handle.index as usize)?;
        if entry.generation != handle.generation {
            return None;
        }

        let value = entry.value.take()?;
        entry.generation = entry.generation.wrapping_add(1);
        self.free_list.push(handle.index);
        Some(value)
    }

    /// Get a reference to a value
    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        let entry = self.entries.get(handle.index as usize)?;
        if entry.generation != handle.generation {
            return None;
        }
        entry.value.as_ref()
    }

    /// Get a mutable reference to a value
    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let entry = self.entries.get_mut(handle.index as usize)?;
        if entry.generation != handle.generation {
            return None;
        }
        entry.value.as_mut()
    }

    /// Check if handle is valid
    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.entries
            .get(handle.index as usize)
            .map(|e| e.generation == handle.generation && e.value.is_some())
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.value.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.free_list.clear();
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_basic() {
        let mut pool = Pool::new();
        let h1 = pool.insert(42);
        let h2 = pool.insert(100);

        assert_eq!(pool.get(h1), Some(&42));
        assert_eq!(pool.get(h2), Some(&100));
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_pool_remove() {
        let mut pool = Pool::new();
        let h = pool.insert(42);

        assert_eq!(pool.remove(h), Some(42));
        assert!(!pool.contains(h));
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_pool_reuse() {
        let mut pool = Pool::new();
        let h1 = pool.insert(1);
        pool.remove(h1);
        let h2 = pool.insert(2);

        // Same index, different generation
        assert_eq!(h1.index(), h2.index());
        assert!(!pool.contains(h1));
        assert!(pool.contains(h2));
    }
}

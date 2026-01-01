//! # Prefab Registry
//!
//! Cache for loaded prefabs to avoid repeated disk reads.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Prefab;
use super::PrefabError;

/// Entry in the prefab cache
#[derive(Debug)]
struct CacheEntry {
    /// The loaded prefab
    prefab: Prefab,
    /// When the file was last modified (for hot reload)
    modified_time: Option<SystemTime>,
    /// When this entry was last accessed
    last_access: std::time::Instant,
}

/// Prefab registry with caching
pub struct PrefabRegistry {
    /// Cached prefabs by path
    cache: HashMap<PathBuf, CacheEntry>,
    /// Base path for resolving relative paths
    base_path: Option<PathBuf>,
    /// Maximum cache size (number of prefabs)
    max_cache_size: usize,
    /// Whether to check for file modifications
    hot_reload: bool,
}

impl Default for PrefabRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefabRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            base_path: None,
            max_cache_size: 100,
            hot_reload: true,
        }
    }

    /// Create with base path
    pub fn with_base_path(base_path: impl Into<PathBuf>) -> Self {
        Self {
            cache: HashMap::new(),
            base_path: Some(base_path.into()),
            max_cache_size: 100,
            hot_reload: true,
        }
    }

    /// Set the base path for resolving relative paths
    pub fn set_base_path(&mut self, path: impl Into<PathBuf>) {
        self.base_path = Some(path.into());
        // Clear cache when base path changes
        self.cache.clear();
    }

    /// Clear the base path
    pub fn clear_base_path(&mut self) {
        self.base_path = None;
    }

    /// Enable or disable hot reload
    pub fn set_hot_reload(&mut self, enabled: bool) {
        self.hot_reload = enabled;
    }

    /// Resolve a path relative to base path
    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref base) = self.base_path {
            base.join(path)
        } else {
            path.to_path_buf()
        }
    }

    /// Get file modification time
    fn get_modified_time(path: &Path) -> Option<SystemTime> {
        std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
    }

    /// Check if a cached entry is stale
    fn is_stale(&self, path: &Path, entry: &CacheEntry) -> bool {
        if !self.hot_reload {
            return false;
        }

        let current_time = Self::get_modified_time(path);
        match (current_time, entry.modified_time) {
            (Some(current), Some(cached)) => current > cached,
            (Some(_), None) => true,
            _ => false,
        }
    }

    /// Load a prefab, using cache if available
    pub fn get(&mut self, path: &Path) -> Result<&Prefab, PrefabError> {
        let resolved = self.resolve_path(path);
        let key = resolved.clone();

        // Check if we need to reload
        let needs_reload = match self.cache.get(&key) {
            Some(entry) => self.is_stale(&resolved, entry),
            None => true,
        };

        if needs_reload {
            // Evict old entries if cache is full
            if self.cache.len() >= self.max_cache_size {
                self.evict_oldest();
            }

            // Load from disk
            let prefab = Prefab::load(&resolved)?;
            let modified_time = Self::get_modified_time(&resolved);

            self.cache.insert(
                key.clone(),
                CacheEntry {
                    prefab,
                    modified_time,
                    last_access: std::time::Instant::now(),
                },
            );
        } else {
            // Update access time
            if let Some(entry) = self.cache.get_mut(&key) {
                entry.last_access = std::time::Instant::now();
            }
        }

        Ok(&self.cache.get(&key).unwrap().prefab)
    }

    /// Load a prefab and clone it (for modification)
    pub fn get_cloned(&mut self, path: &Path) -> Result<Prefab, PrefabError> {
        Ok(self.get(path)?.clone())
    }

    /// Check if a prefab is cached
    pub fn is_cached(&self, path: &Path) -> bool {
        let resolved = self.resolve_path(path);
        self.cache.contains_key(&resolved)
    }

    /// Invalidate a specific prefab
    pub fn invalidate(&mut self, path: &Path) {
        let resolved = self.resolve_path(path);
        self.cache.remove(&resolved);
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Evict the oldest entry
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, e)| e.last_access)
            .map(|(k, _)| k.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            cached_count: self.cache.len(),
            max_size: self.max_cache_size,
        }
    }

    /// Get number of cached prefabs
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get paths of prefabs that have been modified on disk
    pub fn get_modified_prefabs(&self) -> Vec<PathBuf> {
        self.cache
            .iter()
            .filter(|(path, entry)| self.is_stale(path, entry))
            .map(|(path, _)| path.clone())
            .collect()
    }

    /// Reload a specific prefab
    pub fn reload(&mut self, path: &Path) -> Result<(), PrefabError> {
        let resolved = self.resolve_path(path);
        self.cache.remove(&resolved);

        let prefab = Prefab::load(&resolved)?;
        self.cache.insert(
            resolved.clone(),
            CacheEntry {
                prefab,
                modified_time: Self::get_modified_time(&resolved),
                last_access: std::time::Instant::now(),
            },
        );
        Ok(())
    }

    /// Reload all cached prefabs from disk, returns count of reloaded
    pub fn reload_all(&mut self) -> usize {
        let paths: Vec<_> = self.cache.keys().cloned().collect();
        let mut reloaded_count = 0;

        for path in paths {
            self.cache.remove(&path);
            if let Ok(prefab) = Prefab::load(&path) {
                self.cache.insert(
                    path.clone(),
                    CacheEntry {
                        prefab,
                        modified_time: Self::get_modified_time(&path),
                        last_access: std::time::Instant::now(),
                    },
                );
                reloaded_count += 1;
            }
        }

        reloaded_count
    }
}

/// Cache statistics
#[derive(Clone, Debug)]
pub struct CacheStats {
    /// Number of cached prefabs
    pub cached_count: usize,
    /// Maximum cache size
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = PrefabRegistry::new();
        assert_eq!(registry.cache.len(), 0);
    }

    #[test]
    fn test_path_resolution() {
        let mut registry = PrefabRegistry::with_base_path("/project");
        assert_eq!(
            registry.resolve_path(Path::new("prefabs/test.prefab")),
            PathBuf::from("/project/prefabs/test.prefab")
        );

        // Absolute paths should not be modified
        assert_eq!(
            registry.resolve_path(Path::new("/absolute/path.prefab")),
            PathBuf::from("/absolute/path.prefab")
        );

        registry.clear_base_path();
        assert_eq!(
            registry.resolve_path(Path::new("prefabs/test.prefab")),
            PathBuf::from("prefabs/test.prefab")
        );
    }
}

//! # Prefab Instance
//!
//! Tracks prefab instances in the scene with their overrides.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::overrides::PropertyOverrides;

/// Reference to a prefab file
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrefabRef {
    /// Path to the prefab file (relative to project)
    pub path: PathBuf,
    /// Optional UUID for more robust references
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

impl PrefabRef {
    /// Create a new prefab reference from path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            uuid: None,
        }
    }

    /// Create with UUID
    pub fn with_uuid(path: impl Into<PathBuf>, uuid: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            uuid: Some(uuid.into()),
        }
    }

    /// Get prefab name from path
    pub fn name(&self) -> &str {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
    }
}

/// A prefab instance in the scene
///
/// Links a scene object to its source prefab and tracks property overrides.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrefabInstance {
    /// Reference to the source prefab
    pub prefab_ref: PrefabRef,
    /// Index of this object within the prefab
    /// (prefabs can contain multiple objects)
    pub object_index: usize,
    /// Property overrides from the prefab values
    #[serde(default)]
    pub overrides: PropertyOverrides,
    /// Whether this is the root instance
    /// (the first object instantiated from the prefab)
    #[serde(default)]
    pub is_root: bool,
    /// Instance ID for linking objects from the same instantiation
    #[serde(default)]
    pub instance_id: u32,
}

impl PrefabInstance {
    /// Create a new prefab instance
    pub fn new(prefab_ref: PrefabRef, object_index: usize, instance_id: u32) -> Self {
        Self {
            prefab_ref,
            object_index,
            overrides: PropertyOverrides::new(),
            is_root: object_index == 0,
            instance_id,
        }
    }

    /// Create root instance
    pub fn root(prefab_ref: PrefabRef, instance_id: u32) -> Self {
        Self {
            prefab_ref,
            object_index: 0,
            overrides: PropertyOverrides::new(),
            is_root: true,
            instance_id,
        }
    }

    /// Check if this instance has any overrides
    pub fn has_overrides(&self) -> bool {
        !self.overrides.is_empty()
    }

    /// Get the prefab path
    pub fn prefab_path(&self) -> &PathBuf {
        &self.prefab_ref.path
    }

    /// Get prefab name
    pub fn prefab_name(&self) -> &str {
        self.prefab_ref.name()
    }
}

/// Instance group - tracks all objects from a single prefab instantiation
#[derive(Clone, Debug)]
pub struct PrefabInstanceGroup {
    /// Unique ID for this group
    pub instance_id: u32,
    /// Reference to the prefab
    pub prefab_ref: PrefabRef,
    /// IDs of objects in this group
    pub object_ids: Vec<u32>,
}

impl PrefabInstanceGroup {
    /// Create a new instance group
    pub fn new(instance_id: u32, prefab_ref: PrefabRef) -> Self {
        Self {
            instance_id,
            prefab_ref,
            object_ids: Vec::new(),
        }
    }

    /// Add an object to this group
    pub fn add_object(&mut self, id: u32) {
        self.object_ids.push(id);
    }

    /// Get the root object ID (first object)
    pub fn root_id(&self) -> Option<u32> {
        self.object_ids.first().copied()
    }
}

/// Manages instance IDs
pub struct InstanceIdGenerator {
    next_id: u32,
}

impl Default for InstanceIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceIdGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// Create with starting ID
    pub fn with_start(start: u32) -> Self {
        Self { next_id: start }
    }

    /// Generate next ID
    pub fn next_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Update to ensure future IDs don't conflict
    pub fn ensure_after(&mut self, id: u32) {
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefab_ref() {
        let pref = PrefabRef::new("prefabs/player.prefab");
        assert_eq!(pref.name(), "player");

        let pref2 = PrefabRef::with_uuid("prefabs/enemy.prefab", "abc-123");
        assert_eq!(pref2.uuid, Some("abc-123".to_string()));
    }

    #[test]
    fn test_prefab_instance() {
        let pref = PrefabRef::new("prefabs/player.prefab");
        let instance = PrefabInstance::root(pref, 1);

        assert!(instance.is_root);
        assert_eq!(instance.object_index, 0);
        assert!(!instance.has_overrides());
    }

    #[test]
    fn test_instance_id_generator() {
        let mut gen = InstanceIdGenerator::new();
        assert_eq!(gen.next_id(), 1);
        assert_eq!(gen.next_id(), 2);

        gen.ensure_after(100);
        assert_eq!(gen.next_id(), 101);
    }
}

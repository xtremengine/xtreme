//! # Property Overrides
//!
//! Property-level overrides for prefab instances.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A path to a property within an object
///
/// Examples:
/// - "position" - the position vector
/// - "position.x" - just the x component
/// - "color" - the color array
/// - "camera.fov" - fov within camera component
/// - "audio_source.volume" - volume in audio source
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropertyPath(pub String);

impl PropertyPath {
    /// Create a new property path
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Get the path string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Check if this path is a parent of another
    /// e.g. "position" is parent of "position.x"
    pub fn is_parent_of(&self, other: &PropertyPath) -> bool {
        other.0.starts_with(&self.0) && other.0.len() > self.0.len()
    }

    /// Check if this path is a child of another
    pub fn is_child_of(&self, other: &PropertyPath) -> bool {
        other.is_parent_of(self)
    }

    /// Get the first component of the path
    pub fn root(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }

    /// Get the property path without the first component
    pub fn tail(&self) -> Option<PropertyPath> {
        let mut parts = self.0.splitn(2, '.');
        parts.next()?;
        parts.next().map(|s| PropertyPath(s.to_string()))
    }
}

impl From<&str> for PropertyPath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for PropertyPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Tracks which properties have been overridden from the prefab
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PropertyOverrides {
    /// Set of overridden property paths
    pub overridden: HashSet<PropertyPath>,
}

impl PropertyOverrides {
    /// Create empty overrides
    pub fn new() -> Self {
        Self {
            overridden: HashSet::new(),
        }
    }

    /// Check if a property is overridden
    pub fn is_overridden(&self, path: &PropertyPath) -> bool {
        self.overridden.contains(path)
    }

    /// Check if a property or any parent is overridden
    /// e.g. if "position" is overridden, "position.x" counts as overridden
    pub fn is_effectively_overridden(&self, path: &PropertyPath) -> bool {
        if self.overridden.contains(path) {
            return true;
        }

        // Check if any parent path is overridden
        let path_str = path.as_str();
        for overridden in &self.overridden {
            if path_str.starts_with(overridden.as_str()) {
                let remainder = &path_str[overridden.as_str().len()..];
                if remainder.is_empty() || remainder.starts_with('.') {
                    return true;
                }
            }
        }

        false
    }

    /// Mark a property as overridden
    pub fn set_override(&mut self, path: PropertyPath) {
        self.overridden.insert(path);
    }

    /// Remove an override (revert to prefab value)
    pub fn remove_override(&mut self, path: &PropertyPath) {
        self.overridden.remove(path);
    }

    /// Clear all overrides
    pub fn clear(&mut self) {
        self.overridden.clear();
    }

    /// Get number of overrides
    pub fn count(&self) -> usize {
        self.overridden.len()
    }

    /// Check if there are any overrides
    pub fn is_empty(&self) -> bool {
        self.overridden.is_empty()
    }

    /// Get all overridden paths
    pub fn paths(&self) -> impl Iterator<Item = &PropertyPath> {
        self.overridden.iter()
    }

    /// Merge with another set of overrides
    pub fn merge(&mut self, other: &PropertyOverrides) {
        for path in &other.overridden {
            self.overridden.insert(path.clone());
        }
    }
}

/// Standard property paths for scene objects
pub mod paths {
    use super::PropertyPath;

    pub fn name() -> PropertyPath {
        PropertyPath::new("name")
    }

    pub fn position() -> PropertyPath {
        PropertyPath::new("position")
    }

    pub fn position_x() -> PropertyPath {
        PropertyPath::new("position.x")
    }

    pub fn position_y() -> PropertyPath {
        PropertyPath::new("position.y")
    }

    pub fn position_z() -> PropertyPath {
        PropertyPath::new("position.z")
    }

    pub fn rotation() -> PropertyPath {
        PropertyPath::new("rotation")
    }

    pub fn scale() -> PropertyPath {
        PropertyPath::new("scale")
    }

    pub fn color() -> PropertyPath {
        PropertyPath::new("color")
    }

    pub fn visible() -> PropertyPath {
        PropertyPath::new("visible")
    }

    pub fn camera() -> PropertyPath {
        PropertyPath::new("camera")
    }

    pub fn camera_fov() -> PropertyPath {
        PropertyPath::new("camera.fov")
    }

    pub fn audio_source() -> PropertyPath {
        PropertyPath::new("audio_source")
    }

    pub fn audio_source_volume() -> PropertyPath {
        PropertyPath::new("audio_source.volume")
    }

    pub fn particle_emitter() -> PropertyPath {
        PropertyPath::new("particle_emitter")
    }

    pub fn animator() -> PropertyPath {
        PropertyPath::new("animator")
    }

    pub fn texture_path() -> PropertyPath {
        PropertyPath::new("texture_path")
    }

    pub fn shader_path() -> PropertyPath {
        PropertyPath::new("shader_path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_path() {
        let pos = PropertyPath::new("position");
        let pos_x = PropertyPath::new("position.x");

        assert!(pos.is_parent_of(&pos_x));
        assert!(!pos_x.is_parent_of(&pos));
        assert!(pos_x.is_child_of(&pos));

        assert_eq!(pos.root(), "position");
        assert_eq!(pos_x.root(), "position");
        assert_eq!(pos_x.tail(), Some(PropertyPath::new("x")));
        assert_eq!(pos.tail(), None);
    }

    #[test]
    fn test_property_overrides() {
        let mut overrides = PropertyOverrides::new();

        overrides.set_override(PropertyPath::new("position"));

        assert!(overrides.is_overridden(&PropertyPath::new("position")));
        assert!(!overrides.is_overridden(&PropertyPath::new("rotation")));

        // Check effective override
        assert!(overrides.is_effectively_overridden(&PropertyPath::new("position.x")));
        assert!(!overrides.is_effectively_overridden(&PropertyPath::new("rotation.x")));

        overrides.remove_override(&PropertyPath::new("position"));
        assert!(!overrides.is_overridden(&PropertyPath::new("position")));
    }
}

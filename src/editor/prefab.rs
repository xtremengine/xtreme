//! # Prefab System
//!
//! Reusable object templates that can be saved and instantiated.

use std::path::Path;
use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::components::CameraComponent;
use super::selection::SceneObject;

/// Error type for prefab operations
#[derive(Debug)]
pub enum PrefabError {
    Io(std::io::Error),
    Serialize(String),
    Deserialize(String),
    Empty,
}

impl std::fmt::Display for PrefabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefabError::Io(e) => write!(f, "IO error: {}", e),
            PrefabError::Serialize(e) => write!(f, "Serialize error: {}", e),
            PrefabError::Deserialize(e) => write!(f, "Deserialize error: {}", e),
            PrefabError::Empty => write!(f, "Cannot create prefab from empty selection"),
        }
    }
}

impl std::error::Error for PrefabError {}

impl From<std::io::Error> for PrefabError {
    fn from(e: std::io::Error) -> Self {
        PrefabError::Io(e)
    }
}

/// A single object within a prefab
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrefabObject {
    /// Object name
    pub name: String,
    /// Position relative to prefab center
    pub local_position: [f32; 3],
    /// Rotation (euler angles)
    pub rotation: [f32; 3],
    /// Scale
    pub scale: [f32; 3],
    /// Color
    pub color: [f32; 4],
    /// Visibility
    pub visible: bool,
    /// Camera component (optional)
    #[serde(default)]
    pub camera: Option<CameraComponent>,
}

impl PrefabObject {
    /// Convert to Vec3 position
    pub fn position_vec(&self) -> Vec3 {
        Vec3::from_array(self.local_position)
    }

    /// Convert to Vec3 rotation
    pub fn rotation_vec(&self) -> Vec3 {
        Vec3::from_array(self.rotation)
    }

    /// Convert to Vec3 scale
    pub fn scale_vec(&self) -> Vec3 {
        Vec3::from_array(self.scale)
    }
}

/// A prefab - a reusable group of objects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prefab {
    /// Prefab name
    pub name: String,
    /// Objects in this prefab
    pub objects: Vec<PrefabObject>,
    /// Prefab version for compatibility
    pub version: u32,
}

impl Default for Prefab {
    fn default() -> Self {
        Self {
            name: "New Prefab".to_string(),
            objects: Vec::new(),
            version: 1,
        }
    }
}

impl Prefab {
    /// Create a new empty prefab
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            objects: Vec::new(),
            version: 1,
        }
    }

    /// Create a prefab from selected scene objects
    pub fn from_selection(name: impl Into<String>, objects: &[SceneObject]) -> Result<Self, PrefabError> {
        if objects.is_empty() {
            return Err(PrefabError::Empty);
        }

        // Calculate center of selection
        let center = objects.iter()
            .map(|o| o.position)
            .fold(Vec3::ZERO, |a, b| a + b) / objects.len() as f32;

        // Convert to prefab objects with relative positions
        let prefab_objects: Vec<PrefabObject> = objects.iter()
            .map(|obj| PrefabObject {
                name: obj.name.clone(),
                local_position: (obj.position - center).to_array(),
                rotation: obj.rotation.to_array(),
                scale: obj.scale.to_array(),
                color: obj.color,
                visible: obj.visible,
                camera: obj.camera.clone(),
            })
            .collect();

        Ok(Self {
            name: name.into(),
            objects: prefab_objects,
            version: 1,
        })
    }

    /// Instantiate this prefab at a position, returning new scene objects
    pub fn instantiate(&self, position: Vec3, next_id: &mut u32) -> Vec<SceneObject> {
        self.objects.iter()
            .map(|pobj| {
                let id = *next_id;
                *next_id += 1;

                let mut obj = SceneObject::new(id, pobj.name.clone());
                obj.position = position + pobj.position_vec();
                obj.rotation = pobj.rotation_vec();
                obj.scale = pobj.scale_vec();
                obj.color = pobj.color;
                obj.visible = pobj.visible;
                obj.camera = pobj.camera.clone();
                obj
            })
            .collect()
    }

    /// Save prefab to file (RON format)
    pub fn save(&self, path: &Path) -> Result<(), PrefabError> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| PrefabError::Serialize(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load prefab from file
    pub fn load(path: &Path) -> Result<Self, PrefabError> {
        let content = std::fs::read_to_string(path)?;

        // Try RON first, then JSON
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            serde_json::from_str(&content)
                .map_err(|e| PrefabError::Deserialize(e.to_string()))
        } else {
            ron::from_str(&content)
                .map_err(|e| PrefabError::Deserialize(e.to_string()))
        }
    }

    /// Get object count
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefab_from_selection() {
        let mut obj1 = SceneObject::new(1, "Cube1");
        obj1.position = Vec3::new(0.0, 0.0, 0.0);
        obj1.color = [1.0, 0.0, 0.0, 1.0];

        let mut obj2 = SceneObject::new(2, "Cube2");
        obj2.position = Vec3::new(2.0, 0.0, 0.0);
        obj2.color = [0.0, 1.0, 0.0, 1.0];

        let objects = vec![obj1, obj2];

        let prefab = Prefab::from_selection("Test", &objects).unwrap();
        assert_eq!(prefab.objects.len(), 2);
        assert_eq!(prefab.name, "Test");

        // Center should be (1, 0, 0), so local positions are (-1, 0, 0) and (1, 0, 0)
        assert!((prefab.objects[0].local_position[0] - (-1.0)).abs() < 0.001);
        assert!((prefab.objects[1].local_position[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_prefab_instantiate() {
        let prefab = Prefab {
            name: "Test".to_string(),
            objects: vec![
                PrefabObject {
                    name: "Obj".to_string(),
                    local_position: [1.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    visible: true,
                    camera: None,
                },
            ],
            version: 1,
        };

        let mut next_id = 10;
        let objects = prefab.instantiate(Vec3::new(5.0, 0.0, 0.0), &mut next_id);

        assert_eq!(objects.len(), 1);
        assert_eq!(objects[0].id, 10);
        assert_eq!(objects[0].position, Vec3::new(6.0, 0.0, 0.0));
        assert_eq!(next_id, 11);
    }
}

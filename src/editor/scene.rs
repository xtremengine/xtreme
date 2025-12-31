//! # Scene Serialization
//!
//! Save and load scenes to/from files.

use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Serializable scene object
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneObjectData {
    /// Object ID (regenerated on load)
    #[serde(skip)]
    pub id: u32,
    /// Object name
    pub name: String,
    /// Position in world space
    pub position: [f32; 3],
    /// Rotation (euler angles in radians)
    pub rotation: [f32; 3],
    /// Scale
    pub scale: [f32; 3],
    /// Color (RGBA)
    pub color: [f32; 4],
    /// Visibility
    pub visible: bool,
}

impl SceneObjectData {
    /// Convert to Vec3 values
    pub fn position_vec(&self) -> Vec3 {
        Vec3::from_array(self.position)
    }

    pub fn rotation_vec(&self) -> Vec3 {
        Vec3::from_array(self.rotation)
    }

    pub fn scale_vec(&self) -> Vec3 {
        Vec3::from_array(self.scale)
    }
}

/// Scene file format
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneData {
    /// Scene format version
    pub version: u32,
    /// Scene name
    pub name: String,
    /// Camera position hint
    pub camera_target: Option<[f32; 3]>,
    /// Camera distance hint
    pub camera_distance: Option<f32>,
    /// Objects in the scene
    pub objects: Vec<SceneObjectData>,
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            version: 1,
            name: "Untitled".to_string(),
            camera_target: None,
            camera_distance: None,
            objects: Vec::new(),
        }
    }
}

impl SceneData {
    /// Create new empty scene
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: 1,
            name: name.into(),
            camera_target: None,
            camera_distance: None,
            objects: Vec::new(),
        }
    }

    /// Add an object to the scene
    pub fn add_object(&mut self, obj: SceneObjectData) {
        self.objects.push(obj);
    }

    /// Save to RON file
    pub fn save_ron(&self, path: &Path) -> Result<(), SceneError> {
        let config = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .separate_tuple_members(true);

        let data = ron::ser::to_string_pretty(self, config)
            .map_err(|e| SceneError::Serialize(e.to_string()))?;

        fs::write(path, data)
            .map_err(|e| SceneError::Io(e.to_string()))?;

        Ok(())
    }

    /// Load from RON file
    pub fn load_ron(path: &Path) -> Result<Self, SceneError> {
        let data = fs::read_to_string(path)
            .map_err(|e| SceneError::Io(e.to_string()))?;

        let scene: SceneData = ron::from_str(&data)
            .map_err(|e| SceneError::Deserialize(e.to_string()))?;

        Ok(scene)
    }

    /// Save to JSON file
    pub fn save_json(&self, path: &Path) -> Result<(), SceneError> {
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| SceneError::Serialize(e.to_string()))?;

        fs::write(path, data)
            .map_err(|e| SceneError::Io(e.to_string()))?;

        Ok(())
    }

    /// Load from JSON file
    pub fn load_json(path: &Path) -> Result<Self, SceneError> {
        let data = fs::read_to_string(path)
            .map_err(|e| SceneError::Io(e.to_string()))?;

        let scene: SceneData = serde_json::from_str(&data)
            .map_err(|e| SceneError::Deserialize(e.to_string()))?;

        Ok(scene)
    }

    /// Save to file (auto-detect format from extension)
    pub fn save(&self, path: &Path) -> Result<(), SceneError> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => self.save_json(path),
            Some("ron") | _ => self.save_ron(path),
        }
    }

    /// Load from file (auto-detect format from extension)
    pub fn load(path: &Path) -> Result<Self, SceneError> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Self::load_json(path),
            Some("ron") | _ => Self::load_ron(path),
        }
    }
}

/// Scene errors
#[derive(Debug)]
pub enum SceneError {
    /// IO error
    Io(String),
    /// Serialization error
    Serialize(String),
    /// Deserialization error
    Deserialize(String),
}

impl std::fmt::Display for SceneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SceneError::Io(e) => write!(f, "IO error: {}", e),
            SceneError::Serialize(e) => write!(f, "Serialize error: {}", e),
            SceneError::Deserialize(e) => write!(f, "Deserialize error: {}", e),
        }
    }
}

impl std::error::Error for SceneError {}

/// Scene manager handles loading/saving and tracking current scene
pub struct SceneManager {
    /// Current scene path (None if unsaved)
    current_path: Option<std::path::PathBuf>,
    /// Whether scene has unsaved changes
    dirty: bool,
    /// Current scene name
    scene_name: String,
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneManager {
    /// Create new scene manager
    pub fn new() -> Self {
        Self {
            current_path: None,
            dirty: false,
            scene_name: "Untitled".to_string(),
        }
    }

    /// Get current path
    pub fn current_path(&self) -> Option<&Path> {
        self.current_path.as_deref()
    }

    /// Get scene name
    pub fn scene_name(&self) -> &str {
        &self.scene_name
    }

    /// Check if scene has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark scene as dirty (has unsaved changes)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Mark scene as clean (just saved)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Set current path after save
    pub fn set_path(&mut self, path: std::path::PathBuf) {
        self.scene_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        self.current_path = Some(path);
        self.dirty = false;
    }

    /// Clear path for new scene
    pub fn new_scene(&mut self) {
        self.current_path = None;
        self.scene_name = "Untitled".to_string();
        self.dirty = false;
    }

    /// Get window title
    pub fn window_title(&self) -> String {
        let dirty_marker = if self.dirty { "*" } else { "" };
        format!("{}{} - Xtreme Editor", dirty_marker, self.scene_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_scene_roundtrip_ron() {
        let mut scene = SceneData::new("Test Scene");
        scene.add_object(SceneObjectData {
            id: 0,
            name: "Cube".to_string(),
            position: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
            visible: true,
        });

        let mut temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_owned();

        scene.save_ron(&path).unwrap();

        let loaded = SceneData::load_ron(&path).unwrap();
        assert_eq!(loaded.name, "Test Scene");
        assert_eq!(loaded.objects.len(), 1);
        assert_eq!(loaded.objects[0].name, "Cube");
    }

    #[test]
    fn test_scene_manager() {
        let mut manager = SceneManager::new();
        assert!(!manager.is_dirty());
        assert_eq!(manager.scene_name(), "Untitled");

        manager.mark_dirty();
        assert!(manager.is_dirty());

        manager.set_path("test_scene.ron".into());
        assert!(!manager.is_dirty());
        assert_eq!(manager.scene_name(), "test_scene");
    }
}

//! Project management - save/load complete projects.

use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

/// Project metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Author name
    pub author: String,
    /// Description
    pub description: String,
    /// Main scene file (relative to project root)
    pub main_scene: String,
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Splash screen duration in seconds
    pub splash_duration: f32,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "New Project".to_string(),
            version: "0.1.0".to_string(),
            author: String::new(),
            description: String::new(),
            main_scene: "scenes/main.ron".to_string(),
            window_width: 1280,
            window_height: 720,
            splash_duration: 2.0,
        }
    }
}

impl ProjectConfig {
    /// Create a new project config with name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self, String> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read project config: {}", e))?;

        ron::from_str(&contents)
            .map_err(|e| format!("Failed to parse project config: {}", e))
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let contents = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| format!("Failed to serialize project config: {}", e))?;

        std::fs::write(path, contents)
            .map_err(|e| format!("Failed to write project config: {}", e))
    }
}

/// Project folder structure
pub struct Project {
    /// Root path of the project
    pub root: PathBuf,
    /// Project configuration
    pub config: ProjectConfig,
}

impl Project {
    /// Create a new project at the given path
    pub fn create(root: PathBuf, name: &str) -> Result<Self, String> {
        // Create directory structure
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create project directory: {}", e))?;

        let dirs = ["scenes", "scripts", "prefabs", "assets"];
        for dir in dirs {
            let dir_path = root.join(dir);
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create {} directory: {}", dir, e))?;
        }

        // Create project config
        let config = ProjectConfig::new(name);
        let config_path = root.join("project.ron");
        config.save(&config_path)?;

        log::info!("Created new project '{}' at {:?}", name, root);

        Ok(Self { root, config })
    }

    /// Open an existing project
    pub fn open(root: PathBuf) -> Result<Self, String> {
        let config_path = root.join("project.ron");
        if !config_path.exists() {
            return Err(format!("No project.ron found at {:?}", root));
        }

        let config = ProjectConfig::load(&config_path)?;

        log::info!("Opened project '{}' from {:?}", config.name, root);

        Ok(Self { root, config })
    }

    /// Save project config
    pub fn save(&self) -> Result<(), String> {
        let config_path = self.root.join("project.ron");
        self.config.save(&config_path)
    }

    /// Get path to main scene
    pub fn main_scene_path(&self) -> PathBuf {
        self.root.join(&self.config.main_scene)
    }

    /// Get scenes directory
    pub fn scenes_dir(&self) -> PathBuf {
        self.root.join("scenes")
    }

    /// Get scripts directory
    pub fn scripts_dir(&self) -> PathBuf {
        self.root.join("scripts")
    }

    /// Get prefabs directory
    pub fn prefabs_dir(&self) -> PathBuf {
        self.root.join("prefabs")
    }

    /// Get assets directory
    pub fn assets_dir(&self) -> PathBuf {
        self.root.join("assets")
    }
}

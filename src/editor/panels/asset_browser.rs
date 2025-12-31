//! # Asset Browser Panel
//!
//! File browser for project assets (scenes, prefabs, textures, etc.)

use std::path::PathBuf;
use egui::Ui;

/// Type of asset entry
#[derive(Clone, Debug, PartialEq)]
pub enum AssetType {
    Directory,
    Scene,
    Prefab,
    Texture,
    Unknown,
}

impl AssetType {
    /// Get icon for this asset type
    pub fn icon(&self) -> &'static str {
        match self {
            AssetType::Directory => "[D]",
            AssetType::Scene => "[S]",
            AssetType::Prefab => "[P]",
            AssetType::Texture => "[T]",
            AssetType::Unknown => "[?]",
        }
    }

    /// Detect type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "ron" | "json" => AssetType::Scene, // Could also be prefab
            "prefab" => AssetType::Prefab,
            "png" | "jpg" | "jpeg" | "bmp" | "tga" => AssetType::Texture,
            _ => AssetType::Unknown,
        }
    }
}

/// An entry in the asset browser
#[derive(Clone, Debug)]
pub struct AssetEntry {
    /// Entry name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Asset type
    pub asset_type: AssetType,
}

/// Actions returned by the asset browser
#[derive(Clone, Debug)]
pub enum AssetAction {
    None,
    OpenScene(PathBuf),
    LoadPrefab(PathBuf),
    OpenDirectory(PathBuf),
}

/// Asset browser panel
pub struct AssetBrowser {
    /// Root directory
    root: PathBuf,
    /// Current directory
    current_path: PathBuf,
    /// Cached entries
    entries: Vec<AssetEntry>,
    /// Selected entry index
    selected: Option<usize>,
    /// Search filter
    filter: String,
    /// Whether entries need refresh
    needs_refresh: bool,
}

impl Default for AssetBrowser {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl AssetBrowser {
    /// Create a new asset browser
    pub fn new(root: PathBuf) -> Self {
        let current_path = root.clone();
        let mut browser = Self {
            root,
            current_path,
            entries: Vec::new(),
            selected: None,
            filter: String::new(),
            needs_refresh: true,
        };
        browser.refresh();
        browser
    }

    /// Set the root directory
    pub fn set_root(&mut self, root: PathBuf) {
        self.root = root.clone();
        self.current_path = root;
        self.needs_refresh = true;
    }

    /// Navigate to a directory
    pub fn navigate_to(&mut self, path: PathBuf) {
        self.current_path = path;
        self.selected = None;
        self.needs_refresh = true;
    }

    /// Go to parent directory
    pub fn go_up(&mut self) {
        if self.current_path != self.root {
            if let Some(parent) = self.current_path.parent() {
                self.current_path = parent.to_path_buf();
                self.selected = None;
                self.needs_refresh = true;
            }
        }
    }

    /// Refresh the directory listing
    pub fn refresh(&mut self) {
        self.entries.clear();
        self.needs_refresh = false;

        let Ok(read_dir) = std::fs::read_dir(&self.current_path) else {
            return;
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();

            // Skip hidden files
            if name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                dirs.push(AssetEntry {
                    name,
                    path,
                    asset_type: AssetType::Directory,
                });
            } else {
                let asset_type = path.extension()
                    .and_then(|e| e.to_str())
                    .map(AssetType::from_extension)
                    .unwrap_or(AssetType::Unknown);

                files.push(AssetEntry {
                    name,
                    path,
                    asset_type,
                });
            }
        }

        // Sort: directories first, then files alphabetically
        dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        self.entries.extend(dirs);
        self.entries.extend(files);
    }

    /// Show the asset browser panel
    pub fn show(&mut self, ui: &mut Ui) -> AssetAction {
        if self.needs_refresh {
            self.refresh();
        }

        let mut action = AssetAction::None;

        // Header
        ui.horizontal(|ui| {
            ui.heading("Assets");

            ui.separator();

            // Navigation buttons
            if ui.small_button("^").clicked() {
                self.go_up();
            }

            if ui.small_button("R").clicked() {
                self.needs_refresh = true;
            }

            ui.separator();

            // Current path
            let path_str = self.current_path.to_string_lossy();
            ui.label(&*path_str);
        });

        ui.separator();

        // Filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);
        });

        ui.separator();

        // Content area with scroll
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .show(ui, |ui| {
                let filter_lower = self.filter.to_lowercase();

                for (i, entry) in self.entries.iter().enumerate() {
                    // Apply filter
                    if !self.filter.is_empty() && !entry.name.to_lowercase().contains(&filter_lower) {
                        continue;
                    }

                    let is_selected = self.selected == Some(i);
                    let label = format!("{} {}", entry.asset_type.icon(), entry.name);

                    let response = ui.selectable_label(is_selected, &label);

                    if response.clicked() {
                        self.selected = Some(i);
                    }

                    if response.double_clicked() {
                        match &entry.asset_type {
                            AssetType::Directory => {
                                action = AssetAction::OpenDirectory(entry.path.clone());
                            }
                            AssetType::Scene => {
                                action = AssetAction::OpenScene(entry.path.clone());
                            }
                            AssetType::Prefab => {
                                action = AssetAction::LoadPrefab(entry.path.clone());
                            }
                            _ => {}
                        }
                    }
                }
            });

        // Handle navigation action
        if let AssetAction::OpenDirectory(path) = &action {
            self.navigate_to(path.clone());
            action = AssetAction::None;
        }

        action
    }
}

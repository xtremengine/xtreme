//! Prefab file dialog - directly opens Windows save dialog.

use std::collections::HashSet;

use crate::editor::app::EditorApp;
use crate::editor::prefab::{Prefab, PrefabCreationData};
use crate::editor::selection::SceneObject;

impl EditorApp {
    /// Create prefab from selection and open Windows save dialog directly
    pub fn create_prefab_with_file_dialog(&mut self) {
        let selected_ids = self.selection.all();
        if selected_ids.is_empty() {
            log::warn!("Cannot create prefab: no objects selected");
            return;
        }

        // Collect all objects including children recursively
        let all_ids = self.collect_with_children(selected_ids);

        // Collect objects with their script paths
        let creation_data: Vec<PrefabCreationData> = all_ids
            .iter()
            .filter_map(|id| {
                self.scene_objects.iter().find(|o| o.id == *id).map(|obj| {
                    // Get script paths for this object
                    #[cfg(feature = "scripting")]
                    let script_paths: Vec<String> = obj
                        .scripts
                        .iter()
                        .filter_map(|&script_id| {
                            self.script_runtime
                                .get_script(script_id)
                                .map(|s| s.path.to_string_lossy().to_string())
                        })
                        .collect();

                    #[cfg(not(feature = "scripting"))]
                    let script_paths: Vec<String> = Vec::new();

                    PrefabCreationData {
                        object: obj.clone(),
                        script_paths,
                    }
                })
            })
            .collect();

        if creation_data.is_empty() {
            log::warn!("Cannot create prefab: no valid objects found");
            return;
        }

        // Default filename
        let default_filename = format!("Prefab_{}.xpfb", self.prefabs.len() + 1);

        // Open Windows save dialog directly
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Xtreme Prefab", &["xpfb"])
            .add_filter("All files", &["*"])
            .set_title("Save Prefab")
            .set_file_name(&default_filename)
            .save_file()
        {
            // Ensure .xpfb extension
            let path = if path.extension().is_none() {
                path.with_extension("xpfb")
            } else {
                path
            };

            // Get prefab name from filename (without extension)
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Prefab")
                .to_string();

            // Create the prefab with scripts and hierarchy
            let prefab = match Prefab::from_selection_with_scripts(&name, &creation_data) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to create prefab: {}", e);
                    return;
                }
            };

            // Save to file
            match prefab.save(&path) {
                Ok(()) => {
                    log::info!(
                        "Saved prefab '{}' with {} objects to {:?}",
                        prefab.name,
                        prefab.object_count(),
                        path
                    );
                    // Also add to in-memory list
                    self.prefabs.push(prefab);
                }
                Err(e) => {
                    log::error!("Failed to save prefab: {}", e);
                }
            }
        }
    }

    /// Collect all object IDs including their children recursively
    fn collect_with_children(&self, ids: &[u32]) -> Vec<u32> {
        let mut result: Vec<u32> = Vec::new();
        let mut visited: HashSet<u32> = HashSet::new();

        fn collect_recursive(
            id: u32,
            objects: &[SceneObject],
            result: &mut Vec<u32>,
            visited: &mut HashSet<u32>,
        ) {
            if visited.contains(&id) {
                return;
            }
            visited.insert(id);

            if let Some(obj) = objects.iter().find(|o| o.id == id) {
                result.push(id);

                // Recursively collect children
                for &child_id in &obj.hierarchy.children {
                    collect_recursive(child_id, objects, result, visited);
                }
            }
        }

        for &id in ids {
            collect_recursive(id, &self.scene_objects, &mut result, &mut visited);
        }

        result
    }

    /// Legacy dialog draw - no longer used but kept for compatibility
    pub(crate) fn draw_prefab_dialog(&mut self, _ctx: &egui::Context) {
        // Dialog is now handled directly via Windows file dialog
        // This method is kept for compatibility but does nothing
    }
}

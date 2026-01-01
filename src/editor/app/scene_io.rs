//! Scene save/load actions.

use std::collections::HashMap;
use std::path::PathBuf;

use super::EditorApp;
use crate::editor::scene::{SceneData, SceneObjectData};
use crate::editor::selection::SceneObject;

impl EditorApp {
    /// Save current scene
    pub fn save_scene(&mut self, path: PathBuf) {
        let mut scene_data = SceneData::new(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled"),
        );

        scene_data.camera_target = Some(self.camera.target.to_array());
        scene_data.camera_distance = Some(self.camera.distance);

        // Save timeline sequence if present
        scene_data.timeline = self.timeline_panel.sequence().cloned();

        // Create a map from object id to index in the save order
        let id_to_index: HashMap<u32, usize> = self
            .scene_objects
            .iter()
            .enumerate()
            .map(|(idx, obj)| (obj.id, idx))
            .collect();

        // Collect script paths for each object
        #[cfg(feature = "scripting")]
        let script_paths: HashMap<u32, Vec<String>> = self
            .scene_objects
            .iter()
            .map(|obj| {
                let paths: Vec<String> = obj
                    .scripts
                    .iter()
                    .filter_map(|&script_id| {
                        self.script_runtime
                            .get_script(script_id)
                            .map(|s| s.path.to_string_lossy().to_string())
                    })
                    .collect();
                (obj.id, paths)
            })
            .collect();

        for obj in &self.scene_objects {
            #[cfg(feature = "scripting")]
            let scripts = script_paths.get(&obj.id).cloned().unwrap_or_default();
            #[cfg(not(feature = "scripting"))]
            let scripts = Vec::new();

            // Convert parent id to parent index
            let parent_index = obj
                .hierarchy
                .parent
                .and_then(|parent_id| id_to_index.get(&parent_id).copied());

            // Get prefab instance if this object is a prefab instance
            let prefab_instance = self.prefab_instances.get(&obj.id).cloned();

            scene_data.add_object(SceneObjectData {
                id: obj.id,
                name: obj.name.clone(),
                position: obj.position.to_array(),
                rotation: obj.rotation.to_array(),
                scale: obj.scale.to_array(),
                color: obj.color,
                visible: obj.visible,
                parent_index,
                scripts,
                camera: obj.camera.clone(),
                texture_path: obj.texture_path.clone(),
                shader_path: obj.shader_path.clone(),
                audio_source: obj.audio_source.clone(),
                audio_listener: obj.audio_listener.clone(),
                particle_emitter: obj.particle_emitter.clone(),
                animator: obj.animator.clone(),
                prefab_instance,
                mesh_path: obj.mesh_path.clone(),
            });
        }

        match scene_data.save(&path) {
            Ok(()) => {
                self.scene_manager.set_path(path.clone());
                log::info!("Scene saved to {:?}", path);
            }
            Err(e) => {
                log::error!("Failed to save scene: {}", e);
            }
        }
    }

    /// Load a scene from file
    pub fn load_scene(&mut self, path: PathBuf) {
        match SceneData::load(&path) {
            Ok(scene_data) => {
                // Clear current scene
                self.scene_objects.clear();
                self.selection.clear();
                self.command_history.clear();
                self.prefab_instances.clear();
                self.next_id = 0;

                // Load camera settings
                if let Some(target) = scene_data.camera_target {
                    self.camera.target = glam::Vec3::from_array(target);
                }
                if let Some(distance) = scene_data.camera_distance {
                    self.camera.distance = distance;
                }

                // First pass: Create all objects and store parent indices
                let mut parent_indices: Vec<Option<usize>> = Vec::new();

                for obj_data in &scene_data.objects {
                    let obj_id = self.next_id;
                    let mut obj = SceneObject::new(obj_id, obj_data.name.clone());
                    obj.position = obj_data.position_vec();
                    obj.rotation = obj_data.rotation_vec();
                    obj.scale = obj_data.scale_vec();
                    obj.color = obj_data.color;
                    obj.visible = obj_data.visible;
                    obj.camera = obj_data.camera.clone();
                    obj.texture_path = obj_data.texture_path.clone();
                    obj.shader_path = obj_data.shader_path.clone();
                    obj.audio_source = obj_data.audio_source.clone();
                    obj.audio_listener = obj_data.audio_listener.clone();
                    obj.particle_emitter = obj_data.particle_emitter.clone();
                    obj.animator = obj_data.animator.clone();
                    obj.mesh_path = obj_data.mesh_path.clone();

                    // Track prefab instance if present
                    if let Some(ref prefab_instance) = obj_data.prefab_instance {
                        self.prefab_instances
                            .insert(obj_id, prefab_instance.clone());
                        // Ensure instance ID generator doesn't conflict
                        self.instance_id_generator
                            .ensure_after(prefab_instance.instance_id);
                    }

                    parent_indices.push(obj_data.parent_index);
                    self.scene_objects.push(obj);
                    self.next_id += 1;
                }

                // Second pass: Reconstruct hierarchy from parent indices
                // Build a map from load index to object id
                let index_to_id: Vec<u32> = self.scene_objects.iter().map(|o| o.id).collect();

                for (idx, parent_idx) in parent_indices.iter().enumerate() {
                    if let Some(parent_idx) = parent_idx {
                        if let Some(&parent_id) = index_to_id.get(*parent_idx) {
                            let child_id = index_to_id[idx];

                            // Set parent on child
                            if let Some(child) =
                                self.scene_objects.iter_mut().find(|o| o.id == child_id)
                            {
                                child.hierarchy.parent = Some(parent_id);
                            }

                            // Add child to parent's children list
                            if let Some(parent) =
                                self.scene_objects.iter_mut().find(|o| o.id == parent_id)
                            {
                                parent.hierarchy.add_child(child_id);
                            }
                        }
                    }
                }

                // Third pass: Re-attach scripts
                #[cfg(feature = "scripting")]
                for (idx, obj_data) in scene_data.objects.iter().enumerate() {
                    let obj_id = index_to_id[idx];
                    for script_path in &obj_data.scripts {
                        let script_path_buf = PathBuf::from(script_path);
                        if script_path_buf.exists() {
                            match self
                                .script_runtime
                                .attach_script(script_path_buf.clone(), obj_id)
                            {
                                Ok(script_id) => {
                                    if let Some(obj) =
                                        self.scene_objects.iter_mut().find(|o| o.id == obj_id)
                                    {
                                        obj.scripts.push(script_id);
                                    }
                                    log::info!(
                                        "Re-attached script {:?} to object {}",
                                        script_path,
                                        obj_id
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to re-attach script {:?}: {}",
                                        script_path,
                                        e
                                    );
                                }
                            }
                        } else {
                            log::warn!("Script not found: {:?}", script_path);
                        }
                    }
                }

                self.scene_manager.set_path(path.clone());

                // Restore timeline sequence if present
                if let Some(timeline) = scene_data.timeline {
                    self.timeline_panel.set_sequence(timeline);
                    self.show_timeline = true; // Show timeline panel when loading scene with timeline
                    log::info!("Timeline sequence restored from scene");
                } else {
                    self.timeline_panel.clear_sequence();
                }

                // Mark particles dirty if any objects have particle emitters
                if self
                    .scene_objects
                    .iter()
                    .any(|o| o.particle_emitter.is_some())
                {
                    self.particles_dirty = true;
                }

                log::info!("Scene loaded from {:?}", path);
            }
            Err(e) => {
                log::error!("Failed to load scene: {}", e);
            }
        }
    }

    /// Clear scene for new
    pub fn new_scene(&mut self) {
        self.scene_objects.clear();
        self.selection.clear();
        self.command_history.clear();
        self.prefab_instances.clear();
        self.next_id = 0;
        self.scene_manager.new_scene();
        self.timeline_panel.clear_sequence();
        log::info!("New scene created");
    }
}

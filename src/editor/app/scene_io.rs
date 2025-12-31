//! Scene save/load actions.

use std::path::PathBuf;

use crate::editor::selection::SceneObject;
use crate::editor::scene::{SceneData, SceneObjectData};
use super::EditorApp;

impl EditorApp {
    /// Save current scene
    pub fn save_scene(&mut self, path: PathBuf) {
        let mut scene_data = SceneData::new(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
        );

        scene_data.camera_target = Some(self.camera.target.to_array());
        scene_data.camera_distance = Some(self.camera.distance);

        for obj in &self.scene_objects {
            scene_data.add_object(SceneObjectData {
                id: obj.id,
                name: obj.name.clone(),
                position: obj.position.to_array(),
                rotation: obj.rotation.to_array(),
                scale: obj.scale.to_array(),
                color: obj.color,
                visible: obj.visible,
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
                self.next_id = 0;

                // Load camera settings
                if let Some(target) = scene_data.camera_target {
                    self.camera.target = glam::Vec3::from_array(target);
                }
                if let Some(distance) = scene_data.camera_distance {
                    self.camera.distance = distance;
                }

                // Load objects
                for obj_data in scene_data.objects {
                    let obj = SceneObject {
                        id: self.next_id,
                        name: obj_data.name.clone(),
                        position: obj_data.position_vec(),
                        rotation: obj_data.rotation_vec(),
                        scale: obj_data.scale_vec(),
                        color: obj_data.color,
                        visible: obj_data.visible,
                        scripts: Vec::new(),
                    };
                    self.scene_objects.push(obj);
                    self.next_id += 1;
                }

                self.scene_manager.set_path(path.clone());
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
        self.next_id = 0;
        self.scene_manager.new_scene();
        log::info!("New scene created");
    }
}

//! Script creation and attachment dialogs.

use crate::editor::app::EditorApp;
use std::path::PathBuf;

impl EditorApp {
    /// Draw script creation dialog (opens native file save dialog)
    pub(crate) fn draw_script_dialog(&mut self, _ctx: &egui::Context) {
        if !self.show_script_dialog {
            return;
        }

        // Reset flag immediately
        self.show_script_dialog = false;

        // Open native file save dialog
        let default_dir = if let Some(ref project) = self.current_project {
            project.scripts_dir()
        } else {
            PathBuf::from("scripts")
        };

        // Ensure scripts directory exists
        let _ = std::fs::create_dir_all(&default_dir);

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Python Script", &["py"])
            .set_title("Create New Script")
            .set_file_name("new_script.py")
            .set_directory(&default_dir)
            .save_file()
        {
            // Ensure .py extension
            let path = if path.extension().is_none() {
                path.with_extension("py")
            } else {
                path
            };

            // Create empty script file with template
            self.create_script_at_path(&path);

            // Open in default editor if requested
            if self.open_script_in_editor {
                self.open_file_in_editor(&path);
            }

            // Attach to selected object if scripting is enabled
            #[cfg(feature = "scripting")]
            {
                if let Some(obj_id) = self.selection.first() {
                    match self.script_runtime.attach_script(path.clone(), obj_id) {
                        Ok(script_id) => {
                            if let Some(obj) =
                                self.scene_objects.iter_mut().find(|o| o.id == obj_id)
                            {
                                obj.scripts.push(script_id);
                            }
                            log::info!(
                                "Created and attached script {:?} to object {}",
                                path,
                                obj_id
                            );
                        }
                        Err(e) => {
                            log::error!("Failed to attach script: {}", e);
                        }
                    }
                }
            }

            #[cfg(not(feature = "scripting"))]
            {
                log::info!("Created script: {:?}", path);
            }
        }

        self.script_path_input.clear();
    }

    /// Open a file in the system's default editor
    fn open_file_in_editor(&self, path: &PathBuf) {
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = std::process::Command::new("cmd")
                .args(["/C", "start", "", path.to_str().unwrap_or("")])
                .spawn()
            {
                log::error!("Failed to open file in editor: {}", e);
            } else {
                log::info!("Opened {:?} in default editor", path);
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Err(e) = std::process::Command::new("open").arg(path).spawn() {
                log::error!("Failed to open file in editor: {}", e);
            } else {
                log::info!("Opened {:?} in default editor", path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(e) = std::process::Command::new("xdg-open").arg(path).spawn() {
                log::error!("Failed to open file in editor: {}", e);
            } else {
                log::info!("Opened {:?} in default editor", path);
            }
        }
    }

    fn create_script_at_path(&self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);

            // Ensure xtreme module files exist in the script directory
            self.ensure_xtreme_module(parent);
        }

        let script_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("script");

        let template = format!(
            r#"# {}
# Xtreme Engine Script

import xtreme

def _ready(ctx):
    """Called once when the script is attached"""
    xtreme.log_info("{} ready!")

def _update(ctx, delta):
    """Called every frame"""
    pass

def _physics_update(ctx, delta):
    """Called at fixed physics rate (optional)"""
    pass
"#,
            script_name, script_name
        );

        match std::fs::write(path, template) {
            Ok(_) => log::info!("Created script: {:?}", path),
            Err(e) => log::error!("Failed to create script: {}", e),
        }
    }

    /// Ensure xtreme.py and xtreme.pyi exist in the given directory
    pub(crate) fn ensure_xtreme_module(&self, dir: &std::path::Path) {
        let xtreme_py = dir.join("xtreme.py");
        let xtreme_pyi = dir.join("xtreme.pyi");

        // Create xtreme.py if it doesn't exist
        if !xtreme_py.exists() {
            let content = r#"# xtreme.py - Xtreme Engine Python API
# This module is injected by Rust at runtime via pyo3
# This file provides mock implementations for IDE support

from typing import Dict, List, Any, Optional

def get_position(ctx: Dict[str, Any], object_id: Optional[int] = None) -> List[float]:
    """Returns the position [x, y, z] of the object"""
    return ctx.get("position", [0.0, 0.0, 0.0])

def set_position(ctx: Dict[str, Any], x: float, y: float, z: float) -> None:
    """Sets the position of the object"""
    ctx["position"] = [x, y, z]
    ctx["_position_changed"] = True

def translate(ctx: Dict[str, Any], dx: float, dy: float, dz: float) -> None:
    """Moves the object by (dx, dy, dz)"""
    pos = ctx.get("position", [0.0, 0.0, 0.0])
    pos[0] += dx
    pos[1] += dy
    pos[2] += dz
    ctx["position"] = pos
    ctx["_position_changed"] = True

def get_rotation(ctx: Dict[str, Any]) -> List[float]:
    """Returns the rotation [x, y, z] in radians"""
    return ctx.get("rotation", [0.0, 0.0, 0.0])

def set_rotation(ctx: Dict[str, Any], x: float, y: float, z: float) -> None:
    """Sets the rotation of the object in radians"""
    ctx["rotation"] = [x, y, z]
    ctx["_rotation_changed"] = True

def rotate(ctx: Dict[str, Any], rx: float, ry: float, rz: float) -> None:
    """Rotates the object by (rx, ry, rz) in radians"""
    rot = ctx.get("rotation", [0.0, 0.0, 0.0])
    rot[0] += rx
    rot[1] += ry
    rot[2] += rz
    ctx["rotation"] = rot
    ctx["_rotation_changed"] = True

def get_scale(ctx: Dict[str, Any]) -> List[float]:
    """Returns the scale [x, y, z]"""
    return ctx.get("scale", [1.0, 1.0, 1.0])

def set_scale(ctx: Dict[str, Any], x: float, y: float, z: float) -> None:
    """Sets the scale of the object"""
    ctx["scale"] = [x, y, z]
    ctx["_scale_changed"] = True

def get_time(ctx: Dict[str, Any]) -> float:
    """Returns the elapsed time since play started"""
    return ctx.get("time", 0.0)

def is_key_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Returns true if the key is currently pressed"""
    input_state = ctx.get("input", {})
    keys = input_state.get("keys", [])
    return key in keys

def is_key_just_pressed(ctx: Dict[str, Any], key: str) -> bool:
    """Returns true if the key was just pressed this frame"""
    input_state = ctx.get("input", {})
    keys = input_state.get("keys_just_pressed", [])
    return key in keys

def log_info(message: str) -> None:
    """Log an info message"""
    print(f"[INFO] {message}")

def log_warn(message: str) -> None:
    """Log a warning message"""
    print(f"[WARN] {message}")

def log_error(message: str) -> None:
    """Log an error message"""
    print(f"[ERROR] {message}")
"#;
            if let Err(e) = std::fs::write(&xtreme_py, content) {
                log::error!("Failed to create xtreme.py: {}", e);
            } else {
                log::info!("Created xtreme.py module");
            }
        }

        // Create xtreme.pyi if it doesn't exist
        if !xtreme_pyi.exists() {
            let content = r#"# xtreme.pyi - Type stubs for IDE autocompletion
from typing import Dict, List, Any, Optional

def get_position(ctx: Dict[str, Any], object_id: Optional[int] = None) -> List[float]: ...
def set_position(ctx: Dict[str, Any], x: float, y: float, z: float) -> None: ...
def translate(ctx: Dict[str, Any], dx: float, dy: float, dz: float) -> None: ...
def get_rotation(ctx: Dict[str, Any]) -> List[float]: ...
def set_rotation(ctx: Dict[str, Any], x: float, y: float, z: float) -> None: ...
def rotate(ctx: Dict[str, Any], rx: float, ry: float, rz: float) -> None: ...
def get_scale(ctx: Dict[str, Any]) -> List[float]: ...
def set_scale(ctx: Dict[str, Any], x: float, y: float, z: float) -> None: ...
def get_time(ctx: Dict[str, Any]) -> float: ...
def is_key_pressed(ctx: Dict[str, Any], key: str) -> bool: ...
def is_key_just_pressed(ctx: Dict[str, Any], key: str) -> bool: ...
def log_info(message: str) -> None: ...
def log_warn(message: str) -> None: ...
def log_error(message: str) -> None: ...
"#;
            if let Err(e) = std::fs::write(&xtreme_pyi, content) {
                log::error!("Failed to create xtreme.pyi: {}", e);
            } else {
                log::info!("Created xtreme.pyi stubs");
            }
        }
    }

    /// Draw attach script dialog (opens native file open dialog)
    pub(crate) fn draw_attach_script_dialog(&mut self, _ctx: &egui::Context) {
        if !self.show_attach_script_dialog {
            return;
        }

        // Reset flag immediately
        self.show_attach_script_dialog = false;

        // Open native file open dialog
        let default_dir = if let Some(ref project) = self.current_project {
            project.scripts_dir()
        } else {
            PathBuf::from("scripts")
        };

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Python Script", &["py"])
            .set_title("Attach Existing Script")
            .set_directory(&default_dir)
            .pick_file()
        {
            // Ensure xtreme module exists in the script's directory
            if let Some(parent) = path.parent() {
                self.ensure_xtreme_module(parent);
            }

            // Attach to selected object if scripting is enabled
            #[cfg(feature = "scripting")]
            {
                if let Some(obj_id) = self.selection.first() {
                    match self.script_runtime.attach_script(path.clone(), obj_id) {
                        Ok(script_id) => {
                            if let Some(obj) =
                                self.scene_objects.iter_mut().find(|o| o.id == obj_id)
                            {
                                obj.scripts.push(script_id);
                            }
                            log::info!("Attached script {:?} to object {}", path, obj_id);
                        }
                        Err(e) => {
                            log::error!("Failed to attach script: {}", e);
                        }
                    }
                }
            }

            #[cfg(not(feature = "scripting"))]
            {
                log::info!("Would attach script: {:?}", path);
            }
        }
    }
}

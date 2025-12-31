//! # Script Runtime
//!
//! Manages script execution, lifecycle, and Python interpreter.

use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use super::api::ScriptContext;
use super::script::{Script, ScriptError, ScriptId, ScriptInstance};

/// Runtime for managing Python scripts
pub struct ScriptRuntime {
    /// All loaded scripts
    scripts: HashMap<ScriptId, Script>,
    /// Compiled script instances
    instances: HashMap<ScriptId, ScriptInstance>,
    /// Scripts waiting for _ready call
    pending_ready: Vec<ScriptId>,
    /// Next script ID
    next_id: ScriptId,
    /// Whether runtime is initialized
    initialized: bool,
    /// Scripts directory path (from project)
    scripts_dir: Option<PathBuf>,
}

impl Default for ScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptRuntime {
    /// Create a new script runtime
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            instances: HashMap::new(),
            pending_ready: Vec::new(),
            next_id: 1,
            initialized: false,
            scripts_dir: None,
        }
    }

    /// Set the scripts directory (call before initialize or attach_script)
    pub fn set_scripts_dir(&mut self, dir: PathBuf) {
        // If already initialized, add new path to Python
        if self.initialized {
            Python::attach(|py| {
                if let Ok(sys) = py.import("sys") {
                    if let Ok(path) = sys.getattr("path") {
                        let dir_str = dir.to_string_lossy().to_string();
                        let _ = path.call_method1("insert", (0, dir_str));
                        log::info!("Added scripts directory to Python path: {:?}", dir);
                    }
                }
            });
        }
        self.scripts_dir = Some(dir);
    }

    /// Initialize the Python interpreter
    pub fn initialize(&mut self) -> Result<(), ScriptError> {
        if self.initialized {
            return Ok(());
        }

        // pyo3 auto-initializes with the feature, but we can do setup here
        Python::attach(|py| {
            // Add custom modules or setup paths
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // Add scripts directory to Python path
            if let Some(ref scripts_dir) = self.scripts_dir {
                let dir_str = scripts_dir.to_string_lossy().to_string();
                path.call_method1("insert", (0, &dir_str))?;
                log::info!("Python path: {:?}", scripts_dir);
            } else {
                // Fallback to local scripts folder
                path.call_method1("append", ("scripts",))?;
            }

            log::info!("Python scripting runtime initialized");
            Ok::<(), PyErr>(())
        })?;

        self.initialized = true;
        Ok(())
    }

    /// Load a script from file and attach to an object
    pub fn attach_script(
        &mut self,
        path: PathBuf,
        object_id: u32,
    ) -> Result<ScriptId, ScriptError> {
        if !self.initialized {
            self.initialize()?;
        }

        let id = self.next_id;
        self.next_id += 1;

        let script = Script::from_file(id, path, object_id)?;
        log::info!(
            "Loaded script '{}' (id={}) for object {}",
            script.name,
            id,
            object_id
        );

        // Compile the script
        Python::attach(|py| {
            let instance = ScriptInstance::new(py, &script)?;
            self.instances.insert(id, instance);
            Ok::<(), ScriptError>(())
        })?;

        self.scripts.insert(id, script);
        self.pending_ready.push(id);

        Ok(id)
    }

    /// Attach a script from source code
    pub fn attach_script_source(
        &mut self,
        name: impl Into<String>,
        source: impl Into<String>,
        object_id: u32,
    ) -> Result<ScriptId, ScriptError> {
        if !self.initialized {
            self.initialize()?;
        }

        let id = self.next_id;
        self.next_id += 1;

        let script = Script::from_source(id, name, source, object_id);
        log::info!(
            "Attached inline script '{}' (id={}) to object {}",
            script.name,
            id,
            object_id
        );

        // Compile the script
        Python::attach(|py| {
            let instance = ScriptInstance::new(py, &script)?;
            self.instances.insert(id, instance);
            Ok::<(), ScriptError>(())
        })?;

        self.scripts.insert(id, script);
        self.pending_ready.push(id);

        Ok(id)
    }

    /// Detach a script from an object
    pub fn detach_script(&mut self, script_id: ScriptId) {
        self.scripts.remove(&script_id);
        self.instances.remove(&script_id);
        self.pending_ready.retain(|id| *id != script_id);
        log::info!("Detached script {}", script_id);
    }

    /// Get all scripts attached to an object
    pub fn get_scripts_for_object(&self, object_id: u32) -> Vec<ScriptId> {
        self.scripts
            .iter()
            .filter(|(_, s)| s.object_id == object_id)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Call _ready on all pending scripts
    pub fn call_ready(&mut self, context: &mut ScriptContext) -> Result<(), ScriptError> {
        let pending: Vec<ScriptId> = self.pending_ready.drain(..).collect();

        Python::attach(|py| {
            for id in pending {
                if let Some(instance) = self.instances.get_mut(&id) {
                    let ctx_dict = context.to_py_dict(py);
                    if let Err(e) = instance.call_ready(py, &ctx_dict) {
                        log::error!("Script {} _ready failed: {}", id, e);
                    }
                }
            }
            Ok(())
        })
    }

    /// Call _update on all active scripts
    pub fn call_update(&self, context: &mut ScriptContext, delta: f32) -> Result<(), ScriptError> {
        Python::attach(|py| {
            for (id, instance) in &self.instances {
                // Set current object ID in context
                context.set_current_object(instance.object_id);
                let ctx_dict = context.to_py_dict(py);
                if let Err(e) = instance.call_update(py, &ctx_dict, delta) {
                    log::error!("Script {} _update failed: {}", id, e);
                }
                // Read changes back from Python dict
                context.read_changes_from_dict(&ctx_dict);
            }
            Ok(())
        })
    }

    /// Call _physics_update on all active scripts
    pub fn call_physics_update(
        &self,
        context: &mut ScriptContext,
        delta: f32,
    ) -> Result<(), ScriptError> {
        Python::attach(|py| {
            for (id, instance) in &self.instances {
                context.set_current_object(instance.object_id);
                let ctx_dict = context.to_py_dict(py);
                if let Err(e) = instance.call_physics_update(py, &ctx_dict, delta) {
                    log::error!("Script {} _physics_update failed: {}", id, e);
                }
            }
            Ok(())
        })
    }

    /// Reload a script from disk
    pub fn reload_script(&mut self, script_id: ScriptId) -> Result<(), ScriptError> {
        let script = self
            .scripts
            .get_mut(&script_id)
            .ok_or_else(|| ScriptError::NotFound(format!("Script {}", script_id)))?;

        script.reload()?;

        // Recompile
        Python::attach(|py| {
            let instance = ScriptInstance::new(py, script)?;
            self.instances.insert(script_id, instance);
            Ok::<(), ScriptError>(())
        })?;

        self.pending_ready.push(script_id);
        log::info!("Reloaded script {}", script_id);

        Ok(())
    }

    /// Reload all scripts
    pub fn reload_all(&mut self) -> Result<(), ScriptError> {
        let ids: Vec<ScriptId> = self.scripts.keys().copied().collect();
        for id in ids {
            self.reload_script(id)?;
        }
        Ok(())
    }

    /// Get script count
    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }

    /// Check if a script exists
    pub fn has_script(&self, script_id: ScriptId) -> bool {
        self.scripts.contains_key(&script_id)
    }

    /// Get script info
    pub fn get_script(&self, script_id: ScriptId) -> Option<&Script> {
        self.scripts.get(&script_id)
    }
}

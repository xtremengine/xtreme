//! # Script Definition
//!
//! Types and structures for Python scripts attached to objects.

use std::path::PathBuf;
use std::ffi::CString;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use thiserror::Error;

/// Unique identifier for a script instance
pub type ScriptId = u32;

/// Errors that can occur during script operations
#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Failed to load script: {0}")]
    LoadError(String),
    #[error("Python error: {0}")]
    PythonError(String),
    #[error("Script not found: {0}")]
    NotFound(String),
    #[error("Script compilation error: {0}")]
    CompileError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
}

impl From<PyErr> for ScriptError {
    fn from(err: PyErr) -> Self {
        ScriptError::PythonError(err.to_string())
    }
}

/// A Python script that can be attached to objects
#[derive(Clone, Debug)]
pub struct Script {
    /// Unique script ID
    pub id: ScriptId,
    /// Script name (usually filename without extension)
    pub name: String,
    /// Full path to the script file
    pub path: PathBuf,
    /// Python source code
    pub source: String,
    /// Object ID this script is attached to
    pub object_id: u32,
    /// Whether the script has been initialized
    pub initialized: bool,
}

impl Script {
    /// Create a new script from file path
    pub fn from_file(id: ScriptId, path: PathBuf, object_id: u32) -> Result<Self, ScriptError> {
        let source = std::fs::read_to_string(&path)
            .map_err(|e| ScriptError::LoadError(format!("{}: {}", path.display(), e)))?;

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            id,
            name,
            path,
            source,
            object_id,
            initialized: false,
        })
    }

    /// Create a new script from source code
    pub fn from_source(id: ScriptId, name: impl Into<String>, source: impl Into<String>, object_id: u32) -> Self {
        Self {
            id,
            name: name.into(),
            path: PathBuf::new(),
            source: source.into(),
            object_id,
            initialized: false,
        }
    }

    /// Reload the script from disk
    pub fn reload(&mut self) -> Result<(), ScriptError> {
        if self.path.exists() {
            self.source = std::fs::read_to_string(&self.path)
                .map_err(|e| ScriptError::LoadError(e.to_string()))?;
            self.initialized = false;
        }
        Ok(())
    }
}

/// A compiled script instance ready for execution
pub struct ScriptInstance {
    /// Script ID
    #[allow(dead_code)]
    pub id: ScriptId,
    /// Object ID this is attached to
    pub object_id: u32,
    /// Python module containing the script
    module: Py<PyAny>,
    /// Whether _ready has been called
    ready_called: bool,
}

impl ScriptInstance {
    /// Create a new script instance from source
    pub fn new(py: Python<'_>, script: &Script) -> Result<Self, ScriptError> {
        // Convert strings to CString for the new pyo3 API
        let source = CString::new(script.source.as_str())
            .map_err(|e| ScriptError::CompileError(format!("Invalid source: {}", e)))?;
        let file_name = CString::new(script.path.to_string_lossy().as_bytes())
            .map_err(|e| ScriptError::CompileError(format!("Invalid path: {}", e)))?;
        let module_name = CString::new(script.name.as_str())
            .map_err(|e| ScriptError::CompileError(format!("Invalid name: {}", e)))?;

        // Create a new module for this script
        let module = PyModule::from_code(
            py,
            &source,
            &file_name,
            &module_name,
        )?;

        Ok(Self {
            id: script.id,
            object_id: script.object_id,
            module: module.unbind().into(),
            ready_called: false,
        })
    }

    /// Call the _ready function if it exists
    pub fn call_ready(&mut self, py: Python<'_>, context: &Bound<'_, PyDict>) -> Result<(), ScriptError> {
        if self.ready_called {
            return Ok(());
        }

        let module = self.module.bind(py);
        if let Ok(ready_fn) = module.getattr("_ready") {
            ready_fn.call1((context,))?;
        }

        self.ready_called = true;
        Ok(())
    }

    /// Call the _update function if it exists
    pub fn call_update(&self, py: Python<'_>, context: &Bound<'_, PyDict>, delta: f32) -> Result<(), ScriptError> {
        let module = self.module.bind(py);
        if let Ok(update_fn) = module.getattr("_update") {
            update_fn.call1((context, delta))?;
        }
        Ok(())
    }

    /// Call the _physics_update function if it exists
    pub fn call_physics_update(&self, py: Python<'_>, context: &Bound<'_, PyDict>, delta: f32) -> Result<(), ScriptError> {
        let module = self.module.bind(py);
        if let Ok(physics_fn) = module.getattr("_physics_update") {
            physics_fn.call1((context, delta))?;
        }
        Ok(())
    }

    /// Call a custom function by name
    #[allow(dead_code)]
    pub fn call_function<'py, A>(&self, py: Python<'py>, name: &str, args: A) -> Result<Bound<'py, PyAny>, ScriptError>
    where
        A: IntoPyObject<'py>,
        A::Error: std::fmt::Debug,
    {
        let module = self.module.bind(py);
        let func = module.getattr(name)
            .map_err(|_| ScriptError::NotFound(name.to_string()))?;
        let result = func.call1((args,))?;
        Ok(result)
    }

    /// Check if a function exists in the script
    #[allow(dead_code)]
    pub fn has_function(&self, py: Python<'_>, name: &str) -> bool {
        let module = self.module.bind(py);
        module.getattr(name).is_ok()
    }
}

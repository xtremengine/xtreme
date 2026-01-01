//! # Script API
//!
//! Defines the API exposed to Python scripts for manipulating game objects.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

/// Input state accessible to scripts
#[derive(Clone, Debug, Default)]
pub struct InputState {
    /// Currently pressed keys
    pub keys_pressed: Vec<String>,
    /// Keys just pressed this frame
    pub keys_just_pressed: Vec<String>,
    /// Keys just released this frame
    pub keys_just_released: Vec<String>,
    /// Mouse position (x, y)
    pub mouse_position: (f32, f32),
    /// Mouse buttons pressed
    pub mouse_buttons: Vec<u8>,
}

/// Transform data for an object
#[derive(Clone, Debug, Default)]
pub struct ObjectTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

/// Animation data for an object
#[derive(Clone, Debug, Default)]
pub struct AnimatorData {
    /// Available animation names
    pub animations: Vec<String>,
    /// Current animation name
    pub current_animation: String,
    /// Whether animation is playing
    pub playing: bool,
    /// Playback speed
    pub speed: f32,
}

/// Animation change request
#[derive(Clone, Debug)]
pub enum AnimationChange {
    /// Play an animation by name
    Play(String),
    /// Stop animation
    Stop,
    /// Set speed
    SetSpeed(f32),
}

/// Context provided to scripts during execution
pub struct ScriptContext {
    /// Current object ID being processed
    current_object: u32,
    /// Object transforms (id -> transform)
    transforms: HashMap<u32, ObjectTransform>,
    /// Object visibility
    visibility: HashMap<u32, bool>,
    /// Object names
    names: HashMap<u32, String>,
    /// Input state
    input: InputState,
    /// Time since start
    time: f32,
    /// Queued position changes
    position_changes: Vec<(u32, [f32; 3])>,
    /// Queued rotation changes
    rotation_changes: Vec<(u32, [f32; 3])>,
    /// Queued scale changes
    scale_changes: Vec<(u32, [f32; 3])>,
    /// Objects to spawn
    spawn_queue: Vec<SpawnRequest>,
    /// Objects to destroy
    destroy_queue: Vec<u32>,
    /// Animator data per object
    animator_data: HashMap<u32, AnimatorData>,
    /// Queued animation changes
    animation_changes: Vec<(u32, AnimationChange)>,
}

/// Request to spawn a new object
#[derive(Clone, Debug)]
pub struct SpawnRequest {
    /// Prefab name or object template
    pub template: String,
    /// Position to spawn at
    pub position: [f32; 3],
}

impl Default for ScriptContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptContext {
    /// Create a new script context
    pub fn new() -> Self {
        Self {
            current_object: 0,
            transforms: HashMap::new(),
            visibility: HashMap::new(),
            names: HashMap::new(),
            input: InputState::default(),
            time: 0.0,
            position_changes: Vec::new(),
            rotation_changes: Vec::new(),
            scale_changes: Vec::new(),
            spawn_queue: Vec::new(),
            destroy_queue: Vec::new(),
            animator_data: HashMap::new(),
            animation_changes: Vec::new(),
        }
    }

    /// Set the current object being processed
    pub fn set_current_object(&mut self, id: u32) {
        self.current_object = id;
    }

    /// Update time
    pub fn set_time(&mut self, time: f32) {
        self.time = time;
    }

    /// Update input state
    pub fn set_input(&mut self, input: InputState) {
        self.input = input;
    }

    /// Register an object's transform
    pub fn register_object(
        &mut self,
        id: u32,
        name: String,
        transform: ObjectTransform,
        visible: bool,
    ) {
        self.transforms.insert(id, transform);
        self.names.insert(id, name);
        self.visibility.insert(id, visible);
    }

    /// Clear registered objects (call before each frame)
    pub fn clear_objects(&mut self) {
        self.transforms.clear();
        self.names.clear();
        self.visibility.clear();
        self.animator_data.clear();
    }

    /// Register animator data for an object
    pub fn register_animator(&mut self, id: u32, data: AnimatorData) {
        self.animator_data.insert(id, data);
    }

    /// Get queued position changes
    pub fn drain_position_changes(&mut self) -> Vec<(u32, [f32; 3])> {
        std::mem::take(&mut self.position_changes)
    }

    /// Get queued rotation changes
    pub fn drain_rotation_changes(&mut self) -> Vec<(u32, [f32; 3])> {
        std::mem::take(&mut self.rotation_changes)
    }

    /// Get queued scale changes
    pub fn drain_scale_changes(&mut self) -> Vec<(u32, [f32; 3])> {
        std::mem::take(&mut self.scale_changes)
    }

    /// Get spawn queue
    pub fn drain_spawn_queue(&mut self) -> Vec<SpawnRequest> {
        std::mem::take(&mut self.spawn_queue)
    }

    /// Get destroy queue
    pub fn drain_destroy_queue(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.destroy_queue)
    }

    /// Get animation changes
    pub fn drain_animation_changes(&mut self) -> Vec<(u32, AnimationChange)> {
        std::mem::take(&mut self.animation_changes)
    }

    /// Read changes back from Python dict after script execution
    pub fn read_changes_from_dict(&mut self, dict: &Bound<'_, PyDict>) {
        let object_id = self.current_object;

        // Check if position changed
        if let Ok(Some(changed)) = dict.get_item("_position_changed") {
            if changed.extract::<bool>().unwrap_or(false) {
                if let Ok(Some(pos)) = dict.get_item("position") {
                    if let Ok(pos_vec) = pos.extract::<Vec<f32>>() {
                        if pos_vec.len() == 3 {
                            let new_pos = [pos_vec[0], pos_vec[1], pos_vec[2]];
                            self.position_changes.push((object_id, new_pos));
                            // Update transforms so subsequent scripts see the change
                            if let Some(transform) = self.transforms.get_mut(&object_id) {
                                transform.position = new_pos;
                            }
                        }
                    }
                }
            }
        }

        // Check if rotation changed
        if let Ok(Some(changed)) = dict.get_item("_rotation_changed") {
            if changed.extract::<bool>().unwrap_or(false) {
                if let Ok(Some(rot)) = dict.get_item("rotation") {
                    if let Ok(rot_vec) = rot.extract::<Vec<f32>>() {
                        if rot_vec.len() == 3 {
                            let new_rot = [rot_vec[0], rot_vec[1], rot_vec[2]];
                            self.rotation_changes.push((object_id, new_rot));
                            // Update transforms so subsequent scripts see the change
                            if let Some(transform) = self.transforms.get_mut(&object_id) {
                                transform.rotation = new_rot;
                            }
                        }
                    }
                }
            }
        }

        // Check if scale changed
        if let Ok(Some(changed)) = dict.get_item("_scale_changed") {
            if changed.extract::<bool>().unwrap_or(false) {
                if let Ok(Some(scale)) = dict.get_item("scale") {
                    if let Ok(scale_vec) = scale.extract::<Vec<f32>>() {
                        if scale_vec.len() == 3 {
                            let new_scale = [scale_vec[0], scale_vec[1], scale_vec[2]];
                            self.scale_changes.push((object_id, new_scale));
                            // Update transforms so subsequent scripts see the change
                            if let Some(transform) = self.transforms.get_mut(&object_id) {
                                transform.scale = new_scale;
                            }
                        }
                    }
                }
            }
        }

        // Check if animation changed
        if let Ok(Some(changed)) = dict.get_item("_animation_changed") {
            if changed.extract::<bool>().unwrap_or(false) {
                // Check for play animation
                if let Ok(Some(anim_name)) = dict.get_item("_play_animation") {
                    if let Ok(name) = anim_name.extract::<String>() {
                        self.animation_changes
                            .push((object_id, AnimationChange::Play(name.clone())));
                        // Update local animator data so subsequent scripts see the change
                        if let Some(animator) = self.animator_data.get_mut(&object_id) {
                            animator.current_animation = name;
                            animator.playing = true;
                        }
                    }
                }

                // Check for stop animation
                if let Ok(Some(stop)) = dict.get_item("_stop_animation") {
                    if stop.extract::<bool>().unwrap_or(false) {
                        self.animation_changes
                            .push((object_id, AnimationChange::Stop));
                        if let Some(animator) = self.animator_data.get_mut(&object_id) {
                            animator.playing = false;
                        }
                    }
                }

                // Check for speed change
                if let Ok(Some(speed)) = dict.get_item("_animation_speed") {
                    if let Ok(speed_val) = speed.extract::<f32>() {
                        self.animation_changes
                            .push((object_id, AnimationChange::SetSpeed(speed_val)));
                        if let Some(animator) = self.animator_data.get_mut(&object_id) {
                            animator.speed = speed_val;
                        }
                    }
                }
            }
        }
    }

    /// Convert context to Python dictionary for passing to scripts
    pub fn to_py_dict<'py>(&self, py: Python<'py>) -> Bound<'py, PyDict> {
        let dict = PyDict::new(py);

        // Current object ID
        let _ = dict.set_item("object_id", self.current_object);
        let _ = dict.set_item("time", self.time);

        // Current object's transform
        if let Some(transform) = self.transforms.get(&self.current_object) {
            if let Ok(pos) = PyList::new(py, transform.position) {
                let _ = dict.set_item("position", pos);
            }
            if let Ok(rot) = PyList::new(py, transform.rotation) {
                let _ = dict.set_item("rotation", rot);
            }
            if let Ok(scale) = PyList::new(py, transform.scale) {
                let _ = dict.set_item("scale", scale);
            }
        }

        // Current object's visibility
        if let Some(visible) = self.visibility.get(&self.current_object) {
            let _ = dict.set_item("visible", *visible);
        }

        // Current object's name
        if let Some(name) = self.names.get(&self.current_object) {
            let _ = dict.set_item("name", name.as_str());
        }

        // Input state
        let input_dict = PyDict::new(py);
        if let Ok(keys) = PyList::new(py, &self.input.keys_pressed) {
            let _ = input_dict.set_item("keys", keys);
        }
        if let Ok(just_pressed) = PyList::new(py, &self.input.keys_just_pressed) {
            let _ = input_dict.set_item("keys_just_pressed", just_pressed);
        }
        if let Ok(just_released) = PyList::new(py, &self.input.keys_just_released) {
            let _ = input_dict.set_item("keys_just_released", just_released);
        }
        let _ = input_dict.set_item("mouse_x", self.input.mouse_position.0);
        let _ = input_dict.set_item("mouse_y", self.input.mouse_position.1);
        if let Ok(mouse_buttons) = PyList::new(py, &self.input.mouse_buttons) {
            let _ = input_dict.set_item("mouse_buttons", mouse_buttons);
        }
        let _ = dict.set_item("input", input_dict);

        // Animation data for current object
        if let Some(animator) = self.animator_data.get(&self.current_object) {
            if let Ok(anims) = PyList::new(py, &animator.animations) {
                let _ = dict.set_item("_animations", anims);
            }
            let _ = dict.set_item("_current_animation", animator.current_animation.as_str());
            let _ = dict.set_item("_animation_playing", animator.playing);
            let _ = dict.set_item("_animation_speed", animator.speed);
        }

        dict
    }
}

// Python module for game API
#[pymodule]
#[pyo3(name = "xtreme")]
fn xtreme_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    /// Get the position of an object
    #[pyfunction]
    #[pyo3(signature = (ctx, object_id=None))]
    fn get_position(ctx: &Bound<'_, PyDict>, object_id: Option<u32>) -> PyResult<Vec<f32>> {
        let _id = object_id.unwrap_or_else(|| {
            ctx.get_item("object_id")
                .ok()
                .flatten()
                .and_then(|v| v.extract().ok())
                .unwrap_or(0)
        });

        // Return current position from context
        if let Some(pos) = ctx.get_item("position").ok().flatten() {
            let pos: Vec<f32> = pos.extract()?;
            return Ok(pos);
        }

        Ok(vec![0.0, 0.0, 0.0])
    }

    /// Set position directly
    #[pyfunction]
    fn set_position(ctx: &Bound<'_, PyDict>, x: f32, y: f32, z: f32) -> PyResult<()> {
        let py = ctx.py();
        let new_pos = vec![x, y, z];
        if let Ok(new_pos_list) = PyList::new(py, &new_pos) {
            ctx.set_item("position", new_pos_list)?;
            ctx.set_item("_position_changed", true)?;
        }
        Ok(())
    }

    /// Translate an object by delta
    #[pyfunction]
    fn translate(ctx: &Bound<'_, PyDict>, dx: f32, dy: f32, dz: f32) -> PyResult<()> {
        // Get current position and add delta
        if let Some(pos_item) = ctx.get_item("position").ok().flatten() {
            let pos: Vec<f32> = pos_item.extract()?;
            let new_pos = vec![pos[0] + dx, pos[1] + dy, pos[2] + dz];
            let py = ctx.py();
            if let Ok(new_pos_list) = PyList::new(py, &new_pos) {
                ctx.set_item("position", new_pos_list)?;
                ctx.set_item("_position_changed", true)?;
            }
        }
        Ok(())
    }

    /// Get elapsed time since play started
    #[pyfunction]
    fn get_time(ctx: &Bound<'_, PyDict>) -> PyResult<f32> {
        if let Some(time) = ctx.get_item("time").ok().flatten() {
            return time.extract();
        }
        Ok(0.0)
    }

    /// Rotate an object
    #[pyfunction]
    fn rotate(ctx: &Bound<'_, PyDict>, rx: f32, ry: f32, rz: f32) -> PyResult<()> {
        if let Some(rot_item) = ctx.get_item("rotation").ok().flatten() {
            let rot: Vec<f32> = rot_item.extract()?;
            let new_rot = vec![rot[0] + rx, rot[1] + ry, rot[2] + rz];
            let py = ctx.py();
            if let Ok(new_rot_list) = PyList::new(py, &new_rot) {
                ctx.set_item("rotation", new_rot_list)?;
                ctx.set_item("_rotation_changed", true)?;
            }
        }
        Ok(())
    }

    /// Check if a key is pressed
    #[pyfunction]
    fn is_key_pressed(ctx: &Bound<'_, PyDict>, key: &str) -> PyResult<bool> {
        if let Some(input) = ctx.get_item("input").ok().flatten() {
            let input: &Bound<'_, PyDict> = input.cast()?;
            if let Some(keys) = input.get_item("keys").ok().flatten() {
                let keys: Vec<String> = keys.extract()?;
                return Ok(keys.contains(&key.to_string()));
            }
        }
        Ok(false)
    }

    /// Check if a key was just pressed this frame
    #[pyfunction]
    fn is_key_just_pressed(ctx: &Bound<'_, PyDict>, key: &str) -> PyResult<bool> {
        if let Some(input) = ctx.get_item("input").ok().flatten() {
            let input: &Bound<'_, PyDict> = input.cast()?;
            if let Some(keys) = input.get_item("keys_just_pressed").ok().flatten() {
                let keys: Vec<String> = keys.extract()?;
                return Ok(keys.contains(&key.to_string()));
            }
        }
        Ok(false)
    }

    /// Log a message
    #[pyfunction]
    fn log_info(message: &str) {
        log::info!("[Script] {}", message);
    }

    /// Log a warning
    #[pyfunction]
    fn log_warn(message: &str) {
        log::warn!("[Script] {}", message);
    }

    /// Log an error
    #[pyfunction]
    fn log_error(message: &str) {
        log::error!("[Script] {}", message);
    }

    // Animation functions

    /// Get list of available animations for current object
    #[pyfunction]
    fn get_animations(ctx: &Bound<'_, PyDict>) -> PyResult<Vec<String>> {
        if let Some(anims) = ctx.get_item("_animations").ok().flatten() {
            return anims.extract();
        }
        Ok(Vec::new())
    }

    /// Get current animation name
    #[pyfunction]
    fn get_current_animation(ctx: &Bound<'_, PyDict>) -> PyResult<String> {
        if let Some(anim) = ctx.get_item("_current_animation").ok().flatten() {
            return anim.extract();
        }
        Ok(String::new())
    }

    /// Play an animation by name
    #[pyfunction]
    fn play_animation(ctx: &Bound<'_, PyDict>, name: &str) -> PyResult<()> {
        ctx.set_item("_play_animation", name)?;
        ctx.set_item("_animation_changed", true)?;
        Ok(())
    }

    /// Stop the current animation
    #[pyfunction]
    fn stop_animation(ctx: &Bound<'_, PyDict>) -> PyResult<()> {
        ctx.set_item("_stop_animation", true)?;
        ctx.set_item("_animation_changed", true)?;
        Ok(())
    }

    /// Set animation playback speed
    #[pyfunction]
    fn set_animation_speed(ctx: &Bound<'_, PyDict>, speed: f32) -> PyResult<()> {
        ctx.set_item("_animation_speed", speed)?;
        ctx.set_item("_animation_changed", true)?;
        Ok(())
    }

    /// Check if animation is currently playing
    #[pyfunction]
    fn is_animation_playing(ctx: &Bound<'_, PyDict>) -> PyResult<bool> {
        if let Some(playing) = ctx.get_item("_animation_playing").ok().flatten() {
            return playing.extract();
        }
        Ok(false)
    }

    m.add_function(wrap_pyfunction!(get_position, m)?)?;
    m.add_function(wrap_pyfunction!(set_position, m)?)?;
    m.add_function(wrap_pyfunction!(translate, m)?)?;
    m.add_function(wrap_pyfunction!(get_time, m)?)?;
    m.add_function(wrap_pyfunction!(rotate, m)?)?;
    m.add_function(wrap_pyfunction!(is_key_pressed, m)?)?;
    m.add_function(wrap_pyfunction!(is_key_just_pressed, m)?)?;
    m.add_function(wrap_pyfunction!(log_info, m)?)?;
    m.add_function(wrap_pyfunction!(log_warn, m)?)?;
    m.add_function(wrap_pyfunction!(log_error, m)?)?;
    // Animation functions
    m.add_function(wrap_pyfunction!(get_animations, m)?)?;
    m.add_function(wrap_pyfunction!(get_current_animation, m)?)?;
    m.add_function(wrap_pyfunction!(play_animation, m)?)?;
    m.add_function(wrap_pyfunction!(stop_animation, m)?)?;
    m.add_function(wrap_pyfunction!(set_animation_speed, m)?)?;
    m.add_function(wrap_pyfunction!(is_animation_playing, m)?)?;

    Ok(())
}

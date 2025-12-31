//! # Navigation Mesh (Placeholder)
//!
//! TODO: Full navmesh implementation

use glam::Vec3;
use crate::core::Component;

/// Navigation mesh (placeholder)
pub struct NavMesh {
    _placeholder: (),
}

impl NavMesh {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for NavMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation agent component
#[derive(Clone, Debug)]
pub struct NavAgent {
    pub speed: f32,
    pub acceleration: f32,
    pub radius: f32,
    pub height: f32,
    pub destination: Option<Vec3>,
    pub current_path: Vec<Vec3>,
    pub path_index: usize,
}

impl NavAgent {
    pub fn new() -> Self {
        Self {
            speed: 5.0,
            acceleration: 10.0,
            radius: 0.5,
            height: 2.0,
            destination: None,
            current_path: Vec::new(),
            path_index: 0,
        }
    }

    pub fn set_destination(&mut self, dest: Vec3) {
        self.destination = Some(dest);
        self.path_index = 0;
    }

    pub fn clear_destination(&mut self) {
        self.destination = None;
        self.current_path.clear();
        self.path_index = 0;
    }

    pub fn has_path(&self) -> bool {
        !self.current_path.is_empty()
    }

    pub fn current_waypoint(&self) -> Option<Vec3> {
        self.current_path.get(self.path_index).copied()
    }

    pub fn advance_waypoint(&mut self) {
        if self.path_index < self.current_path.len() {
            self.path_index += 1;
        }
    }

    pub fn is_path_complete(&self) -> bool {
        self.path_index >= self.current_path.len()
    }
}

impl Default for NavAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for NavAgent {}

//! # Material System

use glam::Vec4;

/// Material properties
pub struct Material {
    pub base_color: Vec4,
    pub metallic: f32,
    pub roughness: f32,
}

impl Material {
    pub fn new() -> Self {
        Self {
            base_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            metallic: 0.0,
            roughness: 0.5,
        }
    }

    pub fn with_color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.base_color = Vec4::new(r, g, b, 1.0);
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new()
    }
}

/// Shader handle
pub struct Shader {
    _placeholder: (),
}

impl Shader {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for Shader {
    fn default() -> Self {
        Self::new()
    }
}

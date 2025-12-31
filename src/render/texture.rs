//! # Texture System

/// Texture handle
pub struct Texture {
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// Texture sampler settings
pub struct Sampler {
    _placeholder: (),
}

impl Sampler {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

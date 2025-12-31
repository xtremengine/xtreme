//! # Material System
//!
//! GPU material management with texture support.

use super::texture::{Texture, TextureError};
use glam::{Vec3, Vec4};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Material error types
#[derive(Error, Debug)]
pub enum MaterialError {
    #[error("Texture error: {0}")]
    TextureError(#[from] TextureError),
    #[error("Material not found: {0}")]
    NotFound(MaterialId),
    #[error("Texture not found: {0}")]
    TextureNotFound(TextureId),
}

/// Unique texture identifier
pub type TextureId = u32;

/// Unique material identifier
pub type MaterialId = u32;

/// Material properties for PBR rendering
#[derive(Clone, Debug)]
pub struct Material {
    /// Material name for debugging
    pub name: String,
    /// Base color (used if no texture)
    pub base_color: Vec4,
    /// Optional diffuse/albedo texture
    pub base_color_texture: Option<TextureId>,
    /// Metallic factor (0.0 = dielectric, 1.0 = metal)
    pub metallic: f32,
    /// Roughness factor (0.0 = smooth, 1.0 = rough)
    pub roughness: f32,
    /// Emissive color (self-illumination)
    pub emissive: Vec3,
    /// Alpha cutoff for transparency
    pub alpha_cutoff: f32,
    /// Whether material uses transparency
    pub transparent: bool,
}

impl Material {
    /// Create a new material with default values
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            base_color: Vec4::ONE,
            base_color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            emissive: Vec3::ZERO,
            alpha_cutoff: 0.5,
            transparent: false,
        }
    }

    /// Set base color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.base_color = Vec4::new(r, g, b, a);
        self
    }

    /// Set base color from Vec4
    pub fn with_base_color(mut self, color: Vec4) -> Self {
        self.base_color = color;
        self
    }

    /// Set diffuse texture
    pub fn with_texture(mut self, texture_id: TextureId) -> Self {
        self.base_color_texture = Some(texture_id);
        self
    }

    /// Set metallic factor
    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic.clamp(0.0, 1.0);
        self
    }

    /// Set roughness factor
    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness.clamp(0.0, 1.0);
        self
    }

    /// Set emissive color
    pub fn with_emissive(mut self, emissive: Vec3) -> Self {
        self.emissive = emissive;
        self
    }

    /// Enable transparency
    pub fn with_transparency(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Check if material has a texture
    pub fn has_texture(&self) -> bool {
        self.base_color_texture.is_some()
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Material uniform data for GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    /// Base color RGBA
    pub base_color: [f32; 4],
    /// Emissive RGB + metallic
    pub emissive_metallic: [f32; 4],
    /// Roughness + alpha_cutoff + flags + padding
    pub roughness_alpha_flags: [f32; 4],
}

impl MaterialUniform {
    /// Create uniform data from material
    pub fn from_material(material: &Material) -> Self {
        Self {
            base_color: material.base_color.into(),
            emissive_metallic: [
                material.emissive.x,
                material.emissive.y,
                material.emissive.z,
                material.metallic,
            ],
            roughness_alpha_flags: [
                material.roughness,
                material.alpha_cutoff,
                if material.has_texture() { 1.0 } else { 0.0 },
                if material.transparent { 1.0 } else { 0.0 },
            ],
        }
    }
}

/// Manages materials and textures
pub struct MaterialManager {
    /// Loaded textures by ID
    textures: HashMap<TextureId, Texture>,
    /// Texture bind groups by ID
    texture_bind_groups: HashMap<TextureId, wgpu::BindGroup>,
    /// Defined materials by ID
    materials: HashMap<MaterialId, Material>,
    /// Next texture ID
    next_texture_id: TextureId,
    /// Next material ID
    next_material_id: MaterialId,
    /// Default white texture ID
    default_texture_id: TextureId,
    /// Texture bind group layout
    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl MaterialManager {
    /// Create a new material manager
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, MaterialError> {
        let texture_bind_group_layout = Texture::bind_group_layout(device);

        // Create default white texture
        let default_texture = Texture::white(device, queue)?;
        let default_bind_group =
            default_texture.create_bind_group(device, &texture_bind_group_layout);

        let default_texture_id = 0;

        let mut textures = HashMap::new();
        textures.insert(default_texture_id, default_texture);

        let mut texture_bind_groups = HashMap::new();
        texture_bind_groups.insert(default_texture_id, default_bind_group);

        // Create default material
        let mut materials = HashMap::new();
        let default_material = Material::new("default");
        materials.insert(0, default_material);

        Ok(Self {
            textures,
            texture_bind_groups,
            materials,
            next_texture_id: 1,
            next_material_id: 1,
            default_texture_id,
            texture_bind_group_layout,
        })
    }

    /// Load a texture from file
    pub fn load_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> Result<TextureId, MaterialError> {
        let texture = Texture::from_file(device, queue, path)?;
        let bind_group = texture.create_bind_group(device, &self.texture_bind_group_layout);

        let id = self.next_texture_id;
        self.next_texture_id += 1;

        self.textures.insert(id, texture);
        self.texture_bind_groups.insert(id, bind_group);

        Ok(id)
    }

    /// Load a texture from bytes
    pub fn load_texture_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Result<TextureId, MaterialError> {
        let texture = Texture::from_bytes(device, queue, bytes, label)?;
        let bind_group = texture.create_bind_group(device, &self.texture_bind_group_layout);

        let id = self.next_texture_id;
        self.next_texture_id += 1;

        self.textures.insert(id, texture);
        self.texture_bind_groups.insert(id, bind_group);

        Ok(id)
    }

    /// Create a solid color texture
    pub fn create_solid_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 4],
        label: Option<&str>,
    ) -> Result<TextureId, MaterialError> {
        let texture = Texture::solid_color(device, queue, color, label)?;
        let bind_group = texture.create_bind_group(device, &self.texture_bind_group_layout);

        let id = self.next_texture_id;
        self.next_texture_id += 1;

        self.textures.insert(id, texture);
        self.texture_bind_groups.insert(id, bind_group);

        Ok(id)
    }

    /// Create a checkerboard texture (useful for debugging)
    pub fn create_checkerboard_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: u32,
    ) -> Result<TextureId, MaterialError> {
        let texture = Texture::checkerboard(device, queue, size)?;
        let bind_group = texture.create_bind_group(device, &self.texture_bind_group_layout);

        let id = self.next_texture_id;
        self.next_texture_id += 1;

        self.textures.insert(id, texture);
        self.texture_bind_groups.insert(id, bind_group);

        Ok(id)
    }

    /// Get a texture by ID
    pub fn get_texture(&self, id: TextureId) -> Option<&Texture> {
        self.textures.get(&id)
    }

    /// Get texture bind group by ID
    pub fn get_texture_bind_group(&self, id: TextureId) -> Option<&wgpu::BindGroup> {
        self.texture_bind_groups.get(&id)
    }

    /// Get the default texture bind group
    pub fn default_texture_bind_group(&self) -> &wgpu::BindGroup {
        self.texture_bind_groups
            .get(&self.default_texture_id)
            .expect("Default texture should always exist")
    }

    /// Get texture bind group layout
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    /// Create a new material
    pub fn create_material(&mut self, material: Material) -> MaterialId {
        let id = self.next_material_id;
        self.next_material_id += 1;
        self.materials.insert(id, material);
        id
    }

    /// Get a material by ID
    pub fn get_material(&self, id: MaterialId) -> Option<&Material> {
        self.materials.get(&id)
    }

    /// Get a mutable material by ID
    pub fn get_material_mut(&mut self, id: MaterialId) -> Option<&mut Material> {
        self.materials.get_mut(&id)
    }

    /// Get the bind group for a material's texture
    pub fn get_material_bind_group(&self, material_id: MaterialId) -> &wgpu::BindGroup {
        if let Some(material) = self.materials.get(&material_id) {
            if let Some(texture_id) = material.base_color_texture {
                if let Some(bind_group) = self.texture_bind_groups.get(&texture_id) {
                    return bind_group;
                }
            }
        }
        self.default_texture_bind_group()
    }

    /// Remove a texture
    pub fn remove_texture(&mut self, id: TextureId) -> bool {
        if id == self.default_texture_id {
            return false; // Cannot remove default texture
        }
        self.textures.remove(&id).is_some() && self.texture_bind_groups.remove(&id).is_some()
    }

    /// Remove a material
    pub fn remove_material(&mut self, id: MaterialId) -> bool {
        if id == 0 {
            return false; // Cannot remove default material
        }
        self.materials.remove(&id).is_some()
    }

    /// Get number of loaded textures
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// Get number of materials
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }

    /// Get default texture ID
    pub fn default_texture_id(&self) -> TextureId {
        self.default_texture_id
    }

    /// Get default material ID
    pub fn default_material_id(&self) -> MaterialId {
        0
    }
}

/// Shader module wrapper
pub struct Shader {
    /// The WGPU shader module
    pub module: wgpu::ShaderModule,
    /// Shader label for debugging
    pub label: String,
}

impl Shader {
    /// Create shader from WGSL source
    pub fn from_wgsl(device: &wgpu::Device, source: &str, label: &str) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        Self {
            module,
            label: label.to_string(),
        }
    }

    /// Load shader from file
    pub fn from_file(
        device: &wgpu::Device,
        path: impl AsRef<Path>,
    ) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)?;
        let label = path.file_name().unwrap().to_string_lossy().to_string();
        Ok(Self::from_wgsl(device, &source, &label))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_builder() {
        let material = Material::new("test")
            .with_color(1.0, 0.5, 0.0, 1.0)
            .with_metallic(0.8)
            .with_roughness(0.2)
            .with_emissive(Vec3::new(0.1, 0.0, 0.0));

        assert_eq!(material.name, "test");
        assert_eq!(material.base_color.x, 1.0);
        assert_eq!(material.metallic, 0.8);
        assert_eq!(material.roughness, 0.2);
        assert!(!material.has_texture());
    }

    #[test]
    fn test_material_with_texture() {
        let material = Material::new("textured").with_texture(42);

        assert!(material.has_texture());
        assert_eq!(material.base_color_texture, Some(42));
    }

    #[test]
    fn test_material_uniform() {
        let material = Material::new("test")
            .with_color(1.0, 0.5, 0.25, 1.0)
            .with_metallic(0.5)
            .with_roughness(0.3);

        let uniform = MaterialUniform::from_material(&material);

        assert_eq!(uniform.base_color[0], 1.0);
        assert_eq!(uniform.base_color[1], 0.5);
        assert_eq!(uniform.emissive_metallic[3], 0.5); // metallic
        assert_eq!(uniform.roughness_alpha_flags[0], 0.3); // roughness
    }
}

//! # Texture System
//!
//! GPU texture loading and management.

use std::path::Path;
use thiserror::Error;

/// Texture error types
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("Failed to load image: {0}")]
    LoadError(String),
    #[error("Unsupported image format: {0}")]
    UnsupportedFormat(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Unique texture identifier
pub type TextureId = u32;

/// GPU texture with view and sampler
pub struct Texture {
    /// The GPU texture
    pub texture: wgpu::Texture,
    /// Texture view for binding
    pub view: wgpu::TextureView,
    /// Default sampler
    pub sampler: wgpu::Sampler,
    /// Texture dimensions
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /// Load texture from file path
    pub fn from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: impl AsRef<Path>,
    ) -> Result<Self, TextureError> {
        let path = path.as_ref();
        let img = image::open(path)
            .map_err(|e| TextureError::LoadError(format!("{}: {}", path.display(), e)))?;

        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();

        Self::from_rgba(
            device,
            queue,
            &rgba,
            dimensions.0,
            dimensions.1,
            Some(&path.to_string_lossy()),
        )
    }

    /// Load texture from bytes (PNG, JPG, etc.)
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let img =
            image::load_from_memory(bytes).map_err(|e| TextureError::LoadError(e.to_string()))?;

        let rgba = img.to_rgba8();
        let dimensions = rgba.dimensions();

        Self::from_rgba(device, queue, &rgba, dimensions.0, dimensions.1, label)
    }

    /// Create texture from raw RGBA data
    pub fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rgba: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            width,
            height,
        })
    }

    /// Create a 1x1 solid color texture
    pub fn solid_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        color: [u8; 4],
        label: Option<&str>,
    ) -> Result<Self, TextureError> {
        Self::from_rgba(device, queue, &color, 1, 1, label)
    }

    /// Create default white texture (1x1)
    pub fn white(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, TextureError> {
        Self::solid_color(device, queue, [255, 255, 255, 255], Some("white_texture"))
    }

    /// Create default black texture (1x1)
    pub fn black(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Self, TextureError> {
        Self::solid_color(device, queue, [0, 0, 0, 255], Some("black_texture"))
    }

    /// Create a checkerboard pattern texture (useful for debugging)
    pub fn checkerboard(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: u32,
    ) -> Result<Self, TextureError> {
        let mut data = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let is_white = ((x / 8) + (y / 8)) % 2 == 0;
                let color = if is_white { 255 } else { 128 };
                data.extend_from_slice(&[color, color, color, 255]);
            }
        }
        Self::from_rgba(device, queue, &data, size, size, Some("checkerboard"))
    }

    /// Get the bind group layout for textures
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                // Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    /// Create a bind group for this texture
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }
}

/// Texture sampler configuration
#[derive(Clone, Debug)]
pub struct SamplerConfig {
    /// Address mode for U coordinate
    pub address_mode_u: wgpu::AddressMode,
    /// Address mode for V coordinate
    pub address_mode_v: wgpu::AddressMode,
    /// Magnification filter
    pub mag_filter: wgpu::FilterMode,
    /// Minification filter
    pub min_filter: wgpu::FilterMode,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
        }
    }
}

impl SamplerConfig {
    /// Create sampler with this configuration
    pub fn create_sampler(&self, device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("custom_sampler"),
            address_mode_u: self.address_mode_u,
            address_mode_v: self.address_mode_v,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }

    /// Pixel-perfect sampling (no filtering)
    pub fn nearest() -> Self {
        Self {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    }

    /// Smooth sampling with linear filtering
    pub fn linear() -> Self {
        Self::default()
    }

    /// Clamp to edge (no repeat)
    pub fn clamp() -> Self {
        Self {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sampler_config() {
        let config = SamplerConfig::nearest();
        assert_eq!(config.mag_filter, wgpu::FilterMode::Nearest);

        let config = SamplerConfig::linear();
        assert_eq!(config.mag_filter, wgpu::FilterMode::Linear);
    }
}

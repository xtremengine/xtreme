//! # Render Context
//!
//! WGPU device, queue, and surface management.
//!
//! This module handles the core GPU resources needed for rendering.

use std::sync::Arc;
use wgpu::{
    Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration,
    TextureFormat, PresentMode,
};
use winit::window::Window;

/// Error types for render context operations
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Failed to create surface: {0}")]
    SurfaceCreation(String),

    #[error("No suitable GPU adapter found")]
    NoAdapter,

    #[error("Failed to request device: {0}")]
    DeviceRequest(String),

    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),
}

/// WGPU render context - holds device, queue, surface
pub struct RenderContext {
    /// GPU device for creating resources
    pub device: Device,
    /// Command queue for submitting work
    pub queue: Queue,
    /// Rendering surface (tied to window)
    surface: Surface<'static>,
    /// Surface configuration
    config: SurfaceConfiguration,
    /// GPU adapter info
    adapter: Adapter,
    /// Window size
    size: (u32, u32),
}

impl RenderContext {
    /// Create a new render context for a window
    ///
    /// This initializes WGPU with the following steps:
    /// 1. Create WGPU instance
    /// 2. Create surface from window
    /// 3. Request GPU adapter
    /// 4. Request device and queue
    /// 5. Configure surface
    pub async fn new(window: Arc<Window>) -> Result<Self, RenderError> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Create WGPU instance with best available backend
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: wgpu::InstanceFlags::default(),
            ..Default::default()
        });

        // Create surface from window
        let surface = instance
            .create_surface(window)
            .map_err(|e| RenderError::SurfaceCreation(e.to_string()))?;

        // Request high-performance GPU adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(RenderError::NoAdapter)?;

        log::info!("GPU Adapter: {:?}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Xtreme Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| RenderError::DeviceRequest(e.to_string()))?;

        // Get surface capabilities
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        log::info!("Surface format: {:?}", format);

        // Configure surface
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            adapter,
            size: (width, height),
        })
    }

    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Get current surface size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Get surface texture format
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    /// Get surface configuration
    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }

    /// Get adapter info
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Get the current surface texture for rendering
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    /// Begin a new frame, returns the surface texture and view
    pub fn begin_frame(&self) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), RenderError> {
        let output = self.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok((output, view))
    }

    /// Create a command encoder for this frame
    pub fn create_encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(label),
        })
    }

    /// Submit commands to the queue
    pub fn submit(&self, commands: impl IntoIterator<Item = wgpu::CommandBuffer>) {
        self.queue.submit(commands);
    }
}

/// Builder for RenderContext with custom options
pub struct RenderContextBuilder {
    power_preference: wgpu::PowerPreference,
    present_mode: PresentMode,
    features: wgpu::Features,
}

impl Default for RenderContextBuilder {
    fn default() -> Self {
        Self {
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: PresentMode::AutoVsync,
            features: wgpu::Features::empty(),
        }
    }
}

impl RenderContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set power preference (HighPerformance or LowPower)
    pub fn power_preference(mut self, pref: wgpu::PowerPreference) -> Self {
        self.power_preference = pref;
        self
    }

    /// Set present mode (Vsync, Immediate, etc)
    pub fn present_mode(mut self, mode: PresentMode) -> Self {
        self.present_mode = mode;
        self
    }

    /// Request additional GPU features
    pub fn features(mut self, features: wgpu::Features) -> Self {
        self.features = features;
        self
    }

    /// Build the render context
    pub async fn build(self, window: Arc<Window>) -> Result<RenderContext, RenderError> {
        // For now, use the default implementation
        // In the future, this can be extended to use the builder options
        RenderContext::new(window).await
    }
}

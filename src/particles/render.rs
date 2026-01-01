//! Particle Render Pipeline
//!
//! Billboard rendering for particles.

use super::emitter::BlendMode;
use super::gpu_resources::{CameraUniforms, ParticleBufferPool};
use wgpu::util::DeviceExt;

/// Particle render pipeline
pub struct ParticleRenderPipeline {
    /// Alpha blend pipeline
    alpha_pipeline: wgpu::RenderPipeline,
    /// Additive blend pipeline
    additive_pipeline: wgpu::RenderPipeline,
    /// Camera bind group layout
    camera_bind_group_layout: wgpu::BindGroupLayout,
    /// Texture bind group layout
    texture_bind_group_layout: wgpu::BindGroupLayout,
    /// Camera uniform buffer
    camera_buffer: wgpu::Buffer,
    /// Default white texture bind group
    default_texture_bind_group: wgpu::BindGroup,
}

impl ParticleRenderPipeline {
    /// Create new render pipeline
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Render Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/particle_render.wgsl").into(),
            ),
        });

        // Camera + particles bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Camera Bind Group Layout"),
                entries: &[
                    // Camera uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Particle storage buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Texture Bind Group Layout"),
                entries: &[
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
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipelines for different blend modes
        let alpha_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::BlendState::ALPHA_BLENDING,
            "fs_main",
        );

        let additive_pipeline = Self::create_pipeline(
            device,
            &pipeline_layout,
            &shader,
            surface_format,
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            "fs_additive",
        );

        // Camera uniform buffer
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Camera Buffer"),
            contents: bytemuck::bytes_of(&CameraUniforms::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create default white texture
        let default_texture_bind_group =
            Self::create_default_texture(device, queue, &texture_bind_group_layout);

        Self {
            alpha_pipeline,
            additive_pipeline,
            camera_bind_group_layout,
            texture_bind_group_layout,
            camera_buffer,
            default_texture_bind_group,
        }
    }

    fn create_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        blend: wgpu::BlendState,
        fs_entry: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fs_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None, // Billboards visible from both sides
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Particles don't write depth
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }

    fn create_default_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        // Create a 2x2 white texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Particle Texture"),
            size: wgpu::Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
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
            &[255u8; 16], // 4 white pixels
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(8),
                rows_per_image: Some(2),
            },
            wgpu::Extent3d {
                width: 2,
                height: 2,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Default Particle Texture Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }

    /// Get camera bind group layout
    pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.camera_bind_group_layout
    }

    /// Get texture bind group layout
    pub fn texture_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_bind_group_layout
    }

    /// Update camera uniforms
    pub fn update_camera(&self, queue: &wgpu::Queue, uniforms: &CameraUniforms) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    /// Create camera bind group for a particle buffer
    pub fn create_camera_bind_group(
        &self,
        device: &wgpu::Device,
        pool: &ParticleBufferPool,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Camera Bind Group"),
            layout: &self.camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: pool.read_buffer().as_entire_binding(),
                },
            ],
        })
    }

    /// Render particles
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        texture_bind_group: Option<&'a wgpu::BindGroup>,
        instance_count: u32,
        blend_mode: BlendMode,
    ) {
        if instance_count == 0 {
            return;
        }

        let pipeline = match blend_mode {
            BlendMode::Alpha | BlendMode::PremultipliedAlpha => &self.alpha_pipeline,
            BlendMode::Additive | BlendMode::Multiply => &self.additive_pipeline,
        };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(
            1,
            texture_bind_group.unwrap_or(&self.default_texture_bind_group),
            &[],
        );

        // 6 vertices per particle (2 triangles)
        render_pass.draw(0..6, 0..instance_count);
    }

    /// Render with indirect draw
    pub fn render_indirect<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        texture_bind_group: Option<&'a wgpu::BindGroup>,
        indirect_buffer: &'a wgpu::Buffer,
        blend_mode: BlendMode,
    ) {
        let pipeline = match blend_mode {
            BlendMode::Alpha | BlendMode::PremultipliedAlpha => &self.alpha_pipeline,
            BlendMode::Additive | BlendMode::Multiply => &self.additive_pipeline,
        };

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(
            1,
            texture_bind_group.unwrap_or(&self.default_texture_bind_group),
            &[],
        );

        render_pass.draw_indirect(indirect_buffer, 0);
    }
}

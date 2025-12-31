//! # Viewport
//!
//! 3D viewport with render-to-texture for egui integration.
//!
//! This module is split into:
//! - `types.rs` - Uniforms and constants
//! - `textures.rs` - Texture creation helpers
//! - `render.rs` - Rendering methods

mod render;
mod textures;
mod types;

#[allow(unused_imports)]
pub use types::{GizmoUniforms, GridUniforms};

use wgpu::util::DeviceExt;

use super::gizmos::GizmoVertex;
use crate::render::{GpuMesh, Mesh, RenderContext, Uniforms};
use textures::{create_depth_texture, create_render_texture};
use types::{MAX_GIZMO_VERTICES, MAX_OBJECTS};

/// 3D viewport for the editor
pub struct Viewport {
    /// Viewport size
    pub(crate) size: (u32, u32),
    /// Render texture
    pub(crate) render_texture: wgpu::Texture,
    /// Render texture view
    pub(crate) render_view: wgpu::TextureView,
    /// Depth texture
    pub(crate) depth_texture: wgpu::Texture,
    /// Depth texture view
    pub(crate) depth_view: wgpu::TextureView,
    /// Egui texture ID
    pub(crate) egui_texture_id: egui::TextureId,
    /// Grid pipeline
    pub(crate) grid_pipeline: wgpu::RenderPipeline,
    /// Grid uniform buffer
    pub(crate) grid_uniform_buffer: wgpu::Buffer,
    /// Grid bind group
    pub(crate) grid_bind_group: wgpu::BindGroup,
    /// Mesh pipeline
    pub(crate) mesh_pipeline: wgpu::RenderPipeline,
    /// Mesh uniform buffer (large enough for MAX_OBJECTS)
    pub(crate) mesh_uniform_buffer: wgpu::Buffer,
    /// Mesh bind group
    pub(crate) mesh_bind_group: wgpu::BindGroup,
    /// Uniform alignment requirement
    pub(crate) uniform_alignment: u32,
    /// Test cube mesh
    pub(crate) cube_mesh: GpuMesh,
    /// Gizmo line pipeline
    pub(crate) gizmo_line_pipeline: wgpu::RenderPipeline,
    /// Gizmo triangle pipeline
    pub(crate) gizmo_tri_pipeline: wgpu::RenderPipeline,
    /// Gizmo vertex buffer (lines)
    pub(crate) gizmo_line_buffer: wgpu::Buffer,
    /// Gizmo vertex buffer (triangles)
    pub(crate) gizmo_tri_buffer: wgpu::Buffer,
    /// Gizmo uniform buffer
    pub(crate) gizmo_uniform_buffer: wgpu::Buffer,
    /// Gizmo bind group
    pub(crate) gizmo_bind_group: wgpu::BindGroup,
}

impl Viewport {
    /// Create a new viewport
    pub fn new(
        ctx: &RenderContext,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) -> Self {
        let device = &ctx.device;

        // Create render texture
        let (render_texture, render_view) = create_render_texture(device, width, height);

        // Create depth texture
        let (depth_texture, depth_view) = create_depth_texture(device, width, height);

        // Register texture with egui
        let egui_texture_id =
            egui_renderer.register_native_texture(device, &render_view, wgpu::FilterMode::Linear);

        // Create grid pipeline
        let grid_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Grid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/grid.wgsl").into()),
        });

        let grid_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Grid Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let grid_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[types::GridUniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Grid Bind Group"),
            layout: &grid_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_uniform_buffer.as_entire_binding(),
            }],
        });

        let grid_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Grid Pipeline Layout"),
            bind_group_layouts: &[&grid_bind_group_layout],
            push_constant_ranges: &[],
        });

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&grid_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &grid_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &grid_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create mesh pipeline (using basic shader)
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Mesh Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/basic.wgsl").into()),
        });

        // Get uniform buffer alignment requirement
        let uniform_alignment = ctx.device.limits().min_uniform_buffer_offset_alignment;
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_uniform_size =
            uniform_size.div_ceil(uniform_alignment) * uniform_alignment;

        let mesh_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Mesh Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(uniform_size as u64),
                    },
                    count: None,
                }],
            });

        // Create buffer large enough for MAX_OBJECTS
        let buffer_size = aligned_uniform_size as usize * MAX_OBJECTS;
        let mesh_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Mesh Uniform Buffer"),
            size: buffer_size as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mesh_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mesh Bind Group"),
            layout: &mesh_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &mesh_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(uniform_size as u64),
                }),
            }],
        });

        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh Pipeline Layout"),
            bind_group_layouts: &[&mesh_bind_group_layout],
            push_constant_ranges: &[],
        });

        let mesh_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Pipeline"),
            layout: Some(&mesh_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &mesh_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<crate::render::Vertex>()
                        as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 24,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &mesh_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create test cube
        let cube_mesh = GpuMesh::from_mesh(device, &Mesh::cube(1.0));

        // Create gizmo pipeline
        let gizmo_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gizmo Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/gizmo.wgsl").into()),
        });

        let gizmo_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Gizmo Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let gizmo_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gizmo Uniform Buffer"),
            contents: bytemuck::cast_slice(&[types::GizmoUniforms::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let gizmo_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gizmo Bind Group"),
            layout: &gizmo_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gizmo_uniform_buffer.as_entire_binding(),
            }],
        });

        let gizmo_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Gizmo Pipeline Layout"),
                bind_group_layouts: &[&gizmo_bind_group_layout],
                push_constant_ranges: &[],
            });

        // Gizmo vertex layout (position + color)
        let gizmo_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GizmoVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3, // position
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4, // color
                },
            ],
        };

        // Line pipeline (for axes)
        let gizmo_line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gizmo Line Pipeline"),
            layout: Some(&gizmo_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gizmo_shader,
                entry_point: Some("vs_main"),
                buffers: std::slice::from_ref(&gizmo_vertex_layout),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gizmo_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Gizmos don't write depth
                depth_compare: wgpu::CompareFunction::Always, // Always visible
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Triangle pipeline (for arrow heads, cubes)
        let gizmo_tri_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gizmo Triangle Pipeline"),
            layout: Some(&gizmo_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gizmo_shader,
                entry_point: Some("vs_main"),
                buffers: &[gizmo_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &gizmo_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None, // No culling for gizmos
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Create gizmo vertex buffers
        let gizmo_buffer_size = (MAX_GIZMO_VERTICES * std::mem::size_of::<GizmoVertex>()) as u64;
        let gizmo_line_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Line Buffer"),
            size: gizmo_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let gizmo_tri_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gizmo Triangle Buffer"),
            size: gizmo_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            size: (width, height),
            render_texture,
            render_view,
            depth_texture,
            depth_view,
            egui_texture_id,
            grid_pipeline,
            grid_uniform_buffer,
            grid_bind_group,
            mesh_pipeline,
            mesh_uniform_buffer,
            mesh_bind_group,
            uniform_alignment,
            cube_mesh,
            gizmo_line_pipeline,
            gizmo_tri_pipeline,
            gizmo_line_buffer,
            gizmo_tri_buffer,
            gizmo_uniform_buffer,
            gizmo_bind_group,
        }
    }

    /// Resize the viewport
    pub fn resize(
        &mut self,
        ctx: &RenderContext,
        egui_renderer: &mut egui_wgpu::Renderer,
        width: u32,
        height: u32,
    ) {
        if width == 0 || height == 0 || (width == self.size.0 && height == self.size.1) {
            return;
        }

        self.size = (width, height);

        // Recreate textures
        let (render_texture, render_view) = create_render_texture(&ctx.device, width, height);
        let (depth_texture, depth_view) = create_depth_texture(&ctx.device, width, height);

        self.render_texture = render_texture;
        self.render_view = render_view;
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        // Update egui texture
        egui_renderer.update_egui_texture_from_wgpu_texture(
            &ctx.device,
            &self.render_view,
            wgpu::FilterMode::Linear,
            self.egui_texture_id,
        );
    }

    /// Get the egui texture ID for this viewport
    pub fn texture_id(&self) -> egui::TextureId {
        self.egui_texture_id
    }

    /// Get viewport size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

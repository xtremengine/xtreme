//! Skinned Mesh Support
//!
//! GPU vertex format and resources for skeletal animation.

use super::bone::MAX_BONES;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;

/// Maximum bones that can influence a single vertex
pub const MAX_BONES_PER_VERTEX: usize = 4;

/// Skinned vertex with bone weights
///
/// Total size: 64 bytes (aligned for GPU)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SkinnedVertex {
    /// Position (xyz)
    pub position: [f32; 3],
    /// Normal (xyz)
    pub normal: [f32; 3],
    /// Texture coordinates (uv)
    pub uv: [f32; 2],
    /// Bone indices (up to 4 bones)
    pub bone_indices: [u32; 4],
    /// Bone weights (must sum to 1.0)
    pub bone_weights: [f32; 4],
}

impl SkinnedVertex {
    /// Create a new skinned vertex
    pub fn new(
        position: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
        bone_indices: [u32; 4],
        bone_weights: [f32; 4],
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            bone_indices,
            bone_weights,
        }
    }

    /// Create vertex with single bone influence
    pub fn with_single_bone(
        position: [f32; 3],
        normal: [f32; 3],
        uv: [f32; 2],
        bone_index: u32,
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            bone_indices: [bone_index, 0, 0, 0],
            bone_weights: [1.0, 0.0, 0.0, 0.0],
        }
    }

    /// Get vertex buffer layout for WGPU
    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // UV
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Bone indices
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32x4,
                },
                // Bone weights
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

/// Bone matrices uniform buffer for GPU skinning
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct BoneMatricesUniform {
    /// Skin matrices for each bone
    pub matrices: [[f32; 16]; MAX_BONES],
}

impl Default for BoneMatricesUniform {
    fn default() -> Self {
        Self {
            matrices: [[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]; MAX_BONES],
        }
    }
}

impl BoneMatricesUniform {
    /// Create from skin matrices
    pub fn from_skin_matrices(matrices: &[Mat4]) -> Self {
        let mut data = Self::default();
        for (i, mat) in matrices.iter().take(MAX_BONES).enumerate() {
            data.matrices[i] = mat.to_cols_array();
        }
        data
    }

    /// Update from skin matrices
    pub fn update(&mut self, matrices: &[Mat4]) {
        for (i, mat) in matrices.iter().take(MAX_BONES).enumerate() {
            self.matrices[i] = mat.to_cols_array();
        }
    }

    /// Get size in bytes
    pub fn size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
}

/// Skinned mesh data (CPU-side)
#[derive(Clone, Debug)]
pub struct SkinnedMesh {
    /// Vertices with bone weights
    pub vertices: Vec<SkinnedVertex>,
    /// Triangle indices
    pub indices: Vec<u32>,
    /// Name of the mesh
    pub name: String,
}

impl SkinnedMesh {
    /// Create a new skinned mesh
    pub fn new(name: impl Into<String>, vertices: Vec<SkinnedVertex>, indices: Vec<u32>) -> Self {
        Self {
            name: name.into(),
            vertices,
            indices,
        }
    }

    /// Number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of indices
    pub fn index_count(&self) -> usize {
        self.indices.len()
    }

    /// Number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Normalize bone weights so they sum to 1.0
    pub fn normalize_weights(&mut self) {
        for vertex in &mut self.vertices {
            let sum: f32 = vertex.bone_weights.iter().sum();
            if sum > 0.0 {
                for w in &mut vertex.bone_weights {
                    *w /= sum;
                }
            } else {
                // Default to first bone if no weights
                vertex.bone_weights = [1.0, 0.0, 0.0, 0.0];
            }
        }
    }
}

/// GPU resources for a skinned mesh
pub struct SkinnedMeshGpu {
    /// Vertex buffer
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer
    pub index_buffer: wgpu::Buffer,
    /// Bone matrices uniform buffer
    pub bone_buffer: wgpu::Buffer,
    /// Bind group for bone matrices
    pub bone_bind_group: wgpu::BindGroup,
    /// Number of indices
    pub index_count: u32,
}

impl SkinnedMeshGpu {
    /// Create GPU resources for a skinned mesh
    pub fn new(
        device: &wgpu::Device,
        mesh: &SkinnedMesh,
        bone_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Vertex Buffer", mesh.name)),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Index Buffer", mesh.name)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let bone_uniform = BoneMatricesUniform::default();
        let bone_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Bone Buffer", mesh.name)),
            contents: bytemuck::bytes_of(&bone_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bone_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bone Bind Group", mesh.name)),
            layout: bone_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bone_buffer.as_entire_binding(),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            bone_buffer,
            bone_bind_group,
            index_count: mesh.indices.len() as u32,
        }
    }

    /// Update bone matrices
    pub fn update_bones(&self, queue: &wgpu::Queue, matrices: &[Mat4]) {
        let uniform = BoneMatricesUniform::from_skin_matrices(matrices);
        queue.write_buffer(&self.bone_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    /// Create bind group layout for bone matrices
    pub fn bone_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bone Matrices Bind Group Layout"),
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
        })
    }
}

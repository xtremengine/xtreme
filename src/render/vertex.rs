//! # Vertex Definitions

use bytemuck::{Pod, Zeroable};

/// Standard 3D vertex with position, normal, and UV
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self { position, normal, uv }
    }
}

/// Vertex layout description
pub struct VertexLayout {
    pub stride: u64,
    pub attributes: Vec<VertexAttribute>,
}

pub struct VertexAttribute {
    pub offset: u64,
    pub format: VertexFormat,
}

#[derive(Clone, Copy)]
pub enum VertexFormat {
    Float32x2,
    Float32x3,
    Float32x4,
}

impl Vertex {
    pub fn layout() -> VertexLayout {
        VertexLayout {
            stride: std::mem::size_of::<Vertex>() as u64,
            attributes: vec![
                VertexAttribute { offset: 0, format: VertexFormat::Float32x3 },
                VertexAttribute { offset: 12, format: VertexFormat::Float32x3 },
                VertexAttribute { offset: 24, format: VertexFormat::Float32x2 },
            ],
        }
    }
}

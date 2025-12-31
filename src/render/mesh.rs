//! # Mesh System
//!
//! Mesh data structures for CPU and GPU.

use wgpu::util::DeviceExt;
use super::vertex::Vertex;

/// 3D mesh data
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    /// Create a unit cube centered at origin
    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        let vertices = vec![
            // Front face (+Z)
            Vertex::new([-s, -s,  s], [0.0, 0.0, 1.0], [0.0, 0.0]),
            Vertex::new([ s, -s,  s], [0.0, 0.0, 1.0], [1.0, 0.0]),
            Vertex::new([ s,  s,  s], [0.0, 0.0, 1.0], [1.0, 1.0]),
            Vertex::new([-s,  s,  s], [0.0, 0.0, 1.0], [0.0, 1.0]),
            // Back face (-Z)
            Vertex::new([ s, -s, -s], [0.0, 0.0, -1.0], [0.0, 0.0]),
            Vertex::new([-s, -s, -s], [0.0, 0.0, -1.0], [1.0, 0.0]),
            Vertex::new([-s,  s, -s], [0.0, 0.0, -1.0], [1.0, 1.0]),
            Vertex::new([ s,  s, -s], [0.0, 0.0, -1.0], [0.0, 1.0]),
            // Top face (+Y)
            Vertex::new([-s,  s,  s], [0.0, 1.0, 0.0], [0.0, 0.0]),
            Vertex::new([ s,  s,  s], [0.0, 1.0, 0.0], [1.0, 0.0]),
            Vertex::new([ s,  s, -s], [0.0, 1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-s,  s, -s], [0.0, 1.0, 0.0], [0.0, 1.0]),
            // Bottom face (-Y)
            Vertex::new([-s, -s, -s], [0.0, -1.0, 0.0], [0.0, 0.0]),
            Vertex::new([ s, -s, -s], [0.0, -1.0, 0.0], [1.0, 0.0]),
            Vertex::new([ s, -s,  s], [0.0, -1.0, 0.0], [1.0, 1.0]),
            Vertex::new([-s, -s,  s], [0.0, -1.0, 0.0], [0.0, 1.0]),
            // Right face (+X)
            Vertex::new([ s, -s,  s], [1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([ s, -s, -s], [1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([ s,  s, -s], [1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([ s,  s,  s], [1.0, 0.0, 0.0], [0.0, 1.0]),
            // Left face (-X)
            Vertex::new([-s, -s, -s], [-1.0, 0.0, 0.0], [0.0, 0.0]),
            Vertex::new([-s, -s,  s], [-1.0, 0.0, 0.0], [1.0, 0.0]),
            Vertex::new([-s,  s,  s], [-1.0, 0.0, 0.0], [1.0, 1.0]),
            Vertex::new([-s,  s, -s], [-1.0, 0.0, 0.0], [0.0, 1.0]),
        ];
        let indices = vec![
            0,  1,  2,  2,  3,  0,   // front
            4,  5,  6,  6,  7,  4,   // back
            8,  9,  10, 10, 11, 8,   // top
            12, 13, 14, 14, 15, 12,  // bottom
            16, 17, 18, 18, 19, 16,  // right
            20, 21, 22, 22, 23, 20,  // left
        ];
        Self { vertices, indices }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn index_count(&self) -> usize {
        self.indices.len()
    }
}

/// Builder for creating meshes procedurally
pub struct MeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, vertex: Vertex) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(vertex);
        index
    }

    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.extend_from_slice(&[i0, i1, i2]);
    }

    pub fn build(self) -> Mesh {
        Mesh::new(self.vertices, self.indices)
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Primitive mesh types
pub enum Primitive {
    Cube,
    Sphere,
    Plane,
    Cylinder,
}

/// GPU-ready mesh with vertex and index buffers
pub struct GpuMesh {
    /// Vertex buffer
    pub vertex_buffer: wgpu::Buffer,
    /// Index buffer
    pub index_buffer: wgpu::Buffer,
    /// Number of indices
    pub index_count: u32,
}

impl GpuMesh {
    /// Create a GPU mesh from a CPU mesh
    pub fn from_mesh(device: &wgpu::Device, mesh: &Mesh) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        }
    }

    /// Create a unit cube GPU mesh
    pub fn cube(device: &wgpu::Device, size: f32) -> Self {
        Self::from_mesh(device, &Mesh::cube(size))
    }

    /// Draw this mesh using a render pass
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }

    /// Draw this mesh with instancing
    pub fn draw_instanced<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, instances: u32) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.index_count, 0, 0..instances);
    }
}

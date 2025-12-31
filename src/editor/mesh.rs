//! Mesh loading and management for 3D models.

use std::path::Path;
use glam::Vec3;

/// A loaded mesh with vertices and indices
#[derive(Clone, Debug)]
pub struct LoadedMesh {
    /// Mesh name
    pub name: String,
    /// Vertex positions [x, y, z, x, y, z, ...]
    pub positions: Vec<f32>,
    /// Vertex normals [nx, ny, nz, nx, ny, nz, ...]
    pub normals: Vec<f32>,
    /// Texture coordinates [u, v, u, v, ...]
    pub texcoords: Vec<f32>,
    /// Indices for indexed rendering
    pub indices: Vec<u32>,
    /// Bounding box min
    pub bounds_min: Vec3,
    /// Bounding box max
    pub bounds_max: Vec3,
}

impl LoadedMesh {
    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Get mesh center
    pub fn center(&self) -> Vec3 {
        (self.bounds_min + self.bounds_max) * 0.5
    }

    /// Get mesh size
    pub fn size(&self) -> Vec3 {
        self.bounds_max - self.bounds_min
    }
}

/// Load meshes from an OBJ file
pub fn load_obj(path: &Path) -> Result<Vec<LoadedMesh>, String> {
    let load_options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, _materials) = tobj::load_obj(path, &load_options)
        .map_err(|e| format!("Failed to load OBJ: {}", e))?;

    let mut meshes = Vec::new();

    for model in models {
        let mesh = &model.mesh;

        // Calculate bounds
        let mut bounds_min = Vec3::splat(f32::MAX);
        let mut bounds_max = Vec3::splat(f32::MIN);

        for i in (0..mesh.positions.len()).step_by(3) {
            let pos = Vec3::new(
                mesh.positions[i],
                mesh.positions[i + 1],
                mesh.positions[i + 2],
            );
            bounds_min = bounds_min.min(pos);
            bounds_max = bounds_max.max(pos);
        }

        // Generate default normals if not present
        let normals = if mesh.normals.is_empty() {
            generate_flat_normals(&mesh.positions, &mesh.indices)
        } else {
            mesh.normals.clone()
        };

        // Generate default texcoords if not present
        let texcoords = if mesh.texcoords.is_empty() {
            vec![0.0; mesh.positions.len() / 3 * 2]
        } else {
            mesh.texcoords.clone()
        };

        meshes.push(LoadedMesh {
            name: model.name,
            positions: mesh.positions.clone(),
            normals,
            texcoords,
            indices: mesh.indices.clone(),
            bounds_min,
            bounds_max,
        });
    }

    if meshes.is_empty() {
        Err("No meshes found in file".to_string())
    } else {
        log::info!("Loaded {} meshes from {:?}", meshes.len(), path);
        Ok(meshes)
    }
}

/// Generate flat normals for a mesh
fn generate_flat_normals(positions: &[f32], indices: &[u32]) -> Vec<f32> {
    let vertex_count = positions.len() / 3;
    let mut normals = vec![0.0f32; vertex_count * 3];
    let mut counts = vec![0u32; vertex_count];

    // Calculate face normals and accumulate
    for i in (0..indices.len()).step_by(3) {
        let i0 = indices[i] as usize;
        let i1 = indices[i + 1] as usize;
        let i2 = indices[i + 2] as usize;

        let v0 = Vec3::new(positions[i0 * 3], positions[i0 * 3 + 1], positions[i0 * 3 + 2]);
        let v1 = Vec3::new(positions[i1 * 3], positions[i1 * 3 + 1], positions[i1 * 3 + 2]);
        let v2 = Vec3::new(positions[i2 * 3], positions[i2 * 3 + 1], positions[i2 * 3 + 2]);

        let normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();

        for idx in [i0, i1, i2] {
            normals[idx * 3] += normal.x;
            normals[idx * 3 + 1] += normal.y;
            normals[idx * 3 + 2] += normal.z;
            counts[idx] += 1;
        }
    }

    // Normalize accumulated normals
    for i in 0..vertex_count {
        if counts[i] > 0 {
            let len = (normals[i * 3].powi(2) + normals[i * 3 + 1].powi(2) + normals[i * 3 + 2].powi(2)).sqrt();
            if len > 0.0 {
                normals[i * 3] /= len;
                normals[i * 3 + 1] /= len;
                normals[i * 3 + 2] /= len;
            }
        }
    }

    normals
}

/// Supported mesh file extensions
pub fn supported_extensions() -> &'static [&'static str] {
    &["obj"]
}

/// Check if a file extension is supported
pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| supported_extensions().contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

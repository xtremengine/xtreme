//! GLTF Animation Loader
//!
//! Load skeletons and animations from GLTF files.

use super::bone::{Bone, BoneIndex, BoneTransform, Skeleton};
use super::clip::{AnimationClip, BoneTrack, Keyframe};
use super::skinning::{SkinnedMesh, SkinnedVertex};
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during GLTF loading
#[derive(Error, Debug)]
pub enum GltfError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("GLTF parse error: {0}")]
    Gltf(#[from] gltf::Error),
    #[error("No skeleton found in file")]
    NoSkeleton,
    #[error("No animations found in file")]
    NoAnimations,
    #[error("Invalid bone index: {0}")]
    InvalidBone(usize),
    #[error("Missing data: {0}")]
    MissingData(String),
}

/// Result of loading a GLTF file
pub struct GltfData {
    /// Loaded skeleton
    pub skeleton: Skeleton,
    /// Animation clips
    pub clips: Vec<AnimationClip>,
    /// Skinned meshes
    pub meshes: Vec<SkinnedMesh>,
}

/// Load skeleton, animations, and meshes from a GLTF file
pub fn load_gltf(path: impl AsRef<Path>) -> Result<GltfData, GltfError> {
    let path = path.as_ref();
    let (document, buffers, _images) = gltf::import(path)?;

    // Build parent map by traversing all nodes
    let parent_map = build_parent_map(&document);

    // Find skin (skeleton)
    let skin = document.skins().next().ok_or(GltfError::NoSkeleton)?;

    // Load skeleton
    let skeleton = load_skeleton(&skin, &buffers, &parent_map)?;

    // Build node index to bone index map
    let node_to_bone: HashMap<usize, BoneIndex> = skin
        .joints()
        .enumerate()
        .map(|(i, joint)| (joint.index(), i as BoneIndex))
        .collect();

    // Load animations
    let clips: Vec<AnimationClip> = document
        .animations()
        .map(|anim| load_animation(&anim, &buffers, &node_to_bone))
        .collect::<Result<Vec<_>, _>>()?;

    // Load skinned meshes
    let meshes = load_skinned_meshes(&document, &buffers)?;

    Ok(GltfData {
        skeleton,
        clips,
        meshes,
    })
}

/// Build a map of node index -> parent node index by traversing the scene
fn build_parent_map(document: &gltf::Document) -> HashMap<usize, usize> {
    let mut parent_map = HashMap::new();

    fn traverse_node(node: &gltf::Node, parent_map: &mut HashMap<usize, usize>) {
        for child in node.children() {
            parent_map.insert(child.index(), node.index());
            traverse_node(&child, parent_map);
        }
    }

    for scene in document.scenes() {
        for node in scene.nodes() {
            traverse_node(&node, &mut parent_map);
        }
    }

    parent_map
}

/// Load skeleton from GLTF skin
fn load_skeleton(
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
    parent_map: &HashMap<usize, usize>,
) -> Result<Skeleton, GltfError> {
    let joints: Vec<_> = skin.joints().collect();
    let mut bones = Vec::with_capacity(joints.len());

    // Build node to index map
    let node_to_idx: HashMap<usize, BoneIndex> = joints
        .iter()
        .enumerate()
        .map(|(i, j)| (j.index(), i as BoneIndex))
        .collect();

    // Load inverse bind matrices
    let inverse_bind_matrices: Vec<Mat4> = if let Some(accessor) = skin.inverse_bind_matrices() {
        read_mat4_accessor(&accessor, buffers)?
    } else {
        vec![Mat4::IDENTITY; joints.len()]
    };

    // Create bones
    for (i, joint) in joints.iter().enumerate() {
        let (translation, rotation, scale) = joint.transform().decomposed();
        let local_bind_pose = BoneTransform {
            translation: Vec3::from(translation),
            rotation: Quat::from_array(rotation),
            scale: Vec3::from(scale),
        };

        // Find parent using the parent map
        let parent = parent_map
            .get(&joint.index())
            .and_then(|&parent_node_idx| node_to_idx.get(&parent_node_idx).copied());

        let mut bone = Bone::new(joint.name().unwrap_or(&format!("Bone_{}", i)))
            .with_bind_pose(local_bind_pose)
            .with_inverse_bind_matrix(inverse_bind_matrices[i]);

        if let Some(parent_idx) = parent {
            bone = bone.with_parent(parent_idx);
        }

        bones.push(bone);
    }

    // Set up children
    for i in 0..bones.len() {
        if let Some(parent) = bones[i].parent {
            let child_idx = i as BoneIndex;
            bones[parent as usize].children.push(child_idx);
        }
    }

    Ok(Skeleton::new(bones))
}

/// Load animation from GLTF animation
fn load_animation(
    animation: &gltf::Animation,
    buffers: &[gltf::buffer::Data],
    node_to_bone: &HashMap<usize, BoneIndex>,
) -> Result<AnimationClip, GltfError> {
    let name = animation.name().unwrap_or("Unnamed").to_string();

    let mut tracks: HashMap<BoneIndex, BoneTrack> = HashMap::new();
    let mut duration = 0.0f32;

    for channel in animation.channels() {
        let target = channel.target();
        let node_idx = target.node().index();

        // Skip if not a bone
        let bone_idx = match node_to_bone.get(&node_idx) {
            Some(&idx) => idx,
            None => continue,
        };

        let sampler = channel.sampler();
        let input = sampler.input();
        let output = sampler.output();

        // Read times
        let times = read_f32_accessor(&input, buffers)?;

        // Update duration
        if let Some(&max_time) = times.last() {
            duration = duration.max(max_time);
        }

        // Get or create track
        let track = tracks
            .entry(bone_idx)
            .or_insert_with(|| BoneTrack::new(bone_idx));

        // Read values based on property
        match target.property() {
            gltf::animation::Property::Translation => {
                let values = read_vec3_accessor(&output, buffers)?;
                let keyframes: Vec<Keyframe<Vec3>> = times
                    .iter()
                    .zip(values.iter())
                    .map(|(&t, &v)| Keyframe::new(t, v))
                    .collect();
                track.translation = Some(keyframes);
            }
            gltf::animation::Property::Rotation => {
                let values = read_quat_accessor(&output, buffers)?;
                let keyframes: Vec<Keyframe<Quat>> = times
                    .iter()
                    .zip(values.iter())
                    .map(|(&t, &v)| Keyframe::new(t, v))
                    .collect();
                track.rotation = Some(keyframes);
            }
            gltf::animation::Property::Scale => {
                let values = read_vec3_accessor(&output, buffers)?;
                let keyframes: Vec<Keyframe<Vec3>> = times
                    .iter()
                    .zip(values.iter())
                    .map(|(&t, &v)| Keyframe::new(t, v))
                    .collect();
                track.scale = Some(keyframes);
            }
            gltf::animation::Property::MorphTargetWeights => {
                // Skip morph targets for now
            }
        }
    }

    let clip = AnimationClip {
        name,
        duration,
        tracks: tracks.into_values().collect(),
        looping: true,
        speed: 1.0,
    };

    Ok(clip)
}

/// Load skinned meshes from GLTF
fn load_skinned_meshes(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<SkinnedMesh>, GltfError> {
    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Read positions
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or_else(|| GltfError::MissingData("positions".into()))?
                .collect();

            // Read normals
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            // Read UVs
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            // Read joints
            let joints: Vec<[u16; 4]> = reader
                .read_joints(0)
                .map(|j| j.into_u16().collect())
                .unwrap_or_else(|| vec![[0, 0, 0, 0]; positions.len()]);

            // Read weights
            let weights: Vec<[f32; 4]> = reader
                .read_weights(0)
                .map(|w| w.into_f32().collect())
                .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]);

            // Build vertices
            let vertices: Vec<SkinnedVertex> = positions
                .into_iter()
                .zip(normals)
                .zip(uvs)
                .zip(joints)
                .zip(weights)
                .map(|((((pos, norm), uv), joint), weight)| {
                    SkinnedVertex::new(
                        pos,
                        norm,
                        uv,
                        [
                            joint[0] as u32,
                            joint[1] as u32,
                            joint[2] as u32,
                            joint[3] as u32,
                        ],
                        weight,
                    )
                })
                .collect();

            // Read indices
            let indices: Vec<u32> = reader
                .read_indices()
                .ok_or_else(|| GltfError::MissingData("indices".into()))?
                .into_u32()
                .collect();

            let name = mesh.name().unwrap_or("Unnamed").to_string();
            meshes.push(SkinnedMesh::new(name, vertices, indices));
        }
    }

    Ok(meshes)
}

// Helper functions to read GLTF accessors

fn read_f32_accessor(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<f32>, GltfError> {
    let view = accessor
        .view()
        .ok_or_else(|| GltfError::MissingData("buffer view".into()))?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();

    let data = &buffer[offset..offset + count * 4];
    let values: Vec<f32> = data
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(values)
}

fn read_vec3_accessor(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<Vec3>, GltfError> {
    let view = accessor
        .view()
        .ok_or_else(|| GltfError::MissingData("buffer view".into()))?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();

    let data = &buffer[offset..offset + count * 12];
    let values: Vec<Vec3> = data
        .chunks_exact(12)
        .map(|chunk| {
            let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
            Vec3::new(x, y, z)
        })
        .collect();

    Ok(values)
}

fn read_quat_accessor(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<Quat>, GltfError> {
    let view = accessor
        .view()
        .ok_or_else(|| GltfError::MissingData("buffer view".into()))?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();

    let data = &buffer[offset..offset + count * 16];
    let values: Vec<Quat> = data
        .chunks_exact(16)
        .map(|chunk| {
            let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]);
            let w = f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]);
            Quat::from_xyzw(x, y, z, w)
        })
        .collect();

    Ok(values)
}

fn read_mat4_accessor(
    accessor: &gltf::Accessor,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<Mat4>, GltfError> {
    let view = accessor
        .view()
        .ok_or_else(|| GltfError::MissingData("buffer view".into()))?;
    let buffer = &buffers[view.buffer().index()];
    let offset = view.offset() + accessor.offset();
    let count = accessor.count();

    let data = &buffer[offset..offset + count * 64];
    let values: Vec<Mat4> = data
        .chunks_exact(64)
        .map(|chunk| {
            let mut cols = [[0.0f32; 4]; 4];
            for (col_idx, col) in cols.iter_mut().enumerate() {
                for (row_idx, value) in col.iter_mut().enumerate() {
                    let byte_offset = (col_idx * 4 + row_idx) * 4;
                    *value = f32::from_le_bytes([
                        chunk[byte_offset],
                        chunk[byte_offset + 1],
                        chunk[byte_offset + 2],
                        chunk[byte_offset + 3],
                    ]);
                }
            }
            Mat4::from_cols_array_2d(&cols)
        })
        .collect();

    Ok(values)
}

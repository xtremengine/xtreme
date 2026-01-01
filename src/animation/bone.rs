//! Bone and Skeleton Data Structures
//!
//! Defines the skeletal hierarchy for animation.

use crate::core::Component;
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;

/// Index into skeleton bone array
pub type BoneIndex = u16;

/// Maximum bones supported in a skeleton
pub const MAX_BONES: usize = 128;

/// Transform for a bone (decomposed for interpolation)
#[derive(Clone, Copy, Debug)]
pub struct BoneTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for BoneTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl BoneTransform {
    /// Identity transform
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Create a new bone transform
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Create from translation only
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// Create from rotation only
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            rotation,
            ..Self::IDENTITY
        }
    }

    /// Convert to 4x4 matrix
    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Create from 4x4 matrix (extracts TRS)
    pub fn from_matrix(mat: Mat4) -> Self {
        let (scale, rotation, translation) = mat.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Linear interpolation between two transforms
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    /// Blend multiple transforms with weights
    pub fn blend(transforms: &[Self], weights: &[f32]) -> Self {
        if transforms.is_empty() {
            return Self::IDENTITY;
        }

        if transforms.len() == 1 {
            return transforms[0];
        }

        let mut result_translation = Vec3::ZERO;
        let mut result_scale = Vec3::ZERO;

        // Weighted sum for translation and scale
        for (t, w) in transforms.iter().zip(weights.iter()) {
            result_translation += t.translation * *w;
            result_scale += t.scale * *w;
        }

        // For rotation, use normalized lerp chain
        let mut result_rotation = transforms[0].rotation;
        let mut accumulated_weight = weights[0];

        for (t, w) in transforms[1..].iter().zip(weights[1..].iter()) {
            let blend_factor = *w / (accumulated_weight + *w);
            result_rotation = result_rotation.slerp(t.rotation, blend_factor);
            accumulated_weight += *w;
        }

        Self {
            translation: result_translation,
            rotation: result_rotation.normalize(),
            scale: result_scale,
        }
    }
}

/// A single bone in the skeleton
#[derive(Clone, Debug)]
pub struct Bone {
    /// Bone name for lookup
    pub name: String,
    /// Parent bone index (None for root bones)
    pub parent: Option<BoneIndex>,
    /// Children bone indices
    pub children: Vec<BoneIndex>,
    /// Local bind pose transform (relative to parent)
    pub local_bind_pose: BoneTransform,
    /// Inverse bind matrix (world space -> bone space)
    pub inverse_bind_matrix: Mat4,
}

impl Bone {
    /// Create a new bone
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent: None,
            children: Vec::new(),
            local_bind_pose: BoneTransform::IDENTITY,
            inverse_bind_matrix: Mat4::IDENTITY,
        }
    }

    /// Set the parent bone
    pub fn with_parent(mut self, parent: BoneIndex) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set the local bind pose
    pub fn with_bind_pose(mut self, pose: BoneTransform) -> Self {
        self.local_bind_pose = pose;
        self
    }

    /// Set the inverse bind matrix
    pub fn with_inverse_bind_matrix(mut self, mat: Mat4) -> Self {
        self.inverse_bind_matrix = mat;
        self
    }

    /// Check if this is a root bone
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

/// Skeleton definition (shared across entity instances)
#[derive(Clone, Debug)]
pub struct Skeleton {
    /// All bones in the skeleton
    bones: Vec<Bone>,
    /// Bone name to index lookup
    bone_names: HashMap<String, BoneIndex>,
    /// Root bone indices
    roots: Vec<BoneIndex>,
}

impl Skeleton {
    /// Create a new skeleton from a list of bones
    pub fn new(bones: Vec<Bone>) -> Self {
        let mut bone_names = HashMap::new();
        let mut roots = Vec::new();

        for (i, bone) in bones.iter().enumerate() {
            bone_names.insert(bone.name.clone(), i as BoneIndex);
            if bone.is_root() {
                roots.push(i as BoneIndex);
            }
        }

        Self {
            bones,
            bone_names,
            roots,
        }
    }

    /// Number of bones
    pub fn bone_count(&self) -> usize {
        self.bones.len()
    }

    /// Get a bone by index
    pub fn bone(&self, index: BoneIndex) -> Option<&Bone> {
        self.bones.get(index as usize)
    }

    /// Get a bone by name
    pub fn bone_by_name(&self, name: &str) -> Option<&Bone> {
        self.bone_names.get(name).and_then(|&i| self.bone(i))
    }

    /// Find bone index by name
    pub fn find_bone(&self, name: &str) -> Option<BoneIndex> {
        self.bone_names.get(name).copied()
    }

    /// Get root bone indices
    pub fn roots(&self) -> &[BoneIndex] {
        &self.roots
    }

    /// Iterate all bones
    pub fn bones(&self) -> &[Bone] {
        &self.bones
    }

    /// Create bind pose for this skeleton
    pub fn create_bind_pose(&self) -> SkeletonPose {
        let local_transforms: Vec<BoneTransform> =
            self.bones.iter().map(|b| b.local_bind_pose).collect();

        let mut pose = SkeletonPose {
            local_transforms,
            world_transforms: vec![Mat4::IDENTITY; self.bones.len()],
            skin_matrices: vec![Mat4::IDENTITY; self.bones.len()],
            dirty: true,
        };

        pose.calculate_world_transforms(self);
        pose
    }
}

/// Current pose of a skeleton (component attached to animated entities)
#[derive(Clone, Debug)]
pub struct SkeletonPose {
    /// Current local transforms for each bone
    pub local_transforms: Vec<BoneTransform>,
    /// Cached world transforms (computed from local)
    pub world_transforms: Vec<Mat4>,
    /// Final skin matrices (world * inverse_bind)
    pub skin_matrices: Vec<Mat4>,
    /// Dirty flag for recalculation
    pub dirty: bool,
}

impl Component for SkeletonPose {}

impl SkeletonPose {
    /// Create from a skeleton (bind pose)
    pub fn from_skeleton(skeleton: &Skeleton) -> Self {
        skeleton.create_bind_pose()
    }

    /// Set a bone's local transform
    pub fn set_bone_transform(&mut self, index: BoneIndex, transform: BoneTransform) {
        if let Some(t) = self.local_transforms.get_mut(index as usize) {
            *t = transform;
            self.dirty = true;
        }
    }

    /// Get a bone's local transform
    pub fn get_bone_transform(&self, index: BoneIndex) -> Option<&BoneTransform> {
        self.local_transforms.get(index as usize)
    }

    /// Get a bone's world transform
    pub fn get_world_transform(&self, index: BoneIndex) -> Option<&Mat4> {
        self.world_transforms.get(index as usize)
    }

    /// Calculate world transforms from local transforms
    pub fn calculate_world_transforms(&mut self, skeleton: &Skeleton) {
        if !self.dirty {
            return;
        }

        // Process bones in order (parents before children)
        for (i, bone) in skeleton.bones().iter().enumerate() {
            let local_matrix = self.local_transforms[i].to_matrix();

            let world_matrix = if let Some(parent_idx) = bone.parent {
                self.world_transforms[parent_idx as usize] * local_matrix
            } else {
                local_matrix
            };

            self.world_transforms[i] = world_matrix;
            self.skin_matrices[i] = world_matrix * bone.inverse_bind_matrix;
        }

        self.dirty = false;
    }

    /// Apply sampled animation transforms
    pub fn apply_animation(&mut self, bone_transforms: &[(BoneIndex, BoneTransform)]) {
        for (index, transform) in bone_transforms {
            if let Some(t) = self.local_transforms.get_mut(*index as usize) {
                *t = *transform;
            }
        }
        self.dirty = true;
    }

    /// Blend with another pose
    pub fn blend_with(&mut self, other: &SkeletonPose, factor: f32) {
        for (a, b) in self
            .local_transforms
            .iter_mut()
            .zip(other.local_transforms.iter())
        {
            *a = a.lerp(b, factor);
        }
        self.dirty = true;
    }

    /// Get skin matrices for GPU upload
    pub fn skin_matrices(&self) -> &[Mat4] {
        &self.skin_matrices
    }
}

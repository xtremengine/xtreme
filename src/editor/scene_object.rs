//! # Scene Object
//!
//! Represents a 3D object in the scene with components.

use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::editor::components::{
    AnimatorComponent, AudioListenerComponent, AudioSourceComponent, CameraComponent,
    ParticleEmitterComponent, ParticlePreset,
};
use crate::editor::hierarchy::{helpers::HasHierarchy, Hierarchy};

/// Unique identifier for scene objects
pub type ObjectId = u32;

/// A 3D object in the scene
#[derive(Clone, Debug)]
pub struct SceneObject {
    /// Unique identifier
    pub id: ObjectId,
    /// Display name
    pub name: String,
    /// Position (local space - relative to parent)
    pub position: Vec3,
    /// Rotation (euler angles in radians, local space)
    pub rotation: Vec3,
    /// Scale (local space)
    pub scale: Vec3,
    /// Object color
    pub color: [f32; 4],
    /// Whether the object is visible
    pub visible: bool,
    /// Attached script IDs
    pub scripts: Vec<u32>,
    /// Hierarchy component (parent/children relationships)
    pub hierarchy: Hierarchy,
    /// Camera component (optional)
    pub camera: Option<CameraComponent>,
    /// Texture path (relative to project)
    pub texture_path: Option<String>,
    /// Shader path (relative to project, .wgsl file)
    pub shader_path: Option<String>,
    /// Audio source component (optional)
    pub audio_source: Option<AudioSourceComponent>,
    /// Audio listener component (optional)
    pub audio_listener: Option<AudioListenerComponent>,
    /// Particle emitter component (optional)
    pub particle_emitter: Option<ParticleEmitterComponent>,
    /// Animator component (optional)
    pub animator: Option<AnimatorComponent>,
}

impl SceneObject {
    /// Create a new scene object
    pub fn new(id: ObjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.8, 0.4, 0.2, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
            shader_path: None,
            audio_source: None,
            audio_listener: None,
            particle_emitter: None,
            animator: None,
        }
    }

    /// Create a cube object
    pub fn cube(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Cube {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.8, 0.4, 0.2, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
            shader_path: None,
            audio_source: None,
            audio_listener: None,
            particle_emitter: None,
            animator: None,
        }
    }

    /// Create a camera object
    pub fn camera(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Camera {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.2, 0.6, 0.9, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: Some(CameraComponent::default()),
            texture_path: None,
            shader_path: None,
            audio_source: None,
            audio_listener: None,
            particle_emitter: None,
            animator: None,
        }
    }

    /// Create an audio source object
    pub fn audio_source(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Audio Source {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.2, 0.8, 0.4, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
            shader_path: None,
            audio_source: Some(AudioSourceComponent::default()),
            audio_listener: None,
            particle_emitter: None,
            animator: None,
        }
    }

    /// Create an audio listener object
    pub fn audio_listener(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Audio Listener {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.4, 0.9, 0.6, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
            shader_path: None,
            audio_source: None,
            audio_listener: Some(AudioListenerComponent::default()),
            particle_emitter: None,
            animator: None,
        }
    }

    /// Create a particle emitter object
    pub fn particle_emitter(id: ObjectId, position: Vec3, preset: ParticlePreset) -> Self {
        Self {
            id,
            name: format!("Particle Emitter {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [1.0, 0.5, 0.0, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
            shader_path: None,
            audio_source: None,
            audio_listener: None,
            particle_emitter: Some(ParticleEmitterComponent::with_preset(preset)),
            animator: None,
        }
    }

    /// Create an animated object
    pub fn animated(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Animated Object {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.9, 0.3, 0.9, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
            shader_path: None,
            audio_source: None,
            audio_listener: None,
            particle_emitter: None,
            animator: Some(AnimatorComponent::default()),
        }
    }

    /// Get the local model matrix (relative to parent)
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            Quat::from_euler(
                EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ),
            self.position,
        )
    }

    /// Get the model matrix for this object (legacy - returns local matrix)
    pub fn model_matrix(&self) -> Mat4 {
        self.local_matrix()
    }

    /// Compute world matrix by traversing parent chain
    pub fn world_matrix(&self, objects: &[SceneObject]) -> Mat4 {
        let local = self.local_matrix();

        if let Some(parent_id) = self.hierarchy.parent {
            if let Some(parent) = objects.iter().find(|o| o.id == parent_id) {
                let parent_world = parent.world_matrix(objects);
                return parent_world * local;
            }
        }

        local
    }

    /// Get world position (computed from hierarchy)
    pub fn world_position(&self, objects: &[SceneObject]) -> Vec3 {
        let world = self.world_matrix(objects);
        Vec3::new(world.w_axis.x, world.w_axis.y, world.w_axis.z)
    }

    /// Get axis-aligned bounding box (min, max) in world space
    pub fn aabb(&self) -> (Vec3, Vec3) {
        let half = self.scale * 0.5;
        let min = self.position - half;
        let max = self.position + half;
        (min, max)
    }

    /// Get AABB in world space (considering hierarchy)
    pub fn world_aabb(&self, objects: &[SceneObject]) -> (Vec3, Vec3) {
        let world_pos = self.world_position(objects);
        let half = self.scale * 0.5;
        (world_pos - half, world_pos + half)
    }
}

/// Implement HasHierarchy trait for SceneObject
impl HasHierarchy for SceneObject {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn hierarchy(&self) -> &Hierarchy {
        &self.hierarchy
    }

    fn hierarchy_mut(&mut self) -> &mut Hierarchy {
        &mut self.hierarchy
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn rotation(&self) -> Vec3 {
        self.rotation
    }

    fn scale(&self) -> Vec3 {
        self.scale
    }
}

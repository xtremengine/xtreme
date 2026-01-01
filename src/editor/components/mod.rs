//! Editor components that can be attached to SceneObjects.

mod animator;
mod audio;
mod camera;
mod particles;

pub use animator::{AnimationClip, AnimatorComponent};
pub use audio::{AudioListenerComponent, AudioSourceComponent};
pub use camera::{CameraComponent, CameraProjection};
pub use particles::{ParticleEmitterComponent, ParticlePreset};

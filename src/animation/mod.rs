//! # Animation Module
//!
//! Skeletal animation system with keyframe interpolation and state machines.
//!
//! ## Features
//!
//! - Skeletal animation with up to 128 bones
//! - Keyframe animation with multiple easing functions
//! - Animation state machine for complex character animation
//! - Smooth blending between animation states
//! - GLTF animation loading
//! - GPU-ready skinned mesh format
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use xtreme::animation::*;
//!
//! // Load skeleton and animations from GLTF
//! let gltf_data = load_gltf("assets/character.gltf").unwrap();
//!
//! // Create animation library
//! let mut library = AnimationLibrary::new();
//! for clip in gltf_data.clips {
//!     library.add_clip(clip);
//! }
//!
//! // Create state machine
//! let state_machine = StateMachineBuilder::new()
//!     .add_state("idle", idle_clip_id)
//!     .add_state("walk", walk_clip_id)
//!     .add_transition(
//!         StateTransition::new("idle", "walk")
//!             .with_condition(TransitionCondition::Parameter {
//!                 name: "speed".into(),
//!                 comparison: Comparison::GreaterThan,
//!                 value: 0.1,
//!             })
//!             .with_duration(0.2)
//!     )
//!     .default_state("idle")
//!     .build();
//!
//! // Create animated entity
//! let character = world.spawn()
//!     .with(Transform::default())
//!     .with(SkeletonPose::from_skeleton(&gltf_data.skeleton))
//!     .with(Animator::new(0, state_machine))
//!     .build();
//!
//! // In game loop
//! if let Some(animator) = world.get_mut::<Animator>(character) {
//!     animator.set_parameter("speed", player_speed);
//! }
//! run_animation_systems(&mut world, &library, &[skeleton], dt);
//! ```
//!
//! ## Easing Functions
//!
//! The module includes 30+ easing functions for smooth interpolation:
//! - Linear
//! - Quadratic, Cubic, Quartic, Quintic
//! - Sine, Exponential, Circular
//! - Back, Elastic, Bounce
//!
//! ## State Machine
//!
//! Animation states can be connected with transitions that trigger based on:
//! - Animation completion
//! - Parameter values (float comparisons)
//! - Triggers (events)
//! - Time in state

mod animator;
mod bone;
mod clip;
mod easing;
mod loader;
mod sampler;
mod skinning;
mod state_machine;
mod system;

pub use animator::{
    AnimationBlend, AnimationState, AnimationStateMachine, Animator, Comparison, PlaybackInfo,
    StateMachineBuilder, StateTransition, TransitionCondition,
};
pub use bone::{Bone, BoneIndex, BoneTransform, Skeleton, SkeletonPose, MAX_BONES};
pub use clip::{
    AnimationChannel, AnimationClip, AnimationEvent, AnimationLibrary, BoneTrack, ClipId, Keyframe,
};
pub use easing::{ease, EasingFunction, Lerp};
pub use loader::{load_gltf, GltfData, GltfError};
pub use sampler::{
    blend_additive, blend_multiple, blend_poses, sample_bone_track, sample_clip, sample_f32,
    sample_quat, sample_vec3, AnimationLayer, BlendMode,
};
pub use skinning::{
    BoneMatricesUniform, SkinnedMesh, SkinnedMeshGpu, SkinnedVertex, MAX_BONES_PER_VERTEX,
};
pub use system::{
    animation_update_system, get_animation_state, get_animation_time, play_animation,
    run_animation_systems, set_animation_parameter, set_animation_trigger,
    skeleton_transform_system,
};

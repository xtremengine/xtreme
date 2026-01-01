//! Animation ECS System
//!
//! Systems for updating animation and skeleton poses.

use crate::core::{Entity, World};

use super::animator::Animator;
use super::bone::{BoneTransform, Skeleton, SkeletonPose};
use super::clip::AnimationLibrary;
use super::sampler::sample_clip;

/// Update all animators and sample animations
///
/// Should run in Stage::Update
pub fn animation_update_system(
    world: &mut World,
    animation_library: &AnimationLibrary,
    skeletons: &[Skeleton],
    dt: f32,
) {
    // Get clip duration helper
    let get_clip_duration = |clip_id| {
        animation_library
            .get_clip(clip_id)
            .map(|c| c.duration)
            .unwrap_or(1.0)
    };

    // Update all animators
    if let Some(storage) = world.storage_mut::<Animator>() {
        for (_entity_idx, animator) in storage.iter_mut() {
            animator.update(dt, get_clip_duration);
        }
    }

    // Sample animations and update poses
    update_skeleton_poses(world, animation_library, skeletons);
}

/// Sample animations and update skeleton poses
fn update_skeleton_poses(
    world: &mut World,
    animation_library: &AnimationLibrary,
    _skeletons: &[Skeleton],
) {
    // Collect animator data - store entity index for later lookup
    let mut pose_updates: Vec<(u32, Vec<(u16, BoneTransform)>, Option<BlendData>)> = Vec::new();

    if let Some(animator_storage) = world.storage::<Animator>() {
        for (entity_idx, animator) in animator_storage.iter() {
            if let Some(clip_id) = animator.current_clip_id() {
                if let Some(clip) = animation_library.get_clip(clip_id) {
                    let sampled = sample_clip(clip, animator.playback.time);

                    // Handle blending
                    let blend_data = animator.blend.as_ref().and_then(|blend| {
                        let from_clip_id =
                            animation_library.find_clip(&blend.from_state).or_else(|| {
                                animator
                                    .state_machine
                                    .get_state(&blend.from_state)
                                    .map(|s| s.clip_id)
                            });

                        from_clip_id.and_then(|id| {
                            animation_library.get_clip(id).map(|from_clip| BlendData {
                                from_transforms: sample_clip(from_clip, blend.from_time),
                                factor: blend.factor(),
                            })
                        })
                    });

                    pose_updates.push((entity_idx, sampled, blend_data));
                }
            }
        }
    }

    // Apply pose updates using entity lookup
    if let Some(pose_storage) = world.storage_mut::<SkeletonPose>() {
        for (entity_idx, sampled, blend_data) in pose_updates {
            // Create entity from index (generation 0 for lookup - SparseSet only checks index)
            let entity = Entity::from_bits(entity_idx as u64);
            if let Some(pose) = pose_storage.get_mut(entity) {
                if let Some(blend) = blend_data {
                    // Blend between poses
                    apply_blended_animation(pose, &blend.from_transforms, &sampled, blend.factor);
                } else {
                    // Direct application
                    pose.apply_animation(&sampled);
                }
            }
        }
    }
}

struct BlendData {
    from_transforms: Vec<(u16, BoneTransform)>,
    factor: f32,
}

/// Apply blended animation to a pose
fn apply_blended_animation(
    pose: &mut SkeletonPose,
    from: &[(u16, BoneTransform)],
    to: &[(u16, BoneTransform)],
    factor: f32,
) {
    // Create lookup for 'to' transforms
    let to_map: std::collections::HashMap<u16, &BoneTransform> =
        to.iter().map(|(i, t)| (*i, t)).collect();

    for (bone_idx, from_transform) in from {
        if let Some(to_transform) = to_map.get(bone_idx) {
            let blended = from_transform.lerp(to_transform, factor);
            if let Some(t) = pose.local_transforms.get_mut(*bone_idx as usize) {
                *t = blended;
            }
        }
    }

    // Also apply any transforms that are only in 'to'
    for (bone_idx, transform) in to {
        if !from.iter().any(|(i, _)| i == bone_idx) {
            if let Some(t) = pose.local_transforms.get_mut(*bone_idx as usize) {
                *t = *transform;
            }
        }
    }

    pose.dirty = true;
}

/// Calculate world transforms for all skeletons
///
/// Should run in Stage::PostUpdate after animation sampling
pub fn skeleton_transform_system(world: &mut World, skeletons: &[Skeleton]) {
    // Get skeleton ID from animator - collect entity indices and skeleton ids
    let skeleton_ids: Vec<(u32, u32)> = if let Some(storage) = world.storage::<Animator>() {
        storage
            .iter()
            .map(|(idx, a)| (idx, a.skeleton_id))
            .collect()
    } else {
        Vec::new()
    };

    // Update poses
    if let Some(pose_storage) = world.storage_mut::<SkeletonPose>() {
        for (entity_idx, skeleton_id) in skeleton_ids {
            if let Some(skeleton) = skeletons.get(skeleton_id as usize) {
                // Create entity from index for lookup
                let entity = Entity::from_bits(entity_idx as u64);
                if let Some(pose) = pose_storage.get_mut(entity) {
                    pose.calculate_world_transforms(skeleton);
                }
            }
        }
    }
}

/// Combined animation system that runs both update and transform
pub fn run_animation_systems(
    world: &mut World,
    animation_library: &AnimationLibrary,
    skeletons: &[Skeleton],
    dt: f32,
) {
    animation_update_system(world, animation_library, skeletons, dt);
    skeleton_transform_system(world, skeletons);
}

/// Helper to get animation time for an entity
pub fn get_animation_time(world: &World, entity: Entity) -> Option<f32> {
    world.get::<Animator>(entity).map(|a| a.playback.time)
}

/// Helper to get current animation state name
pub fn get_animation_state(world: &World, entity: Entity) -> Option<String> {
    world
        .get::<Animator>(entity)
        .map(|a| a.playback.state.clone())
}

/// Helper to trigger an animation state change
pub fn play_animation(world: &mut World, entity: Entity, state: &str, blend_time: f32) {
    if let Some(animator) = world.get_mut::<Animator>(entity) {
        animator.play(state, blend_time);
    }
}

/// Helper to set animation parameter
pub fn set_animation_parameter(world: &mut World, entity: Entity, name: &str, value: f32) {
    if let Some(animator) = world.get_mut::<Animator>(entity) {
        animator.set_parameter(name, value);
    }
}

/// Helper to trigger animation trigger
pub fn set_animation_trigger(world: &mut World, entity: Entity, name: &str) {
    if let Some(animator) = world.get_mut::<Animator>(entity) {
        animator.set_trigger(name);
    }
}

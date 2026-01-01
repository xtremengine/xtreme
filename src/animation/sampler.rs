//! Animation Sampler
//!
//! Handles keyframe interpolation and animation sampling.

use super::bone::BoneTransform;
use super::clip::{AnimationClip, BoneTrack, Keyframe};
use glam::{Quat, Vec3};

/// Sample a Vec3 property from keyframes at given time
pub fn sample_vec3(keyframes: &[Keyframe<Vec3>], time: f32) -> Vec3 {
    if keyframes.is_empty() {
        return Vec3::ZERO;
    }

    if keyframes.len() == 1 {
        return keyframes[0].value;
    }

    // Find surrounding keyframes
    let (prev_idx, next_idx) = find_keyframe_pair(keyframes, time);

    if prev_idx == next_idx {
        return keyframes[prev_idx].value;
    }

    let prev = &keyframes[prev_idx];
    let next = &keyframes[next_idx];

    // Calculate interpolation factor
    let duration = next.time - prev.time;
    let local_time = time - prev.time;
    let t = (local_time / duration).clamp(0.0, 1.0);
    let eased_t = prev.easing.apply(t);

    prev.value.lerp(next.value, eased_t)
}

/// Sample a Quat (rotation) property from keyframes at given time
pub fn sample_quat(keyframes: &[Keyframe<Quat>], time: f32) -> Quat {
    if keyframes.is_empty() {
        return Quat::IDENTITY;
    }

    if keyframes.len() == 1 {
        return keyframes[0].value;
    }

    // Find surrounding keyframes
    let (prev_idx, next_idx) = find_keyframe_pair(keyframes, time);

    if prev_idx == next_idx {
        return keyframes[prev_idx].value;
    }

    let prev = &keyframes[prev_idx];
    let next = &keyframes[next_idx];

    // Calculate interpolation factor
    let duration = next.time - prev.time;
    let local_time = time - prev.time;
    let t = (local_time / duration).clamp(0.0, 1.0);
    let eased_t = prev.easing.apply(t);

    // Use slerp for quaternion interpolation
    prev.value.slerp(next.value, eased_t)
}

/// Sample a f32 property from keyframes at given time
pub fn sample_f32(keyframes: &[Keyframe<f32>], time: f32) -> f32 {
    if keyframes.is_empty() {
        return 0.0;
    }

    if keyframes.len() == 1 {
        return keyframes[0].value;
    }

    let (prev_idx, next_idx) = find_keyframe_pair(keyframes, time);

    if prev_idx == next_idx {
        return keyframes[prev_idx].value;
    }

    let prev = &keyframes[prev_idx];
    let next = &keyframes[next_idx];

    let duration = next.time - prev.time;
    let local_time = time - prev.time;
    let t = (local_time / duration).clamp(0.0, 1.0);
    let eased_t = prev.easing.apply(t);

    prev.value + (next.value - prev.value) * eased_t
}

/// Find the pair of keyframes surrounding a given time
fn find_keyframe_pair<T: Clone>(keyframes: &[Keyframe<T>], time: f32) -> (usize, usize) {
    // Before first keyframe
    if time <= keyframes[0].time {
        return (0, 0);
    }

    // After last keyframe
    let last_idx = keyframes.len() - 1;
    if time >= keyframes[last_idx].time {
        return (last_idx, last_idx);
    }

    // Binary search for efficiency
    let mut low = 0;
    let mut high = last_idx;

    while low < high - 1 {
        let mid = (low + high) / 2;
        if keyframes[mid].time <= time {
            low = mid;
        } else {
            high = mid;
        }
    }

    (low, high)
}

/// Sample a bone track at given time
pub fn sample_bone_track(
    track: &BoneTrack,
    time: f32,
    base_transform: &BoneTransform,
) -> BoneTransform {
    let translation = track
        .translation
        .as_ref()
        .map(|kf| sample_vec3(kf, time))
        .unwrap_or(base_transform.translation);

    let rotation = track
        .rotation
        .as_ref()
        .map(|kf| sample_quat(kf, time))
        .unwrap_or(base_transform.rotation);

    let scale = track
        .scale
        .as_ref()
        .map(|kf| sample_vec3(kf, time))
        .unwrap_or(base_transform.scale);

    BoneTransform {
        translation,
        rotation,
        scale,
    }
}

/// Sample an entire animation clip at given time
///
/// Returns a list of (bone_index, transform) pairs for all animated bones.
pub fn sample_clip(clip: &AnimationClip, time: f32) -> Vec<(u16, BoneTransform)> {
    let time = if clip.looping && clip.duration > 0.0 {
        time % clip.duration
    } else {
        time.min(clip.duration)
    };

    clip.tracks
        .iter()
        .map(|track| {
            let transform = sample_bone_track(track, time, &BoneTransform::IDENTITY);
            (track.bone_index, transform)
        })
        .collect()
}

/// Blend two poses together
///
/// Interpolates between pose_a and pose_b using the blend_factor.
/// blend_factor = 0.0 returns pose_a, blend_factor = 1.0 returns pose_b.
pub fn blend_poses(
    pose_a: &[BoneTransform],
    pose_b: &[BoneTransform],
    blend_factor: f32,
) -> Vec<BoneTransform> {
    let factor = blend_factor.clamp(0.0, 1.0);

    pose_a
        .iter()
        .zip(pose_b.iter())
        .map(|(a, b)| a.lerp(b, factor))
        .collect()
}

/// Additively blend a pose on top of a base pose
///
/// Adds the difference from identity to the base pose.
pub fn blend_additive(
    base_pose: &[BoneTransform],
    additive_pose: &[BoneTransform],
    weight: f32,
) -> Vec<BoneTransform> {
    let weight = weight.clamp(0.0, 1.0);

    base_pose
        .iter()
        .zip(additive_pose.iter())
        .map(|(base, add)| {
            // Calculate the difference from identity
            let add_translation = add.translation * weight;
            let add_rotation = Quat::IDENTITY.slerp(add.rotation, weight);
            let add_scale = Vec3::ONE.lerp(add.scale, weight);

            BoneTransform {
                translation: base.translation + add_translation,
                rotation: base.rotation * add_rotation,
                scale: base.scale * add_scale,
            }
        })
        .collect()
}

/// Blend multiple poses with weights
///
/// All poses must have the same length.
pub fn blend_multiple(poses: &[&[BoneTransform]], weights: &[f32]) -> Vec<BoneTransform> {
    if poses.is_empty() {
        return Vec::new();
    }

    if poses.len() == 1 {
        return poses[0].to_vec();
    }

    let bone_count = poses[0].len();

    // Normalize weights
    let total_weight: f32 = weights.iter().sum();
    let normalized: Vec<f32> = if total_weight > 0.0 {
        weights.iter().map(|w| w / total_weight).collect()
    } else {
        vec![1.0 / poses.len() as f32; poses.len()]
    };

    (0..bone_count)
        .map(|bone_idx| {
            let transforms: Vec<BoneTransform> = poses.iter().map(|p| p[bone_idx]).collect();
            BoneTransform::blend(&transforms, &normalized)
        })
        .collect()
}

/// Layer configuration for animation blending
#[derive(Clone, Debug)]
pub struct AnimationLayer {
    /// Layer weight (0.0 - 1.0)
    pub weight: f32,
    /// Blend mode
    pub blend_mode: BlendMode,
    /// Bone mask (if Some, only blend these bones)
    pub bone_mask: Option<Vec<u16>>,
}

/// How layers are blended
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    /// Replace the lower layers (weighted blend)
    #[default]
    Override,
    /// Add on top of lower layers
    Additive,
}

impl Default for AnimationLayer {
    fn default() -> Self {
        Self {
            weight: 1.0,
            blend_mode: BlendMode::Override,
            bone_mask: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_vec3_single() {
        let keyframes = vec![Keyframe::new(0.0, Vec3::new(1.0, 2.0, 3.0))];
        let result = sample_vec3(&keyframes, 0.5);
        assert_eq!(result, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_sample_vec3_interpolation() {
        let keyframes = vec![
            Keyframe::new(0.0, Vec3::ZERO),
            Keyframe::new(1.0, Vec3::ONE),
        ];

        let result = sample_vec3(&keyframes, 0.5);
        assert!((result - Vec3::splat(0.5)).length() < 0.001);
    }

    #[test]
    fn test_find_keyframe_pair() {
        let keyframes = vec![
            Keyframe::new(0.0, Vec3::ZERO),
            Keyframe::new(1.0, Vec3::ONE),
            Keyframe::new(2.0, Vec3::ONE * 2.0),
        ];

        assert_eq!(find_keyframe_pair(&keyframes, 0.5), (0, 1));
        assert_eq!(find_keyframe_pair(&keyframes, 1.5), (1, 2));
        assert_eq!(find_keyframe_pair(&keyframes, -1.0), (0, 0));
        assert_eq!(find_keyframe_pair(&keyframes, 10.0), (2, 2));
    }

    #[test]
    fn test_blend_poses() {
        let pose_a = vec![BoneTransform::from_translation(Vec3::ZERO)];
        let pose_b = vec![BoneTransform::from_translation(Vec3::ONE)];

        let result = blend_poses(&pose_a, &pose_b, 0.5);
        assert!((result[0].translation - Vec3::splat(0.5)).length() < 0.001);
    }
}

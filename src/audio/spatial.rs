//! Spatial Audio Calculations
//!
//! 3D audio positioning with distance attenuation and stereo panning.

use glam::Vec3;

/// Result of spatial audio calculation
#[derive(Clone, Copy, Debug, Default)]
pub struct SpatialParams {
    /// Final volume after distance attenuation (0.0 - 1.0)
    pub volume: f32,
    /// Stereo pan (-1.0 = full left, 0.0 = center, 1.0 = full right)
    pub pan: f32,
}

/// Attenuation model for distance falloff
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AttenuationModel {
    /// No attenuation (2D sound)
    None,
    /// Linear falloff: 1 - (distance / max_distance)
    #[default]
    Linear,
    /// Inverse distance: 1 / (1 + rolloff * distance)
    InverseDistance,
    /// Inverse distance squared (more realistic)
    InverseDistanceSquared,
    /// Exponential decay
    Exponential,
}

/// Calculate spatial audio parameters for a sound source
///
/// # Arguments
/// * `listener_pos` - Position of the listener
/// * `listener_forward` - Forward direction of the listener
/// * `listener_up` - Up direction of the listener
/// * `source_pos` - Position of the sound source
/// * `min_distance` - Distance at which sound is at full volume
/// * `max_distance` - Distance at which sound is inaudible
/// * `rolloff` - Rolloff factor for attenuation curve
/// * `model` - Attenuation model to use
pub fn calculate_spatial(
    listener_pos: Vec3,
    listener_forward: Vec3,
    listener_up: Vec3,
    source_pos: Vec3,
    min_distance: f32,
    max_distance: f32,
    rolloff: f32,
    model: AttenuationModel,
) -> SpatialParams {
    let to_source = source_pos - listener_pos;
    let distance = to_source.length();

    // Calculate volume attenuation
    let volume = calculate_attenuation(distance, min_distance, max_distance, rolloff, model);

    // Calculate stereo pan
    let pan = calculate_pan(to_source, listener_forward, listener_up);

    SpatialParams { volume, pan }
}

/// Calculate distance-based volume attenuation
///
/// Returns a value between 0.0 (silent) and 1.0 (full volume)
pub fn calculate_attenuation(
    distance: f32,
    min_distance: f32,
    max_distance: f32,
    rolloff: f32,
    model: AttenuationModel,
) -> f32 {
    // At or inside min_distance: full volume
    if distance <= min_distance {
        return 1.0;
    }

    // Beyond max_distance: silent
    if distance >= max_distance {
        return 0.0;
    }

    let clamped_distance = distance - min_distance;
    let range = max_distance - min_distance;

    match model {
        AttenuationModel::None => 1.0,

        AttenuationModel::Linear => {
            // Linear falloff from 1 to 0
            1.0 - (clamped_distance / range)
        }

        AttenuationModel::InverseDistance => {
            // 1 / (1 + rolloff * normalized_distance)
            let normalized = clamped_distance / min_distance;
            1.0 / (1.0 + rolloff * normalized)
        }

        AttenuationModel::InverseDistanceSquared => {
            // 1 / (1 + rolloff * normalized_distance^2)
            let normalized = clamped_distance / min_distance;
            1.0 / (1.0 + rolloff * normalized * normalized)
        }

        AttenuationModel::Exponential => {
            // e^(-rolloff * normalized_distance)
            let normalized = clamped_distance / range;
            (-rolloff * normalized).exp()
        }
    }
}

/// Calculate stereo panning based on direction to source
///
/// Returns a value between -1.0 (full left) and 1.0 (full right)
pub fn calculate_pan(to_source: Vec3, listener_forward: Vec3, listener_up: Vec3) -> f32 {
    // If source is at listener position, no panning
    if to_source.length_squared() < 0.0001 {
        return 0.0;
    }

    let direction = to_source.normalize();

    // Calculate right vector from forward and up
    let listener_right = listener_forward.cross(listener_up).normalize();

    // Project direction onto listener's horizontal plane
    // Dot product with right vector gives left-right position
    let pan = direction.dot(listener_right);

    pan.clamp(-1.0, 1.0)
}

/// Apply stereo pan to left/right volumes
///
/// Returns (left_volume, right_volume)
pub fn apply_pan(volume: f32, pan: f32) -> (f32, f32) {
    // Constant power panning for smoother transitions
    let angle = (pan + 1.0) * std::f32::consts::FRAC_PI_4; // 0 to PI/2
    let left = angle.cos() * volume;
    let right = angle.sin() * volume;
    (left, right)
}

/// Calculate 3D distance between two points
pub fn distance_3d(a: Vec3, b: Vec3) -> f32 {
    (b - a).length()
}

/// Calculate squared distance (faster, avoids sqrt)
#[allow(dead_code)]
pub fn distance_squared_3d(a: Vec3, b: Vec3) -> f32 {
    (b - a).length_squared()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attenuation_at_min_distance() {
        let vol = calculate_attenuation(1.0, 1.0, 100.0, 1.0, AttenuationModel::Linear);
        assert!((vol - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_attenuation_at_max_distance() {
        let vol = calculate_attenuation(100.0, 1.0, 100.0, 1.0, AttenuationModel::Linear);
        assert!((vol - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_attenuation_linear_midpoint() {
        let vol = calculate_attenuation(50.0, 0.0, 100.0, 1.0, AttenuationModel::Linear);
        assert!((vol - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_pan_right() {
        let pan = calculate_pan(
            Vec3::X,     // source to the right
            Vec3::NEG_Z, // looking forward (-Z)
            Vec3::Y,     // up is Y
        );
        assert!(pan > 0.9, "Pan should be ~1.0 (right), got {}", pan);
    }

    #[test]
    fn test_pan_left() {
        let pan = calculate_pan(
            Vec3::NEG_X, // source to the left
            Vec3::NEG_Z, // looking forward (-Z)
            Vec3::Y,     // up is Y
        );
        assert!(pan < -0.9, "Pan should be ~-1.0 (left), got {}", pan);
    }

    #[test]
    fn test_pan_center() {
        let pan = calculate_pan(
            Vec3::NEG_Z, // source in front
            Vec3::NEG_Z, // looking forward
            Vec3::Y,
        );
        assert!(pan.abs() < 0.1, "Pan should be ~0.0 (center), got {}", pan);
    }

    #[test]
    fn test_apply_pan_center() {
        let (left, right) = apply_pan(1.0, 0.0);
        // At center, both should be roughly equal (constant power)
        assert!((left - right).abs() < 0.1);
    }

    #[test]
    fn test_apply_pan_full_right() {
        let (left, right) = apply_pan(1.0, 1.0);
        assert!(right > left);
    }
}

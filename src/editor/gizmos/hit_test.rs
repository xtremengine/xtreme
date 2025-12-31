//! Gizmo hit testing for mouse interaction.

use glam::Vec3;

use super::math::{ray_line_distance, ray_plane_intersection};
use super::types::{GizmoAxis, GizmoMode};
use super::Gizmo;
use crate::editor::selection::Ray;

impl Gizmo {
    /// Hit test against gizmo axes
    pub fn hit_test(&self, ray: &Ray, position: Vec3, scale: f32) -> GizmoAxis {
        let threshold = scale * 0.25; // Hit threshold (increased for better detection)

        match self.mode {
            GizmoMode::Translate => self.hit_test_translate(ray, position, scale, threshold),
            GizmoMode::Rotate => self.hit_test_rotate(ray, position, scale, threshold),
            GizmoMode::Scale => self.hit_test_scale(ray, position, scale, threshold),
        }
    }

    /// Hit test for translate gizmo
    fn hit_test_translate(&self, ray: &Ray, pos: Vec3, scale: f32, threshold: f32) -> GizmoAxis {
        let length = scale * 1.5;
        let plane_offset = scale * 0.3;
        let plane_size = scale * 0.4;

        // Test plane handles first (they're in front)
        // XY plane
        let xy_center = pos
            + Vec3::new(
                plane_offset + plane_size * 0.5,
                plane_offset + plane_size * 0.5,
                0.0,
            );
        if let Some(t) = ray_plane_intersection(ray, xy_center, Vec3::Z) {
            let hit = ray.origin + ray.direction * t;
            let local = hit - pos;
            if local.x > plane_offset
                && local.x < plane_offset + plane_size
                && local.y > plane_offset
                && local.y < plane_offset + plane_size
                && local.z.abs() < threshold
            {
                return GizmoAxis::XY;
            }
        }

        // XZ plane
        let xz_center = pos
            + Vec3::new(
                plane_offset + plane_size * 0.5,
                0.0,
                plane_offset + plane_size * 0.5,
            );
        if let Some(t) = ray_plane_intersection(ray, xz_center, Vec3::Y) {
            let hit = ray.origin + ray.direction * t;
            let local = hit - pos;
            if local.x > plane_offset
                && local.x < plane_offset + plane_size
                && local.z > plane_offset
                && local.z < plane_offset + plane_size
                && local.y.abs() < threshold
            {
                return GizmoAxis::XZ;
            }
        }

        // YZ plane
        let yz_center = pos
            + Vec3::new(
                0.0,
                plane_offset + plane_size * 0.5,
                plane_offset + plane_size * 0.5,
            );
        if let Some(t) = ray_plane_intersection(ray, yz_center, Vec3::X) {
            let hit = ray.origin + ray.direction * t;
            let local = hit - pos;
            if local.y > plane_offset
                && local.y < plane_offset + plane_size
                && local.z > plane_offset
                && local.z < plane_offset + plane_size
                && local.x.abs() < threshold
            {
                return GizmoAxis::YZ;
            }
        }

        // Test axis lines
        if let Some(dist) = ray_line_distance(ray, pos, pos + Vec3::X * length) {
            if dist < threshold {
                return GizmoAxis::X;
            }
        }
        if let Some(dist) = ray_line_distance(ray, pos, pos + Vec3::Y * length) {
            if dist < threshold {
                return GizmoAxis::Y;
            }
        }
        if let Some(dist) = ray_line_distance(ray, pos, pos + Vec3::Z * length) {
            if dist < threshold {
                return GizmoAxis::Z;
            }
        }

        GizmoAxis::None
    }

    /// Hit test for rotate gizmo
    fn hit_test_rotate(&self, ray: &Ray, pos: Vec3, scale: f32, threshold: f32) -> GizmoAxis {
        let radius = scale * 1.2;
        let ring_threshold = threshold * 1.5;

        // Test each ring
        // X ring (YZ plane)
        if let Some(t) = ray_plane_intersection(ray, pos, Vec3::X) {
            let hit = ray.origin + ray.direction * t;
            let local = hit - pos;
            let dist_from_ring = (local.length() - radius).abs();
            if dist_from_ring < ring_threshold {
                return GizmoAxis::X;
            }
        }

        // Y ring (XZ plane)
        if let Some(t) = ray_plane_intersection(ray, pos, Vec3::Y) {
            let hit = ray.origin + ray.direction * t;
            let local = hit - pos;
            let dist_from_ring = (local.length() - radius).abs();
            if dist_from_ring < ring_threshold {
                return GizmoAxis::Y;
            }
        }

        // Z ring (XY plane)
        if let Some(t) = ray_plane_intersection(ray, pos, Vec3::Z) {
            let hit = ray.origin + ray.direction * t;
            let local = hit - pos;
            let dist_from_ring = (local.length() - radius).abs();
            if dist_from_ring < ring_threshold {
                return GizmoAxis::Z;
            }
        }

        GizmoAxis::None
    }

    /// Hit test for scale gizmo
    fn hit_test_scale(&self, ray: &Ray, pos: Vec3, scale: f32, threshold: f32) -> GizmoAxis {
        let length = scale * 1.3;
        let cube_size = scale * 0.12;
        let center_size = cube_size * 1.2;

        // Test center cube first
        let center_half = center_size * 0.5;
        if ray
            .intersect_aabb(
                pos - Vec3::splat(center_half),
                pos + Vec3::splat(center_half),
            )
            .is_some()
        {
            return GizmoAxis::XYZ;
        }

        // Test axis cubes
        let end_x = pos + Vec3::X * length;
        let half = cube_size * 0.5;
        if ray
            .intersect_aabb(end_x - Vec3::splat(half), end_x + Vec3::splat(half))
            .is_some()
        {
            return GizmoAxis::X;
        }

        let end_y = pos + Vec3::Y * length;
        if ray
            .intersect_aabb(end_y - Vec3::splat(half), end_y + Vec3::splat(half))
            .is_some()
        {
            return GizmoAxis::Y;
        }

        let end_z = pos + Vec3::Z * length;
        if ray
            .intersect_aabb(end_z - Vec3::splat(half), end_z + Vec3::splat(half))
            .is_some()
        {
            return GizmoAxis::Z;
        }

        // Test axis lines
        if let Some(dist) = ray_line_distance(ray, pos, pos + Vec3::X * length) {
            if dist < threshold {
                return GizmoAxis::X;
            }
        }
        if let Some(dist) = ray_line_distance(ray, pos, pos + Vec3::Y * length) {
            if dist < threshold {
                return GizmoAxis::Y;
            }
        }
        if let Some(dist) = ray_line_distance(ray, pos, pos + Vec3::Z * length) {
            if dist < threshold {
                return GizmoAxis::Z;
            }
        }

        GizmoAxis::None
    }
}

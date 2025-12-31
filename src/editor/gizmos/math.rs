//! Math helper functions for gizmo calculations.

use crate::editor::selection::Ray;
use glam::Vec3;

/// Ray-plane intersection, returns t parameter
pub fn ray_plane_intersection(ray: &Ray, plane_point: Vec3, plane_normal: Vec3) -> Option<f32> {
    let denom = ray.direction.dot(plane_normal);
    if denom.abs() < 0.0001 {
        return None;
    }

    let t = (plane_point - ray.origin).dot(plane_normal) / denom;
    if t < 0.0 {
        return None;
    }

    Some(t)
}

/// Calculate minimum distance from ray to line segment
pub fn ray_line_distance(ray: &Ray, line_start: Vec3, line_end: Vec3) -> Option<f32> {
    let line_dir = line_end - line_start;
    let line_len = line_dir.length();
    if line_len < 0.0001 {
        return None;
    }
    let line_dir = line_dir / line_len;

    // Using the formula for distance between two skew lines
    let w0 = ray.origin - line_start;
    let a = ray.direction.dot(ray.direction);
    let b = ray.direction.dot(line_dir);
    let c = line_dir.dot(line_dir);
    let d = ray.direction.dot(w0);
    let e = line_dir.dot(w0);

    let denom = a * c - b * b;
    if denom.abs() < 0.0001 {
        // Lines are parallel
        return Some(w0.cross(ray.direction).length());
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    // Clamp t to line segment (normalize to 0..1 range)
    let t_param = t / line_len;
    let t_clamped = t_param.clamp(0.0, 1.0);

    let closest_ray = ray.origin + ray.direction * s.max(0.0);
    let closest_line = line_start + (line_end - line_start) * t_clamped;

    Some((closest_ray - closest_line).length())
}

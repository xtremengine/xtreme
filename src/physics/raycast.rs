//! # Raycasting

use glam::Vec3;
use crate::core::Entity;
use super::shapes::AABB;

/// Ray for casting
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Raycast hit result
#[derive(Clone, Copy, Debug)]
pub struct RaycastHit {
    pub entity: Entity,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

/// Raycast query options
pub struct RaycastQuery {
    pub ray: Ray,
    pub max_distance: f32,
    pub layer_mask: u32,
}

impl RaycastQuery {
    pub fn new(ray: Ray) -> Self {
        Self {
            ray,
            max_distance: f32::MAX,
            layer_mask: u32::MAX,
        }
    }

    pub fn with_max_distance(mut self, distance: f32) -> Self {
        self.max_distance = distance;
        self
    }
}

/// Ray vs AABB intersection test
pub fn ray_vs_aabb(ray: &Ray, aabb: &AABB) -> Option<f32> {
    let inv_dir = Vec3::new(
        1.0 / ray.direction.x,
        1.0 / ray.direction.y,
        1.0 / ray.direction.z,
    );

    let t1 = (aabb.min - ray.origin) * inv_dir;
    let t2 = (aabb.max - ray.origin) * inv_dir;

    let tmin = t1.min(t2);
    let tmax = t1.max(t2);

    let tmin = tmin.x.max(tmin.y).max(tmin.z);
    let tmax = tmax.x.min(tmax.y).min(tmax.z);

    if tmax >= tmin && tmax >= 0.0 {
        Some(if tmin >= 0.0 { tmin } else { tmax })
    } else {
        None
    }
}

/// Ray vs plane intersection
pub fn ray_vs_plane(ray: &Ray, plane_normal: Vec3, plane_d: f32) -> Option<f32> {
    let denom = ray.direction.dot(plane_normal);
    if denom.abs() < 1e-6 {
        return None;
    }

    let t = -(ray.origin.dot(plane_normal) + plane_d) / denom;
    if t >= 0.0 { Some(t) } else { None }
}

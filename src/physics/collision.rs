//! # Collision Detection

use super::shapes::AABB;
use crate::core::Entity;
use glam::Vec3;

/// Contact point information
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub point: Vec3,
    pub normal: Vec3,
    pub depth: f32,
}

/// Pair of colliding entities
#[derive(Clone, Copy, Debug)]
pub struct CollisionPair {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contact: Contact,
}

/// Collision world for managing all colliders
pub struct CollisionWorld {
    _placeholder: (),
}

impl CollisionWorld {
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl Default for CollisionWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// AABB vs AABB collision test
pub fn aabb_vs_aabb(a: &AABB, b: &AABB) -> Option<Contact> {
    if !a.intersects(b) {
        return None;
    }

    // Find overlap on each axis
    let overlap_x = (a.max.x.min(b.max.x) - a.min.x.max(b.min.x)).max(0.0);
    let overlap_y = (a.max.y.min(b.max.y) - a.min.y.max(b.min.y)).max(0.0);
    let overlap_z = (a.max.z.min(b.max.z) - a.min.z.max(b.min.z)).max(0.0);

    // Find minimum overlap axis
    let (depth, normal) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        let sign = if a.center().x < b.center().x {
            -1.0
        } else {
            1.0
        };
        (overlap_x, Vec3::new(sign, 0.0, 0.0))
    } else if overlap_y <= overlap_z {
        let sign = if a.center().y < b.center().y {
            -1.0
        } else {
            1.0
        };
        (overlap_y, Vec3::new(0.0, sign, 0.0))
    } else {
        let sign = if a.center().z < b.center().z {
            -1.0
        } else {
            1.0
        };
        (overlap_z, Vec3::new(0.0, 0.0, sign))
    };

    Some(Contact {
        point: (a.center() + b.center()) * 0.5,
        normal,
        depth,
    })
}

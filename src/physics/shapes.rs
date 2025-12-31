//! # Collision Shapes

use glam::Vec3;

/// Shape trait for collision detection
pub trait Shape {
    fn bounds(&self) -> AABB;
}

/// Axis-Aligned Bounding Box
#[derive(Clone, Copy, Debug, Default)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub fn merge(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

impl Shape for AABB {
    fn bounds(&self) -> AABB {
        *self
    }
}

/// Sphere shape
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }

    pub fn intersects(&self, other: &Sphere) -> bool {
        let dist_sq = (self.center - other.center).length_squared();
        let radii_sum = self.radius + other.radius;
        dist_sq <= radii_sum * radii_sum
    }
}

impl Shape for Sphere {
    fn bounds(&self) -> AABB {
        AABB {
            min: self.center - Vec3::splat(self.radius),
            max: self.center + Vec3::splat(self.radius),
        }
    }
}

/// Oriented Bounding Box (placeholder)
#[derive(Clone, Copy, Debug)]
pub struct OBB {
    pub center: Vec3,
    pub half_extents: Vec3,
    // orientation: Quat
}

impl OBB {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self { center, half_extents }
    }
}

impl Shape for OBB {
    fn bounds(&self) -> AABB {
        AABB::from_center_size(self.center, self.half_extents * 2.0)
    }
}

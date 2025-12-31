//! # Selection System
//!
//! Handles object selection in the editor viewport.

use glam::{EulerRot, Mat4, Quat, Vec3};

use crate::editor::components::CameraComponent;
use crate::editor::hierarchy::{helpers::HasHierarchy, Hierarchy};

/// Unique identifier for scene objects
pub type ObjectId = u32;

/// A 3D object in the scene
#[derive(Clone, Debug)]
pub struct SceneObject {
    /// Unique identifier
    pub id: ObjectId,
    /// Display name
    pub name: String,
    /// Position (local space - relative to parent)
    pub position: Vec3,
    /// Rotation (euler angles in radians, local space)
    pub rotation: Vec3,
    /// Scale (local space)
    pub scale: Vec3,
    /// Object color
    pub color: [f32; 4],
    /// Whether the object is visible
    pub visible: bool,
    /// Attached script IDs
    pub scripts: Vec<u32>,
    /// Hierarchy component (parent/children relationships)
    pub hierarchy: Hierarchy,
    /// Camera component (optional)
    pub camera: Option<CameraComponent>,
    /// Texture path (relative to project)
    pub texture_path: Option<String>,
}

impl SceneObject {
    /// Create a new scene object
    pub fn new(id: ObjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.8, 0.4, 0.2, 1.0], // Orange
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
        }
    }

    /// Create a cube object
    pub fn cube(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Cube {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.8, 0.4, 0.2, 1.0],
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: None,
            texture_path: None,
        }
    }

    /// Create a camera object
    pub fn camera(id: ObjectId, position: Vec3) -> Self {
        Self {
            id,
            name: format!("Camera {}", id),
            position,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
            color: [0.2, 0.6, 0.9, 1.0], // Blue for cameras
            visible: true,
            scripts: Vec::new(),
            hierarchy: Hierarchy::new(),
            camera: Some(CameraComponent::default()),
            texture_path: None,
        }
    }

    /// Get the local model matrix (relative to parent)
    pub fn local_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            Quat::from_euler(
                EulerRot::XYZ,
                self.rotation.x,
                self.rotation.y,
                self.rotation.z,
            ),
            self.position,
        )
    }

    /// Get the model matrix for this object (legacy - returns local matrix)
    pub fn model_matrix(&self) -> Mat4 {
        self.local_matrix()
    }

    /// Compute world matrix by traversing parent chain
    pub fn world_matrix(&self, objects: &[SceneObject]) -> Mat4 {
        let local = self.local_matrix();

        if let Some(parent_id) = self.hierarchy.parent {
            if let Some(parent) = objects.iter().find(|o| o.id == parent_id) {
                let parent_world = parent.world_matrix(objects);
                return parent_world * local;
            }
        }

        local
    }

    /// Get world position (computed from hierarchy)
    pub fn world_position(&self, objects: &[SceneObject]) -> Vec3 {
        let world = self.world_matrix(objects);
        Vec3::new(world.w_axis.x, world.w_axis.y, world.w_axis.z)
    }

    /// Get axis-aligned bounding box (min, max) in world space
    pub fn aabb(&self) -> (Vec3, Vec3) {
        let half = self.scale * 0.5;
        let min = self.position - half;
        let max = self.position + half;
        (min, max)
    }

    /// Get AABB in world space (considering hierarchy)
    pub fn world_aabb(&self, objects: &[SceneObject]) -> (Vec3, Vec3) {
        let world_pos = self.world_position(objects);
        let half = self.scale * 0.5;
        (world_pos - half, world_pos + half)
    }
}

/// Implement HasHierarchy trait for SceneObject
impl HasHierarchy for SceneObject {
    fn id(&self) -> ObjectId {
        self.id
    }

    fn hierarchy(&self) -> &Hierarchy {
        &self.hierarchy
    }

    fn hierarchy_mut(&mut self) -> &mut Hierarchy {
        &mut self.hierarchy
    }

    fn position(&self) -> Vec3 {
        self.position
    }

    fn rotation(&self) -> Vec3 {
        self.rotation
    }

    fn scale(&self) -> Vec3 {
        self.scale
    }
}

/// Selection state
#[derive(Clone, Debug, Default)]
pub struct Selection {
    /// Currently selected object IDs
    selected: Vec<ObjectId>,
    /// Hovered object (mouse over)
    hovered: Option<ObjectId>,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self::default()
    }

    /// Select a single object (clears previous selection)
    pub fn select(&mut self, id: ObjectId) {
        self.selected.clear();
        self.selected.push(id);
    }

    /// Add an object to the selection
    pub fn add(&mut self, id: ObjectId) {
        if !self.selected.contains(&id) {
            self.selected.push(id);
        }
    }

    /// Remove an object from the selection
    pub fn remove(&mut self, id: ObjectId) {
        self.selected.retain(|&x| x != id);
    }

    /// Toggle selection of an object
    pub fn toggle(&mut self, id: ObjectId) {
        if self.selected.contains(&id) {
            self.remove(id);
        } else {
            self.add(id);
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.selected.clear();
    }

    /// Check if an object is selected
    pub fn is_selected(&self, id: ObjectId) -> bool {
        self.selected.contains(&id)
    }

    /// Get the first selected object
    pub fn first(&self) -> Option<ObjectId> {
        self.selected.first().copied()
    }

    /// Get all selected objects
    pub fn all(&self) -> &[ObjectId] {
        &self.selected
    }

    /// Get selection count
    pub fn count(&self) -> usize {
        self.selected.len()
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Set hovered object
    pub fn set_hovered(&mut self, id: Option<ObjectId>) {
        self.hovered = id;
    }

    /// Get hovered object
    pub fn hovered(&self) -> Option<ObjectId> {
        self.hovered
    }
}

/// Ray for picking
#[derive(Clone, Debug)]
pub struct Ray {
    /// Ray origin
    pub origin: Vec3,
    /// Ray direction (normalized)
    pub direction: Vec3,
}

impl Ray {
    /// Create a new ray
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Create a ray from screen coordinates
    pub fn from_screen(
        screen_x: f32,
        screen_y: f32,
        viewport_width: f32,
        viewport_height: f32,
        view_proj_inverse: Mat4,
    ) -> Self {
        // Convert to normalized device coordinates [-1, 1]
        let ndc_x = (2.0 * screen_x / viewport_width) - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_y / viewport_height); // Flip Y

        // Near and far points in NDC
        let near_ndc = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let far_ndc = glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

        // Transform to world space
        let near_world = view_proj_inverse * near_ndc;
        let far_world = view_proj_inverse * far_ndc;

        // Perspective divide
        let near = Vec3::new(
            near_world.x / near_world.w,
            near_world.y / near_world.w,
            near_world.z / near_world.w,
        );
        let far = Vec3::new(
            far_world.x / far_world.w,
            far_world.y / far_world.w,
            far_world.z / far_world.w,
        );

        let direction = (far - near).normalize();
        Self::new(near, direction)
    }

    /// Test ray against AABB, returns distance if hit
    pub fn intersect_aabb(&self, min: Vec3, max: Vec3) -> Option<f32> {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (min.x - self.origin.x) * inv_dir.x;
        let t2 = (max.x - self.origin.x) * inv_dir.x;
        let t3 = (min.y - self.origin.y) * inv_dir.y;
        let t4 = (max.y - self.origin.y) * inv_dir.y;
        let t5 = (min.z - self.origin.z) * inv_dir.z;
        let t6 = (max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(if tmin < 0.0 { tmax } else { tmin })
        }
    }
}

/// Pick the closest object hit by a ray
pub fn pick_object(ray: &Ray, objects: &[SceneObject]) -> Option<ObjectId> {
    let mut closest: Option<(ObjectId, f32)> = None;

    for obj in objects {
        if !obj.visible {
            continue;
        }

        let (min, max) = obj.aabb();
        if let Some(dist) = ray.intersect_aabb(min, max) {
            match closest {
                None => closest = Some((obj.id, dist)),
                Some((_, prev_dist)) if dist < prev_dist => {
                    closest = Some((obj.id, dist));
                }
                _ => {}
            }
        }
    }

    closest.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection() {
        let mut sel = Selection::new();
        assert!(sel.is_empty());

        sel.select(1);
        assert!(sel.is_selected(1));
        assert_eq!(sel.count(), 1);

        sel.add(2);
        assert!(sel.is_selected(2));
        assert_eq!(sel.count(), 2);

        sel.toggle(1);
        assert!(!sel.is_selected(1));
        assert_eq!(sel.count(), 1);
    }

    #[test]
    fn test_ray_aabb() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, -10.0), Vec3::new(0.0, 0.0, 1.0));
        let min = Vec3::new(-1.0, -1.0, -1.0);
        let max = Vec3::new(1.0, 1.0, 1.0);

        let hit = ray.intersect_aabb(min, max);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 9.0).abs() < 0.001);
    }
}

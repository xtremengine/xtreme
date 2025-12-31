//! Gizmo drag handling for object manipulation.

use glam::Vec3;

use super::math::ray_plane_intersection;
use super::types::{GizmoAxis, GizmoDelta, GizmoMode};
use super::Gizmo;
use crate::editor::selection::Ray;

impl Gizmo {
    /// Begin dragging on an axis
    pub fn begin_drag(
        &mut self,
        axis: GizmoAxis,
        ray: &Ray,
        gizmo_position: Vec3,
        object_transform: (Vec3, Vec3, Vec3),
    ) {
        self.active_axis = axis;
        self.is_dragging = true;
        self.drag_start_transform = object_transform;
        self.drag_accumulated = Vec3::ZERO;

        // Calculate drag start point based on axis
        self.drag_start_world = self
            .project_ray_to_axis(ray, gizmo_position, axis)
            .unwrap_or(gizmo_position);
    }

    /// Update drag with current ray, returns delta if changed
    pub fn update_drag(&mut self, ray: &Ray, gizmo_position: Vec3) -> Option<GizmoDelta> {
        if !self.is_dragging {
            return None;
        }

        let current_world = self.project_ray_to_axis(ray, gizmo_position, self.active_axis)?;
        let drag_delta = current_world - self.drag_start_world;

        // Only return delta if there's meaningful change (reduced threshold for smoother movement)
        if drag_delta.length_squared() < 0.000001 {
            return None;
        }

        let delta = match self.mode {
            GizmoMode::Translate => {
                let translation = self.constrain_to_axis(drag_delta, self.active_axis);
                GizmoDelta {
                    translation,
                    rotation: Vec3::ZERO,
                    scale: Vec3::ZERO,
                }
            }
            GizmoMode::Rotate => {
                let rotation = self.calculate_rotation_delta(
                    gizmo_position,
                    self.drag_start_world,
                    current_world,
                );
                GizmoDelta {
                    translation: Vec3::ZERO,
                    rotation,
                    scale: Vec3::ZERO,
                }
            }
            GizmoMode::Scale => {
                let scale_factor = self.calculate_scale_delta(drag_delta);
                GizmoDelta {
                    translation: Vec3::ZERO,
                    rotation: Vec3::ZERO,
                    scale: scale_factor,
                }
            }
        };

        // Update start for incremental delta
        self.drag_start_world = current_world;

        Some(delta)
    }

    /// End dragging
    pub fn end_drag(&mut self) {
        self.active_axis = GizmoAxis::None;
        self.is_dragging = false;
        self.drag_accumulated = Vec3::ZERO;
    }

    /// Project ray onto axis constraint plane
    fn project_ray_to_axis(&self, ray: &Ray, center: Vec3, axis: GizmoAxis) -> Option<Vec3> {
        match axis {
            GizmoAxis::X => {
                // Project to XY or XZ plane depending on camera angle
                let normal = if ray.direction.z.abs() > ray.direction.y.abs() {
                    Vec3::Y
                } else {
                    Vec3::Z
                };
                ray_plane_intersection(ray, center, normal).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::Y => {
                let normal = if ray.direction.z.abs() > ray.direction.x.abs() {
                    Vec3::X
                } else {
                    Vec3::Z
                };
                ray_plane_intersection(ray, center, normal).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::Z => {
                let normal = if ray.direction.y.abs() > ray.direction.x.abs() {
                    Vec3::X
                } else {
                    Vec3::Y
                };
                ray_plane_intersection(ray, center, normal).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::XY => {
                ray_plane_intersection(ray, center, Vec3::Z).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::XZ => {
                ray_plane_intersection(ray, center, Vec3::Y).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::YZ => {
                ray_plane_intersection(ray, center, Vec3::X).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::XYZ => {
                // For center/free, use camera-facing plane
                let normal = -ray.direction;
                ray_plane_intersection(ray, center, normal).map(|t| ray.origin + ray.direction * t)
            }
            GizmoAxis::None => None,
        }
    }

    /// Constrain delta to axis
    fn constrain_to_axis(&self, delta: Vec3, axis: GizmoAxis) -> Vec3 {
        match axis {
            GizmoAxis::X => Vec3::new(delta.x, 0.0, 0.0),
            GizmoAxis::Y => Vec3::new(0.0, delta.y, 0.0),
            GizmoAxis::Z => Vec3::new(0.0, 0.0, delta.z),
            GizmoAxis::XY => Vec3::new(delta.x, delta.y, 0.0),
            GizmoAxis::XZ => Vec3::new(delta.x, 0.0, delta.z),
            GizmoAxis::YZ => Vec3::new(0.0, delta.y, delta.z),
            GizmoAxis::XYZ | GizmoAxis::None => delta,
        }
    }

    /// Calculate rotation delta from arc on ring
    fn calculate_rotation_delta(&self, center: Vec3, start: Vec3, current: Vec3) -> Vec3 {
        let start_dir = (start - center).normalize();
        let current_dir = (current - center).normalize();

        // Calculate angle
        let dot = start_dir.dot(current_dir).clamp(-1.0, 1.0);
        let angle = dot.acos();

        // Determine sign using cross product
        let cross = start_dir.cross(current_dir);

        match self.active_axis {
            GizmoAxis::X => Vec3::new(angle * cross.x.signum(), 0.0, 0.0),
            GizmoAxis::Y => Vec3::new(0.0, angle * cross.y.signum(), 0.0),
            GizmoAxis::Z => Vec3::new(0.0, 0.0, angle * cross.z.signum()),
            _ => Vec3::ZERO,
        }
    }

    /// Calculate scale delta
    fn calculate_scale_delta(&self, drag_delta: Vec3) -> Vec3 {
        let sensitivity = 0.01; // Scale sensitivity

        match self.active_axis {
            GizmoAxis::X => Vec3::new(drag_delta.x * sensitivity, 0.0, 0.0),
            GizmoAxis::Y => Vec3::new(0.0, drag_delta.y * sensitivity, 0.0),
            GizmoAxis::Z => Vec3::new(0.0, 0.0, drag_delta.z * sensitivity),
            GizmoAxis::XYZ => {
                // Uniform scale based on average drag
                let avg = (drag_delta.x + drag_delta.y + drag_delta.z) / 3.0 * sensitivity;
                Vec3::splat(avg)
            }
            _ => Vec3::ZERO,
        }
    }
}

//! Gizmo vertex generation for rendering.

use glam::Vec3;
use std::f32::consts::PI;

use super::types::{colors, GizmoAxis, GizmoMode, GizmoVertex};
use super::Gizmo;

impl Gizmo {
    /// Generate vertices for the gizmo at a given position and scale
    pub fn generate_vertices(
        &self,
        position: Vec3,
        scale: f32,
    ) -> (Vec<GizmoVertex>, Vec<GizmoVertex>) {
        let mut lines = Vec::new();
        let mut triangles = Vec::new();

        match self.mode {
            GizmoMode::Translate => {
                self.generate_translate_gizmo(&mut lines, &mut triangles, position, scale);
            }
            GizmoMode::Rotate => {
                self.generate_rotate_gizmo(&mut lines, position, scale);
            }
            GizmoMode::Scale => {
                self.generate_scale_gizmo(&mut lines, &mut triangles, position, scale);
            }
        }

        (lines, triangles)
    }

    /// Generate translate gizmo (arrows)
    pub(super) fn generate_translate_gizmo(
        &self,
        lines: &mut Vec<GizmoVertex>,
        triangles: &mut Vec<GizmoVertex>,
        pos: Vec3,
        scale: f32,
    ) {
        let length = scale * 1.5;
        let arrow_size = scale * 0.15;
        let plane_size = scale * 0.4;
        let plane_offset = scale * 0.3;

        // Get colors (highlight if hovered/active)
        let x_color = self.get_axis_color(GizmoAxis::X);
        let y_color = self.get_axis_color(GizmoAxis::Y);
        let z_color = self.get_axis_color(GizmoAxis::Z);

        // X axis line and arrow
        lines.push(GizmoVertex::new(pos, x_color));
        lines.push(GizmoVertex::new(pos + Vec3::X * length, x_color));
        self.add_arrow_head(
            triangles,
            pos + Vec3::X * length,
            Vec3::X,
            arrow_size,
            x_color,
        );

        // Y axis line and arrow
        lines.push(GizmoVertex::new(pos, y_color));
        lines.push(GizmoVertex::new(pos + Vec3::Y * length, y_color));
        self.add_arrow_head(
            triangles,
            pos + Vec3::Y * length,
            Vec3::Y,
            arrow_size,
            y_color,
        );

        // Z axis line and arrow
        lines.push(GizmoVertex::new(pos, z_color));
        lines.push(GizmoVertex::new(pos + Vec3::Z * length, z_color));
        self.add_arrow_head(
            triangles,
            pos + Vec3::Z * length,
            Vec3::Z,
            arrow_size,
            z_color,
        );

        // Plane handles (small squares)
        let xy_color = self.get_axis_color(GizmoAxis::XY);
        let xz_color = self.get_axis_color(GizmoAxis::XZ);
        let yz_color = self.get_axis_color(GizmoAxis::YZ);

        // XY plane
        let xy_base = pos + Vec3::new(plane_offset, plane_offset, 0.0);
        triangles.push(GizmoVertex::new(xy_base, xy_color));
        triangles.push(GizmoVertex::new(xy_base + Vec3::X * plane_size, xy_color));
        triangles.push(GizmoVertex::new(
            xy_base + Vec3::new(plane_size, plane_size, 0.0),
            xy_color,
        ));
        triangles.push(GizmoVertex::new(xy_base, xy_color));
        triangles.push(GizmoVertex::new(
            xy_base + Vec3::new(plane_size, plane_size, 0.0),
            xy_color,
        ));
        triangles.push(GizmoVertex::new(xy_base + Vec3::Y * plane_size, xy_color));

        // XZ plane
        let xz_base = pos + Vec3::new(plane_offset, 0.0, plane_offset);
        triangles.push(GizmoVertex::new(xz_base, xz_color));
        triangles.push(GizmoVertex::new(xz_base + Vec3::X * plane_size, xz_color));
        triangles.push(GizmoVertex::new(
            xz_base + Vec3::new(plane_size, 0.0, plane_size),
            xz_color,
        ));
        triangles.push(GizmoVertex::new(xz_base, xz_color));
        triangles.push(GizmoVertex::new(
            xz_base + Vec3::new(plane_size, 0.0, plane_size),
            xz_color,
        ));
        triangles.push(GizmoVertex::new(xz_base + Vec3::Z * plane_size, xz_color));

        // YZ plane
        let yz_base = pos + Vec3::new(0.0, plane_offset, plane_offset);
        triangles.push(GizmoVertex::new(yz_base, yz_color));
        triangles.push(GizmoVertex::new(yz_base + Vec3::Y * plane_size, yz_color));
        triangles.push(GizmoVertex::new(
            yz_base + Vec3::new(0.0, plane_size, plane_size),
            yz_color,
        ));
        triangles.push(GizmoVertex::new(yz_base, yz_color));
        triangles.push(GizmoVertex::new(
            yz_base + Vec3::new(0.0, plane_size, plane_size),
            yz_color,
        ));
        triangles.push(GizmoVertex::new(yz_base + Vec3::Z * plane_size, yz_color));
    }

    /// Generate rotate gizmo (rings)
    pub(super) fn generate_rotate_gizmo(
        &self,
        lines: &mut Vec<GizmoVertex>,
        pos: Vec3,
        scale: f32,
    ) {
        let radius = scale * 1.2;
        let segments = 48;

        let x_color = self.get_axis_color(GizmoAxis::X);
        let y_color = self.get_axis_color(GizmoAxis::Y);
        let z_color = self.get_axis_color(GizmoAxis::Z);

        // X ring (YZ plane)
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
            let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

            let p1 = pos + Vec3::new(0.0, angle1.cos() * radius, angle1.sin() * radius);
            let p2 = pos + Vec3::new(0.0, angle2.cos() * radius, angle2.sin() * radius);

            lines.push(GizmoVertex::new(p1, x_color));
            lines.push(GizmoVertex::new(p2, x_color));
        }

        // Y ring (XZ plane)
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
            let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

            let p1 = pos + Vec3::new(angle1.cos() * radius, 0.0, angle1.sin() * radius);
            let p2 = pos + Vec3::new(angle2.cos() * radius, 0.0, angle2.sin() * radius);

            lines.push(GizmoVertex::new(p1, y_color));
            lines.push(GizmoVertex::new(p2, y_color));
        }

        // Z ring (XY plane)
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
            let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

            let p1 = pos + Vec3::new(angle1.cos() * radius, angle1.sin() * radius, 0.0);
            let p2 = pos + Vec3::new(angle2.cos() * radius, angle2.sin() * radius, 0.0);

            lines.push(GizmoVertex::new(p1, z_color));
            lines.push(GizmoVertex::new(p2, z_color));
        }
    }

    /// Generate scale gizmo (cubes at ends)
    pub(super) fn generate_scale_gizmo(
        &self,
        lines: &mut Vec<GizmoVertex>,
        triangles: &mut Vec<GizmoVertex>,
        pos: Vec3,
        scale: f32,
    ) {
        let length = scale * 1.3;
        let cube_size = scale * 0.12;

        let x_color = self.get_axis_color(GizmoAxis::X);
        let y_color = self.get_axis_color(GizmoAxis::Y);
        let z_color = self.get_axis_color(GizmoAxis::Z);
        let center_color = self.get_axis_color(GizmoAxis::XYZ);

        // X axis
        lines.push(GizmoVertex::new(pos, x_color));
        lines.push(GizmoVertex::new(pos + Vec3::X * length, x_color));
        self.add_cube(triangles, pos + Vec3::X * length, cube_size, x_color);

        // Y axis
        lines.push(GizmoVertex::new(pos, y_color));
        lines.push(GizmoVertex::new(pos + Vec3::Y * length, y_color));
        self.add_cube(triangles, pos + Vec3::Y * length, cube_size, y_color);

        // Z axis
        lines.push(GizmoVertex::new(pos, z_color));
        lines.push(GizmoVertex::new(pos + Vec3::Z * length, z_color));
        self.add_cube(triangles, pos + Vec3::Z * length, cube_size, z_color);

        // Center cube (uniform scale)
        self.add_cube(triangles, pos, cube_size * 1.2, center_color);
    }

    /// Add arrow head triangles
    pub(super) fn add_arrow_head(
        &self,
        triangles: &mut Vec<GizmoVertex>,
        tip: Vec3,
        direction: Vec3,
        size: f32,
        color: [f32; 4],
    ) {
        // Create perpendicular vectors
        let perp1 = if direction.y.abs() < 0.9 {
            direction.cross(Vec3::Y).normalize()
        } else {
            direction.cross(Vec3::X).normalize()
        };
        let perp2 = direction.cross(perp1).normalize();

        let base = tip - direction * size * 2.0;
        let cone_segments = 8;

        for i in 0..cone_segments {
            let angle1 = (i as f32 / cone_segments as f32) * 2.0 * PI;
            let angle2 = ((i + 1) as f32 / cone_segments as f32) * 2.0 * PI;

            let p1 = base + (perp1 * angle1.cos() + perp2 * angle1.sin()) * size;
            let p2 = base + (perp1 * angle2.cos() + perp2 * angle2.sin()) * size;

            // Triangle from tip to edge
            triangles.push(GizmoVertex::new(tip, color));
            triangles.push(GizmoVertex::new(p1, color));
            triangles.push(GizmoVertex::new(p2, color));
        }
    }

    /// Add a small cube
    pub(super) fn add_cube(
        &self,
        triangles: &mut Vec<GizmoVertex>,
        center: Vec3,
        size: f32,
        color: [f32; 4],
    ) {
        let half = size * 0.5;

        // Define the 8 corners
        let corners = [
            center + Vec3::new(-half, -half, -half),
            center + Vec3::new(half, -half, -half),
            center + Vec3::new(half, half, -half),
            center + Vec3::new(-half, half, -half),
            center + Vec3::new(-half, -half, half),
            center + Vec3::new(half, -half, half),
            center + Vec3::new(half, half, half),
            center + Vec3::new(-half, half, half),
        ];

        // 6 faces, 2 triangles each
        let faces = [
            [0, 1, 2, 0, 2, 3], // front
            [5, 4, 7, 5, 7, 6], // back
            [4, 0, 3, 4, 3, 7], // left
            [1, 5, 6, 1, 6, 2], // right
            [3, 2, 6, 3, 6, 7], // top
            [4, 5, 1, 4, 1, 0], // bottom
        ];

        for face in faces {
            for idx in face {
                triangles.push(GizmoVertex::new(corners[idx], color));
            }
        }
    }

    /// Get color for an axis (handles hover highlighting)
    pub(super) fn get_axis_color(&self, axis: GizmoAxis) -> [f32; 4] {
        if self.active_axis == axis || self.hovered_axis == axis {
            colors::HOVER
        } else {
            match axis {
                GizmoAxis::X => colors::X,
                GizmoAxis::Y => colors::Y,
                GizmoAxis::Z => colors::Z,
                GizmoAxis::XY => colors::PLANE_XY,
                GizmoAxis::XZ => colors::PLANE_XZ,
                GizmoAxis::YZ => colors::PLANE_YZ,
                GizmoAxis::XYZ => colors::CENTER,
                GizmoAxis::None => [0.5, 0.5, 0.5, 1.0],
            }
        }
    }
}

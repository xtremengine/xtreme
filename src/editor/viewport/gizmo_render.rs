//! Gizmo and wireframe rendering for the viewport.

use glam::{Mat4, Vec3};

use super::types::GizmoUniforms;
use super::Viewport;
use crate::editor::gizmos::{Gizmo, GizmoVertex};
use crate::render::{IsometricCamera, RenderContext};

impl Viewport {
    /// Render a gizmo at the specified position
    pub fn render_gizmo(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        camera: &IsometricCamera,
        gizmo: &Gizmo,
        position: Vec3,
    ) {
        // Calculate gizmo scale based on camera distance
        let gizmo_scale = camera.distance * 0.08;

        // Generate gizmo vertices
        let (lines, triangles) = gizmo.generate_vertices(position, gizmo_scale);

        if lines.is_empty() && triangles.is_empty() {
            return;
        }

        // Update uniforms
        let view_proj = camera.view_projection_matrix();
        let uniforms = GizmoUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
        };
        ctx.queue.write_buffer(
            &self.gizmo_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        // Upload line vertices
        if !lines.is_empty() {
            let line_data: &[u8] = bytemuck::cast_slice(&lines);
            ctx.queue
                .write_buffer(&self.gizmo_line_buffer, 0, line_data);
        }

        // Upload triangle vertices
        if !triangles.is_empty() {
            let tri_data: &[u8] = bytemuck::cast_slice(&triangles);
            ctx.queue.write_buffer(&self.gizmo_tri_buffer, 0, tri_data);
        }

        // Render gizmo pass (on top of everything)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Gizmo Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw triangles first (arrow heads, cubes)
            if !triangles.is_empty() {
                pass.set_pipeline(&self.gizmo_tri_pipeline);
                pass.set_bind_group(0, &self.gizmo_bind_group, &[]);
                pass.set_vertex_buffer(0, self.gizmo_tri_buffer.slice(..));
                pass.draw(0..triangles.len() as u32, 0..1);
            }

            // Draw lines on top
            if !lines.is_empty() {
                pass.set_pipeline(&self.gizmo_line_pipeline);
                pass.set_bind_group(0, &self.gizmo_bind_group, &[]);
                pass.set_vertex_buffer(0, self.gizmo_line_buffer.slice(..));
                pass.draw(0..lines.len() as u32, 0..1);
            }
        }
    }

    /// Render camera wireframes (pyramid shape showing direction)
    pub fn render_camera_wireframes(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        camera: &IsometricCamera,
        cameras: &[(Mat4, [f32; 4])],
    ) {
        if cameras.is_empty() {
            return;
        }

        // Generate pyramid wireframe vertices for all cameras
        let mut lines: Vec<GizmoVertex> = Vec::new();

        for (world_matrix, color) in cameras {
            // Pyramid dimensions (in local space)
            let apex = world_matrix.transform_point3(Vec3::new(0.0, 0.0, 0.5));
            let base_bl = world_matrix.transform_point3(Vec3::new(-0.5, -0.5, -1.5));
            let base_br = world_matrix.transform_point3(Vec3::new(0.5, -0.5, -1.5));
            let base_tl = world_matrix.transform_point3(Vec3::new(-0.5, 0.5, -1.5));
            let base_tr = world_matrix.transform_point3(Vec3::new(0.5, 0.5, -1.5));

            // Base lines (4 lines forming a square)
            lines.push(GizmoVertex::new(base_bl, *color));
            lines.push(GizmoVertex::new(base_br, *color));

            lines.push(GizmoVertex::new(base_br, *color));
            lines.push(GizmoVertex::new(base_tr, *color));

            lines.push(GizmoVertex::new(base_tr, *color));
            lines.push(GizmoVertex::new(base_tl, *color));

            lines.push(GizmoVertex::new(base_tl, *color));
            lines.push(GizmoVertex::new(base_bl, *color));

            // Apex lines (4 lines from apex to base corners)
            lines.push(GizmoVertex::new(apex, *color));
            lines.push(GizmoVertex::new(base_bl, *color));

            lines.push(GizmoVertex::new(apex, *color));
            lines.push(GizmoVertex::new(base_br, *color));

            lines.push(GizmoVertex::new(apex, *color));
            lines.push(GizmoVertex::new(base_tl, *color));

            lines.push(GizmoVertex::new(apex, *color));
            lines.push(GizmoVertex::new(base_tr, *color));
        }

        if lines.is_empty() {
            return;
        }

        // Update gizmo uniforms
        let view_proj = camera.view_projection_matrix();
        let uniforms = GizmoUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(),
        };
        ctx.queue.write_buffer(
            &self.gizmo_uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        // Upload line vertices
        let line_data: &[u8] = bytemuck::cast_slice(&lines);
        ctx.queue
            .write_buffer(&self.gizmo_line_buffer, 0, line_data);

        // Render camera wireframes
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Camera Wireframe Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.gizmo_line_pipeline);
            pass.set_bind_group(0, &self.gizmo_bind_group, &[]);
            pass.set_vertex_buffer(0, self.gizmo_line_buffer.slice(..));
            pass.draw(0..lines.len() as u32, 0..1);
        }
    }
}

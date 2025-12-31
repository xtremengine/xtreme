//! Viewport rendering methods.

use glam::{Mat4, Vec3};

use super::types::{GizmoUniforms, GridUniforms, MAX_OBJECTS};
use super::Viewport;
use crate::editor::gizmos::{Gizmo, GizmoVertex};
use crate::render::{IsometricCamera, RenderContext, Uniforms};

impl Viewport {
    /// Render the viewport
    pub fn render(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        camera: &IsometricCamera,
        objects: &[(Mat4, [f32; 4])],
    ) {
        // Update grid uniforms
        let view_proj = camera.view_projection_matrix();
        let grid_uniforms = GridUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera.position().to_array(),
            grid_scale: 1.0,
        };
        ctx.queue.write_buffer(
            &self.grid_uniform_buffer,
            0,
            bytemuck::cast_slice(&[grid_uniforms]),
        );

        // Calculate aligned uniform size
        let uniform_size = std::mem::size_of::<Uniforms>() as u32;
        let aligned_size = uniform_size.div_ceil(self.uniform_alignment) * self.uniform_alignment;

        // Write ALL uniforms to the buffer at once (before recording any render passes)
        let num_objects = objects.len().min(MAX_OBJECTS);
        for (i, (model, color)) in objects.iter().take(num_objects).enumerate() {
            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: model.to_cols_array_2d(),
                color: *color,
            };
            let offset = (i as u32 * aligned_size) as u64;
            ctx.queue.write_buffer(
                &self.mesh_uniform_buffer,
                offset,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        // Clear pass
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.15,
                            g: 0.15,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Draw all cubes in a single render pass using dynamic offsets
        if num_objects > 0 {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Mesh Pass"),
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

            pass.set_pipeline(&self.mesh_pipeline);

            // Draw each object with its own dynamic offset
            for i in 0..num_objects {
                let dynamic_offset = i as u32 * aligned_size;
                pass.set_bind_group(0, &self.mesh_bind_group, &[dynamic_offset]);
                self.cube_mesh.draw(&mut pass);
            }
        }

        // Draw grid (on top with alpha blending)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Viewport Grid Pass"),
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

            pass.set_pipeline(&self.grid_pipeline);
            pass.set_bind_group(0, &self.grid_bind_group, &[]);
            pass.draw(0..6, 0..1); // Full-screen quad (6 vertices)
        }
    }

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
            model: Mat4::IDENTITY.to_cols_array_2d(), // Vertices already in world space
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
    /// Each camera is represented by: apex (front) + 4 base corners + 8 lines
    pub fn render_camera_wireframes(
        &self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        camera: &IsometricCamera,
        cameras: &[(Mat4, [f32; 4])], // (world_matrix, color) for each camera object
    ) {
        if cameras.is_empty() {
            return;
        }

        // Generate pyramid wireframe vertices for all cameras
        let mut lines: Vec<GizmoVertex> = Vec::new();

        for (world_matrix, color) in cameras {
            // Pyramid dimensions (in local space)
            // Camera looks down -Z, so:
            // - Apex at back (near camera position, +Z)
            // - Base in front (where camera looks, -Z)
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

        // Update gizmo uniforms (reuse existing buffer)
        let view_proj = camera.view_projection_matrix();
        let uniforms = GizmoUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            model: Mat4::IDENTITY.to_cols_array_2d(), // Vertices already in world space
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

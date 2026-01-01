//! Viewport rendering methods.

use glam::Mat4;

use super::types::{GridUniforms, MAX_OBJECTS};
use super::{ObjectRenderData, Viewport};
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
                time: [0.0, 0.0, 0.0, 0.0],
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

    /// Render objects with texture and custom shader support
    pub fn render_objects(
        &mut self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        camera: &IsometricCamera,
        objects: &[ObjectRenderData],
        time: f32,
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
        for (i, obj) in objects.iter().take(num_objects).enumerate() {
            let uniforms = Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                model: obj.model.to_cols_array_2d(),
                color: obj.color,
                time: [time, 0.0, 0.0, 0.0],
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

        // Draw objects
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

            // Draw each object
            for (i, obj) in objects.iter().take(num_objects).enumerate() {
                let dynamic_offset = i as u32 * aligned_size;

                // Check if object has texture in cache
                let has_texture = obj
                    .texture_path
                    .as_ref()
                    .map(|p| self.texture_cache.contains_key(p))
                    .unwrap_or(false);

                // Check if object has custom shader
                let custom_shader = obj
                    .shader_path
                    .as_ref()
                    .and_then(|path| self.shader_cache.get_or_compile(&ctx.device, path));

                if let Some(cached_shader) = custom_shader {
                    // Use custom shader pipeline
                    pass.set_pipeline(&cached_shader.pipeline);
                    pass.set_bind_group(0, &self.mesh_bind_group, &[dynamic_offset]);

                    // Set texture if shader uses it and we have one
                    if cached_shader.uses_texture && has_texture {
                        let texture_path = obj.texture_path.as_ref().unwrap();
                        if let Some(cached) = self.texture_cache.get(texture_path) {
                            pass.set_bind_group(1, &cached.bind_group, &[]);
                        }
                    }
                } else if has_texture {
                    // Use textured pipeline
                    pass.set_pipeline(&self.textured_pipeline);
                    pass.set_bind_group(0, &self.mesh_bind_group, &[dynamic_offset]);

                    // Set texture bind group
                    let texture_path = obj.texture_path.as_ref().unwrap();
                    if let Some(cached) = self.texture_cache.get(texture_path) {
                        pass.set_bind_group(1, &cached.bind_group, &[]);
                    }
                } else {
                    // Use basic pipeline (no texture)
                    pass.set_pipeline(&self.mesh_pipeline);
                    pass.set_bind_group(0, &self.mesh_bind_group, &[dynamic_offset]);
                }

                // Draw the correct mesh (custom mesh or fallback to cube)
                if let Some(mesh_path) = &obj.mesh_path {
                    if let Some(gpu_mesh) = self.mesh_cache.get(mesh_path) {
                        gpu_mesh.draw(&mut pass);
                    } else {
                        self.cube_mesh.draw(&mut pass);
                    }
                } else {
                    self.cube_mesh.draw(&mut pass);
                }
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
}

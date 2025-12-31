//! # Egui Integration
//!
//! Integration layer for egui with wgpu and winit.

use egui::Context;
use egui_wgpu::ScreenDescriptor;
use winit::event::WindowEvent;
use winit::window::Window;

use super::context::RenderContext;

/// Egui integration for the editor
pub struct EguiIntegration {
    /// Egui context
    pub context: Context,
    /// Egui-winit state
    state: egui_winit::State,
    /// Egui-wgpu renderer
    renderer: egui_wgpu::Renderer,
    /// Stored full output from last frame
    last_output: Option<egui::FullOutput>,
}

impl EguiIntegration {
    /// Create a new egui integration
    pub fn new(ctx: &RenderContext, window: &Window) -> Self {
        let egui_context = Context::default();

        // Create egui-winit state
        let viewport_id = egui_context.viewport_id();
        let state = egui_winit::State::new(
            egui_context.clone(),
            viewport_id,
            window,
            Some(window.scale_factor() as f32),
            None,
            None, // max_texture_side
        );

        // Create egui-wgpu renderer
        let renderer = egui_wgpu::Renderer::new(
            &ctx.device,
            ctx.format(),
            None,
            1,
            false,
        );

        Self {
            context: egui_context,
            state,
            renderer,
            last_output: None,
        }
    }

    /// Handle a window event, returns true if egui consumed it
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    /// Begin a new egui frame
    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.context.begin_pass(raw_input);
    }

    /// End the egui frame (must be called before render)
    pub fn end_frame(&mut self, window: &Window) {
        let output = self.context.end_pass();

        // Handle platform output (cursor changes, clipboard, etc.)
        self.state.handle_platform_output(window, output.platform_output.clone());

        self.last_output = Some(output);
    }

    /// Render egui to the screen
    /// Must be called after end_frame and inside a command encoder scope
    pub fn render(
        &mut self,
        ctx: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let Some(output) = self.last_output.take() else {
            return;
        };

        // Tessellate shapes
        let paint_jobs = self.context.tessellate(output.shapes, output.pixels_per_point);

        // Update textures
        for (id, delta) in &output.textures_delta.set {
            self.renderer.update_texture(&ctx.device, &ctx.queue, *id, delta);
        }

        // Get screen descriptor
        let (width, height) = ctx.size();
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: output.pixels_per_point,
        };

        // Upload buffers
        self.renderer.update_buffers(
            &ctx.device,
            &ctx.queue,
            encoder,
            &paint_jobs,
            &screen_descriptor,
        );

        // Render egui
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load, // Don't clear - render on top
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: The render pass is dropped before encoder.finish() is called,
            // which is required for the forget_lifetime to be safe.
            let mut render_pass = render_pass.forget_lifetime();
            self.renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        // Free textures
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }

    /// Get the egui context for building UI
    pub fn ctx(&self) -> &Context {
        &self.context
    }

    /// Get mutable access to the egui-wgpu renderer (for registering textures)
    pub fn renderer_mut(&mut self) -> &mut egui_wgpu::Renderer {
        &mut self.renderer
    }
}

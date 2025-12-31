//! # Window Management
//!
//! Winit window wrapper with event loop integration.
//!
//! This module provides a high-level interface for creating windows
//! and handling the event loop with egui integration.

use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Icon, Window as WinitWindow, WindowAttributes, WindowId},
};

/// Window configuration
#[derive(Clone, Debug)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Initial width
    pub width: u32,
    /// Initial height
    pub height: u32,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether to enable VSync
    pub vsync: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Xtreme Engine".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            vsync: true,
        }
    }
}

impl WindowConfig {
    /// Create a new window config with title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set resizable
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

/// Events emitted by the window
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Window resize
    Resized { width: u32, height: u32 },
    /// Keyboard input
    KeyInput { key: Key, pressed: bool },
    /// Mouse moved
    MouseMoved { x: f64, y: f64 },
    /// Mouse button
    MouseButton { button: u32, pressed: bool },
    /// Mouse wheel
    MouseWheel { delta_x: f32, delta_y: f32 },
    /// Close requested
    CloseRequested,
    /// Redraw requested
    RedrawRequested,
}

/// Trait for applications that run in the window
pub trait App: 'static {
    /// Called when the window is created
    fn init(&mut self, window: Arc<WinitWindow>);

    /// Handle a raw window event (for egui integration)
    /// Returns true if the event was consumed and should not be processed further
    fn raw_event(&mut self, _window: &WinitWindow, _event: &WindowEvent) -> bool {
        false
    }

    /// Handle a window event
    fn event(&mut self, event: AppEvent);

    /// Update the application (called every frame)
    fn update(&mut self);

    /// Render the application
    fn render(&mut self);

    /// Called when the window is closing
    fn shutdown(&mut self) {}

    /// Handle pending window creation (called each frame)
    /// This allows the app to create secondary windows like game preview
    fn handle_pending_windows(&mut self, _event_loop: &ActiveEventLoop) {}

    /// Handle events for secondary windows
    /// Returns true if the event was handled
    fn secondary_window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: &WindowEvent,
    ) -> bool {
        false
    }

    /// Get the main window ID (if set)
    fn main_window_id(&self) -> Option<WindowId> {
        None
    }
}

/// Application handler for winit
struct AppHandler<A: App> {
    app: A,
    config: WindowConfig,
    window: Option<Arc<WinitWindow>>,
}

/// Load window icon from file
fn load_icon() -> Option<Icon> {
    // Try to load icon from assets folder
    let icon_paths = [
        "assets/icon.ico",
        "assets/xtreme-logo.png",
    ];

    for path in &icon_paths {
        if let Ok(img) = image::open(path) {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), width, height) {
                log::info!("Loaded window icon from {}", path);
                return Some(icon);
            }
        }
    }

    log::warn!("Could not load window icon");
    None
}

impl<A: App> ApplicationHandler for AppHandler<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let mut attrs = WindowAttributes::default()
                .with_title(&self.config.title)
                .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
                .with_resizable(self.config.resizable);

            // Set window icon
            if let Some(icon) = load_icon() {
                attrs = attrs.with_window_icon(Some(icon));
            }

            match event_loop.create_window(attrs) {
                Ok(window) => {
                    let window = Arc::new(window);
                    self.window = Some(window.clone());
                    self.app.init(window);
                }
                Err(e) => {
                    log::error!("Failed to create window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // Check if this is a secondary window event
        let main_window_id = self.window.as_ref().map(|w| w.id());
        if main_window_id.is_some() && Some(window_id) != main_window_id {
            // This is a secondary window event
            self.app.secondary_window_event(event_loop, window_id, &event);
            return;
        }

        // Let app handle raw event first (for egui)
        if let Some(window) = &self.window {
            if self.app.raw_event(window, &event) {
                // Event was consumed by app (e.g., egui), skip normal processing
                // But still handle close and redraw
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::RedrawRequested) {
                    // Fall through to normal handling
                } else {
                    return;
                }
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.app.event(AppEvent::CloseRequested);
                self.app.shutdown();
                event_loop.exit();
            }

            WindowEvent::Resized(PhysicalSize { width, height }) => {
                self.app.event(AppEvent::Resized { width, height });
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    logical_key,
                    state,
                    ..
                },
                ..
            } => {
                // Handle Escape to close
                if logical_key == Key::Named(NamedKey::Escape) && state == ElementState::Pressed {
                    self.app.event(AppEvent::CloseRequested);
                    self.app.shutdown();
                    event_loop.exit();
                    return;
                }

                self.app.event(AppEvent::KeyInput {
                    key: logical_key,
                    pressed: state == ElementState::Pressed,
                });
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.app.event(AppEvent::MouseMoved {
                    x: position.x,
                    y: position.y,
                });
            }

            WindowEvent::MouseInput { button, state, .. } => {
                let button_id = match button {
                    winit::event::MouseButton::Left => 0,
                    winit::event::MouseButton::Right => 1,
                    winit::event::MouseButton::Middle => 2,
                    winit::event::MouseButton::Back => 3,
                    winit::event::MouseButton::Forward => 4,
                    winit::event::MouseButton::Other(id) => id as u32,
                };
                self.app.event(AppEvent::MouseButton {
                    button: button_id,
                    pressed: state == ElementState::Pressed,
                });
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        (pos.x as f32 / 100.0, pos.y as f32 / 100.0)
                    }
                };
                self.app.event(AppEvent::MouseWheel {
                    delta_x: dx,
                    delta_y: dy,
                });
            }

            WindowEvent::RedrawRequested => {
                self.app.event(AppEvent::RedrawRequested);
                self.app.update();
                self.app.render();

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Handle pending window creation
        self.app.handle_pending_windows(event_loop);

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// Run an application in a window
pub fn run<A: App>(app: A, config: WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut handler = AppHandler {
        app,
        config,
        window: None,
    };

    event_loop.run_app(&mut handler)?;
    Ok(())
}

// Legacy wrapper for compatibility
/// Window wrapper (legacy)
pub struct Window {
    window: Option<Arc<WinitWindow>>,
}

impl Window {
    /// Create placeholder (use run() function instead)
    pub fn new() -> Self {
        Self { window: None }
    }

    /// Get inner winit window
    pub fn inner(&self) -> Option<&Arc<WinitWindow>> {
        self.window.as_ref()
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

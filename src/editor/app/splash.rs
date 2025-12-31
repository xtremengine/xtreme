//! Splash screen for play mode (deprecated - now using game window).

use super::EditorApp;

impl EditorApp {
    /// Draw splash screen overlay (deprecated - now using game window)
    #[allow(dead_code)]
    pub(super) fn draw_splash_screen(&mut self, _ctx: &egui::Context) {
        // Splash screen is no longer used - game runs in separate window
    }
}

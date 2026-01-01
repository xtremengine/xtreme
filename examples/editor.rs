//! Xtreme Engine - Visual Editor
//!
//! Visual game editor with egui UI.
//!
//! Run with: `cargo run --example editor`

use xtreme::editor::panels::init_log_capture;
use xtreme::editor::EditorApp;
use xtreme::render::{run, WindowConfig};

fn main() {
    // Initialize custom log capture for console panel
    let log_receiver = init_log_capture(log::Level::Debug);

    log::info!("Starting Xtreme Engine Editor v{}", xtreme::VERSION);

    // Create editor application with log receiver
    let mut app = EditorApp::new();
    app.set_log_receiver(log_receiver);

    // Window configuration
    let config = WindowConfig::new("Xtreme Engine Editor")
        .with_size(1280, 720)
        .with_resizable(true);

    // Run
    if let Err(e) = run(app, config) {
        log::error!("Error: {}", e);
        std::process::exit(1);
    }
}

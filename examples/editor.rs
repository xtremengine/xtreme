//! Xtreme Engine - Visual Editor
//!
//! Visual game editor with egui UI.
//!
//! Run with: `cargo run --example editor`

use xtreme::editor::EditorApp;
use xtreme::render::{run, WindowConfig};

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn"),
    )
    .init();

    log::info!("Starting Xtreme Engine Editor v{}", xtreme::VERSION);

    // Create editor application
    let app = EditorApp::new();

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

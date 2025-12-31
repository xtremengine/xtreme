//! Isometric Camera Example
//!
//! Demonstrates the isometric coordinate system and transformations.

use glam::{Vec2, Vec3};
use xtreme::math::isometric::{screen_to_world, world_to_screen, IsometricConfig, IsometricCoord};

fn main() {
    env_logger::init();
    log::info!("Xtreme Engine - Isometric Camera Example");

    // Create standard 2:1 isometric config
    let config = IsometricConfig::standard();
    log::info!("Tile size: {}x{}", config.tile_width, config.tile_height);

    // Test coordinate conversions
    println!("\n=== World to Screen Conversion ===");
    let test_positions = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(2.0, 1.0, 2.0), // With height
    ];

    for world_pos in test_positions {
        let screen_pos = world_to_screen(world_pos, &config);
        println!(
            "World ({:5.1}, {:5.1}, {:5.1}) -> Screen ({:6.1}, {:6.1})",
            world_pos.x, world_pos.y, world_pos.z, screen_pos.x, screen_pos.y
        );
    }

    // Test reverse conversion
    println!("\n=== Screen to World Conversion (y=0 plane) ===");
    let test_screens = [
        Vec2::new(0.0, 0.0),
        Vec2::new(32.0, 16.0),
        Vec2::new(-32.0, 16.0),
        Vec2::new(64.0, 32.0),
    ];

    for screen_pos in test_screens {
        let world_pos = screen_to_world(screen_pos, &config);
        println!(
            "Screen ({:6.1}, {:6.1}) -> World ({:5.1}, {:5.1}, {:5.1})",
            screen_pos.x, screen_pos.y, world_pos.x, world_pos.y, world_pos.z
        );
    }

    // Test tile coordinates
    println!("\n=== Tile Grid (5x5) ===");
    for row in 0..5 {
        for col in 0..5 {
            let _coord = IsometricCoord::flat(col, row);
            let world = Vec3::new(col as f32, 0.0, row as f32);
            let screen = world_to_screen(world, &config);
            print!("({:4.0},{:4.0}) ", screen.x, screen.y);
        }
        println!();
    }

    println!("\nIsometric camera example completed!");
}

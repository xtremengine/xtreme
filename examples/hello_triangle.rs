//! Hello Triangle Example
//!
//! Basic example showing ECS setup with position and velocity components.

use xtreme::core::{Component, World};
use xtreme::math::transform::Position;

/// Simple velocity component
#[derive(Debug, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}

impl Component for Velocity {}

fn main() {
    env_logger::init();
    log::info!("Xtreme Engine - Hello Triangle Example");

    // Create world
    let mut world = World::new();

    // Spawn entities with components
    let entity1 = world
        .spawn()
        .with(Position::new(0.0, 0.0, 0.0))
        .with(Velocity { dx: 1.0, dy: 0.5 })
        .build();

    let entity2 = world
        .spawn()
        .with(Position::new(5.0, 5.0, 0.0))
        .with(Velocity { dx: -0.5, dy: 1.0 })
        .build();

    log::info!("Spawned entities: {:?}, {:?}", entity1, entity2);
    log::info!("Entity count: {}", world.entity_count());

    // Simple update loop simulation
    for frame in 0..10 {
        // Query and update positions
        for (entity, pos) in world.query_one::<Position>() {
            if let Some(vel) = world.get::<Velocity>(entity) {
                log::info!(
                    "Frame {}: Entity {:?} at ({:.2}, {:.2}) vel ({:.2}, {:.2})",
                    frame,
                    entity,
                    pos.x,
                    pos.y,
                    vel.dx,
                    vel.dy
                );
            }
        }
    }

    log::info!("Example completed!");
}

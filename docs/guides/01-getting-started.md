# Getting Started with Xtreme Engine

This guide will help you get up and running with Xtreme Engine quickly.

## Prerequisites

- **Rust 1.75+** - Install from [rustup.rs](https://rustup.rs)
- **GPU Support** - Vulkan, DirectX 12, or Metal compatible graphics
- **Python 3.9+** (optional) - Required only for scripting feature

## Installation

### Option 1: Add as Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
xtreme-engine = "0.1"

# Optional features
# xtreme-engine = { version = "0.1", features = ["ml", "scripting"] }
```

### Option 2: Clone Repository

```bash
git clone https://github.com/xtremengine/xtreme
cd xtreme
cargo build
```

## Running the Editor

The fastest way to start is using the visual editor:

```bash
cargo run --example editor
```

This launches the Xtreme Editor where you can:
- Create and manipulate 3D objects
- Build scene hierarchies
- Save/load scenes
- Test with Play Mode

## Your First Program

Create a simple program using the ECS:

```rust
use xtreme::prelude::*;

fn main() {
    // Create the world (container for all entities)
    let mut world = World::new();

    // Spawn an entity with a transform
    let entity = world
        .spawn()
        .with(Transform::from_position(Vec3::new(0.0, 1.0, 0.0)))
        .build();

    // Query and print all transforms
    for (entity, transform) in world.query_one::<Transform>() {
        println!("Entity {:?} at position {:?}", entity, transform.position);
    }

    // Modify components
    if let Some(transform) = world.get_mut::<Transform>(entity) {
        transform.position.y += 5.0;
    }

    // Clean up
    world.despawn(entity);
}
```

## Project Structure

A typical Xtreme Engine project:

```
my_game/
├── Cargo.toml
├── src/
│   └── main.rs
├── assets/
│   ├── scenes/       # .xtrm scene files
│   ├── scripts/      # Python scripts
│   └── models/       # 3D models
└── target/
```

## Cargo Features

| Feature | Description | Use Case |
|---------|-------------|----------|
| `default` | Core engine | Basic games |
| `ml` | ONNX Runtime | AI agents with trained models |
| `scripting` | Python via pyo3 | Game logic in Python |

Enable in `Cargo.toml`:

```toml
[dependencies]
xtreme-engine = { version = "0.1", features = ["scripting"] }
```

## Build Profiles

For development (faster compilation):

```bash
cargo build
```

For release (optimized):

```bash
cargo build --release
```

## Next Steps

- [ECS Guide](02-ecs.md) - Learn the Entity Component System
- [Editor Guide](03-editor.md) - Master the visual editor
- [Scripting Guide](05-scripting.md) - Add Python scripts
- [Physics Guide](04-physics.md) - Collision and movement

## Common Issues

### "GPU not found" Error

Ensure your graphics drivers are up to date and support Vulkan/DX12/Metal.

### Python Not Found (scripting feature)

Install Python 3.9+ and ensure it's in your PATH:

```bash
python --version  # Should show 3.9+
```

### Slow Compilation

Use the dev profile which has `opt-level = 1` for faster iteration.

# Xtreme Engine

A modular ECS game engine with isometric 3D rendering and ML-powered AI, built in Rust.

[![CI](https://github.com/your-username/xtreme-engine/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/xtreme-engine/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)

## Features

### Core ECS
- **Data-Oriented Design**: Cache-friendly component storage using SparseSet
- **Generational Entities**: Safe entity recycling with automatic stale reference detection
- **Archetype System**: Fast entity queries by component composition
- **System Scheduler**: Ordered execution with configurable stages

### Rendering (WGPU)
- **Cross-Platform**: Vulkan, DirectX 12, Metal, WebGPU backends
- **Isometric Camera**: Orthographic projection with configurable angle
- **Material System**: Shader and texture management
- **Mesh Primitives**: Cubes, spheres, planes, and custom geometry
- **Egui Integration**: Immediate mode UI for editor and debug

### Physics
- **Collision Shapes**: AABB, Sphere, OBB
- **Spatial Grid**: Broadphase optimization for large worlds
- **Raycast Queries**: Fast ray-object intersection
- **Movement System**: Velocity, gravity, and collision response

### AI System
- **ML Inference**: ONNX Runtime for trained models (optional)
- **A* Pathfinding**: Grid-based navigation
- **NavMesh**: Agent-based navigation
- **Perception**: Field of view, sensors, and memory system

### Visual Editor
- **3D Viewport**: Isometric scene view with gizmos
- **Hierarchy Panel**: Entity tree with drag-and-drop
- **Inspector Panel**: Component editing
- **Asset Browser**: File system navigation
- **Toolbar**: Object creation and manipulation tools
- **Undo/Redo**: Full command history
- **Scene I/O**: Save/load in RON or JSON format
- **Prefab System**: Reusable object templates
- **Keyboard Shortcuts**: Customizable bindings
- **Play Mode**: Test scenes in editor

### Scripting (Optional)
- **Python Integration**: pyo3-based scripting
- **Lifecycle Callbacks**: `_ready()`, `_update()`, `_physics_update()`
- **Scene API**: Object manipulation from scripts

## Quick Start

### Requirements

- Rust 1.75+ (stable)
- Windows 10/11, Linux, or macOS
- GPU with Vulkan, DirectX 12, or Metal support

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xtreme-engine = "0.1"
```

Or clone the repository:

```bash
git clone https://github.com/your-username/xtreme-engine.git
cd xtreme-engine
cargo build
```

### Hello World

```rust
use xtreme::prelude::*;

fn main() {
    let mut world = World::new();

    // Spawn an entity with components
    let entity = world
        .spawn()
        .with(Transform::default())
        .with(Mesh::cube(1.0))
        .with(Material::default())
        .build();

    // Query and update
    for transform in world.query::<&mut Transform>() {
        transform.position.x += 1.0;
    }
}
```

### Running the Editor

```bash
cargo run --example editor
```

## Examples

| Example | Description | Command |
|---------|-------------|---------|
| `hello_triangle` | Basic ECS setup | `cargo run --example hello_triangle` |
| `isometric_camera` | Coordinate system demo | `cargo run --example isometric_camera` |
| `editor` | Visual game editor | `cargo run --example editor` |

## Architecture

```
xtreme/
├── src/
│   ├── lib.rs              # Crate root
│   ├── core/               # ECS implementation
│   │   ├── entity.rs       # Entity IDs with generations
│   │   ├── component.rs    # Component storage (SparseSet)
│   │   ├── archetype.rs    # Entity grouping
│   │   ├── world.rs        # Central container
│   │   ├── query.rs        # Component iteration
│   │   └── system.rs       # System scheduler
│   ├── render/             # WGPU rendering
│   │   ├── context.rs      # Device, queue, surface
│   │   ├── window.rs       # Window management
│   │   ├── camera.rs       # Isometric camera
│   │   ├── mesh.rs         # Geometry
│   │   ├── material.rs     # Shaders
│   │   ├── texture.rs      # Texture loading
│   │   ├── pipeline.rs     # Render pipeline
│   │   └── egui_integration.rs # UI rendering
│   ├── input/              # Input handling
│   │   ├── keyboard.rs     # Key states
│   │   ├── mouse.rs        # Mouse input
│   │   └── events.rs       # Event queue
│   ├── physics/            # Collision & movement
│   │   ├── shapes.rs       # AABB, Sphere, OBB
│   │   ├── collision.rs    # Detection
│   │   ├── spatial.rs      # Broadphase grid
│   │   ├── raycast.rs      # Ray queries
│   │   └── movement.rs     # Velocity, RigidBody
│   ├── ai/                 # Game AI
│   │   ├── brain.rs        # Agent component
│   │   ├── perception.rs   # Sensors, FOV
│   │   ├── pathfinding.rs  # A* algorithm
│   │   ├── navmesh.rs      # Navigation mesh
│   │   └── inference.rs    # ONNX runtime (optional)
│   ├── math/               # Math utilities
│   │   ├── transform.rs    # Position, rotation, scale
│   │   └── isometric.rs    # Coordinate conversion
│   ├── editor/             # Visual editor
│   │   ├── app/            # Editor application
│   │   │   ├── state.rs    # Editor state
│   │   │   ├── ui.rs       # Main UI layout
│   │   │   ├── menu.rs     # Menu bar
│   │   │   ├── input.rs    # Input handling
│   │   │   ├── history.rs  # Undo/redo
│   │   │   └── ...
│   │   ├── viewport/       # 3D view
│   │   ├── panels/         # UI panels
│   │   │   ├── hierarchy.rs   # Entity tree
│   │   │   ├── inspector.rs   # Component editor
│   │   │   ├── toolbar.rs     # Tools
│   │   │   └── asset_browser.rs # File browser
│   │   ├── gizmos/         # Transform handles
│   │   ├── selection.rs    # Object selection
│   │   ├── commands.rs     # Undo/redo
│   │   ├── scene.rs        # Scene I/O
│   │   ├── prefab.rs       # Prefab system
│   │   └── shortcuts.rs    # Keyboard shortcuts
│   ├── scripting/          # Python integration (optional)
│   │   ├── script.rs       # Script component
│   │   ├── runtime.rs      # Python runtime
│   │   └── api.rs          # Script API
│   └── utils/              # Pools, timers
├── examples/
│   ├── hello_triangle.rs
│   ├── isometric_camera.rs
│   └── editor.rs
└── docs/
    └── process/            # Development log
```

## Module Overview

### Core (`xtreme::core`)

The ECS implementation follows data-oriented design principles:

```rust
use xtreme::core::*;

// Define components
#[derive(Debug, Clone)]
struct Health { value: f32 }
impl Component for Health {}

#[derive(Debug, Clone)]
struct Velocity { dx: f32, dy: f32 }
impl Component for Velocity {}

// Create world and entities
let mut world = World::new();

// Entity builder pattern
let entity = world
    .spawn()
    .with(Health { value: 100.0 })
    .with(Velocity { dx: 1.0, dy: 0.0 })
    .build();

// Access components
if let Some(health) = world.get_mut::<Health>(entity) {
    health.value -= 10.0;
}

// Query components
for (entity, pos) in world.query_one::<Position>() {
    println!("Entity {:?} at {:?}", entity, pos);
}

// Check entity validity
assert!(world.is_alive(entity));
world.despawn(entity);
assert!(!world.is_alive(entity));
```

### Render (`xtreme::render`)

WGPU-based rendering with isometric support:

```rust
use xtreme::render::*;

// Create isometric camera
let mut camera = IsometricCamera::new();
camera.target = Vec3::ZERO;
camera.distance = 20.0;
camera.zoom = 10.0;
camera.yaw = std::f32::consts::FRAC_PI_4;  // 45 degrees
camera.pitch = 30.0_f32.to_radians();

// Get view-projection matrix
let vp = camera.view_projection();

// Create render context
let context = RenderContext::new(window).await?;
```

### Physics (`xtreme::physics`)

Collision detection and movement:

```rust
use xtreme::physics::*;
use glam::Vec3;

// Create AABB
let aabb = AABB::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));

// Raycast
let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
if let Some(hit) = ray.cast(&aabb) {
    println!("Hit at distance: {}", hit.distance);
}

// Spatial grid for broadphase
let mut grid = SpatialGrid::new(10.0);  // 10 unit cells
grid.insert(entity, position);
let nearby = grid.query_radius(position, 5.0);
```

### AI (`xtreme::ai`)

Pathfinding and ML inference:

```rust
use xtreme::ai::*;

// A* pathfinding
let grid = Grid::new(100, 100);
let path = AStar::find_path(&grid, (0, 0), (50, 50));

// AI Brain component
let brain = AIBrain::new()
    .with_state(AIState::Idle)
    .with_perception(FieldOfView::new(60.0, 20.0));

// ML inference (requires 'ml' feature)
#[cfg(feature = "ml")]
{
    let model = OnnxModel::load("agent.onnx")?;
    let output = model.infer(&observation)?;
}
```

### Math (`xtreme::math`)

Transforms and isometric coordinates:

```rust
use xtreme::math::*;
use glam::Vec3;

// Transform component
let transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));

// Isometric conversion (2:1 ratio)
let config = IsometricConfig::standard();
let screen = world_to_screen(Vec3::new(1.0, 0.0, 1.0), &config);
let world = screen_to_world(screen, &config);
```

### Editor (`xtreme::editor`)

Visual game editor:

```rust
use xtreme::editor::EditorApp;
use xtreme::render::{WindowConfig, run};

fn main() {
    let app = EditorApp::new();
    let config = WindowConfig::new("Xtreme Editor")
        .with_size(1280, 720);
    run(app, config).unwrap();
}
```

## Configuration

### Cargo Features

| Feature | Description | Default |
|---------|-------------|---------|
| `default` | Core engine only | Yes |
| `ml` | ONNX Runtime for ML inference | No |
| `scripting` | Python scripting via pyo3 | No |

### Build Profiles

```toml
[profile.dev]
opt-level = 1  # Faster iteration

[profile.release]
opt-level = 3
lto = true     # Maximum optimization
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# With all features
cargo build --all-features
```

### Testing

```bash
# Run all tests
cargo test

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Specific module
cargo test core::
```

### Linting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features -- -D warnings

# Security audit
cargo audit
```

### Documentation

```bash
# Generate docs
cargo doc --no-deps --all-features

# Open in browser
cargo doc --open
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `wgpu` | 23.0 | Cross-platform GPU rendering |
| `winit` | 0.30 | Window management |
| `glam` | 0.29 | Linear algebra |
| `egui` | 0.30 | Immediate mode UI |
| `serde` | 1.0 | Serialization |
| `ron` | 0.8 | Rusty Object Notation |
| `ort` | 2.0 | ONNX Runtime (optional) |
| `pyo3` | 0.23 | Python bindings (optional) |

## Roadmap

- [x] ECS Core (Entity, Component, World, Query)
- [x] Archetype System
- [x] Isometric math utilities
- [x] Transform components
- [x] Input handling (keyboard/mouse)
- [x] Collision shapes (AABB, Sphere, OBB)
- [x] A* Pathfinding
- [x] AI structure
- [x] Visual Editor
- [x] Prefab System
- [x] Scene Save/Load
- [ ] Complete WGPU rendering pipeline
- [ ] WGSL Shaders
- [ ] Instanced mesh rendering
- [ ] Shadow mapping
- [ ] Audio system
- [ ] Network multiplayer
- [ ] Asset hot-reloading
- [ ] WebGPU/WASM support

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

Please ensure:
- Code passes `cargo fmt` and `cargo clippy`
- All tests pass
- New features include tests
- Documentation is updated

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Felipe Maya

## Acknowledgments

- [wgpu](https://github.com/gfx-rs/wgpu) - Safe Rust graphics API
- [winit](https://github.com/rust-windowing/winit) - Cross-platform window handling
- [egui](https://github.com/emilk/egui) - Immediate mode GUI
- [glam](https://github.com/bitshifter/glam-rs) - Fast math library
- [ort](https://github.com/pykeio/ort) - ONNX Runtime bindings

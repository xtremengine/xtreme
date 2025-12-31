# Xtreme Engine

A modular ECS game engine with isometric 3D rendering and ML-powered AI, built in Rust.

[![CI](https://github.com/xtremengine/xtreme/actions/workflows/ci.yml/badge.svg)](https://github.com/xtremengine/xtreme/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)

## Features

### Core ECS
- **Data-Oriented Design**: Cache-friendly component storage using SparseSet
- **Generational Entities**: Safe entity recycling with automatic stale reference detection
- **Archetype System**: Fast entity queries by component composition
- **System Scheduler**: Ordered execution with configurable stages
- **Parent-Child Hierarchy**: ECS-like transform hierarchy with local/world space transforms

### Rendering (WGPU)
- **Cross-Platform**: Vulkan, DirectX 12, Metal, WebGPU backends
- **Isometric Camera**: Orthographic projection with configurable angle
- **Material System**: Shader and texture management with PBR support
- **Texture Rendering**: Full texture pipeline with caching and alpha blending
- **Custom Shaders**: Per-object WGSL shaders with runtime compilation
- **Animated Shaders**: Time-based effects (noise, color cycling, dissolve, etc.)
- **Mesh Primitives**: Cubes, spheres, planes, and custom geometry
- **Egui Integration**: Immediate mode UI for editor and debug
- **Camera Components**: Per-entity cameras with perspective/orthographic projection

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
- **AI Brain**: State machine with configurable behaviors

### Visual Editor
- **3D Viewport**: Isometric scene view with transform gizmos
- **Hierarchy Panel**: Entity tree with parent-child relationships
- **Inspector Panel**: Component editing with real-time preview
- **Asset Browser**: File system navigation
- **Toolbar**: Object creation and manipulation tools
- **Undo/Redo**: Full command history with unlimited levels
- **Scene I/O**: Save/load in RON or JSON format
- **Prefab System**: Reusable object templates
- **Keyboard Shortcuts**: Customizable bindings (see below)
- **Play Mode**: Test scenes directly in editor
- **Camera Visualization**: Wireframe pyramid showing camera direction

### Scripting (Optional)
- **Python Integration**: pyo3-based scripting system
- **Lifecycle Callbacks**: `_ready()`, `_update(delta)`, `_physics_update(delta)`
- **Scene API**: Object manipulation, transform access, input handling
- **Hot Reload**: Edit scripts while editor is running

## Quick Start

### Requirements

- Rust 1.75+ (stable)
- Windows 10/11, Linux, or macOS
- GPU with Vulkan, DirectX 12, or Metal support
- Python 3.9+ (optional, for scripting feature)

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
xtreme-engine = "0.1"
```

Or clone the repository:

```bash
git clone https://github.com/xtremengine/xtreme
cd xtreme
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

## Editor Guide

### Keyboard Shortcuts

| Category | Shortcut | Action |
|----------|----------|--------|
| **File** | `Ctrl+N` | New Scene |
| | `Ctrl+O` | Open Scene |
| | `Ctrl+S` | Save Scene |
| | `Ctrl+Shift+S` | Save Scene As |
| **Edit** | `Ctrl+Z` | Undo |
| | `Ctrl+Shift+Z` / `Ctrl+Y` | Redo |
| | `Ctrl+X` | Cut |
| | `Ctrl+C` | Copy |
| | `Ctrl+V` | Paste |
| | `Delete` | Delete Selected |
| | `Ctrl+D` | Duplicate |
| | `Ctrl+A` | Select All |
| **Tools** | `Q` | Select Tool |
| | `W` | Move Tool |
| | `E` | Rotate Tool |
| | `R` | Scale Tool |
| **View** | `F` | Focus on Selected |
| | `Home` | Frame All Objects |
| | `Ctrl+1` | Front View |
| | `Ctrl+3` | Side View |
| | `Ctrl+7` | Top View |
| **Object** | `H` | Toggle Visibility |
| **Play** | `F5` | Toggle Play Mode |

### Viewport Navigation

| Action | Control |
|--------|---------|
| Orbit Camera | Right-click + Drag |
| Pan Camera | Middle-click + Drag |
| Zoom | Scroll Wheel |
| Focus Object | Double-click in Hierarchy |

### Creating Objects

1. Use the toolbar buttons: **+ Cube**, **+ Empty**, **+ Camera**
2. Or right-click in hierarchy for context menu
3. Objects can be parented via drag-and-drop or context menu

### Parent-Child Hierarchy

Objects can be organized in a parent-child hierarchy:
- **Parenting**: Right-click object > Set Parent... > Select parent
- **Unparenting**: Right-click object > Unparent (move to root)
- **Local Transforms**: Child transforms are relative to parent
- **World Transforms**: Automatically calculated from hierarchy

### Camera Component

Add cameras to any object for in-game views:
- **Main Camera**: Check "Main Camera" to use in Play Mode
- **Projection**: Perspective or Orthographic
- **FOV**: Field of view (perspective mode)
- **Visualization**: Cameras show as wireframe pyramids in editor

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
│   ├── lib.rs              # Crate root and prelude
│   ├── core/               # ECS implementation
│   │   ├── entity.rs       # Entity IDs with generations
│   │   ├── component.rs    # Component storage (SparseSet)
│   │   ├── archetype.rs    # Entity grouping by components
│   │   ├── world.rs        # Central container
│   │   ├── query.rs        # Component iteration
│   │   └── system.rs       # System scheduler with stages
│   ├── render/             # WGPU rendering
│   │   ├── context.rs      # Device, queue, surface
│   │   ├── window.rs       # Window management
│   │   ├── camera.rs       # Isometric camera
│   │   ├── mesh.rs         # Geometry primitives
│   │   ├── material.rs     # Shaders and materials
│   │   ├── texture.rs      # Texture loading
│   │   ├── pipeline.rs     # Render pipeline
│   │   └── egui_integration.rs # UI rendering
│   ├── input/              # Input handling
│   │   ├── keyboard.rs     # Key states
│   │   ├── mouse.rs        # Mouse input
│   │   └── events.rs       # Event queue
│   ├── physics/            # Collision & movement
│   │   ├── shapes.rs       # AABB, Sphere, OBB
│   │   ├── collision.rs    # Detection algorithms
│   │   ├── spatial.rs      # Broadphase grid
│   │   ├── raycast.rs      # Ray queries
│   │   └── movement.rs     # Velocity, RigidBody
│   ├── ai/                 # Game AI
│   │   ├── brain.rs        # Agent state machine
│   │   ├── perception.rs   # Sensors, FOV
│   │   ├── pathfinding.rs  # A* algorithm
│   │   ├── navmesh.rs      # Navigation mesh
│   │   └── inference.rs    # ONNX runtime (optional)
│   ├── math/               # Math utilities
│   │   ├── transform.rs    # Position, rotation, scale
│   │   └── isometric.rs    # Coordinate conversion
│   ├── editor/             # Visual editor
│   │   ├── app/            # Editor application
│   │   │   ├── state.rs    # Editor state management
│   │   │   ├── ui.rs       # Main UI layout
│   │   │   ├── menu.rs     # Menu bar
│   │   │   ├── input.rs    # Input handling
│   │   │   ├── history.rs  # Undo/redo system
│   │   │   ├── play_mode.rs # Play mode logic
│   │   │   └── dialogs/    # File dialogs
│   │   ├── viewport/       # 3D view rendering
│   │   ├── panels/         # UI panels
│   │   │   ├── hierarchy.rs   # Entity tree
│   │   │   ├── inspector.rs   # Component editor
│   │   │   ├── toolbar.rs     # Tools
│   │   │   └── asset_browser.rs # File browser
│   │   ├── gizmos/         # Transform handles
│   │   ├── components/     # Editor components
│   │   │   └── camera.rs   # CameraComponent
│   │   ├── selection.rs    # Object selection
│   │   ├── commands.rs     # Undo/redo commands
│   │   ├── scene.rs        # Scene serialization
│   │   ├── prefab.rs       # Prefab system
│   │   ├── hierarchy.rs    # Parent-child system
│   │   └── shortcuts.rs    # Keyboard shortcuts
│   ├── scripting/          # Python integration (optional)
│   │   ├── script.rs       # Script component
│   │   ├── runtime.rs      # Python runtime
│   │   └── api.rs          # Script API
│   └── utils/              # Utilities
│       ├── pool.rs         # Object pools
│       └── timer.rs        # Timers
├── examples/
│   ├── hello_triangle.rs
│   ├── isometric_camera.rs
│   └── editor.rs
└── docs/
    └── process/            # Development log
```

## Module Reference

### Core (`xtreme::core`)

The ECS implementation follows data-oriented design principles:

```rust
use xtreme::core::*;

// Define components (any Clone type)
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

// Direct component access
if let Some(health) = world.get_mut::<Health>(entity) {
    health.value -= 10.0;
}

// Query single component type
for (entity, pos) in world.query_one::<Position>() {
    println!("Entity {:?} at {:?}", entity, pos);
}

// Check entity validity (generational safety)
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

// Window configuration
let config = WindowConfig::new("My Game")
    .with_size(1280, 720)
    .with_resizable(true);
```

### Physics (`xtreme::physics`)

Collision detection and movement:

```rust
use xtreme::physics::*;
use glam::Vec3;

// Create collision shapes
let aabb = AABB::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
let sphere = Sphere::new(Vec3::ZERO, 0.5);

// Raycast
let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::NEG_Y);
if let Some(hit) = ray.cast(&aabb) {
    println!("Hit at distance: {}", hit.distance);
}

// Spatial grid for broadphase optimization
let mut grid = SpatialGrid::new(10.0);  // 10 unit cells
grid.insert(entity, position);
let nearby = grid.query_radius(position, 5.0);
```

### AI (`xtreme::ai`)

Pathfinding and ML inference:

```rust
use xtreme::ai::*;

// A* pathfinding on a grid
let mut grid = Grid::new(100, 100);
grid.set_blocked(50, 50);  // Add obstacle

if let Some(path) = AStar::find_path(&grid, (0, 0), (99, 99)) {
    for (x, y) in path.nodes {
        println!("Step: ({}, {})", x, y);
    }
}

// AI Brain component for agents
let mut brain = AIBrain::new();
brain.set_state(AIState::Patrol);

// Update brain (call each frame)
brain.update(delta_time);
if brain.should_think() {
    // Make decision based on current state
    brain.did_think();
}

// ML inference (requires 'ml' feature)
#[cfg(feature = "ml")]
{
    let model = OnnxModel::load("agent.onnx")?;
    let input = ModelInput::from_vec(observation);
    let output = model.infer(&input)?;
}
```

### Math (`xtreme::math`)

Transforms and isometric coordinates:

```rust
use xtreme::math::*;
use glam::Vec3;

// Transform component
let mut transform = Transform::from_position(Vec3::new(1.0, 2.0, 3.0));
transform.rotation = Vec3::new(0.0, 45.0_f32.to_radians(), 0.0);
transform.scale = Vec3::ONE;

// Get 4x4 transformation matrix
let matrix = transform.matrix();

// Isometric conversion (2:1 ratio, standard isometric)
let config = IsometricConfig::standard();
let screen_pos = world_to_screen(Vec3::new(1.0, 0.0, 1.0), &config);
let world_pos = screen_to_world(screen_pos, &config);
```

### Scripting (`xtreme::scripting`)

Python scripting for game logic (requires `scripting` feature):

```rust
// Enable in Cargo.toml:
// [dependencies]
// xtreme-engine = { version = "0.1", features = ["scripting"] }

use xtreme::scripting::*;

// Create runtime
let mut runtime = ScriptRuntime::new();
runtime.set_scripts_dir("./scripts".into());
runtime.initialize()?;

// Load and execute script
let script_id = runtime.load_script("player.py", object_id)?;
runtime.call_ready(script_id, &context)?;

// In game loop
runtime.call_update(script_id, delta_time, &context)?;
```

**Example Python Script** (`scripts/player.py`):

```python
# Player controller script

speed = 5.0
jump_force = 10.0

def _ready(ctx):
    """Called once when script is attached"""
    print(f"Player ready! Object ID: {ctx['object_id']}")

def _update(ctx, delta):
    """Called every frame"""
    transform = ctx['transform']

    # Movement input
    if is_key_pressed(ctx, 'W'):
        transform['position'][2] -= speed * delta
    if is_key_pressed(ctx, 'S'):
        transform['position'][2] += speed * delta
    if is_key_pressed(ctx, 'A'):
        transform['position'][0] -= speed * delta
    if is_key_pressed(ctx, 'D'):
        transform['position'][0] += speed * delta

    # Return modified transform
    return {'transform': transform}

def _physics_update(ctx, delta):
    """Called at fixed physics rate"""
    pass
```

### Editor (`xtreme::editor`)

Visual game editor:

```rust
use xtreme::editor::EditorApp;
use xtreme::render::{WindowConfig, run};

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    let app = EditorApp::new();
    let config = WindowConfig::new("Xtreme Editor")
        .with_size(1280, 720)
        .with_resizable(true);

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

Enable features in `Cargo.toml`:

```toml
[dependencies]
xtreme-engine = { version = "0.1", features = ["ml", "scripting"] }
```

### Build Profiles

```toml
[profile.dev]
opt-level = 1  # Faster iteration

[profile.release]
opt-level = 3
lto = true     # Maximum optimization
```

## Scene File Format

Scenes are saved in RON (Rusty Object Notation) or JSON format:

```ron
// example_scene.xtrm
SceneData(
    version: 2,
    name: "My Scene",
    camera_target: Some((0.0, 0.0, 0.0)),
    camera_distance: Some(20.0),
    objects: [
        SceneObjectData(
            name: "Player",
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [0.2, 0.6, 1.0, 1.0],
            visible: true,
            parent_index: None,
            scripts: ["player.py"],
            camera: None,
        ),
        SceneObjectData(
            name: "Main Camera",
            position: [0.0, 5.0, 10.0],
            rotation: [-0.5, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            visible: true,
            parent_index: None,
            scripts: [],
            camera: Some(CameraComponent(
                projection: Perspective,
                fov: 60.0,
                near: 0.1,
                far: 1000.0,
                is_main: true,
            )),
        ),
    ],
)
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

# With all features
cargo test --all-features
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
| `wgpu` | 27.0 | Cross-platform GPU rendering |
| `winit` | 0.30 | Window management |
| `glam` | 0.29 | Linear algebra (Vec3, Mat4, Quat) |
| `egui` | 0.33 | Immediate mode UI |
| `serde` | 1.0 | Serialization framework |
| `ron` | 0.8 | Rusty Object Notation |
| `ort` | 2.0 | ONNX Runtime (optional) |
| `pyo3` | 0.27 | Python bindings (optional) |

## Roadmap

### Completed
- [x] ECS Core (Entity, Component, World, Query)
- [x] Archetype System
- [x] System Scheduler with Stages
- [x] Parent-Child Hierarchy
- [x] Isometric math utilities
- [x] Transform components
- [x] Input handling (keyboard/mouse)
- [x] Collision shapes (AABB, Sphere, OBB)
- [x] A* Pathfinding
- [x] AI Brain with State Machine
- [x] Visual Editor with egui
- [x] Prefab System
- [x] Scene Save/Load (RON/JSON)
- [x] Undo/Redo System
- [x] Customizable Keyboard Shortcuts
- [x] Python Scripting Integration
- [x] Camera Component System
- [x] Play Mode
- [x] WGPU Rendering Pipeline
- [x] WGSL Shaders (basic + textured)
- [x] Texture/Material Loading with Alpha Blending
- [x] Resizable Asset Browser Panel
- [x] Custom Shaders per Object (runtime compilation)
- [x] Animated Shader Effects (time-based)

### Planned
- [ ] Instanced mesh rendering
- [ ] Shadow mapping
- [ ] Audio system (rodio)
- [ ] Network multiplayer
- [ ] Asset hot-reloading
- [ ] WebGPU/WASM support
- [ ] Animation system
- [ ] Particle effects

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit changes: `git commit -m 'Add amazing feature'`
4. Push to branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

Please ensure:
- Code passes `cargo fmt` and `cargo clippy`
- All tests pass (`cargo test --all-features`)
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
- [pyo3](https://github.com/PyO3/pyo3) - Rust bindings for Python

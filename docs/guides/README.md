# Xtreme Engine Documentation

Welcome to the Xtreme Engine documentation.

## Guides

| Guide | Description |
|-------|-------------|
| [Getting Started](01-getting-started.md) | Installation and first steps |
| [Entity Component System](02-ecs.md) | Core ECS architecture |
| [Visual Editor](03-editor.md) | Using the editor UI |
| [Physics](04-physics.md) | Collision and movement |
| [Scripting](05-scripting.md) | Python integration |
| [AI System](06-ai.md) | Pathfinding and ML |
| [Rendering](07-rendering.md) | WGPU graphics |

## Quick Links

- [Main README](../../README.md) - Project overview
- [API Documentation](https://docs.rs/xtreme-engine) - Generated API docs
- [Examples](../../examples/) - Code examples
- [GitHub Issues](https://github.com/xtremengine/xtreme/issues) - Bug reports

## Learning Path

### Beginner

1. Start with [Getting Started](01-getting-started.md)
2. Run the editor: `cargo run --example editor`
3. Create a simple scene with objects
4. Save and reload your scene

### Intermediate

1. Understand [ECS](02-ecs.md) concepts
2. Add [Python scripts](05-scripting.md) to objects
3. Set up [Physics](04-physics.md) collisions
4. Create reusable prefabs

### Advanced

1. Implement custom [AI](06-ai.md) behaviors
2. Create custom [Rendering](07-rendering.md) pipelines
3. Train and integrate ML models
4. Optimize for performance

## Module Overview

```
xtreme/
├── core/       # ECS (Entity, Component, World, Query, System)
├── render/     # WGPU rendering (Camera, Mesh, Material, Texture)
├── input/      # Input handling (Keyboard, Mouse)
├── physics/    # Collision (AABB, Sphere, OBB, Raycast)
├── ai/         # AI (Pathfinding, Perception, ML Inference)
├── math/       # Math utilities (Transform, Isometric)
├── editor/     # Visual editor (Panels, Gizmos, Scene I/O)
├── scripting/  # Python scripting (Runtime, API)
└── utils/      # Utilities (Pool, Timer)
```

## Features Matrix

| Feature | Default | Optional | Flag |
|---------|---------|----------|------|
| ECS | Yes | - | - |
| Rendering | Yes | - | - |
| Physics | Yes | - | - |
| Editor | Yes | - | - |
| Pathfinding | Yes | - | - |
| ML Inference | - | Yes | `ml` |
| Scripting | - | Yes | `scripting` |

## Common Tasks

### Create a Game Object

```rust
let entity = world.spawn()
    .with(Transform::default())
    .with(Mesh::cube(1.0))
    .build();
```

### Add Physics

```rust
let aabb = AABB::from_center_half(position, Vec3::ONE);
collision_world.add(entity, Shape::AABB(aabb));
```

### Attach a Script

In the editor: Select object > Inspector > + Add Script > Select .py file

### Find a Path

```rust
let path = AStar::find_path(&grid, (0, 0), (100, 100));
```

## Support

- **Documentation**: This guide and API docs
- **Examples**: `cargo run --example <name>`
- **Issues**: [GitHub Issues](https://github.com/xtremengine/xtreme/issues)

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

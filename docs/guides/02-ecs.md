# Entity Component System (ECS)

Xtreme Engine uses a data-oriented ECS architecture for high performance and flexibility.

## Core Concepts

### Entity

An entity is a unique identifier (u64) representing a game object. It has no data itself - just an ID.

```rust
use xtreme::core::*;

let mut world = World::new();

// Spawn an empty entity
let entity = world.spawn_empty();

// Entities are just IDs
println!("Entity ID: {:?}", entity);
```

**Generational IDs**: Entities use generational indices to prevent stale references:
- Lower 32 bits: index into entity array
- Upper 32 bits: generation counter

When an entity is despawned and its slot reused, the generation increments, invalidating old references.

### Component

Components are pure data structs attached to entities. They contain no logic.

```rust
use xtreme::core::Component;

// Define components
#[derive(Debug, Clone)]
struct Position { x: f32, y: f32, z: f32 }
impl Component for Position {}

#[derive(Debug, Clone)]
struct Health { current: f32, max: f32 }
impl Component for Health {}

#[derive(Debug, Clone)]
struct Player { name: String }
impl Component for Player {}
```

**Requirements**: Components must be `Clone` and `'static`.

### World

The world is the central container managing all entities and components.

```rust
let mut world = World::new();

// Spawn with builder pattern
let entity = world
    .spawn()
    .with(Position { x: 0.0, y: 0.0, z: 0.0 })
    .with(Health { current: 100.0, max: 100.0 })
    .build();

// Or spawn with single component
let entity2 = world.spawn_with(Position { x: 5.0, y: 0.0, z: 0.0 });
```

## Working with Components

### Adding Components

```rust
// At spawn time
let entity = world.spawn().with(Position::default()).build();

// After spawn
world.insert(entity, Health { current: 50.0, max: 100.0 });
```

### Reading Components

```rust
// Get immutable reference
if let Some(pos) = world.get::<Position>(entity) {
    println!("Position: ({}, {}, {})", pos.x, pos.y, pos.z);
}

// Get mutable reference
if let Some(health) = world.get_mut::<Health>(entity) {
    health.current -= 10.0;
}
```

### Removing Components

```rust
// Remove single component
world.remove::<Health>(entity);

// Despawn entity (removes all components)
world.despawn(entity);
```

### Checking Components

```rust
// Check if entity has component
if world.has::<Health>(entity) {
    println!("Entity has health!");
}

// Check if entity is alive
if world.is_alive(entity) {
    // Safe to use
}
```

## Querying

Queries iterate over entities with specific components.

### Single Component Query

```rust
// Iterate all entities with Position
for (entity, pos) in world.query_one::<Position>() {
    println!("Entity {:?} at {:?}", entity, pos);
}

// Mutable query
for (idx, pos) in world.query_one_mut::<Position>().iter_mut() {
    pos.x += 1.0;
}
```

### Multi-Component Query

```rust
use xtreme::core::query_two;

// Query entities with both Position and Velocity
for (entity, pos, vel) in query_two::<Position, Velocity>(&world) {
    println!("{:?}: pos={:?}, vel={:?}", entity, pos, vel);
}
```

## Systems

Systems contain game logic that operates on components.

### Defining Systems

```rust
use xtreme::core::{System, World};

struct MovementSystem;

impl System for MovementSystem {
    fn run(&mut self, world: &mut World) {
        // Apply velocity to position
        for (entity, pos, vel) in query_two::<Position, Velocity>(world) {
            // Note: This is read-only, for mutation use different pattern
        }
    }
}
```

### System Scheduler

```rust
use xtreme::core::{SystemScheduler, Stage};

let mut scheduler = SystemScheduler::new();

// Add systems to stages
scheduler.add_system(Stage::Update, Box::new(MovementSystem));
scheduler.add_system(Stage::Update, Box::new(PhysicsSystem));
scheduler.add_system(Stage::PostUpdate, Box::new(RenderSystem));

// Run all systems
scheduler.run(&mut world);
```

### Execution Stages

| Stage | Purpose | Order |
|-------|---------|-------|
| `PreUpdate` | Input processing | 1 |
| `Update` | Main game logic | 2 |
| `PostUpdate` | Cleanup, rendering | 3 |

## Archetypes

Archetypes group entities by their component composition for efficient iteration.

```rust
// Entities with same components share an archetype
let e1 = world.spawn().with(Position::default()).with(Health::default()).build();
let e2 = world.spawn().with(Position::default()).with(Health::default()).build();
// e1 and e2 share the same archetype

let e3 = world.spawn().with(Position::default()).build();
// e3 has a different archetype (no Health)
```

## Component Storage

Components are stored in `SparseSet` for O(1) access:

```rust
// Internal storage structure (simplified)
struct SparseSet<T> {
    sparse: Vec<Option<usize>>,  // Entity index -> dense index
    dense: Vec<T>,               // Actual component data
    entities: Vec<u32>,          // Dense index -> entity index
}
```

Benefits:
- O(1) insert, remove, get
- Cache-friendly iteration
- No memory waste for missing components

## Best Practices

### 1. Keep Components Small

```rust
// Good: Small, focused component
struct Position { x: f32, y: f32, z: f32 }

// Bad: Large, monolithic component
struct GameObject {
    position: Vec3,
    velocity: Vec3,
    health: f32,
    mana: f32,
    // ... many more fields
}
```

### 2. Separate Data and Logic

```rust
// Components: Pure data
struct Velocity { dx: f32, dy: f32 }

// Systems: Pure logic
struct MovementSystem;
impl System for MovementSystem {
    fn run(&mut self, world: &mut World) {
        // Logic here
    }
}
```

### 3. Use Marker Components

```rust
// Mark entities without data
struct Player;
impl Component for Player {}

struct Enemy;
impl Component for Enemy {}

// Query by marker
for (entity, _) in world.query_one::<Player>() {
    // This is a player
}
```

### 4. Handle Despawned Entities

```rust
// Always check if alive before using
if world.is_alive(entity) {
    if let Some(pos) = world.get::<Position>(entity) {
        // Safe to use
    }
}
```

## Example: Complete Game Loop

```rust
use xtreme::core::*;

// Components
#[derive(Clone)] struct Position { x: f32, y: f32 }
impl Component for Position {}

#[derive(Clone)] struct Velocity { dx: f32, dy: f32 }
impl Component for Velocity {}

fn main() {
    let mut world = World::new();

    // Spawn entities
    for i in 0..100 {
        world.spawn()
            .with(Position { x: i as f32, y: 0.0 })
            .with(Velocity { dx: 1.0, dy: 0.5 })
            .build();
    }

    // Game loop
    let delta_time = 0.016; // 60 FPS

    loop {
        // Update positions based on velocity
        for (idx, pos) in world.query_one_mut::<Position>().iter_mut() {
            // In real code, also query velocity
            pos.x += 1.0 * delta_time;
            pos.y += 0.5 * delta_time;
        }

        // Break for example
        break;
    }
}
```

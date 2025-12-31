# Physics System

Xtreme Engine provides collision detection and basic physics simulation.

## Collision Shapes

### AABB (Axis-Aligned Bounding Box)

The simplest collision shape - a box aligned to world axes.

```rust
use xtreme::physics::AABB;
use glam::Vec3;

// Create AABB from min/max points
let aabb = AABB::new(
    Vec3::new(-1.0, -1.0, -1.0),  // min corner
    Vec3::new(1.0, 1.0, 1.0),     // max corner
);

// Or from center and half-extents
let aabb = AABB::from_center_half(
    Vec3::ZERO,                   // center
    Vec3::new(1.0, 1.0, 1.0),    // half-extents
);

// Properties
let center = aabb.center();
let size = aabb.size();
let half = aabb.half_extents();
```

**Use cases:**
- Fast broad-phase collision
- Simple rectangular objects
- Spatial partitioning

### Sphere

A spherical collision shape.

```rust
use xtreme::physics::Sphere;
use glam::Vec3;

let sphere = Sphere::new(
    Vec3::new(0.0, 1.0, 0.0),  // center
    0.5,                        // radius
);

// Properties
let center = sphere.center;
let radius = sphere.radius;
```

**Use cases:**
- Balls, bullets, particles
- Character collision (bounding sphere)
- Fast distance checks

### OBB (Oriented Bounding Box)

A rotated box that can match object orientation.

```rust
use xtreme::physics::OBB;
use glam::{Vec3, Quat};

let obb = OBB::new(
    Vec3::ZERO,                            // center
    Vec3::new(2.0, 1.0, 0.5),             // half-extents
    Quat::from_rotation_y(45.0_f32.to_radians()), // rotation
);
```

**Use cases:**
- Rotated objects
- Tight-fitting bounds
- More accurate than AABB for rotated shapes

## Collision Detection

### Shape vs Shape

```rust
use xtreme::physics::*;
use glam::Vec3;

let aabb1 = AABB::from_center_half(Vec3::ZERO, Vec3::ONE);
let aabb2 = AABB::from_center_half(Vec3::new(1.5, 0.0, 0.0), Vec3::ONE);

// Check intersection
if aabb1.intersects(&aabb2) {
    println!("Collision detected!");
}

// Sphere-sphere
let sphere1 = Sphere::new(Vec3::ZERO, 1.0);
let sphere2 = Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);

if sphere1.intersects(&sphere2) {
    println!("Spheres overlap!");
}
```

### Collision World

Manage multiple colliders:

```rust
use xtreme::physics::{CollisionWorld, Shape};

let mut collision_world = CollisionWorld::new();

// Add colliders
collision_world.add(entity1, Shape::AABB(aabb));
collision_world.add(entity2, Shape::Sphere(sphere));

// Check all collisions
let pairs: Vec<CollisionPair> = collision_world.detect_all();

for pair in pairs {
    println!("Collision: {:?} vs {:?}", pair.entity_a, pair.entity_b);
}
```

### Contact Information

Get detailed collision info:

```rust
if let Some(contact) = collision_world.contact(entity1, entity2) {
    println!("Contact point: {:?}", contact.point);
    println!("Contact normal: {:?}", contact.normal);
    println!("Penetration depth: {}", contact.depth);
}
```

## Raycasting

Cast rays to find intersections.

### Basic Raycast

```rust
use xtreme::physics::{Ray, RaycastHit};
use glam::Vec3;

// Create ray (origin, direction)
let ray = Ray::new(
    Vec3::new(0.0, 10.0, 0.0),  // Start above
    Vec3::NEG_Y,                 // Shoot down
);

// Cast against AABB
let aabb = AABB::from_center_half(Vec3::ZERO, Vec3::ONE);

if let Some(hit) = ray.cast(&aabb) {
    println!("Hit at distance: {}", hit.distance);
    println!("Hit point: {:?}", hit.point);
    println!("Hit normal: {:?}", hit.normal);
}
```

### Raycast Query

Query multiple objects:

```rust
use xtreme::physics::RaycastQuery;

let mut query = RaycastQuery::new(ray);

// Add objects to query
query.add(entity1, &aabb1);
query.add(entity2, &sphere);
query.add(entity3, &aabb2);

// Get closest hit
if let Some((entity, hit)) = query.closest() {
    println!("Closest hit: {:?} at {}", entity, hit.distance);
}

// Get all hits
for (entity, hit) in query.all() {
    println!("Hit {:?} at {}", entity, hit.distance);
}
```

## Spatial Grid

Optimize collision detection with spatial partitioning.

### Creating a Grid

```rust
use xtreme::physics::SpatialGrid;
use glam::Vec3;

// Create grid with 10-unit cells
let mut grid = SpatialGrid::new(10.0);
```

### Adding Objects

```rust
// Insert objects with their positions
grid.insert(entity1, Vec3::new(5.0, 0.0, 5.0));
grid.insert(entity2, Vec3::new(15.0, 0.0, 5.0));
grid.insert(entity3, Vec3::new(100.0, 0.0, 100.0));
```

### Querying

```rust
// Query by radius
let nearby = grid.query_radius(
    Vec3::new(5.0, 0.0, 5.0),  // center
    20.0,                       // radius
);

for entity in nearby {
    println!("Nearby entity: {:?}", entity);
}

// Query by AABB
let in_area = grid.query_aabb(&search_aabb);
```

### Updating Objects

```rust
// Update position (remove and re-insert)
grid.update(entity, old_position, new_position);

// Remove object
grid.remove(entity, position);
```

## Movement & Physics

### Velocity Component

```rust
use xtreme::physics::Velocity;

let velocity = Velocity {
    linear: Vec3::new(5.0, 0.0, 2.0),  // units per second
    angular: Vec3::ZERO,                 // rotation per second
};
```

### Integration

```rust
use xtreme::physics::integrate;

// Apply velocity to transform
let new_position = integrate(
    current_position,
    velocity.linear,
    delta_time,
);
```

### RigidBody

```rust
use xtreme::physics::RigidBody;

let rigidbody = RigidBody {
    mass: 1.0,
    velocity: Vec3::ZERO,
    acceleration: Vec3::ZERO,
    drag: 0.1,
    gravity_scale: 1.0,
};

// Apply force
rigidbody.add_force(Vec3::new(100.0, 0.0, 0.0));

// Apply impulse (immediate velocity change)
rigidbody.add_impulse(Vec3::new(0.0, 10.0, 0.0));
```

## Collision Response

### Simple Bounce

```rust
fn bounce_response(velocity: Vec3, normal: Vec3, bounciness: f32) -> Vec3 {
    let dot = velocity.dot(normal);
    velocity - (1.0 + bounciness) * dot * normal
}

// Usage
if let Some(hit) = raycast_hit {
    let new_velocity = bounce_response(velocity, hit.normal, 0.5);
}
```

### Stop on Collision

```rust
fn stop_response(velocity: Vec3, normal: Vec3) -> Vec3 {
    let dot = velocity.dot(normal);
    if dot < 0.0 {
        // Remove velocity component into surface
        velocity - dot * normal
    } else {
        velocity
    }
}
```

## Performance Tips

### 1. Use Spatial Partitioning

```rust
// Bad: O(n^2) - check everything against everything
for a in &entities {
    for b in &entities {
        if collides(a, b) { ... }
    }
}

// Good: O(n) average - use spatial grid
let grid = SpatialGrid::new(cell_size);
for entity in &entities {
    let nearby = grid.query_radius(entity.position, max_radius);
    for other in nearby {
        if collides(entity, other) { ... }
    }
}
```

### 2. Broad Phase then Narrow Phase

```rust
// Broad phase: Fast AABB check
if aabb_a.intersects(&aabb_b) {
    // Narrow phase: Detailed check only if broad phase passes
    if obb_a.intersects_detailed(&obb_b) {
        // Handle collision
    }
}
```

### 3. Choose Appropriate Shapes

| Shape | Speed | Accuracy | Use When |
|-------|-------|----------|----------|
| AABB | Fastest | Low | Axis-aligned objects |
| Sphere | Fast | Medium | Round objects |
| OBB | Medium | High | Rotated boxes |

### 4. Layer System (Advanced)

```rust
// Define collision layers
const PLAYER: u32 = 1 << 0;
const ENEMY: u32 = 1 << 1;
const PROJECTILE: u32 = 1 << 2;
const ENVIRONMENT: u32 = 1 << 3;

// Define what collides with what
fn should_collide(layer_a: u32, layer_b: u32) -> bool {
    let mask = match layer_a {
        PLAYER => ENEMY | ENVIRONMENT,
        ENEMY => PLAYER | PROJECTILE | ENVIRONMENT,
        PROJECTILE => ENEMY | ENVIRONMENT,
        _ => 0,
    };
    (layer_b & mask) != 0
}
```

## Example: Physics System

```rust
use xtreme::core::*;
use xtreme::physics::*;

struct PhysicsSystem {
    grid: SpatialGrid,
    gravity: Vec3,
}

impl System for PhysicsSystem {
    fn run(&mut self, world: &mut World) {
        let dt = 0.016; // 60 FPS

        // Update physics
        for (entity, rb) in world.query_one_mut::<RigidBody>().iter_mut() {
            // Apply gravity
            rb.velocity += self.gravity * rb.gravity_scale * dt;

            // Apply drag
            rb.velocity *= 1.0 - rb.drag * dt;
        }

        // Apply velocities to transforms
        // (simplified - real implementation would query both)

        // Update spatial grid
        // ...

        // Detect and resolve collisions
        // ...
    }
}
```

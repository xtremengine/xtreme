// Particle Compute Shader
// Handles particle simulation and spawning on GPU

struct Particle {
    position_age: vec4<f32>,      // xyz = position, w = age
    velocity_lifetime: vec4<f32>, // xyz = velocity, w = lifetime
    color: vec4<f32>,             // rgba
    size_rotation_flags: vec4<f32>, // x = size, y = rotation, z = alive flag, w = emitter_id
}

struct EmitterUniforms {
    transform: mat4x4<f32>,
    gravity_dt: vec4<f32>,        // xyz = gravity, w = delta_time
    wind_drag: vec4<f32>,         // xyz = wind, w = drag
    floor_params: vec4<f32>,      // x = floor_y, y = bounce, z = friction, w = enabled
    spawn_params: vec4<f32>,      // x = spawn_count, y = burst, z = time, w = seed
    buffer_params: vec4<u32>,     // x = max_particles, y = alive_count, z = shape_type, w = local_space
    shape_params: vec4<f32>,      // shape-specific parameters
    start_color: vec4<f32>,
    end_color: vec4<f32>,
    size_lifetime: vec4<f32>,     // x = start_size, y = end_size, z = min_lifetime, w = max_lifetime
    velocity_params: vec4<f32>,   // xyz = initial_velocity, w = speed_min
    velocity_params2: vec4<f32>,  // x = randomness, y = inherit, z = speed_max, w = rotation_speed
    camera_pos: vec4<f32>,
    camera_right: vec4<f32>,
    camera_up: vec4<f32>,
}

@group(0) @binding(0) var<storage, read> particles_in: array<Particle>;
@group(0) @binding(1) var<storage, read_write> particles_out: array<Particle>;
@group(0) @binding(2) var<storage, read_write> alive_count: atomic<u32>;
@group(0) @binding(3) var<uniform> emitter: EmitterUniforms;

// PCG random hash
fn pcg_hash(input: u32) -> u32 {
    let state = input * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn random_float(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967295.0;
}

fn random_range(seed: u32, min_val: f32, max_val: f32) -> f32 {
    return min_val + random_float(seed) * (max_val - min_val);
}

fn random_vec3(seed: u32) -> vec3<f32> {
    return vec3<f32>(
        random_float(seed) * 2.0 - 1.0,
        random_float(seed + 1u) * 2.0 - 1.0,
        random_float(seed + 2u) * 2.0 - 1.0
    );
}

fn random_on_sphere(seed: u32) -> vec3<f32> {
    let theta = random_float(seed) * 6.28318;
    let phi = acos(2.0 * random_float(seed + 1u) - 1.0);
    return vec3<f32>(
        sin(phi) * cos(theta),
        sin(phi) * sin(theta),
        cos(phi)
    );
}

fn random_in_sphere(seed: u32) -> vec3<f32> {
    let dir = random_on_sphere(seed);
    let r = pow(random_float(seed + 3u), 0.333);
    return dir * r;
}

// Spawn position based on shape
fn spawn_position(shape_type: u32, params: vec4<f32>, seed: u32) -> vec3<f32> {
    switch shape_type {
        // Point
        case 0u: {
            return vec3<f32>(0.0);
        }
        // Sphere
        case 1u: {
            let radius = params.x;
            return random_in_sphere(seed) * radius;
        }
        // Box
        case 2u: {
            return vec3<f32>(
                random_range(seed, -params.x, params.x),
                random_range(seed + 1u, -params.y, params.y),
                random_range(seed + 2u, -params.z, params.z)
            );
        }
        // Cone
        case 3u: {
            let angle = params.x;
            let radius = params.y;
            let r = random_float(seed) * radius;
            let theta = random_float(seed + 1u) * 6.28318;
            return vec3<f32>(r * cos(theta), 0.0, r * sin(theta));
        }
        // Disk
        case 4u: {
            let radius = params.x;
            let r = sqrt(random_float(seed)) * radius;
            let theta = random_float(seed + 1u) * 6.28318;
            return vec3<f32>(r * cos(theta), 0.0, r * sin(theta));
        }
        // Ring
        case 5u: {
            let inner = params.x;
            let outer = params.y;
            let r = sqrt(random_range(seed, inner * inner, outer * outer));
            let theta = random_float(seed + 1u) * 6.28318;
            return vec3<f32>(r * cos(theta), 0.0, r * sin(theta));
        }
        // Line
        case 6u: {
            let length = params.x;
            return vec3<f32>(0.0, random_range(seed, 0.0, length), 0.0);
        }
        default: {
            return vec3<f32>(0.0);
        }
    }
}

// Simulate existing particles
@compute @workgroup_size(256, 1, 1)
fn cs_simulate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let max_particles = emitter.buffer_params.x;

    if (idx >= max_particles) {
        return;
    }

    var p = particles_in[idx];

    // Check if alive
    let alive = p.size_rotation_flags.z > 0.5;
    if (!alive) {
        return;
    }

    let age = p.position_age.w;
    let lifetime = p.velocity_lifetime.w;
    let dt = emitter.gravity_dt.w;

    // Update age
    let new_age = age + dt;
    if (new_age >= lifetime) {
        // Particle died
        return;
    }

    // Get current state
    var position = p.position_age.xyz;
    var velocity = p.velocity_lifetime.xyz;

    // Apply gravity
    velocity += emitter.gravity_dt.xyz * dt;

    // Apply wind
    velocity += emitter.wind_drag.xyz * dt;

    // Apply drag
    let drag = emitter.wind_drag.w;
    velocity *= (1.0 - drag * dt);

    // Integrate position
    position += velocity * dt;

    // Floor collision
    let floor_enabled = emitter.floor_params.w > 0.5;
    if (floor_enabled && position.y < emitter.floor_params.x) {
        position.y = emitter.floor_params.x;
        let bounce = emitter.floor_params.y;
        let friction = emitter.floor_params.z;
        velocity.y = -velocity.y * bounce;
        velocity.x *= (1.0 - friction);
        velocity.z *= (1.0 - friction);
    }

    // Interpolate color based on lifetime
    let t = new_age / lifetime;
    let color = mix(emitter.start_color, emitter.end_color, t);

    // Interpolate size
    let start_size = emitter.size_lifetime.x;
    let end_size = emitter.size_lifetime.y;
    let size = mix(start_size, end_size, t);

    // Update rotation
    let rotation_speed = emitter.velocity_params2.w;
    let rotation = p.size_rotation_flags.y + rotation_speed * dt;

    // Write output
    p.position_age = vec4<f32>(position, new_age);
    p.velocity_lifetime = vec4<f32>(velocity, lifetime);
    p.color = color;
    p.size_rotation_flags = vec4<f32>(size, rotation, 1.0, p.size_rotation_flags.w);

    // Atomically allocate output slot
    let out_idx = atomicAdd(&alive_count, 1u);
    if (out_idx < max_particles) {
        particles_out[out_idx] = p;
    }
}

// Spawn new particles
@compute @workgroup_size(64, 1, 1)
fn cs_spawn(@builtin(global_invocation_id) gid: vec3<u32>) {
    let spawn_count = u32(emitter.spawn_params.x);
    if (gid.x >= spawn_count) {
        return;
    }

    let max_particles = emitter.buffer_params.x;
    let out_idx = atomicAdd(&alive_count, 1u);
    if (out_idx >= max_particles) {
        return;
    }

    // Generate seed
    let base_seed = u32(emitter.spawn_params.w * 1000000.0);
    let seed = base_seed + gid.x * 7u + u32(emitter.spawn_params.z * 100.0);

    // Spawn position
    let shape_type = emitter.buffer_params.z;
    var local_pos = spawn_position(shape_type, emitter.shape_params, seed);

    // Transform to world space
    let world_pos = (emitter.transform * vec4<f32>(local_pos, 1.0)).xyz;

    // Initial velocity
    let base_velocity = emitter.velocity_params.xyz;
    let randomness = emitter.velocity_params2.x;
    let speed_min = emitter.velocity_params.w;
    let speed_max = emitter.velocity_params2.z;

    var velocity = base_velocity;
    if (randomness > 0.0) {
        velocity += random_vec3(seed + 10u) * randomness;
    }
    velocity = normalize(velocity) * random_range(seed + 20u, speed_min, speed_max);

    // Lifetime
    let min_lifetime = emitter.size_lifetime.z;
    let max_lifetime = emitter.size_lifetime.w;
    let lifetime = random_range(seed + 30u, min_lifetime, max_lifetime);

    // Initial rotation
    let initial_rotation = random_float(seed + 40u) * 6.28318;

    // Create particle
    var p: Particle;
    p.position_age = vec4<f32>(world_pos, 0.0);
    p.velocity_lifetime = vec4<f32>(velocity, lifetime);
    p.color = emitter.start_color;
    p.size_rotation_flags = vec4<f32>(emitter.size_lifetime.x, initial_rotation, 1.0, 0.0);

    particles_out[out_idx] = p;
}

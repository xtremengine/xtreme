// Particle Billboard Render Shader
// Renders particles as camera-facing quads

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    position: vec4<f32>,
    right: vec4<f32>,
    up: vec4<f32>,
}

struct Particle {
    position_age: vec4<f32>,
    velocity_lifetime: vec4<f32>,
    color: vec4<f32>,
    size_rotation_flags: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;

@group(1) @binding(0) var t_particle: texture_2d<f32>;
@group(1) @binding(1) var s_particle: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

// Quad vertices (2 triangles)
const QUAD_UVS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0)
);

const QUAD_OFFSETS: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, -0.5),
    vec2<f32>(0.5, 0.5),
    vec2<f32>(-0.5, -0.5),
    vec2<f32>(0.5, 0.5),
    vec2<f32>(-0.5, 0.5)
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_idx: u32,
    @builtin(instance_index) instance_idx: u32
) -> VertexOutput {
    let p = particles[instance_idx];

    // Skip dead particles
    let alive = p.size_rotation_flags.z > 0.5;
    if (!alive) {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(0.0, 0.0, -1000.0, 1.0);
        out.uv = vec2<f32>(0.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    let world_pos = p.position_age.xyz;
    let size = p.size_rotation_flags.x;
    let rotation = p.size_rotation_flags.y;

    // Get quad offset
    let offset = QUAD_OFFSETS[vertex_idx] * size;

    // Apply rotation
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    let rotated_offset = vec2<f32>(
        offset.x * cos_r - offset.y * sin_r,
        offset.x * sin_r + offset.y * cos_r
    );

    // Billboard: offset in camera space
    let billboard_pos = world_pos
        + camera.right.xyz * rotated_offset.x
        + camera.up.xyz * rotated_offset.y;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(billboard_pos, 1.0);
    out.uv = QUAD_UVS[vertex_idx];
    out.color = p.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_particle, s_particle, in.uv);
    return tex_color * in.color;
}

// Alternative: soft particle (radial fade)
@fragment
fn fs_soft(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_particle, s_particle, in.uv);

    // Radial fade for soft edges
    let center_dist = length(in.uv - vec2<f32>(0.5));
    let alpha = smoothstep(0.5, 0.2, center_dist);

    var color = tex_color * in.color;
    color.a *= alpha;
    return color;
}

// Alternative: simple circle (no texture needed)
@fragment
fn fs_circle(in: VertexOutput) -> @location(0) vec4<f32> {
    let center_dist = length(in.uv - vec2<f32>(0.5));

    // Sharp circle edge
    if (center_dist > 0.5) {
        discard;
    }

    // Soft edge
    let alpha = smoothstep(0.5, 0.3, center_dist);
    var color = in.color;
    color.a *= alpha;
    return color;
}

// Additive blend fragment shader
@fragment
fn fs_additive(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_particle, s_particle, in.uv);

    // For additive, multiply RGB by alpha for proper glow
    var color = tex_color * in.color;
    color = vec4<f32>(color.rgb * color.a, color.a);
    return color;
}

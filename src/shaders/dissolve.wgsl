// Dissolve Shader
// Creates a dissolving effect using noise pattern

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
    time: vec4<f32>,  // time.x = elapsed time in seconds
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) local_pos: vec3<f32>,  // Object space position for stable dissolve pattern
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.local_pos = input.position;  // Use local position for stable pattern
    return output;
}

// Noise functions for dissolve pattern
fn hash3(p: vec3<f32>) -> f32 {
    let h = dot(p, vec3<f32>(127.1, 311.7, 74.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);

    return mix(
        mix(
            mix(hash3(i + vec3<f32>(0.0, 0.0, 0.0)), hash3(i + vec3<f32>(1.0, 0.0, 0.0)), u.x),
            mix(hash3(i + vec3<f32>(0.0, 1.0, 0.0)), hash3(i + vec3<f32>(1.0, 1.0, 0.0)), u.x),
            u.y
        ),
        mix(
            mix(hash3(i + vec3<f32>(0.0, 0.0, 1.0)), hash3(i + vec3<f32>(1.0, 0.0, 1.0)), u.x),
            mix(hash3(i + vec3<f32>(0.0, 1.0, 1.0)), hash3(i + vec3<f32>(1.0, 1.0, 1.0)), u.x),
            u.y
        ),
        u.z
    );
}

fn fbm3d(p: vec3<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var pos = p;

    for (var i = 0; i < 4; i++) {
        value += amplitude * noise3d(pos);
        pos *= 2.0;
        amplitude *= 0.5;
    }

    return value;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.time.x;

    // Dissolve threshold oscillates over time (0 to 1 and back)
    let dissolve_progress = (sin(time * 0.5) * 0.5 + 0.5);

    // Generate noise pattern based on local position (stays fixed relative to object)
    let noise_scale = 3.0;
    let noise_value = fbm3d(input.local_pos * noise_scale);

    // Dissolve edge width for glowing edge
    let edge_width = 0.1;

    // Discard pixels below threshold (dissolve effect)
    if noise_value < dissolve_progress - edge_width {
        discard;
    }

    // Calculate edge glow
    let edge_factor = smoothstep(dissolve_progress - edge_width, dissolve_progress, noise_value);
    let edge_glow = 1.0 - edge_factor;

    // Edge color (bright orange/yellow for burning effect)
    let edge_color = vec3<f32>(1.0, 0.5, 0.1) * edge_glow * 3.0;

    // Base color with lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(input.world_normal, light_dir), 0.0);
    let ambient = 0.3;
    let base_color = uniforms.color.rgb * (ambient + diffuse * 0.7);

    // Combine base color with edge glow
    let final_color = base_color * edge_factor + edge_color;

    return vec4<f32>(final_color, uniforms.color.a);
}

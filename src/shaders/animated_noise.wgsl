// Animated Noise Shader
// Creates a moving noise pattern effect

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
    @location(2) world_pos: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    return output;
}

// Simple noise functions
fn hash(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453123);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);

    let a = hash(i);
    let b = hash(i + vec2<f32>(1.0, 0.0));
    let c = hash(i + vec2<f32>(0.0, 1.0));
    let d = hash(i + vec2<f32>(1.0, 1.0));

    let u = f * f * (3.0 - 2.0 * f);

    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var freq = 1.0;
    var pos = p;

    for (var i = 0; i < 4; i++) {
        value += amplitude * noise(pos * freq);
        freq *= 2.0;
        amplitude *= 0.5;
        pos += vec2<f32>(1.7, 9.2);
    }

    return value;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Animate UV coordinates
    let time = uniforms.time.x;
    let moving_uv = input.uv * 4.0 + vec2<f32>(time * 0.3, time * 0.2);

    // Generate animated noise
    let n = fbm(moving_uv);

    // Create color variation based on noise
    let noise_color = vec3<f32>(
        n * 0.5 + 0.5,
        n * 0.3 + 0.2,
        n * 0.8 + 0.2
    );

    // Mix with object color
    let final_color = uniforms.color.rgb * noise_color;

    // Simple lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(input.world_normal, light_dir), 0.0);
    let ambient = 0.3;
    let lit_color = final_color * (ambient + diffuse * 0.7);

    return vec4<f32>(lit_color, uniforms.color.a);
}

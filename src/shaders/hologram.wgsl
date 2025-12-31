// Hologram Shader
// Creates a sci-fi hologram effect with scanlines and flicker

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
    @location(3) screen_pos: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    output.screen_pos = output.clip_position;
    return output;
}

// Simple hash for noise
fn hash(n: f32) -> f32 {
    return fract(sin(n) * 43758.5453123);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.time.x;

    // Hologram base color (cyan/blue tint)
    let holo_tint = vec3<f32>(0.3, 0.8, 1.0);
    let base_color = uniforms.color.rgb * holo_tint;

    // Scanlines effect (horizontal lines moving up)
    let screen_y = input.screen_pos.y / input.screen_pos.w;
    let scanline_y = input.world_pos.y * 20.0 - time * 3.0;
    let scanline = sin(scanline_y) * 0.5 + 0.5;
    let scanline_intensity = mix(0.7, 1.0, scanline);

    // Additional thin scanlines
    let thin_scanline = step(0.9, fract(input.world_pos.y * 50.0 - time * 5.0));

    // Flicker effect
    let flicker = 0.95 + 0.05 * sin(time * 30.0) * sin(time * 17.3);

    // Edge glow (fresnel-like effect based on normal)
    let view_dir = normalize(vec3<f32>(0.0, 0.0, 1.0)); // Simplified view direction
    let edge_factor = 1.0 - abs(dot(input.world_normal, view_dir));
    let edge_glow = pow(edge_factor, 2.0) * 0.5;

    // Random glitch lines
    let glitch_seed = floor(time * 10.0);
    let glitch = step(0.97, hash(glitch_seed + floor(input.world_pos.y * 10.0)));
    let glitch_offset = glitch * 0.3;

    // Combine effects
    var final_color = base_color;
    final_color *= scanline_intensity;
    final_color += vec3<f32>(thin_scanline * 0.1);
    final_color *= flicker;
    final_color += vec3<f32>(edge_glow) * holo_tint;
    final_color += vec3<f32>(glitch_offset) * holo_tint;

    // Add slight transparency
    let alpha = (0.7 + edge_glow * 0.3) * uniforms.color.a * flicker;

    return vec4<f32>(final_color, alpha);
}

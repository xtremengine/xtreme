// Pulse Glow Shader
// Creates a pulsing glow/breathing effect

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

    // Add slight vertex displacement for "breathing" effect
    let time = uniforms.time.x;
    let pulse = sin(time * 2.0) * 0.02;
    let displaced_pos = input.position + input.normal * pulse;

    let world_pos = uniforms.model * vec4<f32>(displaced_pos, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    output.world_pos = world_pos.xyz;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let time = uniforms.time.x;

    // Pulsing intensity (sine wave between 0.5 and 1.0)
    let pulse = sin(time * 3.0) * 0.25 + 0.75;

    // Secondary faster pulse for shimmer
    let shimmer = sin(time * 8.0) * 0.1 + 0.9;

    // Edge glow (fresnel effect)
    let view_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));
    let edge_factor = 1.0 - abs(dot(input.world_normal, view_dir));
    let edge_glow = pow(edge_factor, 3.0);

    // Base color with pulse
    let base_color = uniforms.color.rgb * pulse * shimmer;

    // Add bright edge glow
    let glow_color = uniforms.color.rgb * 1.5; // Brighter glow
    let final_color = base_color + edge_glow * glow_color * pulse;

    // Simple lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(input.world_normal, light_dir), 0.0);
    let ambient = 0.4;
    let lit_color = final_color * (ambient + diffuse * 0.6);

    // Emissive boost for glow effect
    let emissive = edge_glow * glow_color * pulse * 0.5;

    return vec4<f32>(lit_color + emissive, uniforms.color.a);
}

// Color Cycle Shader
// Smoothly cycles through rainbow colors over time

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
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_pos = uniforms.model * vec4<f32>(input.position, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;
    output.world_normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
    output.uv = input.uv;
    return output;
}

// Convert HSV to RGB
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;

    let c = v * s;
    let x = c * (1.0 - abs(((h * 6.0) % 2.0) - 1.0));
    let m = v - c;

    var rgb: vec3<f32>;
    let hi = i32(floor(h * 6.0)) % 6;

    if hi == 0 {
        rgb = vec3<f32>(c, x, 0.0);
    } else if hi == 1 {
        rgb = vec3<f32>(x, c, 0.0);
    } else if hi == 2 {
        rgb = vec3<f32>(0.0, c, x);
    } else if hi == 3 {
        rgb = vec3<f32>(0.0, x, c);
    } else if hi == 4 {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + vec3<f32>(m, m, m);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Create cycling hue based on time and position
    let time = uniforms.time.x;

    // Base hue cycles over time
    let base_hue = fract(time * 0.2);

    // Add spatial variation based on UV
    let spatial_offset = (input.uv.x + input.uv.y) * 0.3;
    let hue = fract(base_hue + spatial_offset);

    // Convert HSV to RGB (high saturation and value for vivid colors)
    let rainbow = hsv_to_rgb(vec3<f32>(hue, 0.8, 1.0));

    // Mix with object color for tinting
    let tinted = rainbow * uniforms.color.rgb;

    // Simple lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(input.world_normal, light_dir), 0.0);
    let ambient = 0.3;
    let lit_color = tinted * (ambient + diffuse * 0.7);

    return vec4<f32>(lit_color, uniforms.color.a);
}

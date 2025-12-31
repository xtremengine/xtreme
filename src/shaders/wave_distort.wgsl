// Wave Distortion Shader
// Creates a wavy, water-like distortion effect

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
    @location(2) distort_factor: f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let time = uniforms.time.x;

    // Create wave displacement
    let wave1 = sin(input.position.x * 3.0 + time * 2.0) * 0.1;
    let wave2 = sin(input.position.z * 4.0 + time * 1.5) * 0.08;
    let wave3 = cos(input.position.x * 2.0 + input.position.z * 2.0 + time * 3.0) * 0.05;

    let displacement = wave1 + wave2 + wave3;

    // Apply displacement primarily along Y axis
    var displaced_pos = input.position;
    displaced_pos.y += displacement;

    // Also add some X/Z wobble
    displaced_pos.x += sin(input.position.y * 5.0 + time * 2.5) * 0.03;
    displaced_pos.z += cos(input.position.y * 4.0 + time * 2.0) * 0.03;

    let world_pos = uniforms.model * vec4<f32>(displaced_pos, 1.0);
    output.clip_position = uniforms.view_proj * world_pos;

    // Modify normal based on wave slope (approximation)
    var modified_normal = input.normal;
    modified_normal.x += cos(input.position.x * 3.0 + time * 2.0) * 0.3;
    modified_normal.z += cos(input.position.z * 4.0 + time * 1.5) * 0.24;
    output.world_normal = normalize((uniforms.model * vec4<f32>(modified_normal, 0.0)).xyz);

    output.uv = input.uv;
    output.distort_factor = displacement;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Color variation based on distortion
    let distort_color = vec3<f32>(
        0.5 + input.distort_factor * 2.0,
        0.7 + input.distort_factor,
        1.0
    );

    // Mix with object color
    let base_color = uniforms.color.rgb * distort_color;

    // Lighting with modified normals
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(input.world_normal, light_dir), 0.0);
    let ambient = 0.3;

    // Add specular highlight for wet/shiny look
    let view_dir = normalize(vec3<f32>(0.0, 1.0, 1.0));
    let half_vec = normalize(light_dir + view_dir);
    let specular = pow(max(dot(input.world_normal, half_vec), 0.0), 32.0);

    let lit_color = base_color * (ambient + diffuse * 0.6) + vec3<f32>(specular * 0.3);

    return vec4<f32>(lit_color, uniforms.color.a);
}

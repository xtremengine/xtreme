// Simple textured 3D shader for Xtreme Engine Editor
// Supports: MVP transform, texture sampling, basic lighting

// Transform uniforms (Group 0) - same as basic.wgsl
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Texture bindings (Group 1)
@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(1) @binding(1)
var s_diffuse: sampler;

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
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_pos;

    // Transform normal to world space (simplified - assumes uniform scale)
    out.world_normal = normalize((transform.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);

    // Apply color tint
    let base_color = tex_color * transform.color;

    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let normal = normalize(in.world_normal);

    let ambient = 0.3;
    let diffuse = max(dot(normal, light_dir), 0.0) * 0.7;
    let lighting = ambient + diffuse;

    let final_color = base_color.rgb * lighting;
    return vec4<f32>(final_color, base_color.a);
}

// Textured 3D shader for Xtreme Engine
// Supports: MVP transform, texture sampling, PBR-like material, basic lighting

// Transform uniforms (Group 0)
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
}

// Material uniforms (Group 0, binding 1)
struct MaterialUniforms {
    base_color: vec4<f32>,           // RGBA base color
    emissive_metallic: vec4<f32>,    // RGB emissive + metallic
    roughness_alpha_flags: vec4<f32>, // roughness, alpha_cutoff, has_texture, transparent
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

@group(0) @binding(1)
var<uniform> material: MaterialUniforms;

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
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_pos;
    out.world_position = world_pos.xyz;

    // Transform normal to world space (simplified - assumes uniform scale)
    out.world_normal = normalize((transform.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get material properties
    let has_texture = material.roughness_alpha_flags.z > 0.5;
    let alpha_cutoff = material.roughness_alpha_flags.y;
    let is_transparent = material.roughness_alpha_flags.w > 0.5;
    let roughness = material.roughness_alpha_flags.x;
    let metallic = material.emissive_metallic.w;
    let emissive = material.emissive_metallic.xyz;

    // Sample texture or use base color
    var base_color: vec4<f32>;
    if has_texture {
        base_color = textureSample(t_diffuse, s_diffuse, in.uv);
    } else {
        base_color = material.base_color;
    }

    // Multiply by uniform color (tint)
    base_color = base_color * transform.color;

    // Alpha test
    if base_color.a < alpha_cutoff && !is_transparent {
        discard;
    }

    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let normal = normalize(in.world_normal);

    // Ambient + Diffuse
    let ambient = 0.3;
    let n_dot_l = max(dot(normal, light_dir), 0.0);
    let diffuse = n_dot_l * 0.7;

    // Simple specular (Blinn-Phong approximation)
    let view_dir = normalize(vec3<f32>(0.0, 1.0, 1.0)); // Simplified view
    let half_dir = normalize(light_dir + view_dir);
    let specular_power = mix(8.0, 64.0, 1.0 - roughness);
    let specular = pow(max(dot(normal, half_dir), 0.0), specular_power) * (1.0 - roughness) * 0.3;

    // Combine lighting
    let lighting = ambient + diffuse + specular * metallic;

    // Final color with emissive
    let final_color = base_color.rgb * lighting + emissive;

    return vec4<f32>(final_color, base_color.a);
}

// Variant without material uniforms (for simpler use)
@fragment
fn fs_simple(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);

    // Apply tint
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

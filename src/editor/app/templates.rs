//! Asset file templates for new files created in the editor.

/// Template for new WGSL shader files
pub const SHADER_TEMPLATE: &str = r#"// Xtreme Engine Shader
// Vertex and Fragment shader template

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    color: vec4<f32>,
    time: vec4<f32>,
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
    @location(2) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * uniforms.model * vec4(in.position, 1.0);
    out.world_normal = (uniforms.model * vec4(in.normal, 0.0)).xyz;
    out.uv = in.uv;
    out.color = uniforms.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple diffuse lighting
    let light_dir = normalize(vec3(0.3, 1.0, 0.5));
    let ambient = 0.3;
    let diffuse = max(dot(normalize(in.world_normal), light_dir), 0.0);
    let lighting = ambient + diffuse * 0.7;

    return vec4(in.color.rgb * lighting, in.color.a);
}
"#;

/// Template for new Python script files
pub const SCRIPT_TEMPLATE: &str = r#"# Xtreme Engine Script
# Lifecycle methods: _ready(ctx), _update(ctx, delta), _physics_update(ctx, delta)

speed = 5.0

def _ready(ctx):
    """Called once when the script is attached to an object."""
    print(f"Script ready on object {ctx['object_id']}")

def _update(ctx, delta):
    """Called every frame."""
    transform = ctx['transform']

    # Example: move forward over time
    # transform['position'][2] -= speed * delta

    return {'transform': transform}

def _physics_update(ctx, delta):
    """Called at fixed physics rate."""
    pass
"#;

/// Template for new scene files
pub const SCENE_TEMPLATE: &str = r#"SceneData(
    version: 2,
    name: "New Scene",
    camera_target: Some((0.0, 0.0, 0.0)),
    camera_distance: Some(15.0),
    objects: [],
)
"#;

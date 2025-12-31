// Grid shader for infinite ground plane
// Renders a procedural grid that fades with distance

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    grid_scale: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

// Full-screen quad vertices
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Generate a large quad on the XZ plane
    let size = 1000.0;
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-size, -size),
        vec2<f32>( size, -size),
        vec2<f32>( size,  size),
        vec2<f32>(-size, -size),
        vec2<f32>( size,  size),
        vec2<f32>(-size,  size),
    );

    let pos = positions[vertex_index];
    let world_pos = vec3<f32>(pos.x, 0.0, pos.y);

    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    out.world_pos = world_pos;
    return out;
}

// Grid line function
fn grid_line(coord: f32, line_width: f32) -> f32 {
    let d = abs(fract(coord - 0.5) - 0.5);
    return 1.0 - smoothstep(0.0, line_width, d);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scale = uniforms.grid_scale;

    // Calculate grid coordinates
    let coord = in.world_pos.xz / scale;

    // Line widths (in grid units)
    let minor_width = 0.02;
    let major_width = 0.04;

    // Minor grid (every 1 unit)
    let minor_x = grid_line(coord.x, minor_width);
    let minor_z = grid_line(coord.y, minor_width);
    let minor = max(minor_x, minor_z);

    // Major grid (every 10 units)
    let major_coord = coord / 10.0;
    let major_x = grid_line(major_coord.x, major_width);
    let major_z = grid_line(major_coord.y, major_width);
    let major = max(major_x, major_z);

    // Axis lines
    let axis_width = 0.06;
    let axis_x = grid_line(in.world_pos.x / scale, axis_width); // Z axis (blue)
    let axis_z = grid_line(in.world_pos.z / scale, axis_width); // X axis (red)

    // Distance fade
    let dist = length(in.world_pos.xz - uniforms.camera_pos.xz);
    let fade = 1.0 - smoothstep(50.0, 200.0, dist);

    // Combine grids
    var color = vec3<f32>(0.3, 0.3, 0.3); // Minor grid color
    var alpha = minor * 0.3;

    // Major grid overlay
    color = mix(color, vec3<f32>(0.5, 0.5, 0.5), major);
    alpha = max(alpha, major * 0.5);

    // Axis colors
    if (abs(in.world_pos.z) < scale * 0.1) {
        color = mix(color, vec3<f32>(0.8, 0.2, 0.2), axis_z); // X axis - red
        alpha = max(alpha, axis_z * 0.8);
    }
    if (abs(in.world_pos.x) < scale * 0.1) {
        color = mix(color, vec3<f32>(0.2, 0.2, 0.8), axis_x); // Z axis - blue
        alpha = max(alpha, axis_x * 0.8);
    }

    // Apply fade
    alpha *= fade;

    // Discard if too transparent
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(color, alpha);
}

# Rendering System

Xtreme Engine uses WGPU for cross-platform GPU rendering.

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│                Application                   │
├─────────────────────────────────────────────┤
│              RenderContext                   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ Device  │ │  Queue  │ │ Surface │       │
│  └─────────┘ └─────────┘ └─────────┘       │
├─────────────────────────────────────────────┤
│            Render Pipeline                   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ Shaders │ │ Layouts │ │Bindings │       │
│  └─────────┘ └─────────┘ └─────────┘       │
├─────────────────────────────────────────────┤
│                Resources                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐       │
│  │ Meshes  │ │Textures │ │Uniforms │       │
│  └─────────┘ └─────────┘ └─────────┘       │
└─────────────────────────────────────────────┘
```

## Supported Backends

| Platform | Backend |
|----------|---------|
| Windows | Vulkan, DirectX 12, DirectX 11 |
| macOS | Metal |
| Linux | Vulkan |
| Web | WebGPU |

WGPU automatically selects the best available backend.

## Window Management

### Creating a Window

```rust
use xtreme::render::{WindowConfig, run, App};

fn main() {
    let config = WindowConfig::new("My Game")
        .with_size(1280, 720)
        .with_resizable(true);

    run(MyApp::new(), config).unwrap();
}
```

### Window Configuration

```rust
let config = WindowConfig::new("Title")
    .with_size(1920, 1080)      // Initial size
    .with_resizable(true)        // Allow resize
    .with_vsync(true)            // Enable VSync
    .with_fullscreen(false);     // Windowed mode
```

### Application Trait

```rust
use xtreme::render::{App, AppEvent, RenderContext};

struct MyApp {
    // Your state
}

impl App for MyApp {
    fn init(&mut self, ctx: &mut RenderContext) {
        // Initialize resources
    }

    fn update(&mut self, ctx: &mut RenderContext, dt: f32) {
        // Update logic
    }

    fn render(&mut self, ctx: &mut RenderContext) {
        // Draw
    }

    fn event(&mut self, event: AppEvent) {
        // Handle input events
    }

    fn resize(&mut self, ctx: &mut RenderContext, width: u32, height: u32) {
        // Handle window resize
    }
}
```

## Render Context

The `RenderContext` provides access to GPU resources:

```rust
// Device - create GPU resources
let buffer = ctx.device.create_buffer(...);
let texture = ctx.device.create_texture(...);

// Queue - submit commands
ctx.queue.write_buffer(&buffer, 0, &data);
ctx.queue.submit([encoder.finish()]);

// Surface - get current frame
let output = ctx.surface.get_current_texture()?;
```

## Cameras

### Isometric Camera

Optimized for isometric/orthographic games:

```rust
use xtreme::render::IsometricCamera;
use glam::Vec3;

let mut camera = IsometricCamera::new();

// Camera position
camera.target = Vec3::ZERO;      // Look-at point
camera.distance = 20.0;          // Distance from target
camera.zoom = 10.0;              // Orthographic zoom

// Camera angles
camera.yaw = 45.0_f32.to_radians();   // Horizontal rotation
camera.pitch = 30.0_f32.to_radians(); // Vertical angle

// Get matrices
let view = camera.view_matrix();
let projection = camera.projection_matrix(aspect_ratio);
let view_projection = camera.view_projection();
```

### Camera Controller

```rust
use xtreme::render::CameraController;

let mut controller = CameraController::new(&mut camera);

// Orbit (right-click drag)
controller.orbit(delta_x, delta_y);

// Pan (middle-click drag)
controller.pan(delta_x, delta_y);

// Zoom (scroll wheel)
controller.zoom(scroll_delta);

// Focus on point
controller.focus(target_position, distance);
```

## Meshes

### Built-in Primitives

```rust
use xtreme::render::{Mesh, Primitive};

// Create primitives
let cube = Mesh::cube(1.0);           // 1x1x1 cube
let sphere = Mesh::sphere(0.5, 16);   // Radius 0.5, 16 segments
let plane = Mesh::plane(10.0, 10.0);  // 10x10 plane
```

### Mesh Builder

```rust
use xtreme::render::MeshBuilder;
use glam::Vec3;

let mesh = MeshBuilder::new()
    .add_vertex(Vec3::new(-1.0, 0.0, -1.0))
    .add_vertex(Vec3::new( 1.0, 0.0, -1.0))
    .add_vertex(Vec3::new( 0.0, 0.0,  1.0))
    .add_index(0)
    .add_index(1)
    .add_index(2)
    .build();
```

### GPU Mesh

Upload mesh to GPU:

```rust
use xtreme::render::GpuMesh;

let gpu_mesh = GpuMesh::from_mesh(&ctx.device, &mesh);

// During render
render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
render_pass.set_index_buffer(gpu_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
render_pass.draw_indexed(0..gpu_mesh.index_count, 0, 0..1);
```

## Vertices

### Vertex Format

```rust
use xtreme::render::Vertex;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}
```

### Vertex Layout

```rust
use xtreme::render::VertexLayout;

let layout = VertexLayout::new()
    .add_attribute(0, wgpu::VertexFormat::Float32x3)  // position
    .add_attribute(1, wgpu::VertexFormat::Float32x3)  // normal
    .add_attribute(2, wgpu::VertexFormat::Float32x2)  // uv
    .add_attribute(3, wgpu::VertexFormat::Float32x4); // color
```

## Materials

### Basic Material

```rust
use xtreme::render::Material;
use glam::Vec4;

let material = Material {
    color: Vec4::new(1.0, 0.5, 0.2, 1.0),  // RGBA
    metallic: 0.0,
    roughness: 0.5,
    emissive: Vec4::ZERO,
};
```

### Shaders

```rust
use xtreme::render::Shader;

// Load WGSL shader
let shader = Shader::from_wgsl(
    &ctx.device,
    include_str!("shaders/basic.wgsl"),
);
```

## Textures

### Loading Textures

```rust
use xtreme::render::Texture;

// From file
let texture = Texture::from_file(&ctx.device, &ctx.queue, "assets/texture.png")?;

// From bytes
let texture = Texture::from_bytes(&ctx.device, &ctx.queue, &image_data, width, height)?;
```

### Samplers

```rust
use xtreme::render::Sampler;

let sampler = Sampler::new(&ctx.device)
    .filter_linear()       // Bilinear filtering
    .address_repeat()      // Repeat texture
    .build();
```

## Render Pipeline

### Creating a Pipeline

```rust
use xtreme::render::RenderPipeline;

let pipeline = RenderPipeline::new(&ctx.device)
    .with_shader(&shader)
    .with_vertex_layout(&vertex_layout)
    .with_bind_group_layout(&bind_group_layout)
    .with_depth_test(true)
    .with_cull_mode(wgpu::Face::Back)
    .build();
```

### Uniforms

```rust
use xtreme::render::Uniforms;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    model: [[f32; 4]; 4],
    view_projection: [[f32; 4]; 4],
    color: [f32; 4],
}

// Create buffer
let uniform_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Uniform Buffer"),
    contents: bytemuck::cast_slice(&[uniforms]),
    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
});

// Update uniforms
ctx.queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
```

## Render Pass

### Basic Render Loop

```rust
fn render(&mut self, ctx: &mut RenderContext) {
    // Get current frame
    let output = ctx.surface.get_current_texture().unwrap();
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create command encoder
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    // Begin render pass
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1, g: 0.1, b: 0.1, a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(...),
            ..Default::default()
        });

        // Draw
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.mesh.index_count, 0, 0..1);
    }

    // Submit
    ctx.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}
```

## Egui Integration

### Setup

```rust
use xtreme::render::EguiIntegration;

let mut egui = EguiIntegration::new(&ctx.device, surface_format, &window);
```

### Rendering UI

```rust
fn render(&mut self, ctx: &mut RenderContext) {
    // Begin egui frame
    let input = self.egui.take_input(&window);
    let egui_ctx = self.egui.context();

    // Draw UI
    egui::Window::new("Debug").show(&egui_ctx, |ui| {
        ui.label("Hello, World!");
        if ui.button("Click me").clicked() {
            println!("Clicked!");
        }
    });

    // End frame and render
    let output = self.egui.end_frame(&window);
    let paint_jobs = egui_ctx.tessellate(output.shapes);

    // Render egui
    self.egui.render(
        &ctx.device,
        &ctx.queue,
        &mut encoder,
        &view,
        &paint_jobs,
        &output.textures_delta,
    );
}
```

## Performance Tips

### 1. Batch Draw Calls

```rust
// Bad: Many draw calls
for object in objects {
    render_pass.draw_indexed(...);
}

// Good: Instanced rendering
render_pass.draw_indexed(0..index_count, 0, 0..instance_count);
```

### 2. Minimize State Changes

```rust
// Bad: Set pipeline for each object
for object in objects {
    render_pass.set_pipeline(&pipeline);
    render_pass.draw(...);
}

// Good: Group by pipeline
render_pass.set_pipeline(&pipeline_a);
for object in group_a {
    render_pass.draw(...);
}
render_pass.set_pipeline(&pipeline_b);
for object in group_b {
    render_pass.draw(...);
}
```

### 3. Use Dynamic Uniform Buffers

```rust
// Single buffer with offsets for many objects
let uniform_size = std::mem::size_of::<Uniforms>() as u32;
let aligned_size = align_to(uniform_size, min_alignment);

// Draw with offset
render_pass.set_bind_group(0, &bind_group, &[object_index * aligned_size]);
```

### 4. Frustum Culling

```rust
// Only render visible objects
for object in objects {
    if camera.frustum.contains(object.bounds) {
        render_object(object);
    }
}
```

## Shaders (WGSL)

### Basic Vertex Shader

```wgsl
struct Uniforms {
    model: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    color: vec4<f32>,
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
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_projection * world_position;
    out.world_normal = (uniforms.model * vec4<f32>(in.normal, 0.0)).xyz;
    out.uv = in.uv;

    return out;
}
```

### Basic Fragment Shader

```wgsl
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional light
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(in.world_normal);
    let diffuse = max(dot(normal, light_dir), 0.0);

    let ambient = 0.2;
    let lighting = ambient + diffuse * 0.8;

    return vec4<f32>(uniforms.color.rgb * lighting, uniforms.color.a);
}
```

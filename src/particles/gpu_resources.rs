//! GPU Resources for Particle System
//!
//! Manages GPU buffers for compute shader simulation.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;

/// GPU particle data structure
///
/// Optimized layout: 64 bytes per particle (cache-aligned)
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GpuParticle {
    /// Position (xyz) + age (w)
    pub position_age: [f32; 4],
    /// Velocity (xyz) + lifetime (w)
    pub velocity_lifetime: [f32; 4],
    /// Color RGBA
    pub color: [f32; 4],
    /// Size (x), rotation (y), flags (z), emitter_id (w)
    pub size_rotation_flags: [f32; 4],
}

impl Default for GpuParticle {
    fn default() -> Self {
        Self {
            position_age: [0.0, 0.0, 0.0, 0.0],
            velocity_lifetime: [0.0, 0.0, 0.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            size_rotation_flags: [0.1, 0.0, 1.0, 0.0],
        }
    }
}

impl GpuParticle {
    /// Create a new particle
    pub fn new(position: Vec3, velocity: Vec3, lifetime: f32, color: Vec4, size: f32) -> Self {
        Self {
            position_age: [position.x, position.y, position.z, 0.0],
            velocity_lifetime: [velocity.x, velocity.y, velocity.z, lifetime],
            color: [color.x, color.y, color.z, color.w],
            size_rotation_flags: [size, 0.0, 1.0, 0.0],
        }
    }

    /// Check if particle is alive
    pub fn is_alive(&self) -> bool {
        self.position_age[3] < self.velocity_lifetime[3] && self.size_rotation_flags[2] > 0.5
    }

    /// Size in bytes
    pub const SIZE: u64 = std::mem::size_of::<Self>() as u64;
}

/// Emitter uniforms for compute shader
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct EmitterUniforms {
    /// Emitter world transform (4x4 matrix)
    pub transform: [[f32; 4]; 4],
    /// Gravity (xyz) + delta_time (w)
    pub gravity_dt: [f32; 4],
    /// Wind (xyz) + drag (w)
    pub wind_drag: [f32; 4],
    /// Floor: y level, bounce, friction, enabled
    pub floor_params: [f32; 4],
    /// Spawn: rate*dt, burst_count, time, random_seed
    pub spawn_params: [f32; 4],
    /// Max particles, alive count, shape type, local space
    pub buffer_params: [u32; 4],
    /// Shape params
    pub shape_params: [f32; 4],
    /// Start color
    pub start_color: [f32; 4],
    /// End color
    pub end_color: [f32; 4],
    /// Start size, end size, min lifetime, max lifetime
    pub size_lifetime: [f32; 4],
    /// Initial velocity (xyz) + speed_min (w)
    pub velocity_params: [f32; 4],
    /// Velocity randomness, inherit_velocity, speed_max, rotation_speed
    pub velocity_params2: [f32; 4],
    /// Camera position (xyz) + padding
    pub camera_pos: [f32; 4],
    /// Camera right vector (xyz) + padding
    pub camera_right: [f32; 4],
    /// Camera up vector (xyz) + padding
    pub camera_up: [f32; 4],
}

impl Default for EmitterUniforms {
    fn default() -> Self {
        Self {
            transform: Mat4::IDENTITY.to_cols_array_2d(),
            gravity_dt: [0.0, -9.8, 0.0, 0.016],
            wind_drag: [0.0, 0.0, 0.0, 0.1],
            floor_params: [0.0, 0.5, 0.3, 0.0],
            spawn_params: [1.0, 0.0, 0.0, 0.0],
            buffer_params: [1000, 0, 0, 0],
            shape_params: [0.0, 0.0, 0.0, 0.0],
            start_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            size_lifetime: [0.1, 0.05, 1.0, 2.0],
            velocity_params: [0.0, 1.0, 0.0, 1.0],
            velocity_params2: [0.2, 0.0, 5.0, 0.0],
            camera_pos: [0.0, 0.0, 0.0, 0.0],
            camera_right: [1.0, 0.0, 0.0, 0.0],
            camera_up: [0.0, 1.0, 0.0, 0.0],
        }
    }
}

impl EmitterUniforms {
    /// Size in bytes
    pub const SIZE: u64 = std::mem::size_of::<Self>() as u64;
}

/// Indirect draw arguments for GPU-driven rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct DrawIndirectArgs {
    /// Vertices per instance (6 for quad)
    pub vertex_count: u32,
    /// Number of instances (particles)
    pub instance_count: u32,
    /// First vertex
    pub first_vertex: u32,
    /// First instance
    pub first_instance: u32,
}

impl Default for DrawIndirectArgs {
    fn default() -> Self {
        Self {
            vertex_count: 6,
            instance_count: 0,
            first_vertex: 0,
            first_instance: 0,
        }
    }
}

/// Particle buffer pool with ping-pong buffers
pub struct ParticleBufferPool {
    /// Particle data buffers (ping-pong)
    particle_buffers: [wgpu::Buffer; 2],
    /// Alive particle count (atomic)
    count_buffer: wgpu::Buffer,
    /// Indirect draw buffer
    indirect_buffer: wgpu::Buffer,
    /// Emitter uniform buffer
    emitter_buffer: wgpu::Buffer,
    /// Current read buffer index
    read_index: usize,
    /// Max particles
    max_particles: u32,
}

impl ParticleBufferPool {
    /// Create a new buffer pool
    pub fn new(device: &wgpu::Device, max_particles: u32) -> Self {
        let buffer_size = GpuParticle::SIZE * max_particles as u64;

        let particle_buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Particle Buffer A"),
                size: buffer_size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Particle Buffer B"),
                size: buffer_size,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];

        let count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Count Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let indirect_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Indirect Buffer"),
            contents: bytemuck::bytes_of(&DrawIndirectArgs::default()),
            usage: wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let emitter_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Emitter Uniform Buffer"),
            contents: bytemuck::bytes_of(&EmitterUniforms::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            particle_buffers,
            count_buffer,
            indirect_buffer,
            emitter_buffer,
            read_index: 0,
            max_particles,
        }
    }

    /// Swap read/write buffers
    pub fn swap_buffers(&mut self) {
        self.read_index = 1 - self.read_index;
    }

    /// Get read buffer (current particles)
    pub fn read_buffer(&self) -> &wgpu::Buffer {
        &self.particle_buffers[self.read_index]
    }

    /// Get write buffer (next frame particles)
    pub fn write_buffer(&self) -> &wgpu::Buffer {
        &self.particle_buffers[1 - self.read_index]
    }

    /// Get count buffer
    pub fn count_buffer(&self) -> &wgpu::Buffer {
        &self.count_buffer
    }

    /// Get indirect buffer
    pub fn indirect_buffer(&self) -> &wgpu::Buffer {
        &self.indirect_buffer
    }

    /// Get emitter buffer
    pub fn emitter_buffer(&self) -> &wgpu::Buffer {
        &self.emitter_buffer
    }

    /// Max particles
    pub fn max_particles(&self) -> u32 {
        self.max_particles
    }

    /// Reset count to zero
    pub fn reset_count(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.count_buffer, 0, bytemuck::bytes_of(&0u32));
    }

    /// Update emitter uniforms
    pub fn update_emitter(&self, queue: &wgpu::Queue, uniforms: &EmitterUniforms) {
        queue.write_buffer(&self.emitter_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    /// Update indirect draw count
    pub fn update_indirect(&self, queue: &wgpu::Queue, instance_count: u32) {
        let args = DrawIndirectArgs {
            vertex_count: 6,
            instance_count,
            first_vertex: 0,
            first_instance: 0,
        };
        queue.write_buffer(&self.indirect_buffer, 0, bytemuck::bytes_of(&args));
    }
}

/// Camera uniforms for billboard rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CameraUniforms {
    /// View-projection matrix
    pub view_proj: [[f32; 4]; 4],
    /// Camera position
    pub position: [f32; 4],
    /// Camera right vector
    pub right: [f32; 4],
    /// Camera up vector
    pub up: [f32; 4],
}

impl Default for CameraUniforms {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            position: [0.0, 0.0, 0.0, 0.0],
            right: [1.0, 0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0, 0.0],
        }
    }
}

impl CameraUniforms {
    /// Size in bytes
    pub const SIZE: u64 = std::mem::size_of::<Self>() as u64;
}

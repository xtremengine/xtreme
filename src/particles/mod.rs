//! # Particle System Module
//!
//! GPU-accelerated particle system using compute shaders.
//!
//! ## Features
//!
//! - Millions of particles via GPU compute shaders
//! - Configurable emitter shapes (point, sphere, box, cone, disk, ring, line)
//! - Physics simulation (gravity, wind, drag, floor collision)
//! - Billboard rendering with camera-facing quads
//! - Multiple blend modes (alpha, additive)
//! - ECS integration with ParticleEmitter component
//! - Preset effects (fire, smoke, sparkles, rain, explosion)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use xtreme::particles::*;
//! use xtreme::math::Transform;
//!
//! // Create particle manager
//! let mut particles = ParticleManager::new(&device, &queue, surface_format);
//!
//! // Spawn fire emitter
//! let fire = world.spawn()
//!     .with(Transform::from_xyz(0.0, 0.0, 0.0))
//!     .with(ParticleEmitter::fire())
//!     .build();
//!
//! // Or create custom emitter
//! let config = EmitterConfig::new("Custom")
//!     .with_max_particles(10000)
//!     .with_shape(EmitterShape::Sphere { radius: 1.0 })
//!     .with_spawn_rate(500.0)
//!     .with_gravity(Vec3::new(0.0, -9.8, 0.0))
//!     .with_colors(
//!         Vec4::new(1.0, 1.0, 0.0, 1.0),
//!         Vec4::new(1.0, 0.0, 0.0, 0.0),
//!     );
//!
//! let custom = world.spawn()
//!     .with(Transform::default())
//!     .with(ParticleEmitter::new(config))
//!     .build();
//!
//! // In game loop:
//! particles.update(
//!     &device,
//!     &mut encoder,
//!     &queue,
//!     &mut world,
//!     camera_view_proj,
//!     camera_pos,
//!     camera_right,
//!     camera_up,
//!     delta_time,
//! );
//!
//! // In render pass:
//! particles.render(&mut render_pass, &world);
//! ```
//!
//! ## Performance
//!
//! The system uses GPU compute shaders for particle simulation:
//! - 100,000 particles: ~5 MB GPU memory, 60+ FPS
//! - 500,000 particles: ~24 MB GPU memory, 60+ FPS
//! - 1,000,000 particles: ~48 MB GPU memory, 45-60 FPS
//!
//! ## Presets
//!
//! Pre-configured emitter configs are available:
//! - `presets::fire()` - Rising flame particles
//! - `presets::smoke()` - Billowing smoke
//! - `presets::sparkles()` - Magical sparkles
//! - `presets::rain()` - Falling rain
//! - `presets::explosion()` - Burst explosion

mod component;
mod compute;
mod emitter;
mod gpu_resources;
mod render;
mod system;

pub use component::{
    AttractorFalloff, ForceFieldShape, ParticleAffected, ParticleAttractor, ParticleEmitter,
    ParticleForceField, ParticleSystemHandle,
};
pub use compute::ParticleComputePipeline;
pub use emitter::{
    presets, AppearanceConfig, BlendMode, EmitterConfig, EmitterShape, PhysicsConfig, SpawnConfig,
};
pub use gpu_resources::{
    CameraUniforms, DrawIndirectArgs, EmitterUniforms, GpuParticle, ParticleBufferPool,
};
pub use render::ParticleRenderPipeline;
pub use system::ParticleManager;

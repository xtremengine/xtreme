//! Particle System Components
//!
//! ECS components for the particle system.

use super::emitter::EmitterConfig;
use crate::core::Component;

/// Handle to GPU resources for a particle system
#[derive(Clone, Copy, Debug)]
pub struct ParticleSystemHandle {
    /// Index into the buffer pool array
    pub buffer_index: usize,
    /// Currently alive particles
    pub alive_count: u32,
    /// Maximum particles
    pub max_particles: u32,
}

/// Particle emitter component
///
/// Attach this to an entity to emit particles from its position.
#[derive(Clone)]
pub struct ParticleEmitter {
    /// Emitter configuration
    pub config: EmitterConfig,
    /// Whether emitter is enabled
    pub enabled: bool,
    /// GPU resource handle
    pub gpu_handle: Option<ParticleSystemHandle>,
    /// Spawn accumulator (for rate-based spawning)
    pub spawn_accumulator: f32,
    /// Burst timer
    pub burst_timer: f32,
    /// Time since emitter started
    pub time_alive: f32,
    /// Whether emitter has finished (for non-looping)
    pub finished: bool,
    /// Previous position (for velocity inheritance)
    pub previous_position: glam::Vec3,
    /// Random seed for this emitter
    pub random_seed: u32,
}

impl Component for ParticleEmitter {}

impl ParticleEmitter {
    /// Create a new emitter with config
    pub fn new(config: EmitterConfig) -> Self {
        Self {
            config,
            enabled: true,
            gpu_handle: None,
            spawn_accumulator: 0.0,
            burst_timer: 0.0,
            time_alive: 0.0,
            finished: false,
            previous_position: glam::Vec3::ZERO,
            random_seed: rand::random(),
        }
    }

    /// Create from preset
    pub fn fire() -> Self {
        Self::new(super::emitter::presets::fire())
    }

    /// Create from preset
    pub fn smoke() -> Self {
        Self::new(super::emitter::presets::smoke())
    }

    /// Create from preset
    pub fn sparkles() -> Self {
        Self::new(super::emitter::presets::sparkles())
    }

    /// Create from preset
    pub fn rain() -> Self {
        Self::new(super::emitter::presets::rain())
    }

    /// Create from preset
    pub fn explosion() -> Self {
        Self::new(super::emitter::presets::explosion())
    }

    /// Start emitting
    pub fn start(&mut self) {
        self.enabled = true;
        self.finished = false;
        self.time_alive = 0.0;
    }

    /// Stop emitting (particles continue to live)
    pub fn stop(&mut self) {
        self.enabled = false;
    }

    /// Reset emitter
    pub fn reset(&mut self) {
        self.spawn_accumulator = 0.0;
        self.burst_timer = 0.0;
        self.time_alive = 0.0;
        self.finished = false;
    }

    /// Calculate how many particles to spawn this frame
    pub fn calculate_spawn_count(&mut self, dt: f32) -> u32 {
        if !self.enabled || self.finished {
            return 0;
        }

        let mut spawn_count = 0u32;

        // Rate-based spawning
        if self.config.spawn.rate > 0.0 {
            self.spawn_accumulator += self.config.spawn.rate * dt;
            let rate_spawn = self.spawn_accumulator.floor() as u32;
            self.spawn_accumulator -= rate_spawn as f32;
            spawn_count += rate_spawn;
        }

        // Burst spawning
        if self.config.spawn.burst_count > 0 {
            self.burst_timer += dt;
            if self.burst_timer >= self.config.spawn.burst_interval {
                spawn_count += self.config.spawn.burst_count;
                self.burst_timer = 0.0;
            }
        }

        // Check duration
        if self.config.duration > 0.0 && self.time_alive >= self.config.duration {
            if !self.config.looping {
                self.finished = true;
                return 0;
            }
            self.time_alive = 0.0;
        }

        self.time_alive += dt;

        // Limit to max particles
        if let Some(handle) = &self.gpu_handle {
            let available = handle.max_particles.saturating_sub(handle.alive_count);
            spawn_count = spawn_count.min(available);
        }

        spawn_count
    }

    /// Update random seed
    pub fn advance_seed(&mut self) {
        self.random_seed = self
            .random_seed
            .wrapping_mul(1103515245)
            .wrapping_add(12345);
    }
}

/// Marker component for entities affected by particles (optional)
#[derive(Clone, Copy, Debug, Default)]
pub struct ParticleAffected {
    /// Multiplier for particle force effects
    pub force_multiplier: f32,
}

impl Component for ParticleAffected {}

/// Particle attractor component
///
/// Attracts or repels particles within range.
#[derive(Clone, Debug)]
pub struct ParticleAttractor {
    /// Force strength (positive = attract, negative = repel)
    pub strength: f32,
    /// Effect radius
    pub radius: f32,
    /// Falloff type
    pub falloff: AttractorFalloff,
    /// Affect only specific emitters (empty = all)
    pub emitter_filter: Vec<String>,
}

impl Component for ParticleAttractor {}

impl Default for ParticleAttractor {
    fn default() -> Self {
        Self {
            strength: 10.0,
            radius: 5.0,
            falloff: AttractorFalloff::InverseSquare,
            emitter_filter: Vec::new(),
        }
    }
}

impl ParticleAttractor {
    /// Create a new attractor
    pub fn new(strength: f32, radius: f32) -> Self {
        Self {
            strength,
            radius,
            ..Default::default()
        }
    }

    /// Create a repulsor
    pub fn repulsor(strength: f32, radius: f32) -> Self {
        Self::new(-strength.abs(), radius)
    }
}

/// Attractor falloff type
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttractorFalloff {
    /// Constant force within radius
    Constant,
    /// Linear falloff
    Linear,
    /// Inverse square (realistic)
    #[default]
    InverseSquare,
}

/// Particle force field component
///
/// Applies directional force to particles.
#[derive(Clone, Debug)]
pub struct ParticleForceField {
    /// Force direction and magnitude
    pub force: glam::Vec3,
    /// Shape of the field
    pub shape: ForceFieldShape,
    /// Whether force is relative to field rotation
    pub local_force: bool,
}

impl Component for ParticleForceField {}

impl Default for ParticleForceField {
    fn default() -> Self {
        Self {
            force: glam::Vec3::new(0.0, 10.0, 0.0),
            shape: ForceFieldShape::Global,
            local_force: false,
        }
    }
}

/// Force field shape
#[derive(Clone, Debug, Default)]
pub enum ForceFieldShape {
    /// Affects all particles
    #[default]
    Global,
    /// Sphere with radius
    Sphere { radius: f32 },
    /// Box with half extents
    Box { half_extents: glam::Vec3 },
}

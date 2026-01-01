//! Particle Emitter Configuration
//!
//! Defines emitter shapes, spawn settings, and physics configuration.

use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Emitter shape types
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum EmitterShape {
    /// Single point emission
    #[default]
    Point,
    /// Sphere with radius
    Sphere { radius: f32 },
    /// Box with half extents
    Box { half_extents: Vec3 },
    /// Cone with angle (degrees) and base radius
    Cone { angle: f32, radius: f32 },
    /// Disk on XZ plane
    Disk { radius: f32 },
    /// Ring with inner and outer radius
    Ring {
        inner_radius: f32,
        outer_radius: f32,
    },
    /// Line between two points
    Line { length: f32 },
}

impl EmitterShape {
    /// Get shape type ID for GPU
    pub fn type_id(&self) -> u32 {
        match self {
            Self::Point => 0,
            Self::Sphere { .. } => 1,
            Self::Box { .. } => 2,
            Self::Cone { .. } => 3,
            Self::Disk { .. } => 4,
            Self::Ring { .. } => 5,
            Self::Line { .. } => 6,
        }
    }

    /// Get shape parameters for GPU (4 floats)
    pub fn params(&self) -> [f32; 4] {
        match self {
            Self::Point => [0.0, 0.0, 0.0, 0.0],
            Self::Sphere { radius } => [*radius, 0.0, 0.0, 0.0],
            Self::Box { half_extents } => [half_extents.x, half_extents.y, half_extents.z, 0.0],
            Self::Cone { angle, radius } => [angle.to_radians(), *radius, 0.0, 0.0],
            Self::Disk { radius } => [*radius, 0.0, 0.0, 0.0],
            Self::Ring {
                inner_radius,
                outer_radius,
            } => [*inner_radius, *outer_radius, 0.0, 0.0],
            Self::Line { length } => [*length, 0.0, 0.0, 0.0],
        }
    }
}

/// Particle spawn configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnConfig {
    /// Particles per second
    pub rate: f32,
    /// Burst spawn count
    pub burst_count: u32,
    /// Time between bursts
    pub burst_interval: f32,
    /// Initial velocity direction and magnitude
    pub initial_velocity: Vec3,
    /// Random velocity variation (0-1)
    pub velocity_randomness: f32,
    /// Inherit velocity from emitter movement (0-1)
    pub inherit_velocity: f32,
    /// Initial speed range
    pub speed_range: Range<f32>,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            rate: 100.0,
            burst_count: 0,
            burst_interval: 1.0,
            initial_velocity: Vec3::new(0.0, 1.0, 0.0),
            velocity_randomness: 0.2,
            inherit_velocity: 0.0,
            speed_range: 1.0..5.0,
        }
    }
}

/// Particle appearance configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Starting color
    pub start_color: Vec4,
    /// Ending color (interpolated over lifetime)
    pub end_color: Vec4,
    /// Starting size
    pub start_size: f32,
    /// Ending size
    pub end_size: f32,
    /// Rotation speed (radians per second)
    pub rotation_speed: f32,
    /// Initial rotation range
    pub initial_rotation_range: Range<f32>,
    /// Texture ID (optional)
    pub texture_id: Option<u32>,
    /// Blend mode
    pub blend_mode: BlendMode,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            start_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            end_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
            start_size: 0.1,
            end_size: 0.05,
            rotation_speed: 0.0,
            initial_rotation_range: 0.0..std::f32::consts::TAU,
            texture_id: None,
            blend_mode: BlendMode::Alpha,
        }
    }
}

/// Blend mode for particle rendering
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    /// Standard alpha blending
    #[default]
    Alpha,
    /// Additive blending (good for fire, glow)
    Additive,
    /// Multiplicative blending
    Multiply,
    /// Pre-multiplied alpha
    PremultipliedAlpha,
}

/// Physics configuration for particles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhysicsConfig {
    /// Gravity vector
    pub gravity: Vec3,
    /// Wind force
    pub wind: Vec3,
    /// Air drag (0-1)
    pub drag: f32,
    /// Enable floor collision
    pub collision_enabled: bool,
    /// Bounce factor (0 = no bounce, 1 = perfect bounce)
    pub collision_bounce: f32,
    /// Friction on collision (0-1)
    pub collision_friction: f32,
    /// Floor Y level
    pub floor_y: f32,
    /// Noise turbulence strength
    pub turbulence: f32,
    /// Turbulence frequency
    pub turbulence_frequency: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.8, 0.0),
            wind: Vec3::ZERO,
            drag: 0.1,
            collision_enabled: false,
            collision_bounce: 0.5,
            collision_friction: 0.3,
            floor_y: 0.0,
            turbulence: 0.0,
            turbulence_frequency: 1.0,
        }
    }
}

/// Complete emitter configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmitterConfig {
    /// Emitter name
    pub name: String,
    /// Maximum particles this emitter can have alive
    pub max_particles: u32,
    /// Particle lifetime range
    pub lifetime: Range<f32>,
    /// Emission shape
    pub shape: EmitterShape,
    /// Spawn settings
    pub spawn: SpawnConfig,
    /// Appearance settings
    pub appearance: AppearanceConfig,
    /// Physics settings
    pub physics: PhysicsConfig,
    /// Emit in local space (particles move with emitter)
    pub local_space: bool,
    /// Pre-warm (simulate on spawn to fill)
    pub prewarm: bool,
    /// Duration (0 = infinite)
    pub duration: f32,
    /// Looping
    pub looping: bool,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            name: "Particle Emitter".to_string(),
            max_particles: 1000,
            lifetime: 1.0..2.0,
            shape: EmitterShape::Point,
            spawn: SpawnConfig::default(),
            appearance: AppearanceConfig::default(),
            physics: PhysicsConfig::default(),
            local_space: false,
            prewarm: false,
            duration: 0.0,
            looping: true,
        }
    }
}

impl EmitterConfig {
    /// Create a new emitter config with a name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set max particles
    pub fn with_max_particles(mut self, count: u32) -> Self {
        self.max_particles = count;
        self
    }

    /// Set lifetime range
    pub fn with_lifetime(mut self, min: f32, max: f32) -> Self {
        self.lifetime = min..max;
        self
    }

    /// Set emission shape
    pub fn with_shape(mut self, shape: EmitterShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set spawn rate
    pub fn with_spawn_rate(mut self, rate: f32) -> Self {
        self.spawn.rate = rate;
        self
    }

    /// Set gravity
    pub fn with_gravity(mut self, gravity: Vec3) -> Self {
        self.physics.gravity = gravity;
        self
    }

    /// Set colors
    pub fn with_colors(mut self, start: Vec4, end: Vec4) -> Self {
        self.appearance.start_color = start;
        self.appearance.end_color = end;
        self
    }

    /// Set sizes
    pub fn with_sizes(mut self, start: f32, end: f32) -> Self {
        self.appearance.start_size = start;
        self.appearance.end_size = end;
        self
    }

    /// Enable floor collision
    pub fn with_floor_collision(mut self, floor_y: f32, bounce: f32) -> Self {
        self.physics.collision_enabled = true;
        self.physics.floor_y = floor_y;
        self.physics.collision_bounce = bounce;
        self
    }

    /// Set local space
    pub fn with_local_space(mut self, local: bool) -> Self {
        self.local_space = local;
        self
    }
}

/// Preset emitter configurations
pub mod presets {
    use super::*;

    /// Fire/flame effect
    pub fn fire() -> EmitterConfig {
        EmitterConfig::new("Fire")
            .with_max_particles(5000)
            .with_lifetime(0.5, 1.5)
            .with_shape(EmitterShape::Cone {
                angle: 15.0,
                radius: 0.3,
            })
            .with_spawn_rate(500.0)
            .with_gravity(Vec3::new(0.0, 3.0, 0.0))
            .with_colors(Vec4::new(1.0, 0.5, 0.0, 1.0), Vec4::new(1.0, 0.0, 0.0, 0.0))
            .with_sizes(0.2, 0.05)
    }

    /// Smoke effect
    pub fn smoke() -> EmitterConfig {
        EmitterConfig::new("Smoke")
            .with_max_particles(2000)
            .with_lifetime(2.0, 4.0)
            .with_shape(EmitterShape::Disk { radius: 0.5 })
            .with_spawn_rate(100.0)
            .with_gravity(Vec3::new(0.0, 0.5, 0.0))
            .with_colors(Vec4::new(0.3, 0.3, 0.3, 0.5), Vec4::new(0.5, 0.5, 0.5, 0.0))
            .with_sizes(0.1, 0.5)
    }

    /// Sparkles/magic effect
    pub fn sparkles() -> EmitterConfig {
        EmitterConfig::new("Sparkles")
            .with_max_particles(1000)
            .with_lifetime(0.5, 1.0)
            .with_shape(EmitterShape::Sphere { radius: 0.5 })
            .with_spawn_rate(200.0)
            .with_gravity(Vec3::new(0.0, -2.0, 0.0))
            .with_colors(Vec4::new(1.0, 1.0, 0.5, 1.0), Vec4::new(1.0, 0.8, 0.2, 0.0))
            .with_sizes(0.05, 0.02)
    }

    /// Rain effect
    pub fn rain() -> EmitterConfig {
        EmitterConfig::new("Rain")
            .with_max_particles(10000)
            .with_lifetime(1.0, 2.0)
            .with_shape(EmitterShape::Box {
                half_extents: Vec3::new(10.0, 0.1, 10.0),
            })
            .with_spawn_rate(1000.0)
            .with_gravity(Vec3::new(0.0, -15.0, 0.0))
            .with_colors(Vec4::new(0.7, 0.8, 1.0, 0.5), Vec4::new(0.7, 0.8, 1.0, 0.3))
            .with_sizes(0.02, 0.02)
    }

    /// Explosion effect (burst)
    pub fn explosion() -> EmitterConfig {
        let mut config = EmitterConfig::new("Explosion")
            .with_max_particles(500)
            .with_lifetime(0.5, 1.0)
            .with_shape(EmitterShape::Sphere { radius: 0.1 })
            .with_gravity(Vec3::new(0.0, -5.0, 0.0))
            .with_colors(Vec4::new(1.0, 0.8, 0.2, 1.0), Vec4::new(0.5, 0.1, 0.0, 0.0))
            .with_sizes(0.3, 0.1)
            .with_floor_collision(0.0, 0.3);

        config.spawn.rate = 0.0;
        config.spawn.burst_count = 500;
        config.looping = false;
        config.duration = 0.1;
        config
    }
}

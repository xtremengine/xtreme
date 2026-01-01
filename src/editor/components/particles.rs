//! Particle emitter component for the editor.

use serde::{Deserialize, Serialize};

/// Preset particle effects
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticlePreset {
    /// Custom settings (no preset)
    #[default]
    Custom,
    /// Fire/flame effect
    Fire,
    /// Smoke effect
    Smoke,
    /// Magical sparkles
    Sparkles,
    /// Rain effect
    Rain,
    /// Explosion burst
    Explosion,
}

impl ParticlePreset {
    /// Get display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            Self::Custom => "Custom",
            Self::Fire => "Fire",
            Self::Smoke => "Smoke",
            Self::Sparkles => "Sparkles",
            Self::Rain => "Rain",
            Self::Explosion => "Explosion",
        }
    }

    /// Get all presets for iteration
    pub fn all() -> &'static [ParticlePreset] {
        &[
            ParticlePreset::Custom,
            ParticlePreset::Fire,
            ParticlePreset::Smoke,
            ParticlePreset::Sparkles,
            ParticlePreset::Rain,
            ParticlePreset::Explosion,
        ]
    }
}

/// Particle emitter component
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParticleEmitterComponent {
    /// Preset to use (Custom for manual settings)
    pub preset: ParticlePreset,
    /// Maximum number of particles
    pub max_particles: u32,
    /// Particles spawned per second
    pub spawn_rate: f32,
    /// Minimum particle lifetime in seconds
    pub lifetime_min: f32,
    /// Maximum particle lifetime in seconds
    pub lifetime_max: f32,
    /// Start color (RGBA)
    pub start_color: [f32; 4],
    /// End color (RGBA)
    pub end_color: [f32; 4],
    /// Start size
    pub start_size: f32,
    /// End size
    pub end_size: f32,
    /// Gravity vector
    pub gravity: [f32; 3],
    /// Whether the emitter is enabled
    pub enabled: bool,
    /// Whether particles are simulated in local space (follow emitter)
    #[serde(default = "default_local_space")]
    pub local_space: bool,
}

fn default_local_space() -> bool {
    true
}

impl Default for ParticleEmitterComponent {
    fn default() -> Self {
        Self {
            preset: ParticlePreset::Custom,
            max_particles: 1000,
            spawn_rate: 100.0,
            lifetime_min: 1.0,
            lifetime_max: 2.0,
            start_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            start_size: 0.1,
            end_size: 0.05,
            gravity: [0.0, -9.8, 0.0],
            enabled: true,
            local_space: true,
        }
    }
}

impl ParticleEmitterComponent {
    /// Create with a preset
    pub fn with_preset(preset: ParticlePreset) -> Self {
        match preset {
            ParticlePreset::Custom => Self::default(),
            ParticlePreset::Fire => Self {
                preset,
                max_particles: 5000,
                spawn_rate: 500.0,
                lifetime_min: 0.5,
                lifetime_max: 1.5,
                start_color: [1.0, 0.5, 0.0, 1.0],
                end_color: [1.0, 0.0, 0.0, 0.0],
                start_size: 0.2,
                end_size: 0.05,
                gravity: [0.0, 3.0, 0.0],
                enabled: true,
                local_space: true,
            },
            ParticlePreset::Smoke => Self {
                preset,
                max_particles: 2000,
                spawn_rate: 100.0,
                lifetime_min: 2.0,
                lifetime_max: 4.0,
                start_color: [0.3, 0.3, 0.3, 0.5],
                end_color: [0.5, 0.5, 0.5, 0.0],
                start_size: 0.1,
                end_size: 0.5,
                gravity: [0.0, 0.5, 0.0],
                enabled: true,
                local_space: true,
            },
            ParticlePreset::Sparkles => Self {
                preset,
                max_particles: 1000,
                spawn_rate: 200.0,
                lifetime_min: 0.5,
                lifetime_max: 1.0,
                start_color: [1.0, 1.0, 0.5, 1.0],
                end_color: [1.0, 0.8, 0.2, 0.0],
                start_size: 0.05,
                end_size: 0.02,
                gravity: [0.0, -2.0, 0.0],
                enabled: true,
                local_space: true,
            },
            ParticlePreset::Rain => Self {
                preset,
                max_particles: 10000,
                spawn_rate: 1000.0,
                lifetime_min: 1.0,
                lifetime_max: 2.0,
                start_color: [0.7, 0.8, 1.0, 0.5],
                end_color: [0.7, 0.8, 1.0, 0.3],
                start_size: 0.02,
                end_size: 0.02,
                gravity: [0.0, -15.0, 0.0],
                enabled: true,
                local_space: false, // Rain should be world space
            },
            ParticlePreset::Explosion => Self {
                preset,
                max_particles: 500,
                spawn_rate: 0.0, // Burst only
                lifetime_min: 0.5,
                lifetime_max: 1.0,
                start_color: [1.0, 0.8, 0.2, 1.0],
                end_color: [0.5, 0.1, 0.0, 0.0],
                start_size: 0.3,
                end_size: 0.1,
                gravity: [0.0, -5.0, 0.0],
                enabled: true,
                local_space: false, // Explosion should be world space
            },
        }
    }
}

//! # Audio Module
//!
//! Spatial audio system with 3D positional sound using Rodio.
//!
//! ## Features
//!
//! - 3D positional audio with distance attenuation
//! - Stereo panning based on listener orientation
//! - Volume and pitch controls
//! - Looping and one-shot playback
//! - Multiple attenuation models (linear, inverse, exponential)
//! - ECS integration with AudioSource and AudioListener components
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use xtreme::audio::{AudioManager, AudioSource, AudioListener};
//! use xtreme::core::World;
//! use xtreme::math::Transform;
//!
//! // Initialize audio manager
//! let mut audio = AudioManager::new().unwrap();
//! let clip = audio.load_clip("assets/sounds/explosion.ogg").unwrap();
//!
//! // Create listener (typically on camera entity)
//! let camera = world.spawn()
//!     .with(Transform::default())
//!     .with(AudioListener::default())
//!     .build();
//!
//! // Create audio source
//! let enemy = world.spawn()
//!     .with(Transform::from_xyz(10.0, 0.0, 5.0))
//!     .with(AudioSource::new(clip).with_spatial(true))
//!     .build();
//!
//! // Trigger playback
//! if let Some(source) = world.get_mut::<AudioSource>(enemy) {
//!     source.play();
//! }
//!
//! // In game loop:
//! audio_system(&mut world, &mut audio);
//! ```
//!
//! ## Spatial Audio
//!
//! Spatial audio automatically adjusts volume based on distance between
//! the sound source and the listener. Several attenuation models are available:
//!
//! - **Linear**: Simple linear falloff
//! - **InverseDistance**: More realistic 1/d falloff
//! - **InverseDistanceSquared**: Even sharper falloff
//! - **Exponential**: Exponential decay
//!
//! ## Components
//!
//! - `AudioListener` - Marks the entity that "hears" sounds
//! - `AudioSource` - Emits sound from an entity's position

mod components;
mod manager;
mod spatial;
mod system;

pub use components::{AudioClipId, AudioCommand, AudioListener, AudioSource, PlaybackState};
pub use manager::{AudioClip, AudioError, AudioManager};
pub use spatial::{
    apply_pan, calculate_attenuation, calculate_pan, calculate_spatial, distance_3d,
    AttenuationModel, SpatialParams,
};
pub use system::{audio_system, play_sound_at};

//! Audio ECS System
//!
//! System for processing audio components and updating spatial audio.

use glam::Vec3;

use crate::core::{Entity, World};
use crate::math::Transform;

use super::components::{AudioCommand, AudioListener, AudioSource, PlaybackState};
use super::manager::AudioManager;
use super::spatial::{calculate_spatial, AttenuationModel};

/// Process audio sources and update spatial audio
///
/// This system should run in Stage::PostUpdate after entity movement
/// so that positions are up to date for spatial calculations.
pub fn audio_system(world: &mut World, audio_manager: &mut AudioManager) {
    // Get listener data
    let listener_data = find_listener(world);

    // Process all audio sources
    process_audio_sources(world, audio_manager, listener_data);

    // Clean up finished sounds
    let finished = audio_manager.cleanup_finished();
    mark_finished_sources(world, &finished);
}

/// Listener data: (position, forward, up, master_volume)
type ListenerData = (Vec3, Vec3, Vec3, f32);

/// Find the audio listener and get its position/orientation
fn find_listener(world: &World) -> Option<ListenerData> {
    // Query first AudioListener component
    if let Some(storage) = world.storage::<AudioListener>() {
        if let Some((entity_idx, listener)) = storage.iter().next() {
            // Get the entity from index (use from_bits with just index in low bits)
            let entity = Entity::from_bits(entity_idx as u64);

            // Try to get position from Transform
            let pos = world
                .get::<Transform>(entity)
                .map(|t| t.position.to_vec3())
                .unwrap_or(Vec3::ZERO);

            return Some((pos, listener.forward, listener.up, listener.master_volume));
        }
    }
    None
}

/// Source update data collected from ECS
struct SourceUpdate {
    entity_idx: u32,
    clip: Option<u32>,
    volume: f32,
    pitch: f32,
    looping: bool,
    spatial: bool,
    min_distance: f32,
    max_distance: f32,
    rolloff: f32,
    #[allow(dead_code)]
    sink_id: Option<u64>,
    pending_command: Option<AudioCommand>,
    position: Vec3,
}

/// Process all audio sources - handle commands and update spatial audio
fn process_audio_sources(
    world: &mut World,
    audio_manager: &mut AudioManager,
    listener_data: Option<ListenerData>,
) {
    let (listener_pos, listener_forward, listener_up, master_vol) =
        listener_data.unwrap_or((Vec3::ZERO, Vec3::NEG_Z, Vec3::Y, 1.0));

    // Collect source data first (to avoid borrow issues)
    let mut source_updates: Vec<SourceUpdate> = Vec::new();

    if let Some(storage) = world.storage::<AudioSource>() {
        for (entity_idx, source) in storage.iter() {
            let entity = Entity::from_bits(entity_idx as u64);

            // Get source position
            let source_pos = world
                .get::<Transform>(entity)
                .map(|t| t.position.to_vec3())
                .unwrap_or(Vec3::ZERO);

            source_updates.push(SourceUpdate {
                entity_idx,
                clip: source.clip,
                volume: source.volume,
                pitch: source.pitch,
                looping: source.looping,
                spatial: source.spatial,
                min_distance: source.min_distance,
                max_distance: source.max_distance,
                rolloff: source.rolloff,
                sink_id: source.sink_id,
                pending_command: source.pending_command,
                position: source_pos,
            });
        }
    }

    // Process updates
    for update in source_updates {
        let entity = Entity::from_bits(update.entity_idx as u64);

        // Handle pending commands
        if let Some(cmd) = update.pending_command {
            if let Some(source) = world.get_mut::<AudioSource>(entity) {
                match cmd {
                    AudioCommand::Play => {
                        if let Some(clip_id) = update.clip {
                            // Stop existing playback if any
                            if let Some(old_sink) = source.sink_id {
                                audio_manager.stop(old_sink);
                            }

                            // Start new playback
                            let volume = if update.spatial {
                                // Calculate initial spatial volume
                                let spatial = calculate_spatial(
                                    listener_pos,
                                    listener_forward,
                                    listener_up,
                                    update.position,
                                    update.min_distance,
                                    update.max_distance,
                                    update.rolloff,
                                    AttenuationModel::InverseDistance,
                                );
                                update.volume * spatial.volume * master_vol
                            } else {
                                update.volume * master_vol
                            };

                            match audio_manager.play(
                                clip_id,
                                update.entity_idx,
                                volume,
                                update.pitch,
                                update.looping,
                                update.spatial,
                            ) {
                                Ok(sink_id) => {
                                    source.sink_id = Some(sink_id);
                                    source.state = PlaybackState::Playing;
                                }
                                Err(e) => {
                                    log::error!("Failed to play audio: {}", e);
                                }
                            }
                        }
                    }
                    AudioCommand::Pause => {
                        if let Some(sink_id) = source.sink_id {
                            audio_manager.pause(sink_id);
                            source.state = PlaybackState::Paused;
                        }
                    }
                    AudioCommand::Resume => {
                        if let Some(sink_id) = source.sink_id {
                            audio_manager.resume(sink_id);
                            source.state = PlaybackState::Playing;
                        }
                    }
                    AudioCommand::Stop => {
                        if let Some(sink_id) = source.sink_id {
                            audio_manager.stop(sink_id);
                            source.sink_id = None;
                            source.state = PlaybackState::Stopped;
                        }
                    }
                }
                source.pending_command = None;
            }
        }

        // Update spatial audio for playing sources
        if let Some(source) = world.get::<AudioSource>(entity) {
            if source.state == PlaybackState::Playing && source.spatial {
                if let Some(sink_id) = source.sink_id {
                    let spatial = calculate_spatial(
                        listener_pos,
                        listener_forward,
                        listener_up,
                        update.position,
                        update.min_distance,
                        update.max_distance,
                        update.rolloff,
                        AttenuationModel::InverseDistance,
                    );

                    let final_volume = update.volume * spatial.volume * master_vol;
                    audio_manager.set_sink_volume(sink_id, final_volume);
                }
            }
        }
    }
}

/// Mark sources whose playback has finished
fn mark_finished_sources(world: &mut World, finished_sinks: &[u64]) {
    if finished_sinks.is_empty() {
        return;
    }

    if let Some(storage) = world.storage::<AudioSource>() {
        let entities_to_update: Vec<(u32, u64)> = storage
            .iter()
            .filter_map(|(idx, source)| {
                source
                    .sink_id
                    .filter(|id| finished_sinks.contains(id))
                    .map(|id| (idx, id))
            })
            .collect();

        for (entity_idx, _) in entities_to_update {
            let entity = Entity::from_bits(entity_idx as u64);
            if let Some(source) = world.get_mut::<AudioSource>(entity) {
                source.sink_id = None;
                source.state = PlaybackState::Stopped;
            }
        }
    }
}

/// Convenience function to play a one-shot sound at a position
pub fn play_sound_at(
    audio_manager: &mut AudioManager,
    clip_id: u32,
    position: Vec3,
    volume: f32,
    listener_pos: Vec3,
) -> Result<u64, super::manager::AudioError> {
    // Calculate spatial volume
    let distance = (position - listener_pos).length();
    let spatial_volume = if distance < 1.0 {
        volume
    } else if distance > 100.0 {
        0.0
    } else {
        volume / (1.0 + distance * 0.1)
    };

    audio_manager.play(clip_id, 0, spatial_volume, 1.0, false, true)
}

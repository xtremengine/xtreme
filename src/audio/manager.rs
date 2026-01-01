//! Audio Manager
//!
//! Manages audio playback using Rodio. Handles clip loading,
//! sink management, and playback control.

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;

use super::components::AudioClipId;

/// Audio system errors
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to initialize audio output: {0}")]
    InitError(String),
    #[error("Failed to load audio file: {0}")]
    LoadError(String),
    #[error("Audio clip not found: {0}")]
    ClipNotFound(AudioClipId),
    #[error("Playback error: {0}")]
    PlaybackError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Loaded audio clip data
pub struct AudioClip {
    /// Raw audio bytes
    pub data: Arc<Vec<u8>>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Original file path
    pub path: String,
}

/// Active playback sink with metadata
struct ActiveSink {
    sink: Sink,
    entity_id: u32,
    is_spatial: bool,
    base_volume: f32,
}

/// Central audio manager
///
/// Holds the Rodio output stream and manages all audio resources.
pub struct AudioManager {
    /// Rodio output stream (must be kept alive)
    _stream: OutputStream,
    /// Stream handle for creating sinks
    stream_handle: OutputStreamHandle,
    /// Loaded audio clips by ID
    clips: HashMap<AudioClipId, AudioClip>,
    /// Active playback sinks by ID
    sinks: HashMap<u64, ActiveSink>,
    /// Next clip ID to assign
    next_clip_id: AudioClipId,
    /// Next sink ID to assign
    next_sink_id: u64,
    /// Master volume (0.0 - 1.0)
    pub master_volume: f32,
}

impl AudioManager {
    /// Initialize the audio system
    ///
    /// Returns an error if audio output cannot be initialized.
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) =
            OutputStream::try_default().map_err(|e| AudioError::InitError(e.to_string()))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            clips: HashMap::new(),
            sinks: HashMap::new(),
            next_clip_id: 1,
            next_sink_id: 1,
            master_volume: 1.0,
        })
    }

    /// Load an audio file from disk
    ///
    /// Supports formats: WAV, OGG, MP3, FLAC (via symphonia)
    pub fn load_clip(&mut self, path: impl AsRef<Path>) -> Result<AudioClipId, AudioError> {
        let path = path.as_ref();

        // Read file into memory
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut reader, &mut data)?;

        // Decode to get metadata
        let cursor = std::io::Cursor::new(data.clone());
        let source = Decoder::new(cursor).map_err(|e| AudioError::LoadError(e.to_string()))?;

        let sample_rate = source.sample_rate();
        let channels = source.channels();

        let id = self.next_clip_id;
        self.next_clip_id += 1;

        self.clips.insert(
            id,
            AudioClip {
                data: Arc::new(data),
                sample_rate,
                channels,
                path: path.to_string_lossy().to_string(),
            },
        );

        log::debug!("Loaded audio clip {} from {:?}", id, path);
        Ok(id)
    }

    /// Load audio from embedded bytes
    pub fn load_clip_bytes(&mut self, bytes: &[u8], name: &str) -> Result<AudioClipId, AudioError> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let source = Decoder::new(cursor).map_err(|e| AudioError::LoadError(e.to_string()))?;

        let sample_rate = source.sample_rate();
        let channels = source.channels();

        let id = self.next_clip_id;
        self.next_clip_id += 1;

        self.clips.insert(
            id,
            AudioClip {
                data: Arc::new(bytes.to_vec()),
                sample_rate,
                channels,
                path: name.to_string(),
            },
        );

        Ok(id)
    }

    /// Play a sound and return a sink ID for control
    pub fn play(
        &mut self,
        clip_id: AudioClipId,
        entity_id: u32,
        volume: f32,
        pitch: f32,
        looping: bool,
        spatial: bool,
    ) -> Result<u64, AudioError> {
        let clip = self
            .clips
            .get(&clip_id)
            .ok_or(AudioError::ClipNotFound(clip_id))?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackError(e.to_string()))?;

        // Create source from cached bytes
        let cursor = std::io::Cursor::new(clip.data.as_ref().clone());
        let source = Decoder::new(cursor).map_err(|e| AudioError::PlaybackError(e.to_string()))?;

        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        let final_volume = volume * self.master_volume;
        sink.set_volume(final_volume);
        sink.set_speed(pitch);

        let sink_id = self.next_sink_id;
        self.next_sink_id += 1;

        self.sinks.insert(
            sink_id,
            ActiveSink {
                sink,
                entity_id,
                is_spatial: spatial,
                base_volume: volume,
            },
        );

        log::trace!("Playing audio clip {} on sink {}", clip_id, sink_id);
        Ok(sink_id)
    }

    /// Update volume for a playing sound (used for spatial audio)
    pub fn set_sink_volume(&mut self, sink_id: u64, volume: f32) {
        if let Some(active) = self.sinks.get_mut(&sink_id) {
            let final_volume = volume * self.master_volume;
            active.sink.set_volume(final_volume);
        }
    }

    /// Update base volume for a sink
    pub fn set_base_volume(&mut self, sink_id: u64, volume: f32) {
        if let Some(active) = self.sinks.get_mut(&sink_id) {
            active.base_volume = volume;
            let final_volume = volume * self.master_volume;
            active.sink.set_volume(final_volume);
        }
    }

    /// Update pitch for a playing sound
    pub fn set_sink_pitch(&mut self, sink_id: u64, pitch: f32) {
        if let Some(active) = self.sinks.get(&sink_id) {
            active.sink.set_speed(pitch);
        }
    }

    /// Pause a sound
    pub fn pause(&mut self, sink_id: u64) {
        if let Some(active) = self.sinks.get(&sink_id) {
            active.sink.pause();
        }
    }

    /// Resume a paused sound
    pub fn resume(&mut self, sink_id: u64) {
        if let Some(active) = self.sinks.get(&sink_id) {
            active.sink.play();
        }
    }

    /// Stop a sound and remove it
    pub fn stop(&mut self, sink_id: u64) {
        if let Some(active) = self.sinks.remove(&sink_id) {
            active.sink.stop();
        }
    }

    /// Check if a sink is still playing
    pub fn is_playing(&self, sink_id: u64) -> bool {
        self.sinks
            .get(&sink_id)
            .map(|s| !s.sink.empty())
            .unwrap_or(false)
    }

    /// Check if a sink is paused
    pub fn is_paused(&self, sink_id: u64) -> bool {
        self.sinks
            .get(&sink_id)
            .map(|s| s.sink.is_paused())
            .unwrap_or(false)
    }

    /// Clean up finished sinks
    ///
    /// Returns the IDs of sinks that finished playing.
    pub fn cleanup_finished(&mut self) -> Vec<u64> {
        let finished: Vec<u64> = self
            .sinks
            .iter()
            .filter(|(_, s)| s.sink.empty())
            .map(|(id, _)| *id)
            .collect();

        for id in &finished {
            self.sinks.remove(id);
        }

        finished
    }

    /// Get a loaded clip by ID
    pub fn get_clip(&self, id: AudioClipId) -> Option<&AudioClip> {
        self.clips.get(&id)
    }

    /// Unload a clip
    pub fn unload_clip(&mut self, id: AudioClipId) -> bool {
        self.clips.remove(&id).is_some()
    }

    /// Number of loaded clips
    pub fn clip_count(&self) -> usize {
        self.clips.len()
    }

    /// Number of active sounds
    pub fn active_sound_count(&self) -> usize {
        self.sinks.len()
    }

    /// Stop all sounds
    pub fn stop_all(&mut self) {
        for (_, active) in self.sinks.drain() {
            active.sink.stop();
        }
    }

    /// Set master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);

        // Update all active sinks
        for active in self.sinks.values() {
            let final_volume = active.base_volume * self.master_volume;
            active.sink.set_volume(final_volume);
        }
    }

    /// Get entity ID for a sink
    pub fn get_sink_entity(&self, sink_id: u64) -> Option<u32> {
        self.sinks.get(&sink_id).map(|s| s.entity_id)
    }

    /// Check if a sink is spatial
    pub fn is_sink_spatial(&self, sink_id: u64) -> bool {
        self.sinks
            .get(&sink_id)
            .map(|s| s.is_spatial)
            .unwrap_or(false)
    }
}

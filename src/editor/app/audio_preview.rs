//! Audio preview functionality for the editor.

use super::EditorApp;

impl EditorApp {
    /// Play audio preview from a file path
    pub fn play_audio_preview(&mut self, path: &str) {
        // Stop any existing preview first
        self.stop_audio_preview();

        let Some(audio_manager) = &mut self.audio_manager else {
            log::warn!("Audio manager not available");
            return;
        };

        // Load the clip if not already loaded
        match audio_manager.load_clip(path) {
            Ok(clip_id) => {
                // Play the clip
                match audio_manager.play(
                    clip_id, 0,     // entity_id (not used for preview)
                    1.0,   // volume
                    1.0,   // pitch
                    false, // looping
                    false, // spatial
                ) {
                    Ok(sink_id) => {
                        self.audio_preview_sink = Some(sink_id);
                        log::info!("Playing audio preview: {}", path);
                    }
                    Err(e) => {
                        log::error!("Failed to play audio: {}", e);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to load audio clip {}: {}", path, e);
            }
        }
    }

    /// Stop current audio preview
    pub fn stop_audio_preview(&mut self) {
        if let Some(sink_id) = self.audio_preview_sink.take() {
            if let Some(audio_manager) = &mut self.audio_manager {
                audio_manager.stop(sink_id);
                log::info!("Stopped audio preview");
            }
        }
    }
}

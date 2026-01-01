//! Thread-safe log capture for the console panel.

use log::{Level, Log, Metadata, Record};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// A single log entry
#[derive(Clone, Debug)]
pub struct LogEntry {
    /// Log level
    pub level: Level,
    /// Message content
    pub message: String,
    /// Module/target that emitted the log
    pub target: String,
    /// Category for custom filtering
    pub category: LogCategory,
    /// Timestamp (seconds since logger init)
    pub timestamp: f32,
}

/// Log categories for filtering
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogCategory {
    Engine,
    Editor,
    Render,
    Audio,
    Script,
    User,
    Other,
}

impl LogCategory {
    /// Derive category from log target
    pub fn from_target(target: &str) -> Self {
        if target.starts_with("xtreme::editor") {
            LogCategory::Editor
        } else if target.starts_with("xtreme::render") || target.starts_with("wgpu") {
            LogCategory::Render
        } else if target.starts_with("xtreme::audio") || target.starts_with("rodio") {
            LogCategory::Audio
        } else if target.starts_with("xtreme::scripting") {
            LogCategory::Script
        } else if target.starts_with("xtreme") {
            LogCategory::Engine
        } else {
            LogCategory::Other
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            LogCategory::Engine => "Engine",
            LogCategory::Editor => "Editor",
            LogCategory::Render => "Render",
            LogCategory::Audio => "Audio",
            LogCategory::Script => "Script",
            LogCategory::User => "User",
            LogCategory::Other => "Other",
        }
    }

    pub fn all() -> &'static [LogCategory] {
        &[
            LogCategory::Engine,
            LogCategory::Editor,
            LogCategory::Render,
            LogCategory::Audio,
            LogCategory::Script,
            LogCategory::User,
            LogCategory::Other,
        ]
    }
}

/// Thread-safe log capture that sends to a channel
struct LogCapture {
    sender: Sender<LogEntry>,
    start_time: Instant,
    min_level: Level,
}

impl Log for LogCapture {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let entry = LogEntry {
            level: record.level(),
            message: record.args().to_string(),
            target: record.target().to_string(),
            category: LogCategory::from_target(record.target()),
            timestamp: self.start_time.elapsed().as_secs_f32(),
        };

        // Non-blocking send (ignore if receiver dropped)
        let _ = self.sender.send(entry);
    }

    fn flush(&self) {}
}

/// Global receiver storage
static LOG_RECEIVER: OnceLock<Mutex<Receiver<LogEntry>>> = OnceLock::new();

/// Initialize the log capture system.
/// Returns the receiver for polling logs.
/// MUST be called instead of env_logger::init().
pub fn init_log_capture(max_level: Level) -> &'static Mutex<Receiver<LogEntry>> {
    LOG_RECEIVER.get_or_init(|| {
        let (sender, receiver) = channel();

        let capture = LogCapture {
            sender,
            start_time: Instant::now(),
            min_level: max_level,
        };

        // Set as the global logger
        log::set_boxed_logger(Box::new(capture)).expect("Failed to set logger");
        log::set_max_level(max_level.to_level_filter());

        Mutex::new(receiver)
    })
}

/// Poll all pending log entries (non-blocking)
pub fn poll_logs(receiver: &Mutex<Receiver<LogEntry>>) -> Vec<LogEntry> {
    let Ok(guard) = receiver.lock() else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    while let Ok(entry) = guard.try_recv() {
        entries.push(entry);
    }
    entries
}

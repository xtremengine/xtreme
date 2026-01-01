//! # Console Panel
//!
//! Debug log panel with filtering and search.

mod log_capture;

pub use log_capture::{init_log_capture, poll_logs, LogEntry};

use egui::{Color32, RichText, ScrollArea, Ui};
use log::Level;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

/// Console panel settings (persisted with project)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConsoleSettings {
    /// Show debug level logs
    pub show_debug: bool,
    /// Show info level logs
    pub show_info: bool,
    /// Show warn level logs
    pub show_warn: bool,
    /// Show error level logs
    pub show_error: bool,
    /// Auto-scroll to bottom on new logs
    pub auto_scroll: bool,
    /// Show timestamps
    pub show_timestamps: bool,
    /// Maximum log entries to keep
    pub max_entries: usize,
}

impl Default for ConsoleSettings {
    fn default() -> Self {
        Self {
            show_debug: false,
            show_info: true,
            show_warn: true,
            show_error: true,
            auto_scroll: true,
            show_timestamps: true,
            max_entries: 1000,
        }
    }
}

impl ConsoleSettings {
    /// Check if a level should be shown
    pub fn show_level(&self, level: Level) -> bool {
        match level {
            Level::Error => self.show_error,
            Level::Warn => self.show_warn,
            Level::Info => self.show_info,
            Level::Debug | Level::Trace => self.show_debug,
        }
    }
}

/// Actions returned by the console panel
#[derive(Clone, Debug, PartialEq)]
pub enum ConsoleAction {
    /// No action
    None,
    /// Copy logs to clipboard
    CopyToClipboard(String),
}

/// Console panel for displaying logs
pub struct ConsolePanel {
    /// Log buffer (ring buffer)
    buffer: VecDeque<LogEntry>,
    /// Search/filter text
    filter: String,
    /// Panel settings
    settings: ConsoleSettings,
    /// Whether settings popup is open
    show_settings: bool,
    /// Receiver reference for polling
    receiver: Option<&'static Mutex<Receiver<LogEntry>>>,
    /// Counts by level
    error_count: usize,
    warn_count: usize,
    info_count: usize,
    debug_count: usize,
    /// Whether panel is visible
    pub visible: bool,
}

impl Default for ConsolePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolePanel {
    /// Create a new console panel
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(1000),
            filter: String::new(),
            settings: ConsoleSettings::default(),
            show_settings: false,
            receiver: None,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            debug_count: 0,
            visible: true,
        }
    }

    /// Initialize with log receiver
    pub fn with_receiver(mut self, receiver: &'static Mutex<Receiver<LogEntry>>) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Set receiver after construction
    pub fn set_receiver(&mut self, receiver: &'static Mutex<Receiver<LogEntry>>) {
        self.receiver = Some(receiver);
    }

    /// Set settings (from project config)
    pub fn set_settings(&mut self, settings: ConsoleSettings) {
        self.settings = settings;
    }

    /// Get current settings (for project save)
    pub fn settings(&self) -> &ConsoleSettings {
        &self.settings
    }

    /// Poll for new log entries
    pub fn poll(&mut self) {
        let Some(receiver) = self.receiver else {
            return;
        };

        for entry in poll_logs(receiver) {
            // Update counts
            match entry.level {
                Level::Error => self.error_count += 1,
                Level::Warn => self.warn_count += 1,
                Level::Info => self.info_count += 1,
                Level::Debug | Level::Trace => self.debug_count += 1,
            }

            // Add to buffer
            if self.buffer.len() >= self.settings.max_entries {
                self.buffer.pop_front();
            }
            self.buffer.push_back(entry);
        }
    }

    /// Clear all logs
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.error_count = 0;
        self.warn_count = 0;
        self.info_count = 0;
        self.debug_count = 0;
    }

    /// Draw the console panel
    pub fn show(&mut self, ui: &mut Ui) -> ConsoleAction {
        // Poll for new logs each frame
        self.poll();

        let mut action = ConsoleAction::None;

        // Toolbar
        ui.horizontal(|ui| {
            ui.strong("Console");

            ui.separator();

            // Clear button
            if ui.small_button("Clear").clicked() {
                self.clear();
            }

            ui.separator();

            // Level filters with counts
            let error_text = format!("E:{}", self.error_count);
            if ui
                .selectable_label(
                    self.settings.show_error,
                    RichText::new(&error_text).color(Color32::from_rgb(255, 100, 100)),
                )
                .on_hover_text("Toggle Errors")
                .clicked()
            {
                self.settings.show_error = !self.settings.show_error;
            }

            let warn_text = format!("W:{}", self.warn_count);
            if ui
                .selectable_label(
                    self.settings.show_warn,
                    RichText::new(&warn_text).color(Color32::from_rgb(255, 200, 100)),
                )
                .on_hover_text("Toggle Warnings")
                .clicked()
            {
                self.settings.show_warn = !self.settings.show_warn;
            }

            let info_text = format!("I:{}", self.info_count);
            if ui
                .selectable_label(
                    self.settings.show_info,
                    RichText::new(&info_text).color(Color32::from_rgb(100, 180, 255)),
                )
                .on_hover_text("Toggle Info")
                .clicked()
            {
                self.settings.show_info = !self.settings.show_info;
            }

            let debug_text = format!("D:{}", self.debug_count);
            if ui
                .selectable_label(
                    self.settings.show_debug,
                    RichText::new(&debug_text).color(Color32::GRAY),
                )
                .on_hover_text("Toggle Debug")
                .clicked()
            {
                self.settings.show_debug = !self.settings.show_debug;
            }

            ui.separator();

            // Search filter
            ui.label("Filter:");
            ui.add(egui::TextEdit::singleline(&mut self.filter).desired_width(120.0));
            if ui.small_button("x").clicked() {
                self.filter.clear();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Copy button
                if ui.small_button("Copy").clicked() {
                    let text = self.get_visible_logs_text();
                    action = ConsoleAction::CopyToClipboard(text);
                }

                // Settings button
                if ui.small_button("...").on_hover_text("Settings").clicked() {
                    self.show_settings = !self.show_settings;
                }

                // Auto-scroll toggle
                ui.checkbox(&mut self.settings.auto_scroll, "Auto-scroll");
            });
        });

        ui.separator();

        // Settings popup
        if self.show_settings {
            self.draw_settings_popup(ui);
        }

        // Log entries with scroll
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.settings.auto_scroll)
            .show(ui, |ui| {
                self.draw_log_entries(ui);
            });

        action
    }

    /// Draw log entries
    fn draw_log_entries(&self, ui: &mut Ui) {
        let filter_lower = self.filter.to_lowercase();

        for entry in &self.buffer {
            // Level filter
            if !self.settings.show_level(entry.level) {
                continue;
            }

            // Text filter
            if !filter_lower.is_empty() {
                let searchable = format!("{} {}", entry.target, entry.message).to_lowercase();
                if !searchable.contains(&filter_lower) {
                    continue;
                }
            }

            // Draw entry
            ui.horizontal(|ui| {
                // Timestamp
                if self.settings.show_timestamps {
                    ui.label(
                        RichText::new(format!("[{:>7.2}]", entry.timestamp))
                            .color(Color32::GRAY)
                            .monospace(),
                    );
                }

                // Level icon with color
                let (icon, color) = match entry.level {
                    Level::Error => ("E", Color32::from_rgb(255, 100, 100)),
                    Level::Warn => ("W", Color32::from_rgb(255, 200, 100)),
                    Level::Info => ("I", Color32::from_rgb(100, 180, 255)),
                    Level::Debug => ("D", Color32::GRAY),
                    Level::Trace => ("T", Color32::DARK_GRAY),
                };
                ui.label(RichText::new(icon).color(color).strong().monospace());

                // Message
                let msg_color = match entry.level {
                    Level::Error => Color32::from_rgb(255, 150, 150),
                    Level::Warn => Color32::from_rgb(255, 220, 150),
                    _ => Color32::WHITE,
                };
                ui.label(RichText::new(&entry.message).color(msg_color));
            });
        }
    }

    /// Draw settings popup
    fn draw_settings_popup(&mut self, ui: &mut Ui) {
        egui::Window::new("Console Settings")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.checkbox(&mut self.settings.auto_scroll, "Auto-scroll");
                ui.checkbox(&mut self.settings.show_timestamps, "Show timestamps");

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Max entries:");
                    ui.add(egui::DragValue::new(&mut self.settings.max_entries).range(100..=10000));
                });

                ui.separator();

                if ui.button("Close").clicked() {
                    self.show_settings = false;
                }
            });
    }

    /// Get visible logs as text for clipboard
    pub fn get_visible_logs_text(&self) -> String {
        let filter_lower = self.filter.to_lowercase();
        let mut output = String::new();

        for entry in &self.buffer {
            if !self.settings.show_level(entry.level) {
                continue;
            }
            if !filter_lower.is_empty() {
                let searchable = format!("{} {}", entry.target, entry.message).to_lowercase();
                if !searchable.contains(&filter_lower) {
                    continue;
                }
            }

            output.push_str(&format!(
                "[{:>7.2}] {:5} [{}] {}\n",
                entry.timestamp, entry.level, entry.target, entry.message
            ));
        }

        output
    }
}

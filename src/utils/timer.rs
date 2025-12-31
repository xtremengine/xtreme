//! # Timer Utilities
//!
//! Delta time tracking and fixed timestep utilities.

use std::time::{Duration, Instant};

/// Tracks delta time between frames
#[derive(Debug)]
pub struct DeltaTime {
    last_update: Instant,
    delta: Duration,
    total: Duration,
    frame_count: u64,
}

impl DeltaTime {
    pub fn new() -> Self {
        Self {
            last_update: Instant::now(),
            delta: Duration::ZERO,
            total: Duration::ZERO,
            frame_count: 0,
        }
    }

    /// Update and get delta time since last call
    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        self.delta = now - self.last_update;
        self.last_update = now;
        self.total += self.delta;
        self.frame_count += 1;
        self.delta.as_secs_f32()
    }

    /// Get last delta time in seconds
    pub fn delta_secs(&self) -> f32 {
        self.delta.as_secs_f32()
    }

    /// Get last delta time in milliseconds
    pub fn delta_ms(&self) -> f32 {
        self.delta.as_secs_f32() * 1000.0
    }

    /// Get total elapsed time
    pub fn total_secs(&self) -> f32 {
        self.total.as_secs_f32()
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get average FPS
    pub fn fps(&self) -> f32 {
        if self.total.is_zero() {
            return 0.0;
        }
        self.frame_count as f32 / self.total.as_secs_f32()
    }

    /// Get instantaneous FPS (based on last frame)
    pub fn instant_fps(&self) -> f32 {
        if self.delta.is_zero() {
            return 0.0;
        }
        1.0 / self.delta.as_secs_f32()
    }
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixed timestep accumulator for physics
#[derive(Debug)]
pub struct FixedTimestep {
    timestep: Duration,
    accumulator: Duration,
    max_steps: u32,
}

impl FixedTimestep {
    /// Create with specified timestep (e.g., 1/60 second)
    pub fn new(timestep: Duration) -> Self {
        Self {
            timestep,
            accumulator: Duration::ZERO,
            max_steps: 5,
        }
    }

    /// Create for target FPS
    pub fn from_fps(fps: u32) -> Self {
        Self::new(Duration::from_secs_f64(1.0 / fps as f64))
    }

    /// Set maximum steps per frame (prevents spiral of death)
    pub fn with_max_steps(mut self, max: u32) -> Self {
        self.max_steps = max;
        self
    }

    /// Accumulate time and return number of fixed steps to run
    pub fn accumulate(&mut self, delta: Duration) -> u32 {
        self.accumulator += delta;

        let mut steps = 0;
        while self.accumulator >= self.timestep && steps < self.max_steps {
            self.accumulator -= self.timestep;
            steps += 1;
        }

        // Prevent accumulator from growing too large
        if self.accumulator > self.timestep * 2 {
            self.accumulator = self.timestep;
        }

        steps
    }

    /// Get the fixed timestep in seconds
    pub fn timestep_secs(&self) -> f32 {
        self.timestep.as_secs_f32()
    }

    /// Get interpolation alpha for rendering between physics steps
    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.timestep.as_secs_f32()
    }
}

impl Default for FixedTimestep {
    fn default() -> Self {
        Self::from_fps(60)
    }
}

/// Simple countdown timer
#[derive(Debug)]
pub struct Timer {
    duration: Duration,
    elapsed: Duration,
    repeating: bool,
    finished: bool,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            repeating: false,
            finished: false,
        }
    }

    pub fn from_secs(secs: f32) -> Self {
        Self::new(Duration::from_secs_f32(secs))
    }

    pub fn repeating(mut self) -> Self {
        self.repeating = true;
        self
    }

    /// Update timer, returns true if just finished
    pub fn tick(&mut self, delta: Duration) -> bool {
        if self.finished && !self.repeating {
            return false;
        }

        self.elapsed += delta;

        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.finished = true;
            }
            return true;
        }

        false
    }

    /// Check if timer is finished (non-repeating only)
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Progress from 0.0 to 1.0
    pub fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    /// Remaining time
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed)
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.finished = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_timestep() {
        let mut fixed = FixedTimestep::from_fps(60);

        // One 60fps frame should give exactly 1 step
        let steps = fixed.accumulate(Duration::from_secs_f64(1.0 / 60.0));
        assert_eq!(steps, 1);
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::from_secs(1.0);

        assert!(!timer.tick(Duration::from_secs_f32(0.5)));
        assert!(timer.tick(Duration::from_secs_f32(0.6)));
        assert!(timer.is_finished());
    }

    #[test]
    fn test_repeating_timer() {
        let mut timer = Timer::from_secs(0.5).repeating();

        assert!(timer.tick(Duration::from_secs_f32(0.6)));
        assert!(!timer.is_finished()); // Repeating timers don't "finish"
        assert!(timer.tick(Duration::from_secs_f32(0.4)));
    }
}

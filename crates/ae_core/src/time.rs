// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use std::time::Instant;

/// Core Time Manager and Fixed Time Accumulator.
pub struct Time {
    last_frame_time: Instant,
    pub delta_time: f32,      // Time elapsed between frames (in seconds)
    pub total_time: f64,      // Total time elapsed since application launch
    pub total_time_us: u64,   // Total time (microseconds, high-precision tracking)
    pub frame_count: u64,     // Frame counter
    pub fixed_time_step: f32, // Fixed time step duration (e.g., 0.00833s for 120Hz)
    pub accumulator: f32,     // Accumulated time buffer for consuming fixed updates
}

impl Time {
    /// Creates a new Time manager with a default 120Hz fixed step rate.
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            total_time: 0.0,
            total_time_us: 0,
            frame_count: 0,
            fixed_time_step: 1.0 / 120.0,
            accumulator: 0.0,
        }
    }

    /// Updates the timer state, calculating delta and total time elapsed.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_frame_time);
        self.delta_time = duration.as_secs_f32();

        self.total_time_us = self
            .total_time_us
            .saturating_add(duration.as_micros() as u64);
        self.total_time = self.total_time_us as f64 / 1_000_000.0;

        self.frame_count = self.frame_count.wrapping_add(1);

        self.last_frame_time = now;

        self.accumulator += self.delta_time;
        if self.accumulator > 0.25 {
            self.accumulator = 0.25;
        }
    }

    /// Consumes a single `fixed_time_step` unit from the accumulator buffer.
    pub fn consume_fixed_step(&mut self) -> bool {
        if self.accumulator >= self.fixed_time_step {
            self.accumulator -= self.fixed_time_step;
            true
        } else {
            false
        }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::assert_matches;
    use core::range::Range;

    #[test]
    fn test_time_operations_with_new_features() {
        let mut time = Time::new();

        assert_matches!(time.delta_time, 0.0);
        assert_matches!(time.total_time_us, 0);
        assert_matches!(time.frame_count, 0);
        assert_matches!(time.consume_fixed_step(), false);

        time.accumulator += 0.02;

        let mut consumed_count = 0;

        let range = Range { start: 0, end: 5 };
        for _ in range {
            if time.consume_fixed_step() {
                consumed_count += 1;
            }
        }

        assert_matches!(consumed_count, 2);
        assert_matches!(time.consume_fixed_step(), false);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Monotonic frame clock and fixed-timestep accumulator for deterministic simulation ticks.

use std::time::{Duration, Instant};

/// Maximum single-frame duration (100 ms) used to prevent simulation spiral-of-death
/// when returning from breakpoints, window drags, or OS stalls.
const MAX_FRAME_DELTA: Duration = Duration::from_millis(100);

/// Default fixed simulation frequency in Hertz (120 Hz).
const DEFAULT_FIXED_HZ: f32 = 120.0;

/// Monotonic frame clock and fixed-timestep accumulator shared by engine and runtime loops.
///
/// Tracks per-frame `delta_time` (clamped to 100 ms), cumulative application `total_time`
/// in both floating-point seconds and integer microseconds, monotonically incremented
/// `frame_count`, and an `accumulator` drained via [`Time::consume_fixed_step`].
pub struct Time {
    last_frame_time: Instant,
    /// Clamped duration elapsed since the previous frame, in seconds.
    pub delta_time: f32,
    /// Cumulative elapsed simulation time since launch, in seconds.
    pub total_time: f64,
    /// Cumulative elapsed simulation time since launch, in microseconds.
    pub total_time_us: u64,
    /// Number of completed frames advanced through [`Time::tick`] or [`Time::advance_by`].
    pub frame_count: u64,
    /// Fixed simulation step duration in seconds (for example, `1.0 / 120.0` at 120 Hz).
    pub fixed_time_step: f32,
    /// Buffered elapsed seconds awaiting consumption by [`Time::consume_fixed_step`].
    pub accumulator: f32,
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

impl Time {
    /// Creates a new simulation clock anchored to `Instant::now()` with a 120 Hz fixed step.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
            delta_time: 0.0,
            total_time: 0.0,
            total_time_us: 0,
            frame_count: 0,
            fixed_time_step: 1.0 / DEFAULT_FIXED_HZ,
            accumulator: 0.0,
        }
    }

    /// Samples the OS monotonic clock, advances frame counters, and fills the fixed accumulator.
    ///
    /// Raw frame deltas exceeding 100 ms are clamped before updating `delta_time`,
    /// `total_time`, `total_time_us`, and `accumulator`.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        self.advance_by(duration);
    }

    /// Advances the clock deterministically by `duration` without reading wall-clock time.
    ///
    /// Useful for headless tests, fixed replay harnesses, and unit tests. Durations longer
    /// than 100 ms are clamped identically to [`Time::tick`].
    pub fn advance_by(&mut self, duration: Duration) {
        let clamped = duration.min(MAX_FRAME_DELTA);
        self.delta_time = clamped.as_secs_f32();
        self.total_time_us = self
            .total_time_us
            .saturating_add(clamped.as_micros() as u64);
        self.total_time += clamped.as_secs_f64();
        self.accumulator += self.delta_time;
        self.frame_count = self.frame_count.saturating_add(1);
    }

    /// Advances the clock deterministically by `delta_secs` seconds.
    ///
    /// Negative or non-finite values (`NaN`, `INFINITY`) are rejected and treated as zero
    /// elapsed seconds while still advancing `frame_count`.
    pub fn tick_with_delta(&mut self, delta_secs: f32) {
        let duration = if delta_secs.is_finite() && delta_secs > 0.0 {
            Duration::from_secs_f32(delta_secs.min(MAX_FRAME_DELTA.as_secs_f32()))
        } else {
            Duration::ZERO
        };
        self.advance_by(duration);
    }

    /// Configures `fixed_time_step` from a simulation frequency in Hertz.
    ///
    /// Returns `true` when `hz` is finite and strictly positive; otherwise leaves the
    /// current `fixed_time_step` unchanged and returns `false`.
    pub fn set_fixed_hz(&mut self, hz: f32) -> bool {
        if hz.is_finite() && hz > 0.0 {
            self.fixed_time_step = 1.0 / hz;
            true
        } else {
            false
        }
    }

    /// Configures `fixed_time_step` directly in seconds.
    ///
    /// Returns `true` when `step_secs` is finite and strictly positive; otherwise leaves
    /// the current `fixed_time_step` unchanged and returns `false`.
    pub fn set_fixed_time_step(&mut self, step_secs: f32) -> bool {
        if step_secs.is_finite() && step_secs > 0.0 {
            self.fixed_time_step = step_secs;
            true
        } else {
            false
        }
    }

    /// Consumes one fixed simulation step from `accumulator` if enough time has built up.
    ///
    /// Guards against invalid (`<= 0.0` or non-finite) `fixed_time_step` or `accumulator`
    /// values by returning `false` so `while time.consume_fixed_step()` loops terminate cleanly.
    pub fn consume_fixed_step(&mut self) -> bool {
        if !self.fixed_time_step.is_finite()
            || self.fixed_time_step <= 0.0
            || !self.accumulator.is_finite()
        {
            return false;
        }
        if self.accumulator >= self.fixed_time_step {
            self.accumulator -= self.fixed_time_step;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_timestep_accumulation_and_draining() {
        let mut time = Time::new();
        assert!(!time.consume_fixed_step());

        // Use an exact binary-representable step (0.015625 s = 1/64 s = 64 Hz).
        assert!(time.set_fixed_hz(64.0));
        time.advance_by(Duration::from_micros(39_062)); // 2.5 * 1/64 s -> 2 steps

        let mut steps = 0_u32;
        while time.consume_fixed_step() {
            steps += 1;
        }

        assert_eq!(steps, 2);
        assert_eq!(time.frame_count, 1);
        assert!(!time.consume_fixed_step());
    }

    #[test]
    fn test_spiral_of_death_clamping_limits_frame_delta_to_100ms() {
        let mut time = Time::new();
        time.advance_by(Duration::from_secs(5));

        assert!((time.delta_time - 0.1).abs() < 1e-6);
        assert_eq!(time.total_time_us, 100_000);
        assert!((time.total_time - 0.1).abs() < 1e-9);
        assert!((time.accumulator - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_invalid_deltas_and_non_positive_steps_are_rejected_safely() {
        let mut time = Time::new();
        let baseline_step = time.fixed_time_step;

        assert!(!time.set_fixed_hz(0.0));
        assert!(!time.set_fixed_hz(-60.0));
        assert!(!time.set_fixed_hz(f32::NAN));
        assert!(!time.set_fixed_time_step(0.0));
        assert!(!time.set_fixed_time_step(f32::INFINITY));
        assert_eq!(time.fixed_time_step, baseline_step);

        // Non-finite or negative deltas clamp to zero and preserve totals.
        time.tick_with_delta(-0.5);
        time.tick_with_delta(f32::NAN);
        time.tick_with_delta(f32::INFINITY);
        assert_eq!(time.delta_time, 0.0);
        assert_eq!(time.total_time_us, 0);
        assert_eq!(time.total_time, 0.0);
        assert_eq!(time.frame_count, 3);

        // Manual corruption of `fixed_time_step` or `accumulator` never triggers an infinite loop.
        time.accumulator = 1.0;
        time.fixed_time_step = 0.0;
        assert!(!time.consume_fixed_step());
        time.fixed_time_step = -0.01;
        assert!(!time.consume_fixed_step());
        time.fixed_time_step = f32::NAN;
        assert!(!time.consume_fixed_step());
    }
}
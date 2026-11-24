// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame rate budget computation and event-loop pacing tracker shared across host shells.

use std::time::{Duration, Instant};

/// Tracks frame-to-frame wall-clock elapsed time and computes remaining FPS cap sleep budgets.
///
/// Used by windowed host event loops (`ae_engine` and `ae_runtime`) so capped 60 Hz / 120 Hz
/// modes do not unlock and busy-spin under continuous polling.
pub struct FramePacer {
    last_frame_time: Instant,
}

impl Default for FramePacer {
    fn default() -> Self {
        Self::new()
    }
}

impl FramePacer {
    /// Creates a new frame pacer anchored to `Instant::now()`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
        }
    }

    /// Resets the frame boundary timestamp to `Instant::now()`.
    pub fn reset(&mut self) {
        self.last_frame_time = Instant::now();
    }

    /// Returns wall-clock duration elapsed since the previous frame boundary.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.last_frame_time.elapsed()
    }

    /// Computes remaining sleep budget for `target_fps` relative to `self.elapsed()`.
    ///
    /// Returns `None` for uncapped mode (`None`), non-finite/non-positive FPS values,
    /// or when the frame budget has already been exceeded.
    #[must_use]
    pub fn remaining_sleep_for_fps(&self, target_fps: Option<f32>) -> Option<Duration> {
        Self::remaining_sleep_from_elapsed(self.last_frame_time.elapsed(), target_fps)
    }

    /// Pure deterministic calculation of remaining sleep duration given `elapsed` and `target_fps`.
    #[must_use]
    pub fn remaining_sleep_from_elapsed(
        elapsed: Duration,
        target_fps: Option<f32>,
    ) -> Option<Duration> {
        let fps = target_fps?;
        if !fps.is_finite() || fps <= 0.0 {
            return None;
        }
        let target_duration = Duration::from_secs_f32(1.0 / fps);
        compute_frame_pacing_sleep(elapsed, target_duration)
    }
}

/// Computes remaining frame pacing sleep duration for a target frame budget.
///
/// Returns `Some(target_duration - elapsed)` whenever `target_duration` is positive and
/// `elapsed < target_duration`.
#[must_use]
pub fn compute_frame_pacing_sleep(
    elapsed: Duration,
    target_duration: Duration,
) -> Option<Duration> {
    if !target_duration.is_zero() && elapsed < target_duration {
        Some(target_duration - elapsed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{FramePacer, compute_frame_pacing_sleep};
    use std::time::Duration;

    #[test]
    fn test_compute_frame_pacing_sleep_returns_remaining_budget_for_120_fps() {
        let target_120 = Duration::from_secs_f32(1.0 / 120.0);
        let elapsed = Duration::from_millis(2);
        let remaining = compute_frame_pacing_sleep(elapsed, target_120)
            .expect("120 FPS budget with 2ms elapsed must sleep for remaining frame budget");
        assert_eq!(remaining, target_120 - elapsed);
    }

    #[test]
    fn test_compute_frame_pacing_sleep_rejects_expired_and_zero_budgets() {
        let target_120 = Duration::from_secs_f32(1.0 / 120.0);
        assert_eq!(compute_frame_pacing_sleep(target_120, target_120), None);
        assert_eq!(
            compute_frame_pacing_sleep(Duration::from_millis(16), target_120),
            None
        );
        assert_eq!(
            compute_frame_pacing_sleep(Duration::from_millis(1), Duration::ZERO),
            None
        );
    }

    #[test]
    fn test_remaining_sleep_from_elapsed_handles_capped_and_invalid_fps() {
        let elapsed = Duration::from_millis(4);
        let target_60 = Duration::from_secs_f32(1.0 / 60.0);

        assert_eq!(
            FramePacer::remaining_sleep_from_elapsed(elapsed, Some(60.0)),
            Some(target_60 - elapsed)
        );
        assert_eq!(
            FramePacer::remaining_sleep_from_elapsed(elapsed, None),
            None
        );
        assert_eq!(
            FramePacer::remaining_sleep_from_elapsed(elapsed, Some(0.0)),
            None
        );
        assert_eq!(
            FramePacer::remaining_sleep_from_elapsed(elapsed, Some(-120.0)),
            None
        );
        assert_eq!(
            FramePacer::remaining_sleep_from_elapsed(elapsed, Some(f32::NAN)),
            None
        );
        assert_eq!(
            FramePacer::remaining_sleep_from_elapsed(elapsed, Some(f32::INFINITY)),
            None
        );
    }
}
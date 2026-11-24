// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Fixed-capacity frame time ring buffer and pacing statistical analyzer.

/// Fixed-capacity ring buffer for zero-allocation recording of per-frame durations (in ms).
#[derive(Debug, Clone, Copy)]
pub struct FrameRingBuffer<const N: usize = 240> {
    samples: [f32; N],
    head: usize,
    count: usize,
}

impl<const N: usize> Default for FrameRingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> FrameRingBuffer<N> {
    /// Creates a new, empty `FrameRingBuffer`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            samples: [0.0; N],
            head: 0,
            count: 0,
        }
    }

    /// Pushes a finite, non-negative frame time sample (in milliseconds) into the ring buffer.
    ///
    /// Non-finite (`NaN`/`inf`) or negative values are safely clamped to `0.0` so pacing
    /// percentile calculations never become polluted.
    pub fn push(&mut self, sample_ms: f32) {
        if N == 0 {
            return;
        }
        let clean = if sample_ms.is_finite() && sample_ms >= 0.0 {
            sample_ms
        } else {
            0.0
        };
        self.samples[self.head] = clean;
        self.head = (self.head + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    /// Returns the number of valid samples currently recorded in the buffer.
    #[must_use]
    #[inline]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns the maximum capacity of the ring buffer.
    #[must_use]
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns `true` if no samples have been recorded yet.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the most recent frame time sample in milliseconds, or `0.0` if empty.
    #[must_use]
    pub fn latest(&self) -> f32 {
        if self.count == 0 || N == 0 {
            return 0.0;
        }
        let last_idx = if self.head == 0 { N - 1 } else { self.head - 1 };
        self.samples[last_idx]
    }

    /// Returns the sample at `index` in chronological order (`0` = oldest, `count - 1` = newest).
    #[must_use]
    pub fn get_chronological(&self, index: usize) -> Option<f32> {
        if index >= self.count || N == 0 {
            return None;
        }
        let actual_idx = if self.count < N {
            index
        } else {
            (self.head + index) % N
        };
        Some(self.samples[actual_idx])
    }

    /// Computes statistical performance and frame pacing metrics from the recorded samples.
    #[must_use]
    pub fn calculate_stats(&self) -> FramePacingStats {
        if self.count == 0 || N == 0 {
            return FramePacingStats::default();
        }

        let mut sum_ms = 0.0f32;
        let mut spikes_over_16ms = 0u32;
        let mut spikes_over_33ms = 0u32;

        let mut sorted = [0.0f32; N];
        for (i, slot) in sorted.iter_mut().enumerate().take(self.count) {
            if let Some(val) = self.get_chronological(i) {
                *slot = val;
                sum_ms += val;
                if val >= 16.6 {
                    spikes_over_16ms += 1;
                }
                if val >= 33.3 {
                    spikes_over_33ms += 1;
                }
            }
        }

        let avg_frametime_ms = sum_ms / self.count as f32;
        let avg_fps = if avg_frametime_ms > 0.0001 {
            1000.0 / avg_frametime_ms
        } else {
            0.0
        };

        let mut variance_sum = 0.0f32;
        for val in sorted.iter().take(self.count) {
            let diff = *val - avg_frametime_ms;
            variance_sum += diff * diff;
        }
        let jitter_ms = (variance_sum / self.count as f32).sqrt();

        sorted[..self.count].sort_unstable_by(f32::total_cmp);

        let min_ms = sorted[0];
        let max_ms = sorted[self.count - 1];

        let idx_1_pct = ((self.count as f32 * 0.99).floor() as usize).min(self.count - 1);
        let slowest_1_pct_ms = sorted[idx_1_pct];
        let low_1_percent_fps = if slowest_1_pct_ms > 0.0001 {
            1000.0 / slowest_1_pct_ms
        } else {
            0.0
        };

        let idx_0_1_pct = ((self.count as f32 * 0.999).floor() as usize).min(self.count - 1);
        let slowest_0_1_pct_ms = sorted[idx_0_1_pct];
        let low_0_1_percent_fps = if slowest_0_1_pct_ms > 0.0001 {
            1000.0 / slowest_0_1_pct_ms
        } else {
            0.0
        };

        let stutter_rate_percent = (spikes_over_16ms as f32 / self.count as f32) * 100.0;

        FramePacingStats {
            average_fps: avg_fps,
            average_frametime_ms: avg_frametime_ms,
            variance_ms: jitter_ms,
            low_1_percent_fps,
            low_0_1_percent_fps,
            min_frametime_ms: min_ms,
            max_frametime_ms: max_ms,
            spikes_over_16ms,
            spikes_over_33ms,
            stutter_rate_percent,
        }
    }
}

/// Comprehensive frame pacing and stutter statistical metrics.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FramePacingStats {
    /// Average frames per second over the recorded sample window.
    pub average_fps: f32,
    /// Average frame duration in milliseconds.
    pub average_frametime_ms: f32,
    /// Standard deviation of frame times (pacing jitter in milliseconds).
    pub variance_ms: f32,
    /// 1% Low FPS metric representing the 99th percentile slowest frame duration.
    pub low_1_percent_fps: f32,
    /// 0.1% Low FPS metric representing the 99.9th percentile slowest frame duration.
    pub low_0_1_percent_fps: f32,
    /// Minimum frame duration observed in the sample window (in milliseconds).
    pub min_frametime_ms: f32,
    /// Maximum frame duration observed in the sample window (in milliseconds).
    pub max_frametime_ms: f32,
    /// Number of frames exceeding the 16.67ms (60 FPS) threshold in the sample window.
    pub spikes_over_16ms: u32,
    /// Number of frames exceeding the 33.33ms (30 FPS) threshold in the sample window.
    pub spikes_over_33ms: u32,
    /// Percentage of recorded frames exceeding 16.6ms.
    pub stutter_rate_percent: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_ring_buffer_push_wrap_and_guards() {
        let mut zero_cap = FrameRingBuffer::<0>::new();
        zero_cap.push(16.0);
        assert_eq!(zero_cap.capacity(), 0);
        assert_eq!(zero_cap.latest(), 0.0);
        assert_eq!(zero_cap.calculate_stats(), FramePacingStats::default());

        let mut buffer = FrameRingBuffer::<4>::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.count(), 0);
        assert_eq!(buffer.get_chronological(0), None);

        buffer.push(16.0);
        buffer.push(f32::NAN);
        assert_eq!(buffer.latest(), 0.0);
        assert_eq!(buffer.get_chronological(0), Some(16.0));
        assert_eq!(buffer.get_chronological(1), Some(0.0));

        buffer.push(4.0);
        buffer.push(2.0);
        buffer.push(1.0);
        assert_eq!(buffer.count(), 4);
        assert_eq!(buffer.get_chronological(0), Some(0.0));
        assert_eq!(buffer.get_chronological(3), Some(1.0));
        assert_eq!(buffer.latest(), 1.0);
    }

    #[test]
    fn test_frame_pacing_stats_and_percentile_lows() {
        let mut steady = FrameRingBuffer::<5>::new();
        for _ in 0..5 {
            steady.push(10.0);
        }
        let steady_stats = steady.calculate_stats();
        assert!((steady_stats.average_frametime_ms - 10.0).abs() < 1e-3);
        assert!((steady_stats.average_fps - 100.0).abs() < 1e-3);
        assert!(steady_stats.variance_ms < 1e-3);
        assert_eq!(steady_stats.spikes_over_16ms, 0);

        let mut spiked = FrameRingBuffer::<100>::new();
        for _ in 0..99 {
            spiked.push(8.33);
        }
        spiked.push(33.33);
        let spiked_stats = spiked.calculate_stats();
        assert!(spiked_stats.min_frametime_ms <= 8.34);
        assert!(spiked_stats.max_frametime_ms >= 33.32);
        assert_eq!(spiked_stats.spikes_over_16ms, 1);
        assert_eq!(spiked_stats.spikes_over_33ms, 1);
        assert!(spiked_stats.low_1_percent_fps <= 31.0);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Telemetry and Performance Profiling Data Structures.
//!
//! Provides zero-allocation ring buffers, frame pacing statistical analyzers,
//! CPU thread synchronization metrics, GPU pass timings, and VRAM breakdowns.
//!

/// Fixed-capacity ring buffer for zero-allocation recording of per-frame metrics.
/// Stores `N` floating-point frame time samples in a contiguous static array on the stack.
/// Supports push operations with wrapping, chronological iteration, and statistical calculations
/// without heap allocations.
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
    pub const fn new() -> Self {
        Self {
            samples: [0.0; N],
            head: 0,
            count: 0,
        }
    }

    /// Pushes a new frame time sample (in milliseconds) into the ring buffer.
    /// Overwrites the oldest sample once the buffer capacity `N` is reached.
    pub fn push(&mut self, sample_ms: f32) {
        if N == 0 {
            return;
        }
        self.samples[self.head] = sample_ms;
        self.head = (self.head + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    /// Returns the number of valid samples currently recorded in the buffer.
    #[inline]
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Returns the maximum capacity of the ring buffer.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns `true` if no samples have been recorded yet.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the most recent frame time sample in milliseconds, or `0.0` if empty.
    pub fn latest(&self) -> f32 {
        if self.count == 0 || N == 0 {
            return 0.0;
        }
        let last_idx = if self.head == 0 { N - 1 } else { self.head - 1 };
        self.samples[last_idx]
    }

    /// Returns the sample at the given chronological index (0 = oldest, count-1 = newest).
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
    /// Calculates Average FPS, Frame Pacing Variance (Jitter ms), 1% Low FPS (99th percentile slowest),
    /// 0.1% Low FPS (99.9th percentile slowest / worst spike), and stutter rates.
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

        // Variance / Standard Deviation (Jitter ms)
        let mut variance_sum = 0.0f32;
        for val in sorted.iter().take(self.count) {
            let diff = *val - avg_frametime_ms;
            variance_sum += diff * diff;
        }
        let jitter_ms = (variance_sum / self.count as f32).sqrt();

        // Sort slice in ascending order for percentile calculations
        sorted[..self.count].sort_unstable_by(|a, b| a.total_cmp(b));

        let min_ms = sorted[0];
        let max_ms = sorted[self.count - 1];

        // 1% Low (99th percentile slowest frame)
        let idx_1_pct = ((self.count as f32 * 0.99).floor() as usize).min(self.count - 1);
        let slowest_1_pct_ms = sorted[idx_1_pct];
        let low_1_percent_fps = if slowest_1_pct_ms > 0.0001 {
            1000.0 / slowest_1_pct_ms
        } else {
            0.0
        };

        // 0.1% Low (99.9th percentile slowest frame)
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
    /// 0.1% Low FPS metric representing the 99.9th percentile slowest frame duration (peak stutter).
    pub low_0_1_percent_fps: f32,
    /// Minimum frame duration observed in the sample window (in milliseconds).
    pub min_frametime_ms: f32,
    /// Maximum frame duration observed in the sample window (in milliseconds).
    pub max_frametime_ms: f32,
    /// Number of frames exceeding the 16.67ms (60 FPS) threshold in the sample window.
    pub spikes_over_16ms: u32,
    /// Number of frames exceeding the 33.33ms (30 FPS) threshold in the sample window.
    pub spikes_over_33ms: u32,
    /// Percentage of recorded frames that experienced frame pacing stutter (>16.67ms).
    pub stutter_rate_percent: f32,
}

/// High-resolution CPU thread execution and synchronization stage timings (in milliseconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CpuSyncTimings {
    /// ECS system updates, entity hierarchy transformations, and gameplay logic.
    pub main_logic_ms: f32,
    /// Physics simulation stepping, kinematic velocities, and spatial broadphase queries.
    pub physics_ms: f32,
    /// Render preparation, instance buffer staging, and command encoder recording.
    pub render_prep_ms: f32,
    /// Wall-clock time spent waiting on GPU presentation (swapchain present and acquire synchronization).
    pub wait_for_gpu_ms: f32,
    /// Editor UI layout calculation, widget tessellation, and event handling.
    pub ui_editor_ms: f32,
    /// Total combined CPU frame execution duration.
    pub total_cpu_ms: f32,
}

impl CpuSyncTimings {
    /// Determines whether the frame bottleneck is currently CPU-bound or GPU-bound.
    pub fn is_cpu_bound(&self, total_gpu_ms: f32) -> bool {
        let active_cpu = self.main_logic_ms + self.physics_ms + self.render_prep_ms;
        active_cpu > total_gpu_ms && self.wait_for_gpu_ms < 2.0
    }
}

/// Individual GPU render pass execution durations (in milliseconds).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GpuPassTimings {
    /// Cascaded directional shadow map depth pass execution time.
    pub shadow_pass_ms: f32,
    /// Main forward opaque and transparent scene geometry render pass execution time.
    pub main_opaque_pass_ms: f32,
    /// Post-processing bloom downsampling/upsampling and selection outline pass execution time.
    pub post_process_pass_ms: f32,
    /// Egui UI rendering and composite pass execution time.
    pub ui_pass_ms: f32,
    /// Total aggregated GPU render workload duration.
    pub total_gpu_ms: f32,
}

/// Detailed breakdown of GPU draw calls, compute dispatches, and culling statistics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DrawCallBreakdown {
    /// Total aggregate draw calls dispatched to the GPU command encoder.
    pub total_draw_calls: u32,
    /// Primitive batched draw calls (cubes, spheres, cylinders, quads, debug shapes).
    pub batched_draw_calls: u32,
    /// Hardware instanced draw calls for repeated 3D meshes and entities.
    pub instanced_draw_calls: u32,
    /// Dispatched compute shader passes (culling, particle physics, compute processing).
    pub dispatched_compute: u32,
    /// Meshes excluded from rendering by frustum culling or visibility masks.
    pub culled_meshes: u32,
    /// Total rendered triangle count for the active frame.
    pub triangles: u64,
    /// Total rendered vertex count for the active frame.
    pub vertices: u64,
}

impl DrawCallBreakdown {
    /// Returns the hardware instancing efficiency ratio (percentage of instanced draw calls).
    pub fn instancing_ratio_percent(&self) -> f32 {
        if self.total_draw_calls == 0 {
            0.0
        } else {
            (self.instanced_draw_calls as f32 / self.total_draw_calls as f32) * 100.0
        }
    }
}

/// Detailed breakdown of allocated Video RAM (VRAM) consumption across graphics subsystems.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct VramStats {
    /// VRAM allocated for loaded 2D/3D textures (diffuse, normal, specular, default white).
    pub texture_vram_mb: f32,
    /// VRAM allocated for static and dynamic vertex/index buffers (primitive models and loaded 3D assets).
    pub mesh_index_vram_mb: f32,
    /// VRAM allocated for uniform buffers, instance storage, shadow maps, and framebuffer render targets.
    pub dynamic_uniform_vram_mb: f32,
    /// Total combined VRAM allocated by the rendering engine.
    pub total_vram_mb: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_ring_buffer_push_and_wrap() {
        let mut buffer = FrameRingBuffer::<4>::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.count(), 0);

        buffer.push(16.0);
        buffer.push(8.0);
        assert_eq!(buffer.count(), 2);
        assert_eq!(buffer.get_chronological(0), Some(16.0));
        assert_eq!(buffer.get_chronological(1), Some(8.0));
        assert_eq!(buffer.latest(), 8.0);

        buffer.push(4.0);
        buffer.push(2.0);
        assert_eq!(buffer.count(), 4);

        // Wrap around: push 5th item, replacing oldest (16.0)
        buffer.push(1.0);
        assert_eq!(buffer.count(), 4);
        assert_eq!(buffer.get_chronological(0), Some(8.0));
        assert_eq!(buffer.get_chronological(1), Some(4.0));
        assert_eq!(buffer.get_chronological(2), Some(2.0));
        assert_eq!(buffer.get_chronological(3), Some(1.0));
        assert_eq!(buffer.latest(), 1.0);
    }

    #[test]
    fn test_frame_pacing_stats_calculation() {
        let mut buffer = FrameRingBuffer::<5>::new();
        buffer.push(10.0);
        buffer.push(10.0);
        buffer.push(10.0);
        buffer.push(10.0);
        buffer.push(10.0);

        let stats = buffer.calculate_stats();
        assert!((stats.average_frametime_ms - 10.0).abs() < 0.001);
        assert!((stats.average_fps - 100.0).abs() < 0.001);
        assert!(stats.variance_ms < 0.001); // zero jitter
        assert_eq!(stats.spikes_over_16ms, 0);
    }

    #[test]
    fn test_percentile_low_fps() {
        let mut buffer = FrameRingBuffer::<100>::new();
        // 99 frames at 8.33ms (120 FPS), 1 spike frame at 33.33ms (30 FPS)
        for _ in 0..99 {
            buffer.push(8.33);
        }
        buffer.push(33.33);

        let stats = buffer.calculate_stats();
        assert!(stats.min_frametime_ms <= 8.34);
        assert!(stats.max_frametime_ms >= 33.32);
        assert_eq!(stats.spikes_over_16ms, 1);
        assert_eq!(stats.spikes_over_33ms, 1);
        assert!(stats.low_1_percent_fps <= 31.0); // spike captures the bottom 1%
    }
}
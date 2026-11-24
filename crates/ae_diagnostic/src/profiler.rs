// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Per-frame CPU performance profiler and stage duration accumulator.

use crate::frame_pacing::{FramePacingStats, FrameRingBuffer};
use crate::timings::CpuSyncTimings;
use std::time::Instant;

/// Per-frame CPU-side performance profiler and frame pacing analyzer.
#[derive(Default)]
pub struct Profiler {
    /// ECS system execution and entity transformation time in milliseconds.
    pub ecs_time: f32,
    /// Physics simulation stepping and collision synchronization time in milliseconds.
    pub physics_time: f32,
    /// Render preparation and command recording time in milliseconds.
    pub render_time: f32,
    /// Editor UI layout calculation and drawing time in milliseconds.
    pub ui_time: f32,
    /// Total wall-clock time elapsed for the last complete frame in milliseconds.
    pub total_frame_time: f32,
    /// Zero-allocation ring buffer storing the last 240 frame durations.
    pub frame_pacing: FrameRingBuffer<240>,

    ecs_start: Option<Instant>,
    physics_start: Option<Instant>,
    render_start: Option<Instant>,
    ui_start: Option<Instant>,
    frame_start: Option<Instant>,
}

impl Profiler {
    /// Creates a new `Profiler` instance with all timers zeroed and an empty ring buffer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the beginning of a new frame measurement.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    /// Finalizes the frame measurement and records elapsed milliseconds into `frame_pacing`.
    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start {
            self.total_frame_time = start.elapsed().as_secs_f32() * 1000.0;
            self.frame_pacing.push(self.total_frame_time);
        }
    }

    /// Marks the beginning of the ECS update phase.
    pub fn begin_ecs(&mut self) {
        self.ecs_start = Some(Instant::now());
    }

    /// Finalizes the ECS phase measurement and stores elapsed time in `ecs_time` (ms).
    pub fn end_ecs(&mut self) {
        if let Some(start) = self.ecs_start {
            self.ecs_time = start.elapsed().as_secs_f32() * 1000.0;
        }
    }

    /// Marks the beginning of the Physics simulation and sync phase.
    pub fn begin_physics(&mut self) {
        self.physics_start = Some(Instant::now());
    }

    /// Finalizes the Physics phase measurement and stores elapsed time in `physics_time` (ms).
    pub fn end_physics(&mut self) {
        if let Some(start) = self.physics_start {
            self.physics_time = start.elapsed().as_secs_f32() * 1000.0;
        }
    }

    /// Marks the beginning of the GPU render pass phase.
    pub fn begin_render(&mut self) {
        self.render_start = Some(Instant::now());
    }

    /// Finalizes the render phase measurement and stores elapsed time in `render_time` (ms).
    pub fn end_render(&mut self) {
        if let Some(start) = self.render_start {
            self.render_time = start.elapsed().as_secs_f32() * 1000.0;
        }
    }

    /// Marks the beginning of the UI pass phase.
    pub fn begin_ui(&mut self) {
        self.ui_start = Some(Instant::now());
    }

    /// Finalizes the UI phase measurement and stores elapsed time in `ui_time` (ms).
    pub fn end_ui(&mut self) {
        if let Some(start) = self.ui_start {
            self.ui_time = start.elapsed().as_secs_f32() * 1000.0;
        }
    }

    /// Computes and returns current frame pacing statistical metrics from the ring buffer.
    #[must_use]
    pub fn calculate_pacing_stats(&self) -> FramePacingStats {
        self.frame_pacing.calculate_stats()
    }

    /// Extracts structured CPU execution and synchronization stage timings.
    ///
    /// Clamps non-finite or negative `present_wait_ms` inputs safely to `0.0`.
    #[must_use]
    pub fn get_cpu_sync_timings(&self, present_wait_ms: f32) -> CpuSyncTimings {
        let safe_wait = if present_wait_ms.is_finite() && present_wait_ms >= 0.0 {
            present_wait_ms
        } else {
            0.0
        };
        let render_prep_ms = (self.render_time - safe_wait - self.ui_time).max(0.0);
        let total_cpu_ms =
            self.ecs_time + self.physics_time + render_prep_ms + safe_wait + self.ui_time;
        CpuSyncTimings {
            main_logic_ms: self.ecs_time,
            physics_ms: self.physics_time,
            render_prep_ms,
            wait_for_gpu_ms: safe_wait,
            ui_editor_ms: self.ui_time,
            total_cpu_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_clamps_wait_and_ignores_unstarted_end_calls() {
        let mut profiler = Profiler::new();
        profiler.end_frame();
        profiler.end_ecs();
        profiler.end_physics();
        profiler.end_render();
        profiler.end_ui();
        assert_eq!(profiler.total_frame_time, 0.0);
        assert!(profiler.frame_pacing.is_empty());

        profiler.ecs_time = 2.0;
        profiler.physics_time = 1.5;
        profiler.render_time = 5.0;
        profiler.ui_time = 1.0;

        let clamped = profiler.get_cpu_sync_timings(10.0);
        assert_eq!(clamped.render_prep_ms, 0.0);
        assert_eq!(clamped.wait_for_gpu_ms, 10.0);
        assert!((clamped.total_cpu_ms - 14.5).abs() < 1e-4);

        let nan_safe = profiler.get_cpu_sync_timings(f32::NAN);
        assert_eq!(nan_safe.wait_for_gpu_ms, 0.0);
        assert!((nan_safe.render_prep_ms - 4.0).abs() < 1e-4);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use ae_core::telemetry::{CpuSyncTimings, FramePacingStats, FrameRingBuffer};
use std::time::Instant;

/// Per-frame CPU-side performance profiler and frame pacing analyzer.
/// Tracks high-resolution wall-clock durations (in milliseconds) for ECS logic updates,
/// physics simulation, render preparation, VSync present synchronization, and egui UI passes.
/// Houses a zero-allocation 240-frame ring buffer for real-time stutter and percentile analysis.
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
    /// Zero-allocation ring buffer storing the last 240 frame durations for real-time pacing analysis.
    pub frame_pacing: FrameRingBuffer<240>,

    // Internal trackers
    ecs_start: Option<Instant>,
    physics_start: Option<Instant>,
    render_start: Option<Instant>,
    ui_start: Option<Instant>,
    frame_start: Option<Instant>,
}

impl Profiler {
    /// Creates a new Profiler instance with all timers zeroed and an empty 240-frame ring buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the beginning of a new frame measurement.
    /// Must be called at the very start of the engine update loop.
    /// Pair with `end_frame()` at the end of the frame lifecycle.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    /// Finalizes the frame measurement, stores elapsed time in `total_frame_time` (ms),
    /// and pushes the new duration into the 240-frame ring buffer.
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

    /// Marks the beginning of the egui UI draw phase.
    pub fn begin_ui(&mut self) {
        self.ui_start = Some(Instant::now());
    }

    /// Finalizes the UI phase measurement and stores elapsed time in `ui_time` (ms).
    pub fn end_ui(&mut self) {
        if let Some(start) = self.ui_start {
            self.ui_time = start.elapsed().as_secs_f32() * 1000.0;
        }
    }

    /// Computes and returns current frame pacing statistical metrics from the 240-frame ring buffer.
    pub fn calculate_pacing_stats(&self) -> FramePacingStats {
        self.frame_pacing.calculate_stats()
    }

    /// Extracts structured CPU execution and synchronization stage timings.
    pub fn get_cpu_sync_timings(&self, present_wait_ms: f32) -> CpuSyncTimings {
        let render_prep_ms = (self.render_time - present_wait_ms).max(0.0);
        let total_cpu_ms =
            self.ecs_time + self.physics_time + render_prep_ms + present_wait_ms + self.ui_time;
        CpuSyncTimings {
            main_logic_ms: self.ecs_time,
            physics_ms: self.physics_time,
            render_prep_ms,
            wait_for_gpu_ms: present_wait_ms,
            ui_editor_ms: self.ui_time,
            total_cpu_ms,
        }
    }
}
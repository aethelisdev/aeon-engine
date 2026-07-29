// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use std::time::Instant;

/// Per-frame CPU-side performance profiler.
/// Tracks wall-clock time (in milliseconds) for ECS update, render pass,
/// and UI draw phases using `Instant`-based start/stop pairs.
/// Results are displayed in the editor's performance overlay.
/// All timing values are stored as `f32` milliseconds for UI convenience.
#[derive(Default)]
pub struct Profiler {
    pub ecs_time: f32,
    pub render_time: f32,
    pub ui_time: f32,
    pub total_frame_time: f32,

    // Internal trackers
    ecs_start: Option<Instant>,
    render_start: Option<Instant>,
    ui_start: Option<Instant>,
    frame_start: Option<Instant>,
}

impl Profiler {
    /// Creates a new Profiler with all timers zeroed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the beginning of a new frame measurement.
    /// Must be called at the very start of the frame loop.
    /// Pair with `end_frame()` at the end.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    /// Finalizes the frame measurement and stores elapsed time in `total_frame_time` (ms).
    pub fn end_frame(&mut self) {
        if let Some(start) = self.frame_start {
            self.total_frame_time = start.elapsed().as_secs_f32() * 1000.0;
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
}
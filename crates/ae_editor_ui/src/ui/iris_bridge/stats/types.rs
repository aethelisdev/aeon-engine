// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Types
//!
//! Exposes parameter structures, semantic [`UiTag`] identifiers, and panel state
//! for the pure declarative [`UiScope`] Stats & Profiler panel.

use ae_core::telemetry::FrameRingBuffer;
use irisui::prelude::*;

// ── Semantic 64-bit Tags ──────────────────────────────────────────────────

/// Semantic tag assigned to the Wireframe Mode toggle checkbox container.
pub const STATS_TAG_TOGGLE_WIREFRAME: u64 = UiTag::from_str("stats.toggle.wireframe").raw();

/// Semantic tag assigned to the Viewport Coordinate Grid toggle checkbox container.
pub const STATS_TAG_TOGGLE_GRID: u64 = UiTag::from_str("stats.toggle.grid").raw();

/// Semantic tag assigned to the Stats Panel root viewport container.
pub const STATS_TAG_PANEL_ROOT: u64 = UiTag::from_str("stats.panel_root").raw();

/// Semantic tag assigned to the Oscilloscope frametime canvas container.
pub const STATS_TAG_CANVAS: u64 = UiTag::from_str("stats.canvas").raw();

/// Semantic tag assigned to the Stats telemetry scroll viewport container.
pub const STATS_TAG_VIEWPORT: u64 = UiTag::from_str("stats.viewport").raw();

/// Semantic tag assigned to the Average FPS & frametime telemetry label.
pub const STATS_TAG_AVG_FPS: u64 = UiTag::from_str("stats.avg_fps").raw();

/// Semantic tag assigned to the 1% Low FPS telemetry label.
pub const STATS_TAG_LOW_1: u64 = UiTag::from_str("stats.low_1").raw();

/// Semantic tag assigned to the 0.1% Low FPS telemetry label.
pub const STATS_TAG_LOW_01: u64 = UiTag::from_str("stats.low_01").raw();

/// Semantic tag assigned to the Frametime Jitter telemetry label.
pub const STATS_TAG_JITTER: u64 = UiTag::from_str("stats.jitter").raw();

/// Semantic tag assigned to the Frame Pacing Summary footer telemetry label.
pub const STATS_TAG_SUMMARY: u64 = UiTag::from_str("stats.summary").raw();

/// Semantic tag assigned to the CPU Thread Balance status telemetry label.
pub const STATS_TAG_CPU_BALANCE: u64 = UiTag::from_str("stats.cpu.balance").raw();

/// Semantic tag assigned to the CPU ECS/Logic timing telemetry label.
pub const STATS_TAG_CPU_ECS: u64 = UiTag::from_str("stats.cpu.ecs").raw();

/// Semantic tag assigned to the CPU Physics timing telemetry label.
pub const STATS_TAG_CPU_PHYSICS: u64 = UiTag::from_str("stats.cpu.physics").raw();

/// Semantic tag assigned to the CPU Render Prep timing telemetry label.
pub const STATS_TAG_CPU_RENDER: u64 = UiTag::from_str("stats.cpu.render").raw();

/// Semantic tag assigned to the CPU Wait for GPU timing telemetry label.
pub const STATS_TAG_CPU_WAIT: u64 = UiTag::from_str("stats.cpu.wait").raw();

/// Semantic tag assigned to the CPU UI/Editor timing telemetry label.
pub const STATS_TAG_CPU_UI: u64 = UiTag::from_str("stats.cpu.ui").raw();

/// Semantic tag assigned to the CPU Total Frame timing telemetry label.
pub const STATS_TAG_CPU_TOTAL: u64 = UiTag::from_str("stats.cpu.total").raw();

/// Semantic tag assigned to the GPU Total Frame timing telemetry label.
pub const STATS_TAG_GPU_TOTAL: u64 = UiTag::from_str("stats.gpu.total").raw();

const ALL_STATS_TAGS: &[u64] = &[
    STATS_TAG_TOGGLE_WIREFRAME,
    STATS_TAG_TOGGLE_GRID,
    STATS_TAG_PANEL_ROOT,
    STATS_TAG_CANVAS,
    STATS_TAG_VIEWPORT,
    STATS_TAG_AVG_FPS,
    STATS_TAG_LOW_1,
    STATS_TAG_LOW_01,
    STATS_TAG_JITTER,
    STATS_TAG_SUMMARY,
    STATS_TAG_CPU_BALANCE,
    STATS_TAG_CPU_ECS,
    STATS_TAG_CPU_PHYSICS,
    STATS_TAG_CPU_RENDER,
    STATS_TAG_CPU_WAIT,
    STATS_TAG_CPU_UI,
    STATS_TAG_CPU_TOTAL,
    STATS_TAG_GPU_TOTAL,
];

/// Returns `true` if the given 64-bit widget tag belongs to the Stats & Telemetry panel subsystem.
#[inline]
pub fn is_stats_tag(tag: u64) -> bool {
    ALL_STATS_TAGS.contains(&tag)
}

/// Persistent interactive state for the Performance Stats & Telemetry panel overlay.
///
/// Periodic 33ms live refresh timing is governed autonomously by `Workbench`.
#[derive(Debug, Clone)]
pub struct StatsPanelState {
    /// Accumulated frame count within the current 250ms telemetry rolling average window.
    pub frame_counter: u32,
    /// Timestamp of the last visual rolling window update for the FPS text in the Stats panel.
    pub last_fps_refresh: std::time::Instant,
    /// Windowed rolling average FPS displayed in the UI, updated every 250ms for rock-solid readability.
    pub displayed_fps: f32,
    /// Ring buffer snapshot of recent frame pacing samples used to draw the oscilloscope polyline.
    pub frame_ring_buffer: Option<FrameRingBuffer>,
}

impl Default for StatsPanelState {
    fn default() -> Self {
        Self {
            frame_counter: 0,
            last_fps_refresh: std::time::Instant::now(),
            displayed_fps: 60.0,
            frame_ring_buffer: None,
        }
    }
}

/// Semantic alias for [`StatsPanelState`].
pub type StatsState = StatsPanelState;
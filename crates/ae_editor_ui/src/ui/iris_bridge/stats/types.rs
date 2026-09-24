// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Types
//!
//! Exposes parameter structures, interaction actions, semantic 64-bit tags,
//! and panel state for the pure declarative [`UiScope`] Stats & Profiler panel.
//!

use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer, GpuPassTimings, VramStats,
};
use irisui::prelude::*;

// ── Semantic 64-bit Tags ──────────────────────────────────────────────────

/// Semantic tag assigned to the Wireframe Mode toggle checkbox container.
pub const STATS_TAG_TOGGLE_WIREFRAME: u64 = 0x5354_4154_0000_0001;

/// Semantic tag assigned to the Viewport Coordinate Grid toggle checkbox container.
pub const STATS_TAG_TOGGLE_GRID: u64 = 0x5354_4154_0000_0002;

/// Semantic tag assigned to the Stats Panel root viewport container.
pub const STATS_TAG_PANEL_ROOT: u64 = 0x5354_4154_0000_0003;

/// Semantic tag assigned to the Oscilloscope frametime canvas container.
pub const STATS_TAG_CANVAS: u64 = 0x5354_4154_0000_0004;

/// Returns `true` if the given 64-bit widget tag belongs to the Stats & Telemetry panel subsystem.
#[inline]
pub fn is_stats_tag(tag: u64) -> bool {
    (tag & 0xFFFF_FFFF_0000_0000) == 0x5354_4154_0000_0000
}

/// Actions emitted by the Stats & Profiler panel interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum StatsPanelAction {
    /// Toggles wireframe edge rendering.
    ToggleWireframe,
    /// Toggles world grid visibility.
    ToggleGrid,
    /// Vertical scrolling delta in pixels.
    Scroll(f32),
}

/// Parameter context bundle passed into the declarative Stats & Profiler builder.
pub struct StatsPanelParams<'a> {
    /// Bounding rectangle allocated for the stats panel inside docking.
    pub panel_rect: Rect,
    /// Current vertical scroll offset in pixels.
    pub scroll_y: f32,
    /// Mouse cursor coordinates in screen space.
    pub cursor_pos: Point,
    /// Whether wireframe edge mode is enabled.
    pub wireframe_enabled: bool,
    /// Whether the viewport coordinate grid is enabled.
    pub grid_enabled: bool,
    /// Smoothed frames per second.
    pub fps: f32,
    /// Historical frame pacing ring buffer (240 samples).
    pub frame_pacing: &'a FrameRingBuffer<240>,
    /// Calculated frametime variance, 1% low, and 0.1% low stats.
    pub frame_pacing_stats: &'a FramePacingStats,
    /// CPU thread synchronization timings breakdown.
    pub cpu_timings: &'a CpuSyncTimings,
    /// GPU render pass execution durations.
    pub gpu_pass_timings: &'a GpuPassTimings,
    /// Granular draw call metrics and batch counts.
    pub draw_call_stats: &'a DrawCallBreakdown,
    /// Categorized VRAM memory consumption.
    pub vram_stats: &'a VramStats,
    /// Total rendered triangles in current frame.
    pub render_triangles: u64,
    /// Total rendered vertices in current frame.
    pub render_vertices: u64,
    /// Hardware GPU adapter device name.
    pub gpu_adapter_name: &'a str,
    /// Active graphics API backend (e.g. Vulkan, Metal, DX12).
    pub gpu_backend: &'a str,
    /// Count of active entities in the ECS world.
    pub active_entities_count: usize,
    /// Currently selected entity, if any.
    pub selected_entity: Option<hecs::Entity>,
}

/// Persistent interactive state for the Performance Stats & Telemetry panel overlay.
#[derive(Debug, Clone)]
pub struct StatsPanelState {
    /// Pending user interaction actions emitted during events.
    pub actions: Vec<StatsPanelAction>,
    /// Vertical scrolling offset in physical pixels.
    pub scroll_y: f32,
    /// Maximum computed vertical scrollable overflow extent.
    pub max_scroll: f32,
    /// Last bounding rectangle allocated for the Stats & Profiler panel.
    pub last_rect: Option<Rect>,
    /// Accumulated frame count within the current 250ms telemetry rolling average window.
    pub frame_counter: u32,
    /// Timestamp of the last visual rolling window update for the FPS text in the Stats panel.
    pub last_fps_refresh: std::time::Instant,
    /// Windowed rolling average FPS displayed in the UI, updated every 250ms for rock-solid readability.
    pub displayed_fps: f32,
}

impl Default for StatsPanelState {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            scroll_y: 0.0,
            max_scroll: 0.0,
            last_rect: None,
            frame_counter: 0,
            last_fps_refresh: std::time::Instant::now(),
            displayed_fps: 60.0,
        }
    }
}

impl StatsPanelState {
    /// Consumes and returns all pending user interaction actions.
    pub fn take_actions(&mut self) -> Vec<StatsPanelAction> {
        std::mem::take(&mut self.actions)
    }
}
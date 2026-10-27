// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Types
//!
//! Exposes parameter structures, interaction actions, semantic 64-bit tags,
//! and panel state for the pure declarative [`UiScope`] Stats & Profiler panel.

use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer,
    RenderPassRecordingTimings, VramStats,
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

/// Semantic tag assigned to the Stats telemetry scroll viewport container.
pub const STATS_TAG_VIEWPORT: u64 = 0x5354_4154_0000_0005;

/// Semantic tag assigned to the Average FPS & frametime telemetry label.
pub const STATS_TAG_AVG_FPS: u64 = 0x5354_4154_0000_0010;

/// Semantic tag assigned to the 1% Low FPS telemetry label.
pub const STATS_TAG_LOW_1: u64 = 0x5354_4154_0000_0011;

/// Semantic tag assigned to the 0.1% Low FPS telemetry label.
pub const STATS_TAG_LOW_01: u64 = 0x5354_4154_0000_0012;

/// Semantic tag assigned to the Frametime Jitter telemetry label.
pub const STATS_TAG_JITTER: u64 = 0x5354_4154_0000_0013;

/// Semantic tag assigned to the Frame Pacing Summary footer telemetry label.
pub const STATS_TAG_SUMMARY: u64 = 0x5354_4154_0000_0014;

/// Semantic tag assigned to the CPU Thread Balance status telemetry label.
pub const STATS_TAG_CPU_BALANCE: u64 = 0x5354_4154_0000_0020;

/// Semantic tag assigned to the CPU ECS/Logic timing telemetry label.
pub const STATS_TAG_CPU_ECS: u64 = 0x5354_4154_0000_0021;

/// Semantic tag assigned to the CPU Physics timing telemetry label.
pub const STATS_TAG_CPU_PHYSICS: u64 = 0x5354_4154_0000_0022;

/// Semantic tag assigned to the CPU Render Prep timing telemetry label.
pub const STATS_TAG_CPU_RENDER: u64 = 0x5354_4154_0000_0023;

/// Semantic tag assigned to the CPU Wait for GPU timing telemetry label.
pub const STATS_TAG_CPU_WAIT: u64 = 0x5354_4154_0000_0024;

/// Semantic tag assigned to the CPU UI/Editor timing telemetry label.
pub const STATS_TAG_CPU_UI: u64 = 0x5354_4154_0000_0025;

/// Semantic tag assigned to the CPU Total Frame timing telemetry label.
pub const STATS_TAG_CPU_TOTAL: u64 = 0x5354_4154_0000_0026;

/// Semantic tag assigned to the GPU Total Frame timing telemetry label.
pub const STATS_TAG_GPU_TOTAL: u64 = 0x5354_4154_0000_0030;

/// Returns `true` if the given 64-bit widget tag belongs to the Stats & Telemetry panel subsystem.
#[inline]
pub fn is_stats_tag(tag: u64) -> bool {
    (tag & 0xFFFF_FFFF_0000_0000) == 0x5354_4154_0000_0000
}

/// Strongly typed semantic action resolver for Stats & Profiler tags.
#[inline]
pub fn resolve_stats_action(tag: u64) -> Option<StatsPanelAction> {
    match tag {
        STATS_TAG_TOGGLE_WIREFRAME => Some(StatsPanelAction::ToggleWireframe),
        STATS_TAG_TOGGLE_GRID => Some(StatsPanelAction::ToggleGrid),
        _ => None,
    }
}

/// Actions emitted by the Stats & Profiler panel interactions.
#[derive(Debug, Clone, PartialEq)]
pub enum StatsPanelAction {
    /// Toggles wireframe edge rendering.
    ToggleWireframe,
    /// Toggles world grid visibility.
    ToggleGrid,
}

/// Parameter context bundle passed into the declarative Stats & Profiler builder.
pub struct StatsPanelParams<'a> {
    /// Bounding rectangle allocated for the stats panel inside docking.
    pub panel_rect: Rect,
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
    /// CPU command buffer recording and pass orchestration durations.
    pub pass_recording_timings: &'a RenderPassRecordingTimings,
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
    /// Common panel interaction state (search, actions).
    pub interactions: crate::ui::iris_bridge::types::PanelInteractionState<(), StatsPanelAction>,
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
            interactions: crate::ui::iris_bridge::types::PanelInteractionState::default(),
            frame_counter: 0,
            last_fps_refresh: std::time::Instant::now(),
            displayed_fps: 60.0,
            frame_ring_buffer: None,
        }
    }
}

/// Semantic alias for [`StatsPanelState`].
pub type StatsState = StatsPanelState;

impl std::ops::Deref for StatsPanelState {
    type Target = crate::ui::iris_bridge::types::PanelInteractionState<(), StatsPanelAction>;
    fn deref(&self) -> &Self::Target {
        &self.interactions
    }
}

impl std::ops::DerefMut for StatsPanelState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.interactions
    }
}
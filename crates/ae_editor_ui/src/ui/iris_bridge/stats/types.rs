// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Types
//!
//! Exposes parameter structures, persistent widget node handles, interactive hit targets,
//! and action enums for the retained-mode Stats & Profiler panel.

use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer, GpuPassTimings, VramStats,
};
use irisui::prelude::*;

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

/// Hit-testing targets for interactive elements in the Stats & Profiler panel.
#[derive(Debug, Default, Clone)]
pub struct StatsPanelTargets {
    /// Bounding rectangle of the entire stats panel.
    pub panel_rect: Rect,
    /// Checkbox target for wireframe rendering.
    pub wireframe_checkbox_rect: Option<Rect>,
    /// Checkbox target for grid rendering.
    pub grid_checkbox_rect: Option<Rect>,
}

/// Persistent widget node handles for the Stats & Profiler panel in retained mode.
#[derive(Debug, Clone)]
pub struct StatsPanelNodes {
    /// Root node of the stats panel container.
    pub root_id: WidgetId,
    /// 2x2 Metric pills value node IDs (`[Avg FPS, 1% Low, 0.1% Low, Jitter]`).
    pub metric_pill_val_ids: [WidgetId; 4],
    /// Pacing summary footer text node ID.
    pub pacing_footer_id: WidgetId,

    /// CPU Thread Balance value node ID.
    pub cpu_tb_val_id: WidgetId,
    /// CPU multi-segmented bar track node ID and fill segment node IDs.
    pub cpu_bar_seg_ids: [WidgetId; 5],
    /// CPU 5 subsystem timing row value node IDs.
    pub cpu_timing_val_ids: [WidgetId; 5],
    /// CPU total frame value node ID.
    pub cpu_total_val_id: WidgetId,

    /// GPU device name text node ID.
    pub gpu_dev_id: WidgetId,
    /// GPU multi-segmented bar track fill segment node IDs.
    pub gpu_bar_seg_ids: [WidgetId; 4],
    /// GPU 4 pass timing row value node IDs.
    pub gpu_pass_val_ids: [WidgetId; 4],
    /// GPU total workload value node ID.
    pub gpu_total_val_id: WidgetId,

    /// Scene geometry: Draw Calls value node ID.
    pub dc_val_id: WidgetId,
    /// Scene geometry: Instanced ratio value node ID.
    pub inst_pct_id: WidgetId,
    /// Scene geometry: 4 subrow value node IDs (Batched, Instanced, Compute, Culled).
    pub dc_subrow_val_ids: [WidgetId; 4],
    /// Scene geometry: Triangles value node ID.
    pub triangles_val_id: WidgetId,
    /// Scene geometry: Vertices value node ID.
    pub vertices_val_id: WidgetId,
    /// Scene geometry: Entities value node ID.
    pub entities_val_id: WidgetId,

    /// VRAM multi-segmented bar track fill segment node IDs.
    pub vram_bar_seg_ids: [WidgetId; 3],
    /// VRAM 3 row value node IDs.
    pub vram_row_val_ids: [WidgetId; 3],
    /// VRAM total allocated value node ID.
    pub vram_total_val_id: WidgetId,

    /// Wireframe checkbox box node ID and checkmark text node ID.
    pub wireframe_box_id: WidgetId,
    pub wireframe_check_id: WidgetId,
    /// Grid checkbox box node ID and checkmark text node ID.
    pub grid_box_id: WidgetId,
    pub grid_check_id: WidgetId,

    /// Cached bounding rects for bar layouts.
    pub canvas_rect: Rect,
    pub cpu_bar_rect: Rect,
    pub gpu_bar_rect: Rect,
    pub vram_bar_rect: Rect,
}

/// Parameter context bundle passed into the Stats & Profiler builder.
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
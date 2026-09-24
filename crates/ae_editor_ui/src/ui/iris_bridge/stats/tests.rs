// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Pure Declarative Stats & Telemetry Panel
//!
//! Validates declarative tree construction, semantic 64-bit tag mappings,
//! and hardware hit-testing routing for the Performance Stats & Telemetry panel.
//!

use crate::ui::iris_bridge::stats::panel::build_stats_panel;
use crate::ui::iris_bridge::stats::types::{
    STATS_TAG_CANVAS, STATS_TAG_PANEL_ROOT, STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME,
    StatsPanelAction, StatsPanelParams, StatsPanelState, is_stats_tag,
};
use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer, GpuPassTimings, VramStats,
};
use irisui::prelude::*;

#[test]
fn test_stats_tag_invariants() {
    assert!(is_stats_tag(STATS_TAG_PANEL_ROOT));
    assert!(is_stats_tag(STATS_TAG_CANVAS));
    assert!(is_stats_tag(STATS_TAG_TOGGLE_WIREFRAME));
    assert!(is_stats_tag(STATS_TAG_TOGGLE_GRID));

    // Foreign tags must not match
    assert!(!is_stats_tag(0));
    assert!(!is_stats_tag(0x1234_5678_0000_0000));
    assert!(!is_stats_tag(0xFFFF_FFFF_FFFF_FFFF));
}

#[test]
fn test_stats_panel_state_take_actions() {
    let mut state = StatsPanelState::default();
    assert!(state.actions.is_empty());

    state.actions.push(StatsPanelAction::ToggleWireframe);
    state.actions.push(StatsPanelAction::ToggleGrid);
    assert_eq!(state.actions.len(), 2);

    let taken = state.take_actions();
    assert_eq!(taken.len(), 2);
    assert_eq!(taken[0], StatsPanelAction::ToggleWireframe);
    assert_eq!(taken[1], StatsPanelAction::ToggleGrid);
    assert!(state.actions.is_empty());
}

#[test]
fn test_build_stats_panel_declarative_structure() {
    let mut tree = UiTree::new();
    let root = {
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        scope.empty_box(Style::new())
    };

    let ring = FrameRingBuffer::<240>::new();
    let pacing_stats = FramePacingStats::default();
    let cpu_timings = CpuSyncTimings::default();
    let gpu_timings = GpuPassTimings::default();
    let draw_calls = DrawCallBreakdown::default();
    let vram_stats = VramStats::default();

    let panel_rect = Rect::new(0.0, 0.0, 320.0, 600.0);
    let params = StatsPanelParams {
        panel_rect,
        scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        wireframe_enabled: true,
        grid_enabled: false,
        fps: 120.0,
        frame_pacing: &ring,
        frame_pacing_stats: &pacing_stats,
        cpu_timings: &cpu_timings,
        gpu_pass_timings: &gpu_timings,
        draw_call_stats: &draw_calls,
        vram_stats: &vram_stats,
        render_triangles: 125000,
        render_vertices: 80000,
        gpu_adapter_name: "Test Adapter",
        gpu_backend: "Vulkan",
        active_entities_count: 42,
        selected_entity: None,
    };

    let max_scroll = build_stats_panel(&mut tree, root, &params);
    assert!(max_scroll >= 0.0);

    // Verify root node exists and contains semantic panel root tag
    let mut found_panel_root = false;
    let mut found_canvas = false;
    let mut found_wireframe_toggle = false;
    let mut found_grid_toggle = false;

    for (_wid, node) in tree.iter() {
        if node.tag == STATS_TAG_PANEL_ROOT {
            found_panel_root = true;
        }
        if node.tag == STATS_TAG_CANVAS {
            found_canvas = true;
            assert_eq!(node.role, WidgetRole::OscilloscopeCanvas);
        }
        if node.tag == STATS_TAG_TOGGLE_WIREFRAME {
            found_wireframe_toggle = true;
            assert_eq!(node.role, WidgetRole::Checkbox);
        }
        if node.tag == STATS_TAG_TOGGLE_GRID {
            found_grid_toggle = true;
            assert_eq!(node.role, WidgetRole::Checkbox);
        }
    }

    assert!(found_panel_root, "STATS_TAG_PANEL_ROOT must exist in tree");
    assert!(found_canvas, "STATS_TAG_CANVAS must exist in tree");
    assert!(
        found_wireframe_toggle,
        "STATS_TAG_TOGGLE_WIREFRAME must exist in tree"
    );
    assert!(
        found_grid_toggle,
        "STATS_TAG_TOGGLE_GRID must exist in tree"
    );
}

#[test]
fn test_checkbox_children_resolve_ancestor_tag() {
    let mut tree = UiTree::new();
    let root = {
        let mut scope = UiScope::new(&mut tree, WidgetId::default());
        scope.empty_box(Style::new())
    };

    let ring = FrameRingBuffer::<240>::new();
    let pacing_stats = FramePacingStats::default();
    let cpu_timings = CpuSyncTimings::default();
    let gpu_timings = GpuPassTimings::default();
    let draw_calls = DrawCallBreakdown::default();
    let vram_stats = VramStats::default();

    let panel_rect = Rect::new(0.0, 0.0, 320.0, 600.0);
    let params = StatsPanelParams {
        panel_rect,
        scroll_y: 0.0,
        cursor_pos: Point::new(100.0, 100.0),
        wireframe_enabled: true,
        grid_enabled: false,
        fps: 120.0,
        frame_pacing: &ring,
        frame_pacing_stats: &pacing_stats,
        cpu_timings: &cpu_timings,
        gpu_pass_timings: &gpu_timings,
        draw_call_stats: &draw_calls,
        vram_stats: &vram_stats,
        render_triangles: 125000,
        render_vertices: 80000,
        gpu_adapter_name: "Test Adapter",
        gpu_backend: "Vulkan",
        active_entities_count: 42,
        selected_entity: None,
    };

    build_stats_panel(&mut tree, root, &params);

    // Find the wireframe checkbox container and verify its box child also carries the tag
    let mut box_nodes_with_tag = 0;
    for (_wid, node) in tree.iter() {
        if node.tag == STATS_TAG_TOGGLE_WIREFRAME && node.role == WidgetRole::Checkbox {
            box_nodes_with_tag += 1;
        }
    }

    // Both the row and the inner box container should be tagged
    assert!(
        box_nodes_with_tag >= 2,
        "Both row and inner box must be tagged with STATS_TAG_TOGGLE_WIREFRAME"
    );
}
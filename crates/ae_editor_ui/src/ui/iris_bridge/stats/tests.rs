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

    let mut actions = Vec::new();
    let max_scroll = build_stats_panel(&mut tree, root, &params, &mut actions);
    assert!(max_scroll >= 0.0);

    // Verify nodes exist with semantic tags
    assert!(
        tree.node_tagged(STATS_TAG_PANEL_ROOT).is_some(),
        "STATS_TAG_PANEL_ROOT must exist in tree"
    );
    assert_eq!(
        tree.node_tagged(STATS_TAG_CANVAS)
            .expect("STATS_TAG_CANVAS must exist in tree")
            .role,
        WidgetRole::OscilloscopeCanvas
    );
    assert_eq!(
        tree.node_tagged(STATS_TAG_TOGGLE_WIREFRAME)
            .expect("STATS_TAG_TOGGLE_WIREFRAME must exist in tree")
            .role,
        WidgetRole::Checkbox
    );
    assert_eq!(
        tree.node_tagged(STATS_TAG_TOGGLE_GRID)
            .expect("STATS_TAG_TOGGLE_GRID must exist in tree")
            .role,
        WidgetRole::Checkbox
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

    let mut actions = Vec::new();
    build_stats_panel(&mut tree, root, &params, &mut actions);

    // Find the wireframe checkbox and verify it carries the semantic tag and role
    let mut box_nodes_with_tag = 0;
    for (_wid, node) in tree.iter() {
        if node.tag == STATS_TAG_TOGGLE_WIREFRAME && node.role == WidgetRole::Checkbox {
            box_nodes_with_tag += 1;
        }
    }

    assert_eq!(
        box_nodes_with_tag, 1,
        "Declarative checkbox must emit a single tagged checkbox node"
    );
}

#[test]
fn test_stats_panel_state_ring_buffer_snapshot() {
    let mut state = StatsPanelState::default();
    assert!(state.frame_ring_buffer.is_none());

    let mut ring = FrameRingBuffer::new();
    ring.push(8.33);
    ring.push(8.25);
    state.frame_ring_buffer = Some(ring);

    assert!(state.frame_ring_buffer.is_some());
    let copied = state.frame_ring_buffer.unwrap();
    assert_eq!(copied.count(), 2);
}

#[test]
fn test_stats_panel_declarative_action_generation() {
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
        fps: 60.0,
        frame_pacing: &ring,
        frame_pacing_stats: &pacing_stats,
        cpu_timings: &cpu_timings,
        gpu_pass_timings: &gpu_timings,
        draw_call_stats: &draw_calls,
        vram_stats: &vram_stats,
        render_triangles: 5000,
        render_vertices: 3000,
        gpu_adapter_name: "MockGPU",
        gpu_backend: "Vulkan",
        active_entities_count: 10,
        selected_entity: None,
    };

    let mut actions = Vec::new();
    build_stats_panel(&mut tree, root, &params, &mut actions);

    // Initial build without clicks produces zero actions
    assert!(actions.is_empty());

    // When clicked via event state, actions are collected
    tree.event_state.frame_clicked_tag = Some(STATS_TAG_TOGGLE_WIREFRAME);

    let mut clicked_actions = Vec::new();
    build_stats_panel(&mut tree, root, &params, &mut clicked_actions);

    assert_eq!(clicked_actions.len(), 1);
    assert_eq!(clicked_actions[0], StatsPanelAction::ToggleWireframe);
}
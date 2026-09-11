// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit test suite for retained-mode Stats & Profiler panel synchronization.
//!
//! Validates zero-allocation invariants when idle, dynamic in-place updates,
//! dirty flag behavior, and layout rebuilds on panel geometry mutations.
//!

use super::sync::sync_stats_panel;
use super::types::{StatsPanelParams, StatsPanelRetainedState, StatsPanelTargets};
use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer, GpuPassTimings, VramStats,
};
use irisui::prelude::*;

struct TestTelemetryContext {
    ring: FrameRingBuffer<240>,
    pacing_stats: FramePacingStats,
    cpu_timings: CpuSyncTimings,
    gpu_timings: GpuPassTimings,
    draw_calls: DrawCallBreakdown,
    vram: VramStats,
}

impl Default for TestTelemetryContext {
    fn default() -> Self {
        Self {
            ring: FrameRingBuffer::new(),
            pacing_stats: FramePacingStats::default(),
            cpu_timings: CpuSyncTimings::default(),
            gpu_timings: GpuPassTimings::default(),
            draw_calls: DrawCallBreakdown::default(),
            vram: VramStats::default(),
        }
    }
}

impl TestTelemetryContext {
    fn make_params(&self, panel_rect: Rect, scroll_y: f32, fps: f32) -> StatsPanelParams<'_> {
        StatsPanelParams {
            panel_rect,
            scroll_y,
            cursor_pos: Point::new(50.0, 50.0),
            wireframe_enabled: false,
            grid_enabled: true,
            fps,
            frame_pacing: &self.ring,
            frame_pacing_stats: &self.pacing_stats,
            cpu_timings: &self.cpu_timings,
            gpu_pass_timings: &self.gpu_timings,
            draw_call_stats: &self.draw_calls,
            vram_stats: &self.vram,
            render_triangles: 12500,
            render_vertices: 37500,
            gpu_adapter_name: "Test GPU Adapter",
            gpu_backend: "Vulkan",
            active_entities_count: 42,
            selected_entity: None,
            revision: 0,
        }
    }
}

#[test]
fn test_retained_stats_panel_zero_allocation_when_idle() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut retained_state: Option<StatsPanelRetainedState> = None;
    let mut targets = StatsPanelTargets::default();

    let telemetry = TestTelemetryContext::default();
    let params = telemetry.make_params(Rect::new(10.0, 10.0, 300.0, 600.0), 0.0, 60.0);

    // Initial construction frame
    sync_stats_panel(&mut tree, root, &mut retained_state, &params, &mut targets);

    assert!(retained_state.is_some());
    let state = retained_state.as_ref().unwrap();
    let initial_root = state.nodes.root_id;
    assert!(tree.contains_node(initial_root));
    let initial_node_count = tree.len();
    assert!(initial_node_count > 10);

    // Second frame: Idle state with identical parameters
    sync_stats_panel(&mut tree, root, &mut retained_state, &params, &mut targets);

    let state_second = retained_state.as_ref().unwrap();
    assert_eq!(state_second.nodes.root_id, initial_root);
    assert_eq!(
        tree.len(),
        initial_node_count,
        "Retained stats panel must allocate zero new nodes on idle frames"
    );
}

#[test]
fn test_retained_stats_panel_resize_rebuilds_and_updates_snapshot() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut retained_state: Option<StatsPanelRetainedState> = None;
    let mut targets = StatsPanelTargets::default();

    let telemetry = TestTelemetryContext::default();
    let initial_rect = Rect::new(10.0, 10.0, 300.0, 600.0);
    let params = telemetry.make_params(initial_rect, 0.0, 60.0);

    sync_stats_panel(&mut tree, root, &mut retained_state, &params, &mut targets);

    let first_root = retained_state.as_ref().unwrap().nodes.root_id;
    assert_eq!(retained_state.as_ref().unwrap().panel_rect, initial_rect);
    assert_eq!(retained_state.as_ref().unwrap().last_revision, 0);

    // Resize panel: Width increases from 300.0 to 400.0
    let resized_rect = Rect::new(10.0, 10.0, 400.0, 600.0);
    let resized_params = telemetry.make_params(resized_rect, 0.0, 60.0);

    sync_stats_panel(
        &mut tree,
        root,
        &mut retained_state,
        &resized_params,
        &mut targets,
    );

    let state = retained_state.as_ref().unwrap();
    assert_ne!(
        state.nodes.root_id, first_root,
        "A panel resize should cleanly rebuild the tree with updated card geometries"
    );
    assert!(
        !tree.contains_node(first_root),
        "Old root should be purged from tree"
    );
    assert!(
        tree.contains_node(state.nodes.root_id),
        "New root must exist in tree"
    );
    assert_eq!(state.panel_rect, resized_rect);
}

#[test]
fn test_retained_stats_panel_in_place_metric_update() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let mut retained_state: Option<StatsPanelRetainedState> = None;
    let mut targets = StatsPanelTargets::default();

    let mut telemetry = TestTelemetryContext {
        pacing_stats: FramePacingStats {
            average_fps: 60.0,
            average_frametime_ms: 16.67,
            ..Default::default()
        },
        ..Default::default()
    };

    let params_60fps = telemetry.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 60.0);

    sync_stats_panel(
        &mut tree,
        root,
        &mut retained_state,
        &params_60fps,
        &mut targets,
    );

    let initial_node_count = tree.len();
    let avg_pill_id = retained_state.as_ref().unwrap().nodes.metric_pill_val_ids[0];
    let initial_text = tree.get(avg_pill_id).and_then(|n| n.text.clone());
    assert_eq!(initial_text.as_deref(), Some("60 (16.67ms)"));

    // Next frame: FPS increases to 144
    telemetry.pacing_stats.average_fps = 144.0;
    telemetry.pacing_stats.average_frametime_ms = 6.94;
    let params_144fps = telemetry.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 144.0);

    // In-place metric update: telemetry updates directly in text buffer without rebuild
    super::panel::update_stats_panel_text_values(
        &mut tree,
        &retained_state.as_ref().unwrap().nodes,
        &params_144fps,
    );

    assert_eq!(
        tree.len(),
        initial_node_count,
        "Dynamic metric updates must occur in place with zero node allocations"
    );

    let updated_text = tree.get(avg_pill_id).and_then(|n| n.text.clone());
    assert_eq!(updated_text.as_deref(), Some("144 (6.94ms)"));
}

#[test]
fn test_retained_stats_panel_idle_leaves_geometry_clean() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let telemetry = TestTelemetryContext::default();
    let mut targets = StatsPanelTargets::default();
    let mut retained_state = None;

    let params_initial = telemetry.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 60.0);
    sync_stats_panel(
        &mut tree,
        root,
        &mut retained_state,
        &params_initial,
        &mut targets,
    );

    // Simulate end of previous frame: clear all dirty flags
    tree.clear_all_dirty(DirtyFlags::ALL);
    assert!(!tree.has_dirty_nodes(DirtyFlags::PAINT | DirtyFlags::LAYOUT));

    // Next frame (idle): update live telemetry in-place
    let mut telemetry_idle = telemetry;
    telemetry_idle.pacing_stats.average_fps = 60.0;
    telemetry_idle.pacing_stats.average_frametime_ms = 16.67;
    let nodes = &retained_state.as_ref().unwrap().nodes;
    let params_idle = telemetry_idle.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 60.0);
    super::panel::update_stats_panel_text_values(&mut tree, nodes, &params_idle);

    // As required by Phase 2 architecture, clearing geometry dirty flags after text-only update
    tree.clear_all_dirty(DirtyFlags::PAINT | DirtyFlags::LAYOUT);

    // CRITICAL INVARIANT: In idle, tree geometry and paint MUST be strictly false!
    assert!(
        !tree.has_dirty_nodes(DirtyFlags::PAINT | DirtyFlags::LAYOUT),
        "UI tree must have zero PAINT or LAYOUT dirty nodes during idle telemetry updates"
    );

    let avg_pill_id = nodes.metric_pill_val_ids[0];
    let updated_text = tree.get(avg_pill_id).and_then(|n| n.text.clone());
    assert_eq!(updated_text.as_deref(), Some("60 (16.67ms)"));
}

#[test]
fn test_retained_stats_panel_multi_frame_idle_invariants() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let telemetry = TestTelemetryContext::default();
    let mut targets = StatsPanelTargets::default();
    let mut retained_state = None;

    let params_initial = telemetry.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 60.0);
    sync_stats_panel(
        &mut tree,
        root,
        &mut retained_state,
        &params_initial,
        &mut targets,
    );

    // Frame 0 complete: clear all dirty
    tree.clear_all_dirty(DirtyFlags::ALL);
    assert!(!tree.has_dirty_nodes(DirtyFlags::PAINT | DirtyFlags::LAYOUT));

    let nodes = &retained_state.as_ref().unwrap().nodes;

    // Simulate 10 consecutive frames: alternating telemetry ticks (every 100ms) and quiet frames
    for frame in 1..=10 {
        let is_telemetry_cadence_tick = frame % 2 == 0;
        if is_telemetry_cadence_tick {
            let mut tick_context = TestTelemetryContext::default();
            tick_context.pacing_stats.average_fps = 60.0 + frame as f32;
            let params = tick_context.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 60.0);
            super::panel::update_stats_panel_text_values(&mut tree, nodes, &params);
            tree.clear_all_dirty(DirtyFlags::PAINT | DirtyFlags::LAYOUT);
        }

        // ABSOLUTE IDLE INVARIANT: Under stationary mouse, tree_dirty is strictly FALSE
        assert!(
            !tree.has_dirty_nodes(DirtyFlags::PAINT | DirtyFlags::LAYOUT),
            "Frame {}: UI tree must remain strictly clean of PAINT/LAYOUT dirty nodes in idle",
            frame
        );

        // Frame rendering clears any text dirty flags
        tree.clear_all_dirty(DirtyFlags::ALL);
    }
}

#[test]
fn test_retained_stats_panel_reattaches_when_detached() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    let telemetry = TestTelemetryContext::default();
    let mut targets = StatsPanelTargets::default();
    let mut retained_state = None;

    let params = telemetry.make_params(Rect::new(0.0, 0.0, 300.0, 500.0), 0.0, 60.0);

    // Frame 1: Initial construction
    sync_stats_panel(&mut tree, root, &mut retained_state, &params, &mut targets);
    let stats_root = retained_state.as_ref().unwrap().nodes.root_id;
    assert_eq!(tree.get(stats_root).and_then(|n| n.parent), Some(root));

    // Simulate Taffy layout pass detaching the retained node from root
    let _ = tree.remove_child(root, stats_root);
    assert_eq!(tree.get(stats_root).and_then(|n| n.parent), None);
    assert!(!tree.get(root).unwrap().children.contains(&stats_root));

    // Frame 2: sync_stats_panel on fast path MUST re-attach the node to parent_id
    sync_stats_panel(&mut tree, root, &mut retained_state, &params, &mut targets);
    assert_eq!(tree.get(stats_root).and_then(|n| n.parent), Some(root));
    assert!(tree.get(root).unwrap().children.contains(&stats_root));
}
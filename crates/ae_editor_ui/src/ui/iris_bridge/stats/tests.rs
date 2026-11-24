// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Pure Declarative Stats & Telemetry Panel
//!
//! Validates declarative tree construction, semantic [`UiTag`] mappings,
//! zero-allocation stack text updates, and direct [`ActionBus`] dispatching.

use crate::ui::iris_bridge::stats::cpu_breakdown::update_cpu_breakdown_texts;
use crate::ui::iris_bridge::stats::graph::update_oscilloscope_texts;
use crate::ui::iris_bridge::stats::panel::build_stats_panel;
use crate::ui::iris_bridge::stats::types::{
    STATS_TAG_AVG_FPS, STATS_TAG_CANVAS, STATS_TAG_CPU_BALANCE, STATS_TAG_CPU_TOTAL,
    STATS_TAG_PANEL_ROOT, STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME, STATS_TAG_VIEWPORT,
    StatsPanelState, is_stats_tag,
};
use crate::ui::types::{EditorTelemetrySnapshot, EngineUiAction};
use ae_diagnostic::{CpuSyncTimings, FramePacingStats, FrameRingBuffer};
use irisui::prelude::*;
use irisui::workbench::ActionBus;

#[test]
fn test_stats_tag_invariants() {
    assert!(is_stats_tag(STATS_TAG_PANEL_ROOT));
    assert!(is_stats_tag(STATS_TAG_CANVAS));
    assert!(is_stats_tag(STATS_TAG_TOGGLE_WIREFRAME));
    assert!(is_stats_tag(STATS_TAG_TOGGLE_GRID));
    assert_ne!(STATS_TAG_TOGGLE_WIREFRAME, STATS_TAG_TOGGLE_GRID);

    assert!(!is_stats_tag(0));
    assert!(!is_stats_tag(0x1234_5678_0000_0000));
    assert!(!is_stats_tag(0xFFFF_FFFF_FFFF_FFFF));
}

#[test]
fn test_build_stats_panel_declarative_structure() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let telemetry = EditorTelemetrySnapshot {
        render_triangles: 125_000,
        render_vertices: 80_000,
        ..Default::default()
    };

    let mut bus = ActionBus::new();
    crate::ui::panel_layout::PanelId::Stats
        .surface()
        .expect("Stats surface")
        .show(&mut UiScope::new(&mut tree, root), |s| {
            build_stats_panel(
                s,
                &telemetry,
                ("Test Adapter", "Vulkan"),
                42,
                (true, false),
                &mut bus,
            );
        });
    let max_scroll = tree.max_scroll(STATS_TAG_VIEWPORT);
    assert!(max_scroll >= 0.0);

    assert!(tree.node_tagged(STATS_TAG_PANEL_ROOT).is_some());
    let canvas_node = tree
        .node_tagged(STATS_TAG_CANVAS)
        .expect("STATS_TAG_CANVAS must exist in tree");
    assert_eq!(canvas_node.role, WidgetRole::ImmediateCanvas);
    assert!(tree.canvas_primitives(canvas_node.id).is_some());
    assert_eq!(
        tree.node_tagged(STATS_TAG_TOGGLE_WIREFRAME)
            .expect("STATS_TAG_TOGGLE_WIREFRAME must exist in tree")
            .role,
        WidgetRole::Checkbox
    );
    assert!(tree.node_tagged(STATS_TAG_AVG_FPS).is_some());
    assert!(tree.node_tagged(STATS_TAG_CPU_BALANCE).is_some());
    assert_eq!(
        tree.node_tagged(STATS_TAG_TOGGLE_GRID)
            .expect("STATS_TAG_TOGGLE_GRID must exist in tree")
            .role,
        WidgetRole::Checkbox
    );
}

#[test]
fn test_stats_panel_state_ring_buffer_snapshot_and_live_text_refresh() {
    let mut state = StatsPanelState::default();
    assert!(state.frame_ring_buffer.is_none());

    let mut ring = FrameRingBuffer::new();
    ring.push(8.33);
    ring.push(8.25);
    state.frame_ring_buffer = Some(ring);
    assert_eq!(state.frame_ring_buffer.unwrap().count(), 2);

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root");
    let mut bus = ActionBus::new();
    let mut cpu_timings = CpuSyncTimings::default();
    let telemetry = EditorTelemetrySnapshot {
        frame_pacing: ring,
        render_triangles: 100,
        render_vertices: 50,
        ..Default::default()
    };
    build_stats_panel(
        &mut UiScope::new(&mut tree, root),
        &telemetry,
        ("TestGpu", "Vulkan"),
        1,
        (false, true),
        &mut bus,
    );

    cpu_timings.main_logic_ms = 5.5;
    let next_pacing = FramePacingStats {
        average_fps: 144.0,
        average_frametime_ms: 6.94,
        ..Default::default()
    };
    let mut scope = UiScope::new(&mut tree, root);
    assert!(update_oscilloscope_texts(&mut scope, &next_pacing));
    assert!(update_cpu_breakdown_texts(&mut scope, &cpu_timings));
    assert_eq!(
        tree.node_tagged(STATS_TAG_AVG_FPS)
            .and_then(|n| n.text.as_deref()),
        Some("144 (6.94ms)")
    );
    assert_eq!(
        tree.node_tagged(STATS_TAG_CPU_TOTAL)
            .and_then(|n| n.text.as_deref()),
        Some("5.50 ms")
    );
}

#[test]
fn test_stats_panel_declarative_action_bus_emission() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let telemetry = EditorTelemetrySnapshot {
        render_triangles: 5000,
        render_vertices: 3000,
        ..Default::default()
    };

    let mut initial_bus = ActionBus::new();
    build_stats_panel(
        &mut UiScope::new(&mut tree, root),
        &telemetry,
        ("MockGPU", "Vulkan"),
        10,
        (true, false),
        &mut initial_bus,
    );
    assert!(initial_bus.is_empty());

    tree.event_state.frame_clicked_tag = Some(STATS_TAG_TOGGLE_WIREFRAME);
    let mut clicked_bus = ActionBus::new();
    build_stats_panel(
        &mut UiScope::new(&mut tree, root),
        &telemetry,
        ("MockGPU", "Vulkan"),
        10,
        (true, false),
        &mut clicked_bus,
    );

    let drained: Vec<EngineUiAction> = clicked_bus.drain().collect();
    assert_eq!(drained.len(), 1);
    assert!(matches!(drained[0], EngineUiAction::SetWireframe(false)));
}
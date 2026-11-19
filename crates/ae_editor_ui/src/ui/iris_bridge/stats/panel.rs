// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Orchestrator
//!
//! Assembles the root panel container, scrollable telemetry viewport, and
//! individual telemetry cards purely using [`UiScope`] and percentage-flex cards.

use super::cpu_breakdown::build_cpu_breakdown_content;
use super::gpu_breakdown::build_gpu_breakdown_content;
use super::graph::build_frame_pacing_content;
use super::metrics::{build_scene_geometry_content, build_vram_breakdown_content};
use super::overlays::build_viewport_overlays_content;
use super::types::{STATS_TAG_PANEL_ROOT, STATS_TAG_VIEWPORT, StatsPanelParams};
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Builds the complete Performance Stats & Telemetry panel tree purely using [`UiScope`].
pub fn build_stats_panel(
    scope: &mut UiScope<'_>,
    params: &StatsPanelParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope
        .panel("StatsPanelRoot")
        .tagged(STATS_TAG_PANEL_ROOT)
        .background(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .show(|panel_scope| {
            panel_scope.scroll_area("StatsViewport", STATS_TAG_VIEWPORT, None, |vp_scope| {
                vp_scope.telemetry_card("📈", "Frame Pacing & Stutter Analyzer", |card_scope| {
                    build_frame_pacing_content(card_scope, params);
                });
                vp_scope.telemetry_card("⏱", "CPU Thread & Synchronization", |card_scope| {
                    build_cpu_breakdown_content(card_scope, params);
                });
                vp_scope.telemetry_card("⚡", "GPU Render Passes", |card_scope| {
                    build_gpu_breakdown_content(card_scope, params);
                });
                vp_scope.telemetry_card("📐", "Scene & Geometry Metrics", |card_scope| {
                    build_scene_geometry_content(card_scope, params);
                });
                vp_scope.telemetry_card("💾", "Video RAM & Memory Allocations", |card_scope| {
                    build_vram_breakdown_content(card_scope, params);
                });
                vp_scope.telemetry_card("🎛", "Viewport Overlays", |card_scope| {
                    build_viewport_overlays_content(card_scope, params, action_bus);
                });
            });
        });
}
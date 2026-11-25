// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Orchestrator
//!
//! Assembles the root panel container, scrollable telemetry viewport, and
//! individual telemetry cards purely using [`UiScope`] and percentage-flex cards.

use super::cpu_breakdown::{build_cpu_breakdown_content, update_cpu_breakdown_texts};
use super::gpu_breakdown::build_gpu_breakdown_content;
use super::graph::{build_frame_pacing_content, refresh_live_oscilloscope};
use super::metrics::{build_scene_geometry_content, build_vram_breakdown_content};
use super::overlays::{ViewportOverlaySettings, build_viewport_overlays_content};
use super::types::{STATS_TAG_VIEWPORT, StatsPanelState};
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::{EditorTelemetrySnapshot, EngineUiAction};
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl StatsPanelState {
    /// Draws the Performance Stats & Telemetry profiler panel directly inside the active dock leaf.
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        build_stats_panel(
            scope,
            ctx.data.telemetry,
            (ctx.data.gpu_adapter_name, ctx.data.gpu_backend),
            ctx.data.world.len() as usize,
            ViewportOverlaySettings {
                wireframe_enabled: ctx.data.editor_state.wireframe_enabled,
                grid_enabled: ctx.data.editor_state.grid_enabled,
                show_all_colliders: ctx.data.editor_state.show_all_colliders,
            },
            ctx.action_bus,
        );
        self.frame_ring_buffer = Some(ctx.data.telemetry.frame_pacing);
    }

    /// Performs in-place oscilloscope waveform and CPU timing text refreshes on scheduled ticks.
    pub(crate) fn refresh_panel(
        &mut self,
        scope: &mut UiScope<'_>,
        ctx: &mut EditorPanelContext<'_>,
    ) {
        if refresh_live_oscilloscope(
            scope,
            &ctx.data.telemetry.frame_pacing,
            &ctx.data.telemetry.frame_pacing_stats,
        ) | update_cpu_breakdown_texts(scope, &ctx.data.telemetry.cpu_timings)
        {
            self.frame_ring_buffer = Some(ctx.data.telemetry.frame_pacing);
        }
    }
}

/// Builds the complete Performance Stats & Telemetry panel tree purely using [`UiScope`].
pub fn build_stats_panel(
    scope: &mut UiScope<'_>,
    telemetry: &EditorTelemetrySnapshot,
    (gpu_adapter_name, gpu_backend): (&str, &str),
    active_entities_count: usize,
    overlays: ViewportOverlaySettings,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.scroll_area("StatsViewport", STATS_TAG_VIEWPORT, None, |vp_scope| {
        vp_scope.telemetry_card("📈", "Frame Pacing & Stutter Analyzer", |card_scope| {
            build_frame_pacing_content(
                card_scope,
                &telemetry.frame_pacing,
                &telemetry.frame_pacing_stats,
            );
        });
        vp_scope.telemetry_card("⏱", "CPU Thread & Synchronization", |card_scope| {
            build_cpu_breakdown_content(card_scope, &telemetry.cpu_timings);
        });
        vp_scope.telemetry_card("⚡", "GPU Render Passes", |card_scope| {
            build_gpu_breakdown_content(
                card_scope,
                &telemetry.pass_recording_timings,
                gpu_adapter_name,
                gpu_backend,
            );
        });
        vp_scope.telemetry_card("📐", "Scene & Geometry Metrics", |card_scope| {
            build_scene_geometry_content(
                card_scope,
                &telemetry.draw_call_stats,
                telemetry.render_triangles,
                telemetry.render_vertices,
                active_entities_count,
            );
        });
        vp_scope.telemetry_card("💾", "Video RAM & Memory Allocations", |card_scope| {
            build_vram_breakdown_content(card_scope, &telemetry.vram_stats);
        });
        vp_scope.telemetry_card("🎛", "Viewport Overlays", |card_scope| {
            build_viewport_overlays_content(card_scope, overlays, action_bus);
        });
    });
}
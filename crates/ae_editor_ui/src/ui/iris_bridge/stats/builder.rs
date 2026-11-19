// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Profiler Builder
//!
//! Autonomous builder coordinating telemetry panel presentation and
//! periodic [`ScopedWorkbenchPanel::refresh_live`](irisui::workbench::ScopedWorkbenchPanel::refresh_live)
//! oscilloscope & CPU readout refreshes.

use super::cpu_breakdown::update_cpu_breakdown_texts;
use super::graph::refresh_live_oscilloscope;
use super::panel::build_stats_panel;
use super::types::{STATS_TAG_CANVAS, StatsPanelParams, StatsPanelState};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use crate::ui::types::EngineUiAction;
use ae_core::telemetry::{CpuSyncTimings, FramePacingStats, FrameRingBuffer};
use irisui::prelude::*;
use irisui::workbench::ActionBus;

impl StatsPanelState {
    /// Builds the Performance Stats & Telemetry profiler panel directly inside the active dock leaf.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        action_bus: &mut ActionBus<EngineUiAction>,
    ) {
        if scope.is_collapsed() {
            self.frame_ring_buffer = None;
            return;
        }

        build_stats_panel(
            scope,
            &StatsPanelParams {
                wireframe_enabled: params.viewport.wireframe_enabled,
                grid_enabled: params.viewport.grid_enabled,
                fps: self.displayed_fps,
                frame_pacing: params.telemetry.frame_pacing,
                frame_pacing_stats: params.telemetry.frame_pacing_stats,
                cpu_timings: params.telemetry.cpu_timings,
                pass_recording_timings: params.telemetry.pass_recording_timings,
                draw_call_stats: params.telemetry.draw_call_stats,
                vram_stats: params.telemetry.vram_stats,
                render_triangles: params.telemetry.render_triangles as u64,
                render_vertices: params.telemetry.render_vertices as u64,
                gpu_adapter_name: params.telemetry.gpu_adapter_name,
                gpu_backend: params.telemetry.gpu_backend,
                active_entities_count: params.scene.active_entities_count,
                selected_entity: params.scene.selected_entity,
            },
            action_bus,
        );
        self.frame_ring_buffer = Some(*params.telemetry.frame_pacing);
    }

    /// Performs in-place oscilloscope waveform and CPU timing text refreshes on scheduled ticks.
    pub(crate) fn refresh_live(
        &mut self,
        scope: &mut UiScope<'_>,
        frame_pacing: &FrameRingBuffer<240>,
        frame_pacing_stats: &FramePacingStats,
        cpu_timings: &CpuSyncTimings,
    ) {
        if scope.node_id_tagged(STATS_TAG_CANVAS).is_none() {
            return;
        }
        refresh_live_oscilloscope(scope, frame_pacing, frame_pacing_stats);
        update_cpu_breakdown_texts(scope, cpu_timings);
        self.frame_ring_buffer = Some(*frame_pacing);
    }
}
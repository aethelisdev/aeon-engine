// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Profiler Builder
//!
//! Autonomous builder coordinating telemetry panel geometry allocation,
//! ring buffer tracking, and frame metrics visualization.

use super::panel::build_stats_panel;
use super::types::StatsPanelParams;
use crate::ui::iris_bridge::types::{IrisEditorOverlay, OverlayUpdateParams};
use irisui::prelude::*;

impl IrisEditorOverlay {
    /// Builds the Performance Stats & Telemetry profiler panel if active in either docked or floating mode.
    ///
    /// Evaluates allocated physical geometry and constructs the complete telemetry tree:
    /// oscillograph frame pacing curve, CPU pass breakdown, GPU render passes, draw call counters,
    /// and VRAM memory footprint cards directly within unified docking coordinates.
    pub(crate) fn build_stats_panel_if_active(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
    ) {
        let panel_rect = scope.parent_rect();
        if panel_rect.width <= 20.0 || panel_rect.height <= 20.0 {
            self.stats.frame_ring_buffer = None;
            return;
        }

        let stats_params = StatsPanelParams {
            panel_rect,
            cursor_pos: self.cursor_pos(),
            wireframe_enabled: params.viewport.wireframe_enabled,
            grid_enabled: params.viewport.grid_enabled,
            fps: self.stats.displayed_fps,
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
        };

        build_stats_panel(scope, &stats_params, &mut self.stats.actions);
        self.stats.frame_ring_buffer = Some(*params.telemetry.frame_pacing);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Profiler Builder
//!
//! Autonomous builder coordinating telemetry panel geometry allocation,
//! ring buffer tracking, and frame metrics visualization.

use super::panel::build_stats_panel;
use super::types::{StatsPanelParams, StatsPanelState};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use irisui::prelude::*;

impl StatsPanelState {
    /// Builds the Performance Stats & Telemetry profiler panel directly inside the active dock leaf.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &OverlayUpdateParams<'_>,
        cursor_pos: Point,
    ) {
        if scope.is_collapsed() {
            self.frame_ring_buffer = None;
            return;
        }

        build_stats_panel(
            scope,
            &StatsPanelParams {
                panel_rect: scope.parent_rect(),
                cursor_pos,
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
            &mut self.actions,
        );
        self.frame_ring_buffer = Some(*params.telemetry.frame_pacing);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor Stats and High-Resolution Performance Profiler Panel.
//!
//! Orchestrates the frame pacing oscillograph, CPU thread synchronization breakdown,
//! GPU pass execution metrics, draw call distribution, and granular VRAM consumption cards.
//!

use crate::ui::EngineUi;
use ae_core::telemetry::{
    CpuSyncTimings, DrawCallBreakdown, FramePacingStats, FrameRingBuffer, GpuPassTimings, VramStats,
};

use super::breakdown::{draw_cpu_breakdown_card, draw_gpu_breakdown_card};
use super::graph::draw_frame_pacing_card;
use super::metrics::{SceneMetricsParams, draw_scene_geometry_card, draw_vram_breakdown_card};

/// Telemetry data and viewport overlay references passed into the Stats & Profiler panel.
pub struct StatsPanelContext<'a> {
    pub wireframe_enabled: &'a mut bool,
    pub grid_enabled: &'a mut bool,
    pub fps: f32,
    pub frame_pacing: &'a FrameRingBuffer<240>,
    pub frame_pacing_stats: FramePacingStats,
    pub cpu_timings: CpuSyncTimings,
    pub gpu_pass_timings: GpuPassTimings,
    pub draw_call_stats: DrawCallBreakdown,
    pub vram_stats: VramStats,
    pub render_triangles: u64,
    pub render_vertices: u64,
    pub gpu_adapter_name: &'a str,
    pub gpu_backend: &'a str,
    pub active_entities_count: usize,
    pub selected_entity: Option<hecs::Entity>,
}

/// Helper function to draw a section card with a dark background and subtle border.
pub fn draw_stats_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    icon: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let frame = egui::Frame::NONE
        .fill(egui::Color32::from_rgb(18, 20, 26))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 42, 54)))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(10, 8));

    frame
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} {}", icon, title))
                        .strong()
                        .color(egui::Color32::from_rgb(220, 225, 235)),
                );
            });
            ui.add_space(4.0);
            add_contents(ui)
        })
        .inner
}

impl EngineUi {
    /// Renders the internal content of the Stats & Performance Profiler panel.
    /// Presents real-time telemetry, 240-frame timeline oscillograph, CPU/GPU pass breakdowns,
    /// granular draw call metrics, and VRAM memory distribution.
    pub fn draw_stats_content(ui: &mut egui::Ui, ctx: StatsPanelContext<'_>) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);

            // 1. Frame Pacing & Stutter Analyzer Card
            draw_stats_card(ui, "Frame Pacing & Stutter Analyzer", "📈", |ui| {
                draw_frame_pacing_card(ui, ctx.frame_pacing, &ctx.frame_pacing_stats);
            });

            ui.add_space(6.0);

            // 2. CPU Thread & Synchronization Breakdown Card
            draw_stats_card(ui, "CPU Thread & Synchronization", "⏱", |ui| {
                draw_cpu_breakdown_card(ui, &ctx.cpu_timings, ctx.gpu_pass_timings.total_gpu_ms);
            });

            ui.add_space(6.0);

            // 3. GPU Render Passes Card
            draw_stats_card(ui, "GPU Render Passes", "⚡", |ui| {
                draw_gpu_breakdown_card(
                    ui,
                    &ctx.gpu_pass_timings,
                    ctx.gpu_adapter_name,
                    ctx.gpu_backend,
                );
            });

            ui.add_space(6.0);

            // 4. Scene & Geometry Metrics Card
            draw_stats_card(ui, "Scene & Geometry Metrics", "📐", |ui| {
                draw_scene_geometry_card(
                    ui,
                    SceneMetricsParams {
                        draw_calls: &ctx.draw_call_stats,
                        triangles: ctx.render_triangles,
                        vertices: ctx.render_vertices,
                        active_entities: ctx.active_entities_count,
                        selected_entity: ctx.selected_entity,
                    },
                );
            });

            ui.add_space(6.0);

            // 5. Video RAM & Memory Allocations Card
            draw_stats_card(ui, "Video RAM & Memory Allocations", "💾", |ui| {
                draw_vram_breakdown_card(ui, &ctx.vram_stats);
            });

            ui.add_space(6.0);

            // 6. Viewport Overlays Card
            draw_stats_card(ui, "Viewport Overlays", "🎛", |ui| {
                ui.checkbox(ctx.wireframe_enabled, "🕸 Wireframe Mode (Edges)");
                ui.checkbox(ctx.grid_enabled, "🔲 Show Grid");
            });

            ui.add_space(6.0);
        });
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance Stats & Telemetry Panel Orchestrator
//!
//! Assembles the root panel container, scrollable telemetry viewport, and
//! individual telemetry cards purely using [`UiScope`] and declarative flexbox primitives.
//!

use super::cpu_breakdown::build_cpu_breakdown_content;
use super::gpu_breakdown::build_gpu_breakdown_content;
use super::graph::build_frame_pacing_content;
use super::metrics::{build_scene_geometry_content, build_vram_breakdown_content};
use super::overlays::build_viewport_overlays_content;
use super::types::{STATS_TAG_PANEL_ROOT, StatsPanelAction, StatsPanelParams};
use irisui::prelude::*;

/// Builds the complete Performance Stats & Telemetry panel tree purely using [`UiScope`].
pub fn build_stats_panel(
    scope: &mut UiScope<'_>,
    params: &StatsPanelParams<'_>,
    actions: &mut Vec<StatsPanelAction>,
) {
    let card_w = (params.panel_rect.width - 12.0).max(10.0);

    scope
        .panel("StatsPanelRoot")
        .tagged(STATS_TAG_PANEL_ROOT)
        .background(Color::rgba(0.06, 0.07, 0.09, 1.0))
        .show(|panel_scope| {
            panel_scope.scroll_area(
                "StatsViewport",
                super::types::STATS_TAG_VIEWPORT,
                None,
                |vp_scope| {
                    // 1. Frame Pacing & Stutter Analyzer Card
                    build_stats_card(
                        vp_scope,
                        "📈",
                        "Frame Pacing & Stutter Analyzer",
                        card_w,
                        |card_scope| {
                            build_frame_pacing_content(card_scope, params);
                        },
                    );

                    // 2. CPU Thread & Synchronization Card
                    build_stats_card(
                        vp_scope,
                        "⏱",
                        "CPU Thread & Synchronization",
                        card_w,
                        |card_scope| {
                            build_cpu_breakdown_content(card_scope, params);
                        },
                    );

                    // 3. GPU Render Passes Card
                    build_stats_card(vp_scope, "⚡", "GPU Render Passes", card_w, |card_scope| {
                        build_gpu_breakdown_content(card_scope, params);
                    });

                    // 4. Scene & Geometry Metrics Card
                    build_stats_card(
                        vp_scope,
                        "📐",
                        "Scene & Geometry Metrics",
                        card_w,
                        |card_scope| {
                            build_scene_geometry_content(card_scope, params);
                        },
                    );

                    // 5. Video RAM & Memory Allocations Card
                    build_stats_card(
                        vp_scope,
                        "💾",
                        "Video RAM & Memory Allocations",
                        card_w,
                        |card_scope| {
                            build_vram_breakdown_content(card_scope, params);
                        },
                    );

                    // 6. Viewport Overlays Card
                    build_stats_card(vp_scope, "🎛", "Viewport Overlays", card_w, |card_scope| {
                        build_viewport_overlays_content(card_scope, params, actions);
                    });
                },
            );
        });
}

/// Helper function building a dark styled card container with an icon, title header, and content slot.
fn build_stats_card<F>(scope: &mut UiScope<'_>, icon: &str, title: &str, width: f32, content: F)
where
    F: FnOnce(&mut UiScope<'_>),
{
    let card_style = Style::new()
        .flex_col()
        .width(width)
        .gap(6.0)
        .padding_insets(Insets::new(8.0, 10.0, 8.0, 10.0))
        .background(Color::rgba(0.07, 0.08, 0.10, 0.95))
        .border(1.0, Color::rgba(0.15, 0.16, 0.21, 0.90))
        .border_radius(5.0);

    scope.container(card_style, |c| {
        // Card Header Row
        let header_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(6.0)
            .height(18.0);

        c.container(header_style, |h| {
            h.text_colored(icon, Color::rgba(0.86, 0.88, 0.92, 1.0));
            h.text_colored(title, Color::rgba(0.86, 0.88, 0.92, 1.0));
        });

        // Content Area
        content(c);
    });
}
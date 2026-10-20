// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Panel Root Builder
//!
//! Assembles the complete UI Designer panel hierarchy in the Iris UI tree,
//! including the virtual canvas, 34px elevated toolbar, and interactive dropdown popups
//! purely using declarative [`UiScope`] containers and 64-bit hardware semantic tags.
//!

use super::canvas::build_designer_canvas;
use super::popups::{build_add_element_popup, build_aspect_ratio_popup};
use super::toolbar::build_designer_toolbar;
use super::types::{
    UI_DESIGNER_TAG_PANEL_ROOT, UiDesignerCanvasMetrics, UiDesignerPanelParams,
    UiDesignerPanelState,
};
use irisui::prelude::*;

/// Assembles the complete 2D Visual UI Designer panel tree in native Iris UI.
pub fn build_ui_designer_panel(
    scope: &mut UiScope<'_>,
    params: &UiDesignerPanelParams<'_>,
    designer: &mut UiDesignerPanelState,
) -> UiDesignerCanvasMetrics {
    designer.drag_contexts.clear();

    let canvas_metrics = scope
        .panel("UiDesignerPanelRoot")
        .tagged(UI_DESIGNER_TAG_PANEL_ROOT)
        .bounds(params.panel_rect)
        .background(Color::rgba(0.065, 0.070, 0.082, 1.0))
        .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.70))
        .show(|panel_scope| {
            // ── 1. Virtual Canvas & Elements Viewport ──────────────────────────
            let metrics = build_designer_canvas(panel_scope, params, designer);

            // ── 2. Elevated 34px Top Control Bar ──────────────────────────────
            build_designer_toolbar(panel_scope, params, designer);
            metrics
        })
        .inner;

    designer.canvas_metrics = canvas_metrics;

    // ── 4. Dropdown Popups (Rendered on top in UiLayer::Popup via UiScope) ────
    build_ui_designer_overlays(scope, params, designer);

    canvas_metrics
}

/// Builds top-level floating popups (Aspect Ratio and Add Element palettes)
/// into the designated overlay or portal scope, preventing panel scissor clipping (Mine 2).
pub fn build_ui_designer_overlays(
    scope: &mut UiScope<'_>,
    params: &UiDesignerPanelParams<'_>,
    designer: &mut UiDesignerPanelState,
) {
    build_aspect_ratio_popup(scope, params, designer);
    build_add_element_popup(scope, params, designer);
}
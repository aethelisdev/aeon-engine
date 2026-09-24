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
    UiElementDragContext,
};
use irisui::prelude::*;

/// Assembles the complete 2D Visual UI Designer panel tree in native Iris UI.
pub fn build_ui_designer_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
    out_contexts: &mut Vec<UiElementDragContext>,
) -> UiDesignerCanvasMetrics {
    out_contexts.clear();

    let mut scope = UiScope::with_tagged_interactions(tree, parent_id, &[], params.hovered_tag);

    let root_style = Style::new()
        .flex_col()
        .background(Color::rgba(0.065, 0.070, 0.082, 1.0))
        .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.70))
        .clip_children(true);

    let mut canvas_metrics = UiDesignerCanvasMetrics::default();

    scope.container_tagged(
        "UiDesignerPanelRoot",
        root_style,
        WidgetRole::Default,
        UI_DESIGNER_TAG_PANEL_ROOT,
        |panel_scope| {
            // ── 1. Virtual Canvas & Elements Viewport ──────────────────────────
            canvas_metrics = build_designer_canvas(panel_scope, params, out_contexts);

            // ── 2. Elevated 34px Top Control Bar ──────────────────────────────
            build_designer_toolbar(panel_scope, params);

            // ── 3. Finalize Subtree Flow Layout to Assign Computed Rects ────────
            panel_scope.finish_layout(params.panel_rect);
        },
    );

    // ── 4. Dropdown Popups (Rendered on top in UiLayer::Popup via UiScope) ────
    build_aspect_ratio_popup(&mut scope, params);
    build_add_element_popup(&mut scope, params);

    canvas_metrics
}
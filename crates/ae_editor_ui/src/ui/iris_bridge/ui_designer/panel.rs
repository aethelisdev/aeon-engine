// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # 2D Visual UI Designer Panel Root Builder
//!
//! Assembles the complete UI Designer panel hierarchy in the Iris UI tree,
//! including the virtual canvas, 34px elevated toolbar, and interactive dropdown popups.
//!

use super::canvas::build_designer_canvas;
use super::popups::{build_add_element_popup, build_aspect_ratio_popup};
use super::toolbar::build_designer_toolbar;
use super::types::{UiDesignerPanelParams, UiDesignerPanelTargets};
use irisui::prelude::*;

/// Assembles the complete 2D Visual UI Designer panel tree in native Iris UI.
pub fn build_ui_designer_panel(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &UiDesignerPanelParams<'_>,
) -> UiDesignerPanelTargets {
    let mut targets = UiDesignerPanelTargets {
        panel_rect: params.panel_rect,
        ..Default::default()
    };

    // ── 1. Root Panel Container with Child Clipping ───────────────────────────
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.set_name("UiDesignerPanelRoot");
        node.computed_rect = params.panel_rect;
        node.style = Style::new()
            .background(Color::rgba(0.065, 0.070, 0.082, 1.0))
            .border(1.0, Color::rgba(0.14, 0.15, 0.18, 0.70))
            .clip_children(true);
    }
    let _ = tree.add_child(parent_id, root_id);

    // ── 2. Virtual Canvas & Elements Viewport ──────────────────────────────────
    build_designer_canvas(tree, root_id, params, &mut targets);

    // ── 3. Elevated 34px Top Control Bar ──────────────────────────────────────
    build_designer_toolbar(tree, root_id, params, &mut targets);

    // ── 4. Dropdown Popups (Rendered on top of toolbar) ───────────────────────
    build_aspect_ratio_popup(tree, root_id, params, &mut targets);
    build_add_element_popup(tree, root_id, params, &mut targets);

    targets
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Footer Status Bar Builder
//!
//! Renders the bottom object count and selection state telemetry line.

use super::types::HierarchyPanelParams;
use irisui::prelude::*;

/// Builds the static Scene Hierarchy footer status line in the `UiTree`.
pub fn build_hierarchy_footer(
    tree: &mut UiTree,
    parent_id: WidgetId,
    total_objects: usize,
    params: &HierarchyPanelParams<'_>,
) {
    let padding_x = 6.0;
    let footer_h = 24.0;
    let footer_y = params.panel_rect.bottom() - footer_h;
    let footer_w = params.panel_rect.width - padding_x * 2.0;

    // 1. Separator Line
    let sep_id = tree.create_node();
    if let Some(node) = tree.get_mut(sep_id) {
        node.set_name("HierarchyFooterSeparator");
        node.computed_rect = Rect::new(params.panel_rect.x, footer_y, params.panel_rect.width, 1.0);
        node.style = Style::new().background(Color::rgba(0.14, 0.16, 0.22, 0.70));
    }
    let _ = tree.add_child(parent_id, sep_id);

    // 2. Status Text
    let sel_str = if params.selected_entity.is_some() {
        " • 1 Selected"
    } else {
        ""
    };
    let footer_text = format!(
        "{} Object{}{}",
        total_objects,
        if total_objects == 1 { "" } else { "s" },
        sel_str
    );

    let text_id = tree.create_node();
    if let Some(node) = tree.get_mut(text_id) {
        node.set_name("HierarchyFooterText");
        node.set_text(footer_text);
        node.font_size = 11.0;
        node.line_height = footer_h;
        node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
        node.computed_rect = Rect::new(
            params.panel_rect.x + padding_x,
            footer_y + 1.0,
            footer_w,
            footer_h - 1.0,
        );
    }
    let _ = tree.add_child(parent_id, text_id);
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Bottom Action Bar Builder
//!
//! Renders `➕ Add Component` and `💾 Save as Prefab` action buttons.

use super::types::{InspectorPanelParams, InspectorPanelTargets};
use irisui::prelude::*;

/// Output node handles created during Inspector footer construction.
pub struct FooterNodes {
    /// Add component button node ID.
    pub add_comp_btn_id: WidgetId,
    /// Save prefab button node ID.
    pub save_prefab_btn_id: WidgetId,
}

/// Builds the Inspector bottom action bar and returns the computed height.
pub fn build_inspector_footer(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &InspectorPanelParams<'_>,
    targets: &mut InspectorPanelTargets,
) -> (f32, FooterNodes) {
    let padding_x = 8.0;
    let footer_h = 24.0;
    let footer_y = params.panel_rect.bottom() - footer_h - 6.0;
    let base_x = params.panel_rect.x + padding_x; // Sola yasla (Left-aligned)
    let btn_gap = 8.0;
    let btn_w = 112.0; // Kompakt genişlik (Image 2 & 3)

    // 1. `➕ Add Component` Button
    let add_rect = Rect::new(base_x, footer_y, btn_w, footer_h);
    targets.add_component_btn_rect = add_rect;
    let is_add_hovered = add_rect.contains_point(params.cursor_pos);

    let add_comp_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(add_comp_btn_id) {
        node.set_name("AddComponentBtn");
        node.computed_rect = add_rect;
        let (bg, border, text_col) = if params.is_add_menu_open {
            (
                Color::rgba(0.118, 0.125, 0.145, 1.0),
                Color::rgba(0.353, 0.376, 0.439, 0.95),
                Color::WHITE,
            )
        } else if is_add_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.886, 0.894, 0.918, 1.0),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
        node.set_text("➕ Add Component");
        node.font_size = 11.0;
        node.line_height = footer_h;
        node.text_align = TextAlign::Center;
        node.text_color = text_col;
    }
    let _ = tree.add_child(parent_id, add_comp_btn_id);

    // 2. `💾 Save as Prefab` Button
    let save_rect = Rect::new(base_x + btn_w + btn_gap, footer_y, btn_w, footer_h);
    targets.save_prefab_btn_rect = save_rect;
    let is_save_hovered = save_rect.contains_point(params.cursor_pos);

    let save_prefab_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(save_prefab_btn_id) {
        node.set_name("SavePrefabBtn");
        node.computed_rect = save_rect;
        let (bg, border, text_col) = if is_save_hovered {
            (
                Color::rgba(0.200, 0.208, 0.235, 1.0),
                Color::rgba(0.271, 0.282, 0.329, 0.95),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.157, 0.165, 0.188, 0.98),
                Color::rgba(0.212, 0.220, 0.259, 0.85),
                Color::rgba(0.886, 0.894, 0.918, 1.0),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(5.0);
        node.set_text("💾 Save as Prefab");
        node.font_size = 11.0;
        node.line_height = footer_h;
        node.text_align = TextAlign::Center;
        node.text_color = text_col;
    }
    let _ = tree.add_child(parent_id, save_prefab_btn_id);

    (
        footer_h,
        FooterNodes {
            add_comp_btn_id,
            save_prefab_btn_id,
        },
    )
}
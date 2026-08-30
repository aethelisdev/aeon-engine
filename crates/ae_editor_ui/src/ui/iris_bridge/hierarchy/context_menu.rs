// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Right-Click Entity Context Menu Builder
//!
//! Renders the floating context menu for deleting entities or toggling visibility.

use super::types::{HierarchyPanelParams, HierarchyPanelTargets};
use irisui::prelude::*;

/// Builds the right-click entity context menu in the `UiTree` if active.
pub fn build_context_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) {
    targets.active_context_menu = None;

    let Some((target_entity, click_pos)) = params.active_context_menu else {
        return;
    };

    let menu_w = 160.0;
    let menu_h = 56.0;
    let menu_x = click_pos
        .x
        .min(params.panel_rect.right() - menu_w - 4.0)
        .max(4.0);
    let menu_y = click_pos
        .y
        .min(params.panel_rect.bottom() - menu_h - 4.0)
        .max(4.0);

    let card_rect = Rect::new(menu_x, menu_y, menu_w, menu_h);

    let card_id = tree.create_node();
    if let Some(node) = tree.get_mut(card_id) {
        node.set_name("EntityContextMenuCard");
        node.computed_rect = card_rect;
        node.style = Style::new()
            .background(Color::rgba(0.06, 0.07, 0.10, 0.98))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
            .border_radius(6.0)
            .box_shadow(0.0, 6.0, 18.0, Color::rgba(0.0, 0.0, 0.0, 0.80));
    }
    let _ = tree.add_child(parent_id, card_id);

    // 1. Delete Entity Item
    let del_rect = Rect::new(menu_x + 4.0, menu_y + 4.0, menu_w - 8.0, 22.0);
    let is_del_hovered = del_rect.contains_point(params.cursor_pos);
    let (del_bg, del_text_col) = if is_del_hovered {
        (
            Color::rgba(0.40, 0.08, 0.08, 0.90),
            Color::rgba(1.0, 0.45, 0.45, 1.0),
        )
    } else {
        (Color::TRANSPARENT, Color::rgba(0.88, 0.90, 0.96, 1.0))
    };

    let del_id = tree.create_node();
    if let Some(node) = tree.get_mut(del_id) {
        node.set_name("ContextDeleteEntity");
        node.computed_rect = del_rect;
        node.style = Style::new().background(del_bg).border_radius(4.0);
    }
    let _ = tree.add_child(card_id, del_id);

    let del_ic_id = tree.create_node();
    if let Some(node) = tree.get_mut(del_ic_id) {
        node.set_name("DeleteIcon");
        node.set_text("🗑");
        node.font_size = 11.0;
        node.line_height = 22.0;
        node.computed_rect = Rect::new(del_rect.x + 6.0, del_rect.y, 16.0, 22.0);
    }
    let _ = tree.add_child(del_id, del_ic_id);

    let del_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(del_lbl_id) {
        node.set_name("DeleteLabel");
        node.set_text("Delete Entity");
        node.font_size = 11.0;
        node.line_height = 22.0;
        node.text_color = del_text_col;
        node.computed_rect = Rect::new(del_rect.x + 24.0, del_rect.y, del_rect.width - 28.0, 22.0);
    }
    let _ = tree.add_child(del_id, del_lbl_id);

    // 2. Toggle Visibility Item
    let vis_rect = Rect::new(menu_x + 4.0, menu_y + 28.0, menu_w - 8.0, 22.0);
    let is_vis_hovered = vis_rect.contains_point(params.cursor_pos);
    let (vis_bg, vis_text_col) = if is_vis_hovered {
        (
            Color::rgba(0.0, 0.35, 0.45, 0.80),
            Color::rgba(0.0, 0.95, 1.0, 1.0),
        )
    } else {
        (Color::TRANSPARENT, Color::rgba(0.88, 0.90, 0.96, 1.0))
    };

    let vis_id = tree.create_node();
    if let Some(node) = tree.get_mut(vis_id) {
        node.set_name("ContextToggleVisibility");
        node.computed_rect = vis_rect;
        node.style = Style::new().background(vis_bg).border_radius(4.0);
    }
    let _ = tree.add_child(card_id, vis_id);

    let vis_ic_id = tree.create_node();
    if let Some(node) = tree.get_mut(vis_ic_id) {
        node.set_name("VisibilityIcon");
        node.set_text("👁");
        node.font_size = 11.0;
        node.line_height = 22.0;
        node.computed_rect = Rect::new(vis_rect.x + 6.0, vis_rect.y, 16.0, 22.0);
    }
    let _ = tree.add_child(vis_id, vis_ic_id);

    let vis_lbl_id = tree.create_node();
    if let Some(node) = tree.get_mut(vis_lbl_id) {
        node.set_name("VisibilityLabel");
        node.set_text("Toggle Visibility");
        node.font_size = 11.0;
        node.line_height = 22.0;
        node.text_color = vis_text_col;
        node.computed_rect = Rect::new(vis_rect.x + 24.0, vis_rect.y, vis_rect.width - 28.0, 22.0);
    }
    let _ = tree.add_child(vis_id, vis_lbl_id);

    targets.active_context_menu = Some((target_entity, card_rect, del_rect, vis_rect));
}
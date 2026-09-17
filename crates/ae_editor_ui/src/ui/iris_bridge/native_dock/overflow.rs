// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Dock tab overflow popup dropdown menu builder.
//!

use irisui::dock::DockNode;
use irisui::prelude::*;

use super::super::theme::*;
use super::builder::{add_icon_node, add_rect_node, add_text_node};
use super::types::{NativeDockFrame, NativeDockOverflowItemTarget, NativeDockOverflowMenuParams};

/// Renders the floating popup dropdown menu listing all tabs for an active dock leaf with overflow.
pub fn build_native_dock_overflow_menu(
    tree: &mut UiTree,
    parent: WidgetId,
    params: NativeDockOverflowMenuParams<'_>,
    frame: &mut NativeDockFrame,
) {
    let Some(node) = params.layout_state.dock_state.tree.get(params.leaf_id) else {
        return;
    };
    let DockNode::Leaf { tabs, active_tab } = node else {
        return;
    };
    if tabs.is_empty() {
        return;
    }

    let item_height: f32 = 26.0;
    let menu_padding: f32 = 4.0;
    let menu_width: f32 = 180.0;
    let menu_height = (tabs.len() as f32 * item_height) + (menu_padding * 2.0);

    // Anchor below the chevron button, aligned to its right edge
    let menu_x = (params.anchor_rect.right() - menu_width).max(0.0);
    let menu_y = params.anchor_rect.bottom() + 2.0;
    let menu_rect = Rect::new(menu_x, menu_y, menu_width, menu_height);

    frame.active_overflow_rect = Some(menu_rect);

    // Menu container card
    add_rect_node(
        tree,
        parent,
        menu_rect,
        "IrisDockOverflowMenu",
        Style::new()
            .background(Color::from_u8(22, 26, 36, 250))
            .border(1.0, Color::from_u8(50, 60, 80, 200))
            .corner_radii(CornerRadii::all(4.0))
            .clip_children(true),
    );

    for (index, panel) in tabs.iter().enumerate() {
        let is_active = index == *active_tab;
        let item_y = menu_y + menu_padding + (index as f32 * item_height);
        let item_rect = Rect::new(
            menu_x + menu_padding,
            item_y,
            menu_width - (menu_padding * 2.0),
            item_height,
        );
        let is_hovered = !params.is_cursor_occluded && item_rect.contains_point(params.cursor_pos);

        let item_bg = if is_hovered {
            ELEVATION_3_HOVERED_PILL
        } else if is_active {
            Color::from_u8(32, 42, 58, 220)
        } else {
            Color::TRANSPARENT
        };
        add_rect_node(
            tree,
            parent,
            item_rect,
            "IrisDockOverflowItem",
            Style::new()
                .background(item_bg)
                .corner_radii(CornerRadii::all(3.0)),
        );

        let atlas_icon = panel.atlas_icon();
        let title = if atlas_icon.is_some() {
            panel.title().to_string()
        } else {
            format!("{} {}", panel.icon(), panel.title())
        };
        if let Some(uv) = atlas_icon {
            let tint = if is_active {
                ACCENT_CYAN
            } else if is_hovered {
                Color::WHITE
            } else {
                TEXT_REGULAR
            };
            add_icon_node(
                tree,
                parent,
                Rect::new(item_rect.x + 6.0, item_rect.y + 5.0, 16.0, 16.0),
                "IrisDockOverflowItemIcon",
                uv,
                tint,
            );
        }

        let label_offset_x = if atlas_icon.is_some() { 26.0 } else { 8.0 };
        let label_width =
            (item_rect.width - label_offset_x - if is_active { 20.0 } else { 4.0 }).max(10.0);
        let label_rect = Rect::new(
            item_rect.x + label_offset_x,
            item_rect.y + 4.0,
            label_width,
            18.0,
        );
        let text_col = if is_active {
            ACCENT_CYAN
        } else if is_hovered {
            TEXT_BRIGHT
        } else {
            TEXT_REGULAR
        };
        add_text_node(tree, parent, label_rect, &title, text_col, TextAlign::Left);

        if is_active {
            let check_rect = Rect::new(item_rect.right() - 20.0, item_rect.y + 4.0, 16.0, 18.0);
            add_text_node(
                tree,
                parent,
                check_rect,
                "✓",
                ACCENT_CYAN,
                TextAlign::Center,
            );
        }

        frame
            .overflow_item_targets
            .push(NativeDockOverflowItemTarget {
                leaf: params.leaf_id,
                tab_index: index,
                panel: *panel,
                rect: item_rect,
            });
    }
}
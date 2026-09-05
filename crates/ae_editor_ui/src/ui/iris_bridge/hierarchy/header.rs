// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Header and Search Bar Builder
//!
//! Renders the top search bar input box, clear button, `➕` Add Menu button,
//! and `🗑` Delete Selected entity button with pixel-perfect visual styling.

use super::types::{HierarchyPanelParams, HierarchyPanelTargets};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;

/// Output node handles created during header initialization.
pub struct HeaderNodes {
    /// Search input container node ID.
    pub search_box_id: WidgetId,
    /// Search text/placeholder node ID.
    pub search_text_id: WidgetId,
    /// Search clear `✖` button node ID (if created).
    pub clear_btn_id: Option<WidgetId>,
    /// `➕` Add Menu button node ID.
    pub add_btn_id: WidgetId,
    /// `🗑` Delete Selected button node ID.
    pub delete_btn_id: Option<WidgetId>,
}

/// Builds the static Scene Hierarchy header layout.
pub fn build_hierarchy_header(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    targets: &mut HierarchyPanelTargets,
) -> HeaderNodes {
    let padding_x = 6.0;
    let header_y = params.panel_rect.y + 4.0;
    let header_h = 24.0;
    let header_w = params.panel_rect.width - padding_x * 2.0;

    let btn_size = 24.0;
    let btn_gap = 4.0;

    // Both `+` and `🗑` buttons are permanently visible in the header bar
    let right_btns_w = btn_size * 2.0 + btn_gap;

    let search_w = (header_w - right_btns_w - 4.0).max(60.0);
    let search_x = params.panel_rect.x + padding_x;
    let search_rect = Rect::new(search_x, header_y, search_w, header_h);
    targets.search_input_rect = search_rect;

    // 1. Search Bar Container
    let search_box_id = tree.create_node();
    if let Some(node) = tree.get_mut(search_box_id) {
        node.set_name("HierarchySearchBox");
        node.computed_rect = search_rect;
        let border_color = if params.is_search_focused {
            Color::rgba(0.0, 0.90, 1.0, 0.90) // Active Cyan ring
        } else {
            Color::rgba(0.18, 0.20, 0.26, 0.80)
        };
        node.style = Style::new()
            .background(Color::rgba(0.04, 0.05, 0.07, 0.95))
            .border(1.0, border_color)
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, search_box_id);

    // Search Icon "🔍"
    let icon_id = tree.create_node();
    if let Some(node) = tree.get_mut(icon_id) {
        node.set_name("SearchIcon");
        node.set_text("🔍");
        node.font_size = 11.0;
        node.line_height = header_h;
        node.text_color = Color::rgba(0.55, 0.58, 0.68, 1.0);
        node.computed_rect = Rect::new(search_x + 6.0, header_y, 14.0, header_h);
    }
    let _ = tree.add_child(search_box_id, icon_id);

    // Search Query or Hint Text
    let search_text_id = tree.create_node();
    let display_text = if params.search_query.is_empty() {
        "Search..."
    } else {
        params.search_query
    };
    let text_color = if params.search_query.is_empty() {
        Color::rgba(0.42, 0.45, 0.55, 1.0)
    } else {
        Color::rgba(0.92, 0.94, 0.98, 1.0)
    };

    let text_w = (search_w
        - 24.0
        - if params.search_query.is_empty() {
            0.0
        } else {
            18.0
        })
    .max(20.0);
    let text_start_x = if params.is_search_focused && params.search_query.is_empty() {
        search_x + 24.5
    } else {
        search_x + 22.0
    };
    if let Some(node) = tree.get_mut(search_text_id) {
        node.set_name("SearchQueryText");
        node.set_text(display_text);
        node.font_size = 11.5;
        node.line_height = header_h;
        node.text_color = text_color;
        node.computed_rect = Rect::new(text_start_x, header_y, text_w, header_h);
    }
    let _ = tree.add_child(search_box_id, search_text_id);

    // Blinking Caret Cursor (500ms cycle)
    if params.is_search_focused && params.blink_caret {
        let caret_x = if params.search_query.is_empty() {
            search_x + 22.0
        } else {
            (search_x + 22.0 + (params.search_query.len() as f32 * 6.8))
                .min(search_x + search_w - 24.0)
        };
        let caret_id = tree.create_node();
        if let Some(node) = tree.get_mut(caret_id) {
            node.set_name("HierarchySearchCaret");
            node.computed_rect = Rect::new(caret_x, header_y + 4.0, 1.5, header_h - 8.0);
            node.style = Style::new()
                .background(Color::rgba(0.0, 0.90, 1.0, 1.0))
                .border_radius(0.75);
        }
        let _ = tree.add_child(search_box_id, caret_id);
    }

    // Clear Search "✖" Button
    let mut clear_btn_id = None;
    if !params.search_query.is_empty() {
        let clear_rect = Rect::new(search_x + search_w - 18.0, header_y + 3.0, 16.0, 18.0);
        targets.search_clear_btn_rect = Some(clear_rect);

        let clr_id = tree.create_node();
        if let Some(node) = tree.get_mut(clr_id) {
            node.set_name("SearchClearButton");
            node.set_text("✖");
            node.font_size = 9.5;
            node.line_height = 18.0;
            node.text_align = TextAlign::Center;
            node.text_color = Color::rgba(0.60, 0.63, 0.72, 1.0);
            node.computed_rect = clear_rect;
        }
        let _ = tree.add_child(search_box_id, clr_id);
        clear_btn_id = Some(clr_id);
    } else {
        targets.search_clear_btn_rect = None;
    }

    // 2. "➕" Add Entity Button (Elevated Slate `#383d4a`)
    let add_x = search_x + search_w + 4.0;
    let add_rect = Rect::new(add_x, header_y, btn_size, header_h);
    targets.add_btn_rect = add_rect;

    let is_add_hovered = add_rect.contains_point(params.cursor_pos);
    let (bg, border, icon_col) = if params.is_add_menu_open {
        (
            Color::rgba(0.0, 0.38, 0.50, 0.95),
            Color::rgba(0.0, 0.90, 1.0, 0.90),
            Color::rgba(0.0, 0.95, 1.0, 1.0),
        )
    } else if is_add_hovered {
        (
            Color::rgba(0.30, 0.34, 0.42, 0.95),
            Color::rgba(0.45, 0.50, 0.60, 0.85),
            Color::WHITE,
        )
    } else {
        (
            Color::rgba(0.24, 0.27, 0.34, 0.95), // Elevated slate
            Color::rgba(0.35, 0.39, 0.48, 0.70),
            Color::rgba(0.85, 0.88, 0.95, 1.0),
        )
    };

    let add_btn_id = tree.create_node();
    if let Some(node) = tree.get_mut(add_btn_id) {
        node.set_name("AddEntityButton");
        node.computed_rect = add_rect;
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(4.0);
    }
    let _ = tree.add_child(parent_id, add_btn_id);

    let plus_size = 14.0;
    let plus_x = add_rect.x + (btn_size - plus_size) * 0.5;
    let plus_y = add_rect.y + (header_h - plus_size) * 0.5;
    let plus_id = tree.create_node();
    if let Some(node) = tree.get_mut(plus_id) {
        node.set_name("AddEntityPlusIcon");
        node.computed_rect = Rect::new(plus_x, plus_y, plus_size, plus_size);
        node.set_texture_uv(ICON_PLUS);
        node.set_texture_tint(icon_col);
    }
    let _ = tree.add_child(add_btn_id, plus_id);

    // 3. "🗑" Delete Entity Button (Elevated Slate `#383d4a`, permanently visible)
    let del_x = add_x + btn_size + btn_gap;
    let del_rect = Rect::new(del_x, header_y, btn_size, header_h);
    targets.delete_btn_rect = Some(del_rect);

    let has_selection = params.selected_entity.is_some();
    let is_del_hovered = del_rect.contains_point(params.cursor_pos);
    let del_id = tree.create_node();
    if let Some(node) = tree.get_mut(del_id) {
        node.set_name("DeleteSelectedButton");
        node.computed_rect = del_rect;
        let (bg, border, text_col) = if has_selection && is_del_hovered {
            (
                Color::rgba(0.42, 0.12, 0.12, 0.95),
                Color::rgba(0.92, 0.28, 0.28, 0.90),
                Color::rgba(1.0, 0.40, 0.40, 1.0),
            )
        } else if is_del_hovered {
            (
                Color::rgba(0.30, 0.34, 0.42, 0.95),
                Color::rgba(0.45, 0.50, 0.60, 0.85),
                Color::WHITE,
            )
        } else if has_selection {
            (
                Color::rgba(0.24, 0.27, 0.34, 0.95),
                Color::rgba(0.35, 0.39, 0.48, 0.70),
                Color::rgba(0.85, 0.88, 0.95, 1.0),
            )
        } else {
            (
                Color::rgba(0.20, 0.22, 0.28, 0.80),
                Color::rgba(0.28, 0.31, 0.38, 0.50),
                Color::rgba(0.55, 0.58, 0.66, 0.70),
            )
        };
        node.style = Style::new()
            .background(bg)
            .border(1.0, border)
            .border_radius(4.0);
        node.set_text("🗑");
        node.font_size = 12.0;
        node.line_height = header_h;
        node.text_align = TextAlign::Center;
        node.text_color = text_col;
    }
    let _ = tree.add_child(parent_id, del_id);

    HeaderNodes {
        search_box_id,
        search_text_id,
        clear_btn_id,
        add_btn_id,
        delete_btn_id: Some(del_id),
    }
}
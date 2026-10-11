// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Header and Search Bar Builder
//!
//! Renders the top search bar input box, clear button, `➕` Add Menu button,
//! and `🗑` Delete Selected entity button with clean visual alignment and
//! pure declarative [`UiScope`] semantics.
//!

use super::types::{
    HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_SEARCH_CLEAR,
    HIERARCHY_TAG_SEARCH_INPUT, HierarchyPanelParams,
};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;

/// Builds the declarative Scene Hierarchy header toolbar containing search bar,
/// Add entity menu button, and Delete selected entity button.
///
/// Attaches declarative widgets directly into the active [`UiScope`] flexbox flow.
pub fn build_hierarchy_header(scope: &mut UiScope<'_>, params: &HierarchyPanelParams<'_>) {
    let header_bar_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(4.0)
        .height(31.0)
        .padding_insets(Insets::new(4.0, 6.0, 3.0, 6.0));

    scope.container_named("HierarchyHeaderBar", header_bar_style, |bar| {
        // 1. Search Bar Input Box via standard declarative input_box_tagged widget
        let search_width = (params.panel_rect.width - 68.0).max(60.0);
        let search_props = InputBoxProps::new(
            params.search_query,
            "Search...",
            search_width,
            params.is_search_focused,
            params.is_search_focused && params.blink_caret,
        )
        .with_icon("🔍")
        .with_cursor_byte_idx(params.cursor_byte_idx)
        .with_clear_tag(HIERARCHY_TAG_SEARCH_CLEAR);
        bar.input_box_tagged(&search_props, HIERARCHY_TAG_SEARCH_INPUT);

        // 2. "➕" Add Entity Button
        let (add_bg, add_border, add_icon_col, is_active) = if params.is_add_menu_open {
            (
                Color::rgba(0.0, 0.38, 0.50, 0.95),
                Color::rgba(0.0, 0.90, 1.0, 0.90),
                Color::rgba(0.0, 0.95, 1.0, 1.0),
                true,
            )
        } else {
            (
                Color::rgba(0.24, 0.27, 0.34, 0.95), // Elevated slate
                Color::rgba(0.35, 0.39, 0.48, 0.70),
                Color::rgba(0.85, 0.88, 0.95, 1.0),
                false,
            )
        };

        let mut add_btn_style = Style::new()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .background(add_bg)
            .border(1.0, add_border)
            .border_radius(4.0);

        if !is_active {
            add_btn_style = add_btn_style
                .hover_background(Color::rgba(0.30, 0.35, 0.45, 0.98))
                .hover_border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85));
        }

        bar.container_tagged(
            "AddEntityButton",
            add_btn_style,
            WidgetRole::Button,
            HIERARCHY_TAG_ADD_BUTTON,
            |add_btn| {
                let icon_id = add_btn.icon(ICON_PLUS, add_icon_col, 14.0);
                if !is_active {
                    add_btn.set_hover_texture_tint(icon_id, Color::WHITE);
                }
            },
        );

        // 3. "🗑" Delete Entity Button (Permanently visible)
        let has_selection = params.selected_entity.is_some();
        let (del_bg, del_border, del_text_col) = if !has_selection {
            (
                Color::rgba(0.20, 0.22, 0.28, 0.80),
                Color::rgba(0.28, 0.31, 0.38, 0.50),
                Color::rgba(0.55, 0.58, 0.66, 0.70),
            )
        } else {
            (
                Color::rgba(0.24, 0.27, 0.34, 0.95),
                Color::rgba(0.35, 0.39, 0.48, 0.70),
                Color::rgba(0.85, 0.88, 0.95, 1.0),
            )
        };

        let mut del_btn_style = Style::new()
            .flex_row()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .background(del_bg)
            .border(1.0, del_border)
            .border_radius(4.0);

        if has_selection {
            del_btn_style = del_btn_style
                .hover_background(Color::rgba(0.55, 0.18, 0.22, 0.95))
                .hover_border(1.0, Color::rgba(0.95, 0.35, 0.40, 0.90));
        }

        bar.container_tagged(
            "DeleteSelectedButton",
            del_btn_style,
            WidgetRole::Button,
            HIERARCHY_TAG_DELETE_BUTTON,
            |del_btn| {
                let txt_id =
                    del_btn.label_with_width("🗑", 24.0, 11.0, del_text_col, TextAlign::Center);
                if has_selection {
                    del_btn.set_hover_text_color(txt_id, Color::WHITE);
                }
            },
        );
    });
}
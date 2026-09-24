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
        // 1. Search Bar Container (Tagged with HIERARCHY_TAG_SEARCH_INPUT)
        let is_search_hovered = params.hovered_tag == Some(HIERARCHY_TAG_SEARCH_INPUT);
        let border_color = if params.is_search_focused {
            Color::rgba(0.0, 0.90, 1.0, 0.90) // Active Cyan ring
        } else if is_search_hovered {
            Color::rgba(0.35, 0.40, 0.52, 0.95)
        } else {
            Color::rgba(0.18, 0.20, 0.26, 0.80)
        };
        let search_bg = if is_search_hovered {
            Color::rgba(0.06, 0.08, 0.11, 0.98)
        } else {
            Color::rgba(0.04, 0.05, 0.07, 0.95)
        };

        let search_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .flex_grow(1.0)
            .height(24.0)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
            .gap(6.0)
            .background(search_bg)
            .border(1.0, border_color)
            .border_radius(4.0);

        bar.container_tagged(
            "HierarchySearchBox",
            search_style,
            WidgetRole::Default,
            HIERARCHY_TAG_SEARCH_INPUT,
            |s| {
                // Search Icon "🔍"
                let icon_col = if is_search_hovered {
                    Color::rgba(0.85, 0.88, 0.98, 1.0)
                } else {
                    Color::rgba(0.55, 0.58, 0.68, 1.0)
                };
                s.label_with_width("🔍", 14.0, 11.0, icon_col, TextAlign::Center);

                // Search Query or Hint Text
                let (display_text, text_color) = if params.search_query.is_empty() {
                    let hint_col = if is_search_hovered {
                        Color::rgba(0.55, 0.58, 0.68, 1.0)
                    } else {
                        Color::rgba(0.42, 0.45, 0.55, 1.0)
                    };
                    ("Search...", hint_col)
                } else {
                    (params.search_query, Color::rgba(0.92, 0.94, 0.98, 1.0))
                };

                s.label_flex(display_text, 11.5, text_color, TextAlign::Left);

                // Blinking Caret Cursor (500ms cycle)
                if params.is_search_focused && params.blink_caret {
                    s.label_with_width(
                        "|",
                        6.0,
                        11.5,
                        Color::rgba(0.0, 0.90, 1.0, 1.0),
                        TextAlign::Left,
                    );
                }

                // Clear Search "✖" Button
                if !params.search_query.is_empty() {
                    let is_clear_hovered = params.hovered_tag == Some(HIERARCHY_TAG_SEARCH_CLEAR);
                    let clear_btn_style = Style::new()
                        .flex_row()
                        .width(16.0)
                        .height(18.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center);

                    s.container_tagged(
                        "SearchClearButton",
                        clear_btn_style,
                        WidgetRole::Button,
                        HIERARCHY_TAG_SEARCH_CLEAR,
                        |clr| {
                            let clear_col = if is_clear_hovered {
                                Color::WHITE
                            } else {
                                Color::rgba(0.60, 0.63, 0.72, 1.0)
                            };
                            clr.label_with_width("✖", 16.0, 9.5, clear_col, TextAlign::Center);
                        },
                    );
                }
            },
        );

        // 2. "➕" Add Entity Button
        let is_add_hovered = params.hovered_tag == Some(HIERARCHY_TAG_ADD_BUTTON);
        let (add_bg, add_border, add_icon_col) = if params.is_add_menu_open {
            (
                Color::rgba(0.0, 0.38, 0.50, 0.95),
                Color::rgba(0.0, 0.90, 1.0, 0.90),
                Color::rgba(0.0, 0.95, 1.0, 1.0),
            )
        } else if is_add_hovered {
            (
                Color::rgba(0.30, 0.35, 0.45, 0.98), // Brightened hover
                Color::rgba(0.0, 0.85, 1.0, 0.85),   // Cyan hover ring
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.24, 0.27, 0.34, 0.95), // Elevated slate
                Color::rgba(0.35, 0.39, 0.48, 0.70),
                Color::rgba(0.85, 0.88, 0.95, 1.0),
            )
        };

        let add_btn_style = Style::new()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .background(add_bg)
            .border(1.0, add_border)
            .border_radius(4.0);

        bar.container_tagged(
            "AddEntityButton",
            add_btn_style,
            WidgetRole::Button,
            HIERARCHY_TAG_ADD_BUTTON,
            |add_btn| {
                add_btn.icon(ICON_PLUS, add_icon_col, 14.0);
            },
        );

        // 3. "🗑" Delete Entity Button (Permanently visible)
        let is_del_hovered = params.hovered_tag == Some(HIERARCHY_TAG_DELETE_BUTTON);
        let has_selection = params.selected_entity.is_some();
        let (del_bg, del_border, del_text_col) = if !has_selection {
            (
                Color::rgba(0.20, 0.22, 0.28, 0.80),
                Color::rgba(0.28, 0.31, 0.38, 0.50),
                Color::rgba(0.55, 0.58, 0.66, 0.70),
            )
        } else if is_del_hovered {
            (
                Color::rgba(0.55, 0.18, 0.22, 0.95), // Red danger hover
                Color::rgba(0.95, 0.35, 0.40, 0.90),
                Color::WHITE,
            )
        } else {
            (
                Color::rgba(0.24, 0.27, 0.34, 0.95),
                Color::rgba(0.35, 0.39, 0.48, 0.70),
                Color::rgba(0.85, 0.88, 0.95, 1.0),
            )
        };

        let del_btn_style = Style::new()
            .flex_row()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .background(del_bg)
            .border(1.0, del_border)
            .border_radius(4.0);

        bar.container_tagged(
            "DeleteSelectedButton",
            del_btn_style,
            WidgetRole::Button,
            HIERARCHY_TAG_DELETE_BUTTON,
            |del_btn| {
                del_btn.label_with_width("🗑", 24.0, 11.0, del_text_col, TextAlign::Center);
            },
        );
    });
}
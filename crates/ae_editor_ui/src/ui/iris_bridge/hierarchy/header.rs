// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Header and Search Bar Builder
//!
//! Renders the top search bar input box, clear button, `➕` Add Menu button,
//! and `🗑` Delete Selected entity button with clean visual alignment and
//! pure declarative [`UiScope`] semantics.

use super::types::{
    HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_SEARCH_CLEAR,
    HIERARCHY_TAG_SEARCH_INPUT, HierarchyAction, HierarchyPanelParams,
};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use irisui::prelude::*;
use irisui::widgets::declarative::InputBoxProps;

/// Builds the [`Style`] for the Hierarchy `➕` Add Entity toolbar button based on active popup state.
#[must_use]
fn hierarchy_add_button_style(is_active: bool) -> Style {
    let base = Style::new()
        .width(24.0)
        .height(24.0)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .border_radius(4.0);
    if is_active {
        base.background(Color::rgba(0.0, 0.38, 0.50, 0.95))
            .border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.90))
    } else {
        base.background(Color::rgba(0.24, 0.27, 0.34, 0.95))
            .border(1.0, Color::rgba(0.35, 0.39, 0.48, 0.70))
            .hover_background(Color::rgba(0.30, 0.35, 0.45, 0.98))
            .hover_border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
    }
}

/// Builds the declarative Scene Hierarchy header toolbar containing search bar,
/// Add entity menu button, and Delete selected entity button.
///
/// Attaches declarative widgets directly into the active [`UiScope`] flexbox flow.
pub fn build_hierarchy_header(
    scope: &mut UiScope<'_>,
    params: &HierarchyPanelParams<'_>,
    actions: &mut Vec<HierarchyAction>,
) {
    scope.container_named(
        "HierarchyHeaderBar",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(4.0)
            .height(31.0)
            .padding_insets(Insets::new(4.0, 6.0, 3.0, 6.0)),
        |bar| {
            // 1. Search Bar Input Box via standard declarative input_box_tagged widget
            _ = bar.input_box_tagged(
                &InputBoxProps::new(
                    params.search_query,
                    "Search...",
                    (params.panel_rect.width - 68.0).max(60.0),
                    false,
                    params.blink_caret,
                )
                .with_icon("🔍")
                .with_cursor_byte_idx(params.cursor_byte_idx)
                .with_clear_tag(HIERARCHY_TAG_SEARCH_CLEAR),
                HIERARCHY_TAG_SEARCH_INPUT,
            );
            if bar.is_tag_clicked(HIERARCHY_TAG_SEARCH_CLEAR) {
                actions.push(HierarchyAction::ClearSearchQuery);
            }

            // 2. "➕" Add Entity Button (scoped directly to UiTree::active_popup_tags)
            bar.container_tagged_popup_toggle(
                "AddEntityButton",
                hierarchy_add_button_style(bar.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON)),
                WidgetRole::Button,
                HIERARCHY_TAG_ADD_BUTTON,
                HIERARCHY_TAG_ADD_BUTTON,
                |add_btn, is_active| {
                    if is_active {
                        add_btn.icon(ICON_PLUS, Color::rgba(0.0, 0.95, 1.0, 1.0), 14.0);
                    } else {
                        add_btn.icon_hover(
                            ICON_PLUS,
                            Color::rgba(0.85, 0.88, 0.95, 1.0),
                            Color::WHITE,
                            14.0,
                        );
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

            let del_resp = bar.container_tagged_resp(
                "DeleteSelectedButton",
                del_btn_style,
                WidgetRole::Button,
                HIERARCHY_TAG_DELETE_BUTTON,
                |del_btn| {
                    if has_selection {
                        del_btn.label_styled_hover_passive(
                            "DeleteSelectedIcon",
                            "🗑",
                            11.0,
                            (del_text_col, Color::WHITE),
                            TextAlign::Center,
                            Style::new().width(24.0),
                        );
                    } else {
                        del_btn.label_with_width("🗑", 24.0, 11.0, del_text_col, TextAlign::Center);
                    }
                },
            );
            if has_selection && del_resp.clicked() {
                actions.push(HierarchyAction::DeleteSelected);
            }
        },
    );
}
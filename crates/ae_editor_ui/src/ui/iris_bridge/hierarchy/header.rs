// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Header and Search Bar Builder
//!
//! Renders the top search bar input box, clear button, `➕` Add Menu button,
//! and `🗑` Delete Selected entity button using pure declarative [`UiScope`] expressions.

use super::types::{
    HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_SEARCH_CLEAR,
    HIERARCHY_TAG_SEARCH_INPUT, HierarchyPanelParams,
};
use crate::ui::iris_bridge::icons::ICON_PLUS;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::widgets::declarative::InputBoxProps;
use irisui::workbench::ActionBus;

#[must_use]
fn hierarchy_add_button_style(is_active: bool) -> Style {
    if is_active {
        Style::new()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .border_radius(4.0)
            .background(Color::rgba(0.0, 0.38, 0.50, 0.95))
            .border(1.0, Color::rgba(0.0, 0.90, 1.0, 0.90))
    } else {
        Style::new()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .border_radius(4.0)
            .background(Color::rgba(0.24, 0.27, 0.34, 0.95))
            .border(1.0, Color::rgba(0.35, 0.39, 0.48, 0.70))
            .hover_background(Color::rgba(0.30, 0.35, 0.45, 0.98))
            .hover_border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85))
    }
}

#[must_use]
fn hierarchy_delete_button_style(has_selection: bool) -> Style {
    if has_selection {
        Style::new()
            .flex_row()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .background(Color::rgba(0.24, 0.27, 0.34, 0.95))
            .border(1.0, Color::rgba(0.35, 0.39, 0.48, 0.70))
            .border_radius(4.0)
            .hover_background(Color::rgba(0.55, 0.18, 0.22, 0.95))
            .hover_border(1.0, Color::rgba(0.95, 0.35, 0.40, 0.90))
    } else {
        Style::new()
            .flex_row()
            .width(24.0)
            .height(24.0)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .background(Color::rgba(0.20, 0.22, 0.28, 0.80))
            .border(1.0, Color::rgba(0.28, 0.31, 0.38, 0.50))
            .border_radius(4.0)
    }
}

/// Builds the declarative Scene Hierarchy header toolbar containing search bar,
/// Add entity menu trigger, and Delete selected entity button.
pub fn build_hierarchy_header(
    scope: &mut UiScope<'_>,
    params: &HierarchyPanelParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
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
            bar.input_box_tagged(
                &InputBoxProps::new(
                    params.search_query,
                    "Search...",
                    (params.panel_rect.width - 68.0).max(60.0),
                    false,
                    params.blink_caret,
                )
                .with_icon("🔍")
                .with_cursor_byte_idx(params.cursor_byte_idx)
                .with_clear_tag(HIERARCHY_TAG_SEARCH_CLEAR.raw()),
                HIERARCHY_TAG_SEARCH_INPUT.raw(),
            );

            bar.container_tagged_popup_toggle(
                "AddEntityButton",
                hierarchy_add_button_style(bar.is_popup_tag_active(HIERARCHY_TAG_ADD_BUTTON.raw())),
                WidgetRole::Button,
                HIERARCHY_TAG_ADD_BUTTON.raw(),
                HIERARCHY_TAG_ADD_BUTTON.raw(),
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

            bar.container_tagged_resp(
                "DeleteSelectedButton",
                hierarchy_delete_button_style(params.selected_entity.is_some()),
                WidgetRole::Button,
                HIERARCHY_TAG_DELETE_BUTTON.raw(),
                |del_btn| render_delete_icon(del_btn, params.selected_entity.is_some()),
            )
            .on_click(|| {
                if params.selected_entity.is_some() {
                    action_bus.emit(EngineUiAction::DeleteSelected);
                }
            });
        },
    );
}

fn render_delete_icon(del_btn: &mut UiScope<'_>, has_selection: bool) {
    if has_selection {
        del_btn.label_styled_hover_passive(
            "DeleteSelectedIcon",
            "🗑",
            11.0,
            (Color::rgba(0.85, 0.88, 0.95, 1.0), Color::WHITE),
            TextAlign::Center,
            Style::new().width(24.0),
        );
    } else {
        del_btn.label_with_width(
            "🗑",
            24.0,
            11.0,
            Color::rgba(0.55, 0.58, 0.66, 0.70),
            TextAlign::Center,
        );
    }
}
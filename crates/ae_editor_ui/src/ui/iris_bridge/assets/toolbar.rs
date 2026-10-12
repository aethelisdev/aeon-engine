// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top Navigation Header Toolbar for the Iris UI Content / Asset Browser.
//!
//! Provides a declarative Taffy flexbox toolbar containing:
//! - Canonical vector folder logo (`ICON_FOLDER`) and breadcrumb hierarchy navigation buttons
//! - "Engine" content toggle button with vector gear icon (`ICON_GEAR`)
//! - Action buttons: "+ Import" (`ICON_PLUS`), "Reveal", and "Clean" (sweeping unreferenced VRAM assets)
//! - View mode toggles: "⊞ Grid" vs "☰ List"
//! - Search input box with live query filter, blinking caret, and "✖" clear button
//!

use super::components::asset_breadcrumb_bar;
use super::panel::ASSETS_TOP_BAR_HEIGHT;
use super::types::{
    ASSETS_TAG_CLEAN_VRAM, ASSETS_TAG_ENGINE_CONTENT, ASSETS_TAG_IMPORT, ASSETS_TAG_REVEAL,
    ASSETS_TAG_SEARCH_CLEAR, ASSETS_TAG_SEARCH_INPUT, ASSETS_TAG_VIEW_GRID, ASSETS_TAG_VIEW_LIST,
    AssetsPanelParams,
};
use crate::assets::types::AssetViewMode;
use crate::ui::iris_bridge::icons::{ICON_GEAR, ICON_PLUS};
use irisui::prelude::*;

/// Declarative construction of the Asset Browser top toolbar.
///
/// Divides the toolbar into a left breadcrumb group and a right control group using pure
/// Taffy flexbox alignment with zero manual pixel offsets or absolute coordinates.
pub fn build_asset_toolbar_scope(scope: &mut UiScope<'_>, params: &AssetsPanelParams<'_>) {
    scope.container_named(
        "AssetsTopToolbar",
        Style::new()
            .flex_row()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::SpaceBetween)
            .width(params.panel_rect.width)
            .padding_insets(Insets::symmetric(0.0, 8.0))
            .background(crate::ui::iris_bridge::theme::ELEVATION_2_HEADER)
            .height(ASSETS_TOP_BAR_HEIGHT),
        |toolbar| {
            // 1. Left Group: Breadcrumb navigation hierarchy
            asset_breadcrumb_bar(toolbar, params.current_folder);

            // 2. Right Group: Engine toggle, action buttons, view mode selectors, search box
            toolbar.container_named(
                "ToolbarRightGroup",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .width(564.0)
                    .gap(6.0),
                |right| {
                    // "Engine" Content Visibility Toggle Button
                    build_engine_toggle_btn(right, params.show_engine_content);

                    // "+ Import" Action Button
                    build_import_btn(right);

                    // "Reveal" Action Button
                    build_action_btn(right, "Reveal", 60.0, ASSETS_TAG_REVEAL);

                    // "Clean" Action Button
                    build_action_btn(right, "Clean", 56.0, ASSETS_TAG_CLEAN_VRAM);

                    // View Mode Toggles: Grid vs List
                    build_mode_toggle_btn(
                        right,
                        "☰ List",
                        46.0,
                        params.view_mode == AssetViewMode::List,
                        ASSETS_TAG_VIEW_LIST,
                    );
                    build_mode_toggle_btn(
                        right,
                        "⊞ Grid",
                        46.0,
                        params.view_mode == AssetViewMode::Grid,
                        ASSETS_TAG_VIEW_GRID,
                    );

                    // Search Input Box
                    build_search_box(right, params);
                },
            );
        },
    );
}

/// Helper to render the search input box inside the flex toolbar matching Hierarchy Search UX.
fn build_search_box(scope: &mut UiScope<'_>, params: &AssetsPanelParams<'_>) {
    let is_search_focused = params.focus_state.is_tag_focused(ASSETS_TAG_SEARCH_INPUT);
    let search_props = InputBoxProps::new(
        params.search_query,
        "Search assets...",
        180.0,
        is_search_focused,
        is_search_focused && params.blink_caret,
    )
    .with_icon("🔍")
    .with_cursor_byte_idx(params.cursor_byte_idx)
    .with_clear_tag(ASSETS_TAG_SEARCH_CLEAR);
    scope.input_box_tagged(&search_props, ASSETS_TAG_SEARCH_INPUT);
}

/// Helper to render an action button in the toolbar.
fn build_action_btn(scope: &mut UiScope<'_>, label: &'static str, width: f32, tag: u64) {
    let style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(width)
        .height(24.0)
        .background(Color::rgba(0.10, 0.12, 0.16, 0.90))
        .hover_background(Color::rgba(0.18, 0.22, 0.30, 1.0))
        .border_radius(4.0)
        .border(1.0, Color::rgba(0.18, 0.22, 0.30, 0.60))
        .hover_border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.70));

    let text_c = Color::rgba(0.0, 0.85, 1.0, 0.95);

    scope.container_tagged("ActionBtn", style, WidgetRole::Button, tag, |btn| {
        let txt_id = btn.label_styled_passive(
            "ActionBtnText",
            label,
            11.0,
            text_c,
            TextAlign::Center,
            Style::new(),
        );
        btn.set_hover_text_color(txt_id, Color::WHITE);
    });
}

/// Helper to render the "+ Import" button with vector plus icon.
fn build_import_btn(scope: &mut UiScope<'_>) {
    let style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .gap(4.0)
        .width(68.0)
        .height(24.0)
        .background(Color::rgba(0.0, 0.52, 0.64, 0.95))
        .hover_background(Color::rgba(0.0, 0.65, 0.78, 1.0))
        .border_radius(4.0)
        .border(1.0, Color::rgba(0.0, 0.75, 0.90, 0.80))
        .hover_border(1.0, Color::rgba(0.0, 0.90, 1.0, 1.0));

    scope.container_tagged(
        "ImportBtn",
        style,
        WidgetRole::Button,
        ASSETS_TAG_IMPORT,
        |btn| {
            btn.icon(ICON_PLUS, Color::WHITE, 12.0);
            btn.label_styled_passive(
                "ImportBtnText",
                "Import",
                11.0,
                Color::WHITE,
                TextAlign::Left,
                Style::new(),
            );
        },
    );
}

/// Helper to render a selectable view mode toggle button.
fn build_mode_toggle_btn(
    scope: &mut UiScope<'_>,
    label: &'static str,
    width: f32,
    is_active: bool,
    tag: u64,
) {
    let mut style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width(width)
        .height(24.0)
        .border_radius(4.0);

    if is_active {
        style = style
            .background(Color::rgba(0.08, 0.15, 0.22, 0.95))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.90));
    } else {
        style = style
            .background(Color::rgba(0.08, 0.09, 0.12, 0.60))
            .hover_background(Color::rgba(0.14, 0.16, 0.22, 0.80))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.40))
            .hover_border(1.0, Color::rgba(0.30, 0.35, 0.45, 0.60));
    }

    let text_c = if is_active {
        Color::rgba(0.0, 0.90, 1.0, 1.0)
    } else {
        Color::rgba(0.65, 0.70, 0.80, 1.0)
    };

    scope.container_tagged("ModeToggleBtn", style, WidgetRole::Button, tag, |btn| {
        let txt_id = btn.label_styled_passive(
            "ModeToggleText",
            label,
            11.0,
            text_c,
            TextAlign::Center,
            Style::new(),
        );
        if !is_active {
            btn.set_hover_text_color(txt_id, Color::WHITE);
        }
    });
}

/// Helper to render the "Engine" content toggle button featuring a vector gear icon.
fn build_engine_toggle_btn(scope: &mut UiScope<'_>, is_active: bool) {
    let mut style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .gap(5.0)
        .width(74.0)
        .height(24.0)
        .border_radius(4.0);

    if is_active {
        style = style
            .background(Color::rgba(0.12, 0.18, 0.26, 0.95))
            .border(1.0, Color::rgba(0.0, 0.85, 1.0, 0.85));
    } else {
        style = style
            .background(Color::rgba(0.08, 0.09, 0.12, 0.60))
            .hover_background(Color::rgba(0.14, 0.16, 0.22, 0.80))
            .border(1.0, Color::rgba(0.16, 0.18, 0.24, 0.40))
            .hover_border(1.0, Color::rgba(0.30, 0.35, 0.45, 0.60));
    }

    scope.container_tagged(
        "EngineToggleBtn",
        style,
        WidgetRole::Button,
        ASSETS_TAG_ENGINE_CONTENT,
        |btn| {
            let tint = if is_active {
                Color::rgba(0.0, 0.95, 1.0, 1.0)
            } else {
                Color::rgba(0.65, 0.70, 0.80, 1.0)
            };
            let icon_id = btn.icon_named("EngineGearIcon", ICON_GEAR, tint, 14.0);
            if !is_active {
                btn.set_hover_texture_tint(icon_id, Color::WHITE);
            }

            let text_c = if is_active {
                Color::rgba(0.0, 0.95, 1.0, 1.0)
            } else {
                Color::rgba(0.70, 0.75, 0.85, 1.0)
            };
            let txt_id = btn.label_styled_passive(
                "EngineBtnText",
                "Engine",
                11.0,
                text_c,
                TextAlign::Left,
                Style::new(),
            );
            if !is_active {
                btn.set_hover_text_color(txt_id, Color::WHITE);
            }
        },
    );
}
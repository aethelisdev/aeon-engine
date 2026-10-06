// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Dialog Builder
//!
//! Assembles the complete hardware-accelerated GPU SDF Preferences modal dialog
//! declaratively using [`UiScope`], including glassmorphic card framing, titlebar,
//! sidebar tab navigation, content routing, scrollbar, and active dropdown popups.

use super::tabs::*;
use super::types::{
    PREF_TAG_CARD, PREF_TAG_CLOSE, PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PREF_TAG_TITLEBAR, PreferencesParams, encode_dropdown_item_tag,
    encode_tab_tag,
};
use irisui::prelude::*;

/// Width of the preferences modal card in physical pixels.
pub const PREF_CARD_WIDTH: f32 = 760.0;
/// Height of the preferences modal card in physical pixels.
pub const PREF_CARD_HEIGHT: f32 = 540.0;
/// Width of the left sidebar navigation tab strip in physical pixels.
pub const SIDEBAR_WIDTH: f32 = 160.0;
/// Height of the titlebar in physical pixels.
pub const TITLEBAR_HEIGHT: f32 = 36.0;

/// Sidebar tabs descriptor list: `(label, tab_index)`.
pub const SIDEBAR_TABS: [(&str, u8); 10] = [
    ("General", 0),
    ("Graphics", 1),
    ("Editor", 2),
    ("Navigation", 3),
    ("Input", 7),
    ("Keymap", 4),
    ("System", 5),
    ("Add-ons", 6),
    ("Modules", 9),
    ("Experimental", 8),
];

/// Returns the virtual scrollable height for a given preferences tab.
#[inline]
pub fn tab_virtual_height(tab_idx: u8) -> f32 {
    match tab_idx {
        0 => 500.0,
        1 => 1800.0, // Graphics tab: complete Shadows, Performance, AA, Bloom, Atmosphere, Clouds & Fog
        2 => 1100.0,
        3 => 650.0,
        4 => 1100.0,
        5 => 650.0,
        6 => 700.0,
        7 => 600.0,
        8 => 650.0,
        9 => 850.0,
        _ => 600.0,
    }
}

/// Constructs the complete Preferences dialog tree declaratively using [`UiScope`].
///
/// Returns `(card_id, card_rect, content_rect, max_scroll_y)`.
pub fn build_preferences_dialog(
    tree: &mut UiTree,
    mut params: PreferencesParams<'_>,
) -> (WidgetId, Rect, Rect, f32) {
    let screen_width = params.screen_width;
    let screen_height = params.screen_height;

    let (left, top) = if let Some(pos) = params.window_pos {
        let max_x = (screen_width - PREF_CARD_WIDTH).max(0.0);
        let max_y = (screen_height - PREF_CARD_HEIGHT).max(28.0);
        (
            pos.x.clamp(0.0, max_x).round(),
            pos.y.clamp(28.0, max_y).round(),
        )
    } else {
        (
            ((screen_width - PREF_CARD_WIDTH) * 0.5).max(0.0).round(),
            ((screen_height - PREF_CARD_HEIGHT) * 0.5).max(28.0).round(),
        )
    };

    let card_rect = Rect::new(left, top, PREF_CARD_WIDTH, PREF_CARD_HEIGHT);
    let content_w = PREF_CARD_WIDTH - SIDEBAR_WIDTH - 1.0;
    let content_h = PREF_CARD_HEIGHT - TITLEBAR_HEIGHT;
    let content_rect = Rect::new(
        left + SIDEBAR_WIDTH + 1.0,
        top + TITLEBAR_HEIGHT,
        content_w,
        content_h,
    );

    let root_id = tree
        .root()
        .or_else(|| tree.create_root().ok())
        .unwrap_or_default();
    let mut root_scope =
        UiScope::with_tagged_interactions(tree, root_id, params.events, params.hovered_tag)
            .with_active_text_input(params.active_number_input);

    let mut scroll_container_id: Option<WidgetId> = None;

    // 1. Floating Preferences Dialog Card
    let card_id = root_scope.container_tagged(
        "PreferencesDialogCard",
        Style::new()
            .position_absolute()
            .left(left)
            .top(top)
            .width(PREF_CARD_WIDTH)
            .height(PREF_CARD_HEIGHT)
            .flex_col()
            .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
            .border(1.0, Color::rgba(0.20, 0.23, 0.32, 1.0))
            .border_radius(10.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.75))
            .clip_children(true),
        WidgetRole::ModalWindow,
        PREF_TAG_CARD,
        |card| {
            // 2. Titlebar Strip - unified deep black matching sidebar with matching top rounded corners
            card.container_tagged(
                "PreferencesTitlebar",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .height(TITLEBAR_HEIGHT)
                    .padding_insets(Insets::new(0.0, 14.0, 0.0, 10.0))
                    .background(Color::rgba(0.07, 0.08, 0.10, 0.98))
                    .corner_radii(CornerRadii::new(10.0, 10.0, 0.0, 0.0)),
                WidgetRole::Default,
                PREF_TAG_TITLEBAR,
                |titlebar| {
                    titlebar.icon(
                        crate::ui::iris_bridge::icons::ICON_GEAR,
                        Color::rgba(0.85, 0.88, 0.94, 1.0),
                        15.0,
                    );

                    titlebar.empty_box_passive_named(
                        "PrefTitleGap",
                        Style::new().width(8.0).height(1.0),
                    );

                    titlebar.label_styled_passive(
                        "PrefTitleText",
                        "Preferences",
                        13.0,
                        Color::rgba(1.0, 1.0, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().flex_grow(1.0),
                    );

                    // Close Button
                    let is_close_hovered = params.hovered_tag == Some(PREF_TAG_CLOSE);
                    let close_bg = if is_close_hovered {
                        Color::rgba(0.85, 0.20, 0.25, 0.85)
                    } else {
                        Color::TRANSPARENT
                    };

                    titlebar.container_tagged(
                        "PrefCloseButton",
                        Style::new()
                            .width(26.0)
                            .height(26.0)
                            .background(close_bg)
                            .border_radius(4.0)
                            .align_items(AlignItems::Center)
                            .justify_content(JustifyContent::Center),
                        WidgetRole::Button,
                        PREF_TAG_CLOSE,
                        |btn| {
                            btn.label_styled_passive(
                                "PrefCloseIcon",
                                "✕",
                                12.0,
                                Color::rgba(0.90, 0.92, 0.96, 1.0),
                                TextAlign::Center,
                                Style::new(),
                            );
                        },
                    );
                },
            );

            // 1px Horizontal Titlebar Divider
            card.divider(Color::rgba(0.20, 0.22, 0.30, 0.70));

            // 3. Body: Sidebar + Divider + Content View
            card.container_named(
                "PreferencesBody",
                Style::new().flex_row().flex_grow(1.0).height(content_h),
                |body| {
                    // Left Sidebar
                    body.container_named(
                        "PreferencesSidebar",
                        Style::new()
                            .width(SIDEBAR_WIDTH)
                            .height(content_h)
                            .flex_col()
                            .padding_insets(Insets::new(8.0, 6.0, 8.0, 6.0))
                            .background(Color::rgba(0.07, 0.08, 0.10, 0.90)),
                        |sidebar| {
                            for &(label, tab_idx) in &SIDEBAR_TABS {
                                let tab_tag = encode_tab_tag(tab_idx);
                                let is_active = params.active_tab == tab_idx;
                                let is_hovered = params.hovered_tag == Some(tab_tag);

                                let (bg, text_col) = if is_active {
                                    (
                                        Color::rgba(0.0, 0.65, 0.85, 0.12),
                                        Color::rgba(0.0, 0.90, 1.0, 1.0),
                                    )
                                } else if is_hovered {
                                    (
                                        Color::rgba(0.18, 0.20, 0.28, 0.50),
                                        Color::rgba(1.0, 1.0, 1.0, 1.0),
                                    )
                                } else {
                                    (Color::TRANSPARENT, Color::rgba(0.80, 0.83, 0.90, 1.0))
                                };

                                sidebar.container_tagged(
                                    "PrefSidebarTab",
                                    Style::new()
                                        .flex_row()
                                        .align_items(AlignItems::Center)
                                        .height(30.0)
                                        .padding_insets(Insets::new(0.0, 10.0, 0.0, 0.0))
                                        .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0))
                                        .border_radius(4.0)
                                        .clip_children(true)
                                        .background(bg),
                                    WidgetRole::Button,
                                    tab_tag,
                                    |btn| {
                                        // Vertical Cyan Accent Bar for Active Tab anchored to the exact left edge
                                        let bar_color = if is_active {
                                            Color::rgba(0.0, 0.85, 1.0, 1.0)
                                        } else {
                                            Color::TRANSPARENT
                                        };

                                        btn.empty_box_passive_named(
                                            "PrefTabAccentBar",
                                            Style::new()
                                                .width(3.0)
                                                .height(30.0)
                                                .background(bar_color),
                                        );

                                        // 11px spacer ensures exactly 14.0px clean left padding from outer edge
                                        btn.empty_box_passive_named(
                                            "PrefTabSpacer",
                                            Style::new().width(11.0).height(1.0),
                                        );

                                        btn.label_styled_passive(
                                            "PrefTabLabel",
                                            label,
                                            12.0,
                                            text_col,
                                            TextAlign::Left,
                                            Style::new().flex_grow(1.0),
                                        );
                                    },
                                );
                            }
                        },
                    );

                    // Vertical Divider
                    body.vertical_divider(content_h, Color::rgba(0.20, 0.22, 0.30, 0.70));

                    // Right Content Area (Clipped Viewport)
                    body.container_tagged(
                        "PreferencesContentArea",
                        Style::new()
                            .width(content_w)
                            .height(content_h)
                            .clip_children(true)
                            .padding_insets(Insets::new(14.0, 18.0, 14.0, 18.0)),
                        WidgetRole::Default,
                        PREF_TAG_CONTENT_VIEW,
                        |content| {
                            // Virtual Scrolling Container
                            let sc_id = content.container_named(
                                "PreferencesScrollContainer",
                                Style::new()
                                    .flex_col()
                                    .gap(14.0)
                                    .scroll_offset_y(params.scroll_offset_y),
                                |scroll_col| {
                                    match params.active_tab {
                                        0 => build_general_tab(scroll_col, &mut params),
                                        1 => build_graphics_tab(scroll_col, &mut params),
                                        2 => build_editor_tab(scroll_col, &mut params),
                                        3 => build_navigation_tab(scroll_col, &mut params),
                                        4 => build_keymap_tab(scroll_col, &params),
                                        5 => build_system_tab(scroll_col, &params),
                                        6 => build_addons_tab(scroll_col, &params),
                                        7 => build_input_tab(scroll_col, &mut params),
                                        8 => build_experimental_tab(scroll_col, &params),
                                        9 => build_modules_tab(scroll_col, &params),
                                        _ => {}
                                    }
                                    scroll_col.empty_box_passive_named(
                                        "PrefScrollBottomSpacer",
                                        Style::new().height(24.0),
                                    );
                                },
                            );
                            scroll_container_id = Some(sc_id);
                        },
                    );
                },
            );

            // Calculate and assign layout geometry to the entire preferences dialog hierarchy
            card.finish_layout_with_hover(card_rect, params.cursor_pos);
        },
    );

    // Measure dynamic accumulated height of all tab content sections from layout pass
    let measured_total_h = scroll_container_id
        .and_then(|id| root_scope.tree().get(id))
        .map(|node| {
            node.children
                .iter()
                .filter_map(|&cid| root_scope.tree().get(cid))
                .map(|child| child.computed_rect.bottom() + params.scroll_offset_y - content_rect.y)
                .fold(0.0_f32, f32::max)
        })
        .unwrap_or(0.0);

    let total_h = measured_total_h
        .max(tab_virtual_height(params.active_tab))
        .max(content_h);
    let max_scroll_y = (total_h - content_h).max(0.0);

    // 4. Custom Scrollbar Indicator if virtual height exceeds content height
    if total_h > content_h {
        let style = ScrollAreaStyle {
            thickness: 6.0,
            inset: 3.0,
            ..ScrollAreaStyle::dark_default()
        };
        if let Some(geom) = ScrollBarGeometry::compute_vertical(
            content_rect,
            total_h,
            params.scroll_offset_y,
            &style,
        ) {
            root_scope.scrollbar_vertical(
                geom,
                Rect::ZERO,
                params.is_scrollbar_dragging,
                Some(params.cursor_pos),
                PREF_TAG_SCROLLBAR_TRACK,
                PREF_TAG_SCROLLBAR_THUMB,
            );
        }
    }

    // 5. Active Floating Dropdown Popup Menu Overlay (UiLayer::Popup)
    if let Some(dd_id) = params.active_dropdown {
        let options = dd_id.options();
        let popup_w = 180.0;
        let item_h = 24.0;
        let popup_h = options.len() as f32 * item_h + 8.0;

        // Position popup anchored directly below the ComboBox trigger button
        let (popup_x, popup_y) = if let Some(trigger) = params.dropdown_trigger_rect {
            let x = trigger.x.clamp(
                content_rect.x,
                (content_rect.right() - popup_w).max(content_rect.x),
            );
            // Place below trigger button, or flip above if it would overflow dialog
            let y = if trigger.bottom() + popup_h > card_rect.bottom() - 10.0 {
                (trigger.y - popup_h - 2.0).max(card_rect.y + TITLEBAR_HEIGHT + 4.0)
            } else {
                trigger.bottom() + 2.0
            };
            (x, y)
        } else {
            (
                (params.cursor_pos.x - 20.0).clamp(
                    content_rect.x,
                    (content_rect.right() - popup_w).max(content_rect.x),
                ),
                (params.cursor_pos.y + 10.0).clamp(
                    content_rect.y,
                    (content_rect.bottom() - popup_h).max(content_rect.y),
                ),
            )
        };
        let popup_rect = Rect::new(popup_x, popup_y, popup_w, popup_h);

        root_scope.container_tagged(
            "PrefDropdownPopupCard",
            Style::new()
                .position_absolute()
                .left(popup_x)
                .top(popup_y)
                .width(popup_w)
                .height(popup_h)
                .flex_col()
                .padding_insets(Insets::new(4.0, 4.0, 4.0, 4.0))
                .background(Color::rgba(0.08, 0.09, 0.12, 0.98))
                .border(1.0, Color::rgba(0.24, 0.28, 0.38, 1.0))
                .border_radius(5.0)
                .box_shadow(0.0, 6.0, 16.0, Color::rgba(0.0, 0.0, 0.0, 0.80)),
            WidgetRole::DropdownPopup,
            0,
            |menu| {
                for (idx, &item_label) in options.iter().enumerate() {
                    let item_tag = encode_dropdown_item_tag(idx);
                    let is_item_hovered = params.hovered_tag == Some(item_tag);

                    let item_bg = if is_item_hovered {
                        Color::rgba(0.0, 0.65, 0.85, 0.35)
                    } else {
                        Color::TRANSPARENT
                    };

                    menu.container_tagged(
                        "PrefDropdownMenuItem",
                        Style::new()
                            .flex_row()
                            .align_items(AlignItems::Center)
                            .height(item_h)
                            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0))
                            .border_radius(3.0)
                            .background(item_bg),
                        WidgetRole::DropdownItem,
                        item_tag,
                        |item| {
                            item.label_styled_passive(
                                "PrefDropdownMenuText",
                                item_label,
                                11.5,
                                if is_item_hovered {
                                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                                } else {
                                    Color::rgba(0.85, 0.88, 0.94, 1.0)
                                },
                                TextAlign::Left,
                                Style::new().flex_grow(1.0),
                            );
                        },
                    );
                }
                menu.finish_layout_with_hover(popup_rect, params.cursor_pos);
            },
        );
    }

    (card_id, card_rect, content_rect, max_scroll_y)
}
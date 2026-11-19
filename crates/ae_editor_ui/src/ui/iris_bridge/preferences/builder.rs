// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Dialog Builder
//!
//! Assembles the complete hardware-accelerated GPU SDF Preferences modal dialog
//! declaratively using [`UiScope`], including glassmorphic card framing, titlebar,
//! sidebar tab navigation, content routing, and declarative [`VerticalScrollbarSpec`].

use super::tabs::*;
use super::types::{
    PREF_TAG_CARD, PREF_TAG_CLOSE, PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLL_CONTAINER,
    PREF_TAG_SCROLLBAR_THUMB, PREF_TAG_SCROLLBAR_TRACK, PREF_TAG_TITLEBAR, PreferencesDialogState,
    PreferencesParams, encode_tab_tag, preferences_scroll_style,
};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
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

/// Sidebar tabs descriptor list with explicit semantic tags: `(label, tab_index, semantic_tag)`.
pub const SIDEBAR_TABS_TAGGED: [(&str, u8, u64); 10] = [
    ("General", 0, encode_tab_tag(0)),
    ("Graphics", 1, encode_tab_tag(1)),
    ("Editor", 2, encode_tab_tag(2)),
    ("Navigation", 3, encode_tab_tag(3)),
    ("Input", 7, encode_tab_tag(7)),
    ("Keymap", 4, encode_tab_tag(4)),
    ("System", 5, encode_tab_tag(5)),
    ("Add-ons", 6, encode_tab_tag(6)),
    ("Modules", 9, encode_tab_tag(9)),
    ("Experimental", 8, encode_tab_tag(8)),
];

/// Returns the virtual scrollable height for a given preferences tab.
#[inline]
pub fn tab_virtual_height(tab_idx: u8) -> f32 {
    match tab_idx {
        0 => 680.0,
        1 => 1800.0,
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

/// Calculates the clamped preferences floating window position during titlebar dragging.
pub fn calculate_preferences_drag_pos(
    cursor: Point,
    drag_offset: Point,
    screen_width: f32,
    screen_height: f32,
) -> Point {
    irisui::widgets::floating_panel::calculate_floating_drag_pos(
        cursor,
        drag_offset,
        (PREF_CARD_WIDTH, PREF_CARD_HEIGHT),
        screen_width,
        screen_height,
        crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT,
    )
}

/// Constructs the complete Preferences dialog tree declaratively using [`UiScope`].
///
/// Returns `(card_id, card_rect, content_rect, max_scroll_y)`.
pub fn build_preferences_dialog(
    tree: &mut UiTree,
    params: PreferencesParams<'_>,
    actions: &mut Vec<super::types::PreferencesAction>,
) -> (WidgetId, Rect, Rect, f32) {
    build_preferences_dialog_with_card(
        irisui::widgets::calculate_floating_card_rect(
            Size::new(params.screen_width, params.screen_height),
            Size::new(PREF_CARD_WIDTH, PREF_CARD_HEIGHT),
            params.window_pos,
            crate::ui::iris_bridge::menubar::MENUBAR_HEIGHT,
        ),
        (params.active_number_input, params.blink_caret),
        params.events,
        tree.root()
            .or_else(|| tree.create_root().ok())
            .unwrap_or_default(),
        tree,
        params,
        actions,
    )
}

fn build_preferences_dialog_with_card(
    card_rect: Rect,
    (active_number_input, blink_caret): (Option<(u64, &str, bool)>, bool),
    events: &[(u64, InteractionEvent)],
    root_id: WidgetId,
    tree: &mut UiTree,
    mut params: PreferencesParams<'_>,
    actions: &mut Vec<super::types::PreferencesAction>,
) -> (WidgetId, Rect, Rect, f32) {
    render_preferences_dialog_in_scope(
        UiScope::with_tagged_interactions(tree, root_id, events, None)
            .with_active_text_input_blink(active_number_input, blink_caret),
        &mut params,
        actions,
        (
            card_rect,
            Rect::new(
                card_rect.x + SIDEBAR_WIDTH + 1.0,
                card_rect.y + TITLEBAR_HEIGHT,
                PREF_CARD_WIDTH - SIDEBAR_WIDTH - 1.0,
                PREF_CARD_HEIGHT - TITLEBAR_HEIGHT,
            ),
        ),
    )
}

fn render_preferences_dialog_in_scope(
    mut root_scope: UiScope<'_>,
    params: &mut PreferencesParams<'_>,
    actions: &mut Vec<super::types::PreferencesAction>,
    (card_rect, content_rect): (Rect, Rect),
) -> (WidgetId, Rect, Rect, f32) {
    root_scope.container_tagged(
        "PreferencesDialogCard",
        Style::new()
            .position_absolute()
            .left(card_rect.x)
            .top(card_rect.y)
            .width(PREF_CARD_WIDTH)
            .height(PREF_CARD_HEIGHT)
            .flex_col()
            .background(root_scope.theme().modal_bg)
            .border(1.0, root_scope.theme().modal_border)
            .border_radius(10.0)
            .box_shadow(0.0, 8.0, 24.0, Color::rgba(0.0, 0.0, 0.0, 0.75))
            .clip_children(true),
        WidgetRole::FloatingWindow,
        PREF_TAG_CARD,
        |card| {
            card.container_tagged_resp(
                "PreferencesTitlebar",
                Style::new()
                    .flex_row()
                    .align_items(AlignItems::Center)
                    .justify_content(JustifyContent::SpaceBetween)
                    .height(TITLEBAR_HEIGHT)
                    .padding_insets(Insets::new(0.0, 14.0, 0.0, 10.0))
                    .background(card.theme().toolbar_bg)
                    .corner_radii(CornerRadii::new(10.0, 10.0, 0.0, 0.0)),
                WidgetRole::FloatingTitlebar,
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

                    if titlebar
                        .container_tagged_resp(
                            "PrefCloseButton",
                            Style::new()
                                .width(26.0)
                                .height(26.0)
                                .background(Color::TRANSPARENT)
                                .hover_background(Color::rgba(0.85, 0.20, 0.25, 0.85))
                                .border_radius(4.0)
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::Center),
                            WidgetRole::Button,
                            PREF_TAG_CLOSE,
                            |btn| {
                                btn.label_styled_hover_passive(
                                    "PrefCloseIcon",
                                    "✕",
                                    12.0,
                                    (Color::rgba(0.90, 0.92, 0.96, 1.0), Color::WHITE),
                                    TextAlign::Center,
                                    Style::new(),
                                );
                            },
                        )
                        .clicked()
                    {
                        actions.push(super::types::PreferencesAction::Close);
                    }
                },
            );

            card.divider_themed();
            card.container_named(
                "PreferencesBody",
                Style::new()
                    .flex_row()
                    .flex_grow(1.0)
                    .height(content_rect.height),
                |body| {
                    build_preferences_view(
                        body,
                        params,
                        actions,
                        content_rect.width,
                        content_rect.height,
                    );
                },
            );
            card.finish_layout(card_rect);
        },
    );

    (
        root_scope.node_id_tagged(PREF_TAG_CARD).unwrap_or_default(),
        card_rect,
        content_rect,
        root_scope.sync_and_render_vertical_scrollbar(
            VerticalScrollbarSpec::new(
                PREF_TAG_CONTENT_VIEW,
                PREF_TAG_SCROLL_CONTAINER,
                PREF_TAG_SCROLLBAR_TRACK,
                PREF_TAG_SCROLLBAR_THUMB,
                content_rect,
            )
            .host(card_rect, Some(PREF_TAG_CARD))
            .min_virtual_height(tab_virtual_height(*params.active_tab))
            .cursor(Some(params.cursor_pos))
            .style(preferences_scroll_style()),
        ),
    )
}

/// Renders the complete body of the preferences view (sidebar tabs, vertical divider, scrollable content area).
pub fn build_preferences_view(
    body: &mut UiScope<'_>,
    params: &mut PreferencesParams<'_>,
    actions: &mut Vec<super::types::PreferencesAction>,
    content_w: f32,
    content_h: f32,
) {
    body.sidebar_tabs_tagged(
        &SIDEBAR_TABS_TAGGED,
        params.active_tab,
        Some(PREF_TAG_CONTENT_VIEW),
    )
    .clicked_tab
    .into_iter()
    .for_each(|clicked| actions.push(super::types::PreferencesAction::SelectTab(clicked)));

    body.vertical_divider_themed(content_h);

    body.container_tagged(
        "PreferencesContentArea",
        Style::new()
            .width(content_w)
            .height(content_h)
            .clip_children(true)
            .scroll_offset_y(body.scroll_offset(PREF_TAG_CONTENT_VIEW))
            .padding_insets(Insets::new(14.0, 18.0, 14.0, 18.0)),
        WidgetRole::ScrollAreaViewport,
        PREF_TAG_CONTENT_VIEW,
        |content| {
            content.container_tagged(
                "PreferencesScrollContainer",
                Style::new().flex_col().gap(14.0),
                WidgetRole::Default,
                PREF_TAG_SCROLL_CONTAINER,
                |scroll_col| {
                    match *params.active_tab {
                        0 => build_general_tab(scroll_col, params, actions),
                        1 => build_graphics_tab(scroll_col, params),
                        2 => build_editor_tab(scroll_col, params, actions),
                        3 => build_navigation_tab(scroll_col, params),
                        4 => build_keymap_tab(scroll_col, params),
                        5 => build_system_tab(scroll_col, params),
                        6 => build_addons_tab(scroll_col, params),
                        7 => build_input_tab(scroll_col, params),
                        8 => build_experimental_tab(scroll_col, params),
                        9 => build_modules_tab(scroll_col, params, actions),
                        _ => {}
                    }
                    scroll_col.empty_box_passive_named(
                        "PrefScrollBottomSpacer",
                        Style::new().height(24.0),
                    );
                },
            );
        },
    );
}

impl PreferencesDialogState {
    /// Builds the Preferences panel content directly using mutable preferences references and `UiScope`.
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &mut OverlayUpdateParams<'_>,
        (cursor_pos, blink_caret): (Point, bool),
    ) {
        if scope.is_collapsed() {
            return;
        }

        self.build_view_in_rect(
            scope,
            scope.parent_rect(),
            params,
            (cursor_pos, blink_caret),
        );
    }

    fn build_view_in_rect(
        &mut self,
        scope: &mut UiScope<'_>,
        panel_rect: Rect,
        params: &mut OverlayUpdateParams<'_>,
        (cursor_pos, blink_caret): (Point, bool),
    ) {
        scope.with_scoped_text_input_blink(
            self.active_number_input
                .as_ref()
                .map(|(tag, text, selected)| (*tag, text.as_str(), *selected)),
            blink_caret,
            |scoped_scope| {
                scoped_scope.container_named(
                    "PreferencesPanelBody",
                    Style::new()
                        .flex_row()
                        .width(panel_rect.width)
                        .height(panel_rect.height)
                        .background(scoped_scope.theme().panel_bg),
                    |body| {
                        build_preferences_view(
                            body,
                            &mut PreferencesParams {
                                screen_width: panel_rect.width,
                                screen_height: panel_rect.height,
                                window_pos: None,
                                active_tab: &mut self.tab,
                                collapsed_sections: &self.collapsed_sections,
                                active_number_input: self
                                    .active_number_input
                                    .as_ref()
                                    .map(|(tag, text, selected)| (*tag, text.as_str(), *selected)),
                                blink_caret,
                                cursor_pos: body.resolved_cursor_pos(cursor_pos),
                                zoom_factor: params.context.zoom_factor,
                                graphics_settings: &mut *params.preferences.graphics_settings,
                                snapping_settings: &mut *params.preferences.snapping_settings,
                                editor_config: &mut *params.preferences.editor_config,
                                enable_live_updates: &mut *params.preferences.enable_live_updates,
                                enabled_modules: params.preferences.enabled_modules,
                                events: &self.pending_interaction_events,
                                current_theme: self.theme_preset,
                            },
                            &mut self.actions,
                            (panel_rect.width - SIDEBAR_WIDTH - 1.0).max(100.0),
                            panel_rect.height,
                        );
                    },
                );
            },
        );
        self.pending_interaction_events.clear();

        scope.sync_and_render_vertical_scrollbar(
            VerticalScrollbarSpec::new(
                PREF_TAG_CONTENT_VIEW,
                PREF_TAG_SCROLL_CONTAINER,
                PREF_TAG_SCROLLBAR_TRACK,
                PREF_TAG_SCROLLBAR_THUMB,
                Rect::new(
                    panel_rect.x + SIDEBAR_WIDTH + 1.0,
                    panel_rect.y,
                    (panel_rect.width - SIDEBAR_WIDTH - 1.0).max(100.0),
                    panel_rect.height,
                ),
            )
            .host(panel_rect, None)
            .min_virtual_height(tab_virtual_height(self.tab))
            .cursor(Some(cursor_pos))
            .style(preferences_scroll_style()),
        );
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Panel Builder
//!
//! Assembles the dockable GPU SDF Preferences configuration panel declaratively
//! using [`UiScope`], including sidebar tab navigation, scrollable content view,
//! declarative [`VerticalScrollbarSpec`], and direct [`ActionBus<EngineUiAction>`] emission.

use super::tabs::*;
use super::types::{
    PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLL_CONTAINER, PREF_TAG_SCROLLBAR_THUMB,
    PREF_TAG_SCROLLBAR_TRACK, PreferencesDialogState, PreferencesParams, preferences_scroll_style,
};
use crate::ui::iris_bridge::types::OverlayUpdateParams;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

/// Width of the left sidebar navigation tab strip in logical pixels.
pub const SIDEBAR_WIDTH: f32 = 160.0;

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
pub const fn tab_virtual_height(tab_idx: u8) -> f32 {
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

/// Renders the complete body of the preferences panel (sidebar tabs, divider, scrollable content area).
pub fn build_preferences_view(
    body: &mut UiScope<'_>,
    params: &mut PreferencesParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
    content_w: f32,
    content_h: f32,
) {
    body.sidebar_tabs(
        &SIDEBAR_TABS,
        params.active_tab,
        Some(PREF_TAG_CONTENT_VIEW),
    );
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
                        0 => build_general_tab(scroll_col, params, action_bus),
                        1 => build_graphics_tab(scroll_col, params),
                        2 => build_editor_tab(scroll_col, params),
                        3 => build_navigation_tab(scroll_col, params),
                        4 => build_keymap_tab(scroll_col),
                        5 => build_system_tab(scroll_col),
                        6 => build_addons_tab(scroll_col),
                        7 => build_input_tab(scroll_col, params),
                        8 => build_experimental_tab(scroll_col),
                        9 => build_modules_tab(scroll_col, params, action_bus),
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
    /// Builds the Preferences panel content directly using mutable preferences references and [`UiScope`].
    pub(crate) fn build_view(
        &mut self,
        scope: &mut UiScope<'_>,
        params: &mut OverlayUpdateParams<'_>,
        (cursor_pos, blink_caret): (Point, bool),
        action_bus: &mut ActionBus<EngineUiAction>,
    ) {
        if scope.is_collapsed() {
            return;
        }

        self.build_view_in_rect(
            scope,
            scope.parent_rect(),
            params,
            (cursor_pos, blink_caret),
            action_bus,
        );
    }

    fn build_view_in_rect(
        &mut self,
        scope: &mut UiScope<'_>,
        panel_rect: Rect,
        params: &mut OverlayUpdateParams<'_>,
        (cursor_pos, blink_caret): (Point, bool),
        action_bus: &mut ActionBus<EngineUiAction>,
    ) {
        scope.with_scoped_text_input_blink(None, blink_caret, |scoped_scope| {
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
                            active_tab: &mut self.tab,
                            zoom_factor: params.context.zoom_factor,
                            graphics_settings: &mut *params.preferences.graphics_settings,
                            snapping_settings: &mut *params.preferences.snapping_settings,
                            editor_config: &mut *params.preferences.editor_config,
                            enable_live_updates: &mut *params.preferences.enable_live_updates,
                            enabled_modules: params.preferences.enabled_modules,
                            current_theme: &mut self.theme_preset,
                        },
                        action_bus,
                        (panel_rect.width - SIDEBAR_WIDTH - 1.0).max(100.0),
                        panel_rect.height,
                    );
                },
            );
        });

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
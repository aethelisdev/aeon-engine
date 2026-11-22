// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Preferences Panel Builder
//!
//! Assembles the dockable GPU SDF Preferences configuration panel declaratively
//! using [`UiScope`], [`ScrollAreaView`], [`UiScope::edit_with_state`], and
//! direct [`ActionBus<EngineUiAction>`] emission.

use super::tabs::*;
use super::types::{
    PREF_TAG_CONTENT_VIEW, PREF_TAG_SCROLLBAR_THUMB, PREF_TAG_SCROLLBAR_TRACK,
    PreferencesDialogState, preferences_scroll_style,
};
use crate::ui::panel_registry::EditorPanelContext;
use crate::ui::types::{EditorFrameData, EngineUiAction};
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

/// Renders the complete body of the preferences panel (sidebar tabs, divider, autonomous scrollable content area).
pub fn build_preferences_view(
    body: &mut UiScope<'_>,
    state: &mut PreferencesDialogState,
    frame: &EditorFrameData<'_>,
    zoom_factor: f32,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    body.container_named(
        "PrefSidebarContainer",
        Style::new()
            .flex_col()
            .width(SIDEBAR_WIDTH)
            .flex_shrink(0.0),
        |sb| {
            sb.sidebar_tabs(&SIDEBAR_TABS, &mut state.tab, Some(PREF_TAG_CONTENT_VIEW));
        },
    );
    body.vertical_divider_fill_themed();

    ScrollAreaView::new("PreferencesContentArea")
        .tag(PREF_TAG_CONTENT_VIEW)
        .padding(Insets::new(14.0, 18.0, 14.0, 18.0))
        .gap(14.0)
        .scrollbar_tags(PREF_TAG_SCROLLBAR_TRACK, PREF_TAG_SCROLLBAR_THUMB)
        .scrollbar_style(preferences_scroll_style())
        .scroll_key(u64::from(state.tab))
        .show_scrolled(body, |scroll_col| {
            render_active_preferences_tab(scroll_col, state, frame, zoom_factor, action_bus);
            scroll_col.empty_box_passive_named("PrefScrollBottomSpacer", Style::new().height(24.0));
        });
}

fn render_active_preferences_tab(
    scroll_col: &mut UiScope<'_>,
    state: &mut PreferencesDialogState,
    frame: &EditorFrameData<'_>,
    zoom_factor: f32,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    match state.tab {
        0 => build_general_tab(scroll_col, zoom_factor, &mut state.theme_preset, action_bus),
        1 => scroll_col.edit_with_state(
            action_bus,
            *frame.graphics_settings,
            |s, _bus, gfx| build_graphics_tab(s, gfx),
            |bus, updated| bus.emit(EngineUiAction::UpdateGraphicsSettings(updated)),
        ),
        2 => scroll_col.edit_with_state(
            action_bus,
            *frame.snapping_settings,
            |s, bus, snap| {
                s.edit_with_state(
                    bus,
                    *frame.editor_config,
                    |s2, bus2, cfg| {
                        s2.edit_with_state(
                            bus2,
                            frame.enable_live_updates,
                            |s3, _bus3, live| build_editor_tab(s3, snap, cfg, live),
                            |b, updated_live| {
                                b.emit(EngineUiAction::SetLiveEditorUpdates(updated_live))
                            },
                        );
                    },
                    |b, updated_cfg| b.emit(EngineUiAction::UpdateEditorConfig(updated_cfg)),
                );
            },
            |b, updated_snap| b.emit(EngineUiAction::UpdateSnapSettings(updated_snap)),
        ),
        3 => scroll_col.edit_with_state(
            action_bus,
            *frame.editor_config,
            |s, _bus, cfg| build_navigation_tab(s, cfg),
            |bus, updated| bus.emit(EngineUiAction::UpdateEditorConfig(updated)),
        ),
        4 => build_keymap_tab(scroll_col),
        5 => build_system_tab(scroll_col),
        6 => build_addons_tab(scroll_col),
        7 => scroll_col.edit_with_state(
            action_bus,
            *frame.editor_config,
            |s, _bus, cfg| build_input_tab(s, cfg),
            |bus, updated| bus.emit(EngineUiAction::UpdateEditorConfig(updated)),
        ),
        8 => build_experimental_tab(scroll_col),
        9 => build_modules_tab(scroll_col, frame.enabled_modules, action_bus),
        _ => {}
    }
}

impl PreferencesDialogState {
    /// Entrypoint invoked directly by [`irisui::workbench::ScopedPanelRegistry`].
    pub(crate) fn draw_panel(&mut self, scope: &mut UiScope<'_>, ctx: &mut EditorPanelContext<'_>) {
        build_preferences_view(scope, self, &ctx.data, ctx.zoom_factor, ctx.action_bus);
    }
}
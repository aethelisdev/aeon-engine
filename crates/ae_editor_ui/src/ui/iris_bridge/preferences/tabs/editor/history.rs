// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # History Preferences Card
//!
//! Renders undo history limit configuration declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{pref_section_card, pref_slider_row};
use crate::ui::iris_bridge::preferences::types::{PreferencesParams, PreferencesSliderId};
use ae_editor::editor_state::EditorConfig;
use irisui::prelude::*;

/// Builds the History settings card declaratively using [`UiScope`].
pub fn build_history_card(
    scope: &mut UiScope<'_>,
    params: &PreferencesParams<'_>,
    cfg: &EditorConfig,
) {
    let is_collapsed = params.collapsed_sections.contains("editor_history");

    pref_section_card(
        scope,
        "editor_history",
        "📝  History Settings",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_slider_row(
                body,
                PreferencesSliderId::UndoHistoryLimit,
                "Undo History Limit",
                cfg.max_undo_history as f32,
                params.active_number_input,
                params.blink_caret,
                params.hovered_tag,
            );
        },
    );
}
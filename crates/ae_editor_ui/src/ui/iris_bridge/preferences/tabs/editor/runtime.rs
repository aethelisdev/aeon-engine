// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Runtime Preferences Card
//!
//! Renders engine hot-reload and live editor runtime execution settings
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::pref_section_card;
use crate::ui::iris_bridge::preferences::types::{
    PreferencesAction, PreferencesParams, PreferencesToggleId, encode_toggle_tag,
};
use irisui::prelude::*;

/// Builds the Runtime settings card declaratively using [`UiScope`].
pub fn build_runtime_card(
    scope: &mut UiScope<'_>,
    params: &mut PreferencesParams<'_>,
    actions: &mut Vec<PreferencesAction>,
) {
    pref_section_card(
        scope,
        "editor_runtime",
        "⚙  Runtime Settings",
        params.collapsed_sections.contains("editor_runtime"),
        |body| {
            body.checkbox_tagged(
                "Enable Live Editor Updates (Hot Reload)",
                encode_toggle_tag(PreferencesToggleId::LiveUpdatesEnabled),
                params.enable_live_updates,
            )
            .on_click(|| {
                actions.push(PreferencesAction::Toggle(
                    PreferencesToggleId::LiveUpdatesEnabled,
                ));
            });
        },
    );
}
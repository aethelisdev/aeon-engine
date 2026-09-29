// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Runtime Preferences Card
//!
//! Renders engine hot-reload and live editor runtime execution settings
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{pref_section_card, pref_toggle_row};
use crate::ui::iris_bridge::preferences::types::{PreferencesParams, PreferencesToggleId};
use irisui::prelude::*;

/// Builds the Runtime settings card declaratively using [`UiScope`].
pub fn build_runtime_card(
    scope: &mut UiScope<'_>,
    params: &PreferencesParams<'_>,
    enable_live_updates: bool,
) {
    let is_collapsed = params.collapsed_sections.contains("editor_runtime");

    pref_section_card(
        scope,
        "editor_runtime",
        "⚙  Runtime Settings",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_toggle_row(
                body,
                PreferencesToggleId::LiveUpdatesEnabled,
                "Enable Live Editor Updates (Hot Reload)",
                enable_live_updates,
                params.hovered_tag,
            );
        },
    );
}
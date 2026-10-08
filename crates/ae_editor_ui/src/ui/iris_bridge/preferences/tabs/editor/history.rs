// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # History Preferences Card
//!
//! Renders undo history limit configuration declaratively using [`UiScope`] and two-way property primitives.

use crate::ui::iris_bridge::preferences::components::pref_section_card;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Builds the History settings card declaratively using [`UiScope`].
///
/// Binds maximum undo/redo limit directly to `cfg.max_undo_history` via immediate two-way property primitives.
pub fn build_history_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("editor_history");
    let cfg = &mut *params.editor_config;

    pref_section_card(
        scope,
        "editor_history",
        "📝  History Settings",
        is_collapsed,
        |body| {
            let mut undo_limit = cfg.max_undo_history as f32;
            let opts = PropertySliderOptions::new(10.0, 5000.0, 10.0)
                .with_format("{:.0}")
                .with_subtitle(
                    "Maximum number of actions stored in RAM. Lower values prevent memory bloat during extremely long sessions.",
                );

            body.property_slider_with_options("Undo History Limit", &mut undo_limit, opts);
            cfg.max_undo_history = undo_limit.round() as usize;
        },
    );
}
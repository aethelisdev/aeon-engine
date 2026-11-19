// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # History & Auto-Save Preferences Card
//!
//! Renders undo history limit and periodic background auto-save configuration
//! declaratively using [`UiScope`] and two-way property primitives.

use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PropertySliderOptions;

/// Builds the History & Auto-Save settings card declaratively using [`UiScope`].
pub fn build_history_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.collapsing_section("editor_history", "📝  History & Auto-Save", true, |body| {
        body.property_slider_usize_with_options(
            "Undo History Limit",
            &mut params.editor_config.max_undo_history,
            PropertySliderOptions::new(10.0, 5000.0, 10.0)
                .with_format("{:.0}")
                .with_subtitle(
                    "Maximum number of actions stored in RAM. Lower values prevent memory bloat during extremely long sessions.",
                ),
        );
        body.divider(body.theme().card_border);
        body.property_checkbox(
            "Enable Periodic Auto-Save",
            &mut params.editor_config.autosave_enabled,
        );
        body.when(params.editor_config.autosave_enabled, |s| {
            s.property_slider_with_options(
                "Auto-Save Interval",
                &mut params.editor_config.autosave_interval_secs,
                PropertySliderOptions::new(30.0, 1800.0, 5.0)
                    .with_format("{:.0} s")
                    .with_subtitle(
                        "Seconds of Edit-mode activity between rotating recovery snapshots inside Saved/Autosaves/.",
                    ),
            );
        });
    });
}
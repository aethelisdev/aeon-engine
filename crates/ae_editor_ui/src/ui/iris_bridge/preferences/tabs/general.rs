// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # General Preferences Tab
//!
//! Renders global engine settings, language configuration, and display / UI scale settings.

use super::super::components::{pref_dropdown_row, pref_heading, pref_section_card};
use super::super::types::{PreferencesDropdownId, PreferencesParams};
use irisui::prelude::*;

/// Scales table supported in the UI scale ComboBox.
pub const UI_SCALES: [(f32, &str); 7] = [
    (0.75, "75%"),
    (0.80, "80%"),
    (0.90, "90%"),
    (1.00, "100% (Default)"),
    (1.10, "110%"),
    (1.25, "125%"),
    (1.50, "150%"),
];

/// Builds the General & Interface preferences tab content declaratively using [`UiScope`].
pub fn build_general_tab(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "General & Interface",
        "Global engine settings, language, and display scaling.",
    );

    let is_collapsed = params.collapsed_sections.contains("general_scale");
    pref_section_card(
        scope,
        "general_scale",
        "🔍  Display & UI Scale",
        is_collapsed,
        params.hovered_tag,
        |body| {
            body.label_styled_passive(
                "ScaleDesc",
                "Adjust interface scale for different monitor resolutions (or use Ctrl + / Ctrl - shortcuts):",
                11.5,
                Color::rgba(0.70, 0.73, 0.80, 1.0),
                TextAlign::Left,
                Style::new().margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
            );

            let current_zoom = params.zoom_factor;
            let selected_label = UI_SCALES
                .iter()
                .find(|(val, _)| (current_zoom - *val).abs() < 0.01)
                .map(|(_, l)| *l)
                .unwrap_or("100% (Default)");

            let is_open = params.active_dropdown == Some(PreferencesDropdownId::UiScale);
            pref_dropdown_row(
                body,
                PreferencesDropdownId::UiScale,
                "Interface Scale",
                selected_label,
                is_open,
                params.hovered_tag,
            );
        },
    );
}
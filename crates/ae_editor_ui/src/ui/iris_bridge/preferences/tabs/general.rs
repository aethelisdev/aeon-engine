// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # General Preferences Tab
//!
//! Renders global engine settings, language configuration, display / UI scale settings,
//! and viewport synchronization options declaratively using [`UiScope`].

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
pub fn build_general_tab(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    pref_heading(
        scope,
        "General & Interface",
        "Global engine settings, display scaling, theme, and viewport synchronization.",
    );

    let is_scale_collapsed = params.collapsed_sections.contains("general_scale");
    let hovered_tag = params.hovered_tag;

    pref_section_card(
        scope,
        "general_scale",
        "🔍  Display & UI Scale",
        is_scale_collapsed,
        hovered_tag,
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
                hovered_tag,
            );
        },
    );

    let is_sync_collapsed = params.collapsed_sections.contains("general_viewport");

    pref_section_card(
        scope,
        "general_viewport",
        "🖥  Viewport & Synchronization",
        is_sync_collapsed,
        hovered_tag,
        |body| {
            body.container_named(
                "SyncInfoBox",
                Style::new()
                    .flex_col()
                    .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                    .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                    .border_radius(6.0)
                    .padding_insets(Insets::new(10.0, 12.0, 10.0, 12.0)),
                |card| {
                    card.label_styled_passive(
                        "SyncInfoTitle",
                        "⚡ Frame Presentation Cadence",
                        12.0,
                        Color::rgba(1.0, 1.0, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    );
                    card.label_styled_passive(
                        "SyncInfoDesc",
                        "VSync locks presentation to monitor refresh rate to eliminate tearing. Framerate caps, MSAA samples, and GPU pacing options can be tuned under the Graphics tab.",
                        11.0,
                        Color::rgba(0.65, 0.68, 0.76, 1.0),
                        TextAlign::Left,
                        Style::new(),
                    );
                },
            );
        },
    );
}
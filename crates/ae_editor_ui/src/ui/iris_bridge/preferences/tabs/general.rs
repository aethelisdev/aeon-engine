// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # General Preferences Tab
//!
//! Renders global display / UI scale settings, theme selection, and viewport
//! synchronization details declaratively using [`UiScope`].

use super::super::types::PreferencesParams;
use super::PREF_COMBO_LAYOUT;
use crate::ui::iris_bridge::theme::ThemePreset;
use crate::ui::types::EngineUiAction;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

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

const UI_SCALE_LABELS: [&str; 7] = [
    "75%",
    "80%",
    "90%",
    "100% (Default)",
    "110%",
    "125%",
    "150%",
];

const THEME_LABELS: [&str; 2] = [
    "Obsidian Deep (High Contrast)",
    "Cold Slate (Studio Neutral)",
];

/// Builds the General & Interface preferences tab content declaratively using [`UiScope`].
pub fn build_general_tab(
    scope: &mut UiScope<'_>,
    params: &mut PreferencesParams<'_>,
    action_bus: &mut ActionBus<EngineUiAction>,
) {
    scope.section_heading(
        "General & Interface",
        "Global engine settings, display scaling, theme, and viewport synchronization.",
    );

    scope.collapsing_section(
        "general_scale",
        "🔍  Display & UI Scale",
        true,
        |body| {
            body.label_styled_passive(
                "ScaleDesc",
                "Adjust interface scale for different monitor resolutions (or use Ctrl + / Ctrl - shortcuts):",
                11.5,
                Color::rgba(0.70, 0.73, 0.80, 1.0),
                TextAlign::Left,
                Style::new().margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
            );

            body.prop_combobox_mapped_styled_on_change(
                ComboBoxMapConfig::new(
                    "Interface Scale",
                    UI_SCALES
                        .iter()
                        .position(|(val, _)| (params.zoom_factor - *val).abs() < 0.01)
                        .unwrap_or(3),
                    &UI_SCALE_LABELS,
                )
                .with_layout(PREF_COMBO_LAYOUT),
                &mut params.zoom_factor,
                |idx| UI_SCALES.get(idx).map_or(1.0, |(val, _)| *val),
                |&scale| action_bus.emit(EngineUiAction::SetUiScale(scale)),
            );
        },
    );

    scope.collapsing_section("general_theme", "🎨  Workspace Theme", true, |body| {
        body.label_styled_passive(
            "ThemeDesc",
            "Select workspace color elevation palette (Obsidian Deep or Cold Slate):",
            11.5,
            Color::rgba(0.70, 0.73, 0.80, 1.0),
            TextAlign::Left,
            Style::new().margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
        );

        body.prop_combobox_mapped_styled_on_change(
            ComboBoxMapConfig::new(
                "Theme Preset",
                match *params.current_theme {
                    ThemePreset::ObsidianDeep => 0,
                    ThemePreset::ColdSlate => 1,
                },
                &THEME_LABELS,
            )
            .with_layout(PREF_COMBO_LAYOUT),
            params.current_theme,
            |idx| {
                if idx == 0 {
                    ThemePreset::ObsidianDeep
                } else {
                    ThemePreset::ColdSlate
                }
            },
            |&preset| action_bus.emit(EngineUiAction::SetThemePreset(preset)),
        );
    });

    scope.collapsing_section(
        "general_viewport",
        "🖥  Viewport & Synchronization",
        true,
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
                        "SyncInfoDesc1",
                        "VSync locks presentation to monitor refresh rate to eliminate tearing.",
                        11.0,
                        Color::rgba(0.65, 0.68, 0.76, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
                    );
                    card.label_styled_passive(
                        "SyncInfoDesc2",
                        "Framerate caps, MSAA samples, and GPU pacing options can be tuned under the Graphics tab.",
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
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Snapping Preferences Card
//!
//! Renders the snapping mode selection ComboBox and grid size slider
//! declaratively using [`UiScope`] and two-way property primitives.

use crate::ui::iris_bridge::preferences::components::{pref_dropdown_row, pref_section_card};
use crate::ui::iris_bridge::preferences::types::{PreferencesDropdownId, PreferencesParams};
use ae_editor::snapping::SnapMode;
use irisui::prelude::*;

/// Predefined selectable snapping modes.
pub const SNAP_MODE_OPTIONS: [(&str, SnapMode); 3] = [
    ("Off", SnapMode::Off),
    ("Hold (Ctrl)", SnapMode::Hold),
    ("Toggle", SnapMode::Toggle),
];

/// Builds the Snapping settings card declaratively using [`UiScope`].
///
/// Binds grid size directly to `snapping.grid_size` via immediate two-way property primitives.
pub fn build_snapping_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("editor_snapping");
    let active_dropdown = params.active_dropdown;
    let snapping = &mut *params.snapping_settings;

    let snap_mode_label = match snapping.mode {
        SnapMode::Off => "Off",
        SnapMode::Hold => "Hold (Ctrl)",
        SnapMode::Toggle => "Toggle",
    };

    let is_dropdown_active = active_dropdown == Some(PreferencesDropdownId::SnapMode);

    pref_section_card(
        scope,
        "editor_snapping",
        "🧲  Snapping",
        is_collapsed,
        |body| {
            pref_dropdown_row(
                body,
                PreferencesDropdownId::SnapMode,
                "Snap Mode",
                snap_mode_label,
                is_dropdown_active,
            );

            let opts = PropertySliderOptions::new(0.1, 10.0, 0.05)
                .with_format("{:.2}")
                .with_subtitle(
                    "Grid spacing unit for viewport translation and object placement snapping.",
                );

            body.property_slider_with_options("Grid Size", &mut snapping.grid_size, opts);
        },
    );
}
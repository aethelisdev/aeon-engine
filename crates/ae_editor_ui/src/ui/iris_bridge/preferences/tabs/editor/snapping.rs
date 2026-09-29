// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Snapping Preferences Card
//!
//! Renders the snapping mode selection ComboBox and grid size slider
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{
    pref_dropdown_row, pref_section_card, pref_slider_row,
};
use crate::ui::iris_bridge::preferences::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId,
};
use ae_editor::snapping::{SnapMode, SnapSettings};
use irisui::prelude::*;

/// Predefined selectable snapping modes.
pub const SNAP_MODE_OPTIONS: [(&str, SnapMode); 3] = [
    ("Off", SnapMode::Off),
    ("Hold (Ctrl)", SnapMode::Hold),
    ("Toggle", SnapMode::Toggle),
];

/// Builds the Snapping settings card declaratively using [`UiScope`].
pub fn build_snapping_card(
    scope: &mut UiScope<'_>,
    params: &PreferencesParams<'_>,
    snapping: &SnapSettings,
) {
    let is_collapsed = params.collapsed_sections.contains("editor_snapping");

    let snap_mode_label = match snapping.mode {
        SnapMode::Off => "Off",
        SnapMode::Hold => "Hold (Ctrl)",
        SnapMode::Toggle => "Toggle",
    };

    pref_section_card(
        scope,
        "editor_snapping",
        "🧲  Snapping",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_dropdown_row(
                body,
                PreferencesDropdownId::SnapMode,
                "Snap Mode",
                snap_mode_label,
                params.active_dropdown == Some(PreferencesDropdownId::SnapMode),
                params.hovered_tag,
            );

            pref_slider_row(
                body,
                PreferencesSliderId::GridSize,
                "Grid Size",
                snapping.grid_size,
                params.active_number_input,
                params.blink_caret,
                params.hovered_tag,
            );
        },
    );
}
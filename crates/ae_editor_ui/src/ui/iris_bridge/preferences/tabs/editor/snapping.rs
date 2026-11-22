// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Snapping Preferences Card
//!
//! Renders the snapping mode selection ComboBox and grid size slider
//! declaratively using [`UiScope`] and two-way property primitives.

use crate::ui::iris_bridge::preferences::tabs::PREF_COMBO_LAYOUT;
use ae_editor::snapping::{SnapMode, SnapSettings};
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PropertySliderOptions;

const SNAP_MODE_OPTIONS: [SnapMode; 3] = [SnapMode::Off, SnapMode::Hold, SnapMode::Toggle];
const SNAP_MODE_LABELS: [&str; 3] = ["Off", "Hold (Ctrl)", "Toggle"];

/// Builds the Snapping settings card declaratively using [`UiScope`].
pub fn build_snapping_card(scope: &mut UiScope<'_>, snapping: &mut SnapSettings) {
    scope.collapsing_section("editor_snapping", "🧲  Snapping", true, |body| {
        body.prop_combobox_mapped_styled(
            "Snap Mode",
            match snapping.mode {
                SnapMode::Off => 0,
                SnapMode::Hold => 1,
                SnapMode::Toggle => 2,
            },
            &SNAP_MODE_LABELS,
            PREF_COMBO_LAYOUT,
            &mut snapping.mode,
            |idx| SNAP_MODE_OPTIONS.get(idx).copied().unwrap_or(SnapMode::Off),
        );

        body.property_slider_with_options(
            "Grid Size",
            &mut snapping.grid_size,
            PropertySliderOptions::new(0.1, 10.0, 0.05)
                .with_format("{:.2}")
                .with_subtitle(
                    "Grid spacing unit for viewport translation and object placement snapping.",
                ),
        );
    });
}
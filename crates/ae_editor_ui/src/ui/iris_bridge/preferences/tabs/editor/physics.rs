// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Preferences Card
//!
//! Renders physics simulation frequency settings declaratively using [`UiScope`] and two-way property primitives.

use ae_editor::editor_state::EditorConfig;
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PropertySliderOptions;

/// Builds the Physics settings card declaratively using [`UiScope`].
pub fn build_physics_card(scope: &mut UiScope<'_>, editor_config: &mut EditorConfig) {
    scope.collapsing_section("editor_physics", "🎮  Physics Settings", true, |body| {
        body.property_slider_with_options(
            "Fixed Update Frequency",
            &mut editor_config.physics_hz,
            PropertySliderOptions::new(30.0, 240.0, 1.0)
                .with_format("{:.0} Hz")
                .with_subtitle(
                    "Physics simulation frequency. Higher values improve simulation accuracy but increase CPU usage.",
                ),
        );
    });
}
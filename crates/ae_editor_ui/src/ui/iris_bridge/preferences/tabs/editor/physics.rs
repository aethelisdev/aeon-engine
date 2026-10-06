// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Preferences Card
//!
//! Renders physics simulation frequency settings declaratively using [`UiScope`] and two-way property primitives.

use crate::ui::iris_bridge::preferences::components::pref_section_card;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Builds the Physics settings card declaratively using [`UiScope`].
///
/// Binds simulation update rate directly to `cfg.physics_hz` via immediate two-way property primitives.
pub fn build_physics_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("editor_physics");
    let hovered_tag = params.hovered_tag;
    let cfg = &mut *params.editor_config;

    pref_section_card(
        scope,
        "editor_physics",
        "🎮  Physics Settings",
        is_collapsed,
        hovered_tag,
        |body| {
            let opts = PropertySliderOptions::new(30.0, 240.0, 1.0)
                .with_format("{:.0} Hz")
                .with_subtitle(
                    "Physics simulation frequency. Higher values improve simulation accuracy but increase CPU usage.",
                );

            body.property_slider_with_options("Fixed Update Frequency", &mut cfg.physics_hz, opts);
        },
    );
}
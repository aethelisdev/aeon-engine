// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Physics Preferences Card
//!
//! Renders physics simulation frequency settings declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{pref_section_card, pref_slider_row};
use crate::ui::iris_bridge::preferences::types::{
    PHYSICS_HZ_PRESETS, PreferencesParams, PreferencesSliderId,
};
use ae_editor::editor_state::EditorConfig;
use irisui::prelude::*;

/// Builds the Physics settings card declaratively using [`UiScope`].
pub fn build_physics_card(
    scope: &mut UiScope<'_>,
    params: &PreferencesParams<'_>,
    cfg: &EditorConfig,
) {
    let is_collapsed = params.collapsed_sections.contains("editor_physics");

    let snapped_hz = PHYSICS_HZ_PRESETS
        .iter()
        .copied()
        .min_by(|a, b| {
            (a - cfg.physics_hz)
                .abs()
                .total_cmp(&(b - cfg.physics_hz).abs())
        })
        .unwrap_or(cfg.physics_hz);

    pref_section_card(
        scope,
        "editor_physics",
        "🎮  Physics Settings",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_slider_row(
                body,
                PreferencesSliderId::PhysicsFrequency,
                "Fixed Update Frequency",
                snapped_hz,
                params.active_number_input,
                params.blink_caret,
                params.hovered_tag,
            );
        },
    );
}
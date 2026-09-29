// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Preferences Tab Panel Builder
//!
//! Orchestrates the assembly of editor settings cards including snapping, history limits,
//! physics simulation rate, and live runtime reload declaratively using [`UiScope`].

use super::history::build_history_card;
use super::physics::build_physics_card;
use super::runtime::build_runtime_card;
use super::snapping::build_snapping_card;
use crate::ui::iris_bridge::preferences::components::pref_heading;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Builds the Editor preferences tab content declaratively using [`UiScope`].
pub fn build_editor_tab(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "Editor Settings",
        "Configure viewport snapping, undo history depth, physics update rate, and live reloading.",
    );

    build_snapping_card(scope, params, params.snapping_settings);
    build_history_card(scope, params, params.editor_config);
    build_physics_card(scope, params, params.editor_config);
    build_runtime_card(scope, params, params.enable_live_updates);
}
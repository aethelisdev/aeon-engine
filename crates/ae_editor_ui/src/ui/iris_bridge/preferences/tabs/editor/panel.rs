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
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use irisui::prelude::*;

/// Builds the Editor preferences tab content declaratively using [`UiScope`].
pub fn build_editor_tab(
    scope: &mut UiScope<'_>,
    snapping: &mut SnapSettings,
    editor_config: &mut EditorConfig,
    enable_live_updates: &mut bool,
) {
    scope.section_heading(
        "Editor Settings",
        "Configure viewport snapping, undo history depth, periodic auto-save, physics update rate, and live reloading.",
    );

    build_snapping_card(scope, snapping);
    build_history_card(scope, editor_config);
    build_physics_card(scope, editor_config);
    build_runtime_card(scope, enable_live_updates);
}
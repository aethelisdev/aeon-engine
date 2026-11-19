// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Runtime Preferences Card
//!
//! Renders engine hot-reload and live editor runtime execution settings
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Builds the Runtime settings card declaratively using [`UiScope`].
pub fn build_runtime_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.collapsing_section("editor_runtime", "⚙  Runtime Settings", true, |body| {
        body.checkbox(
            "Enable Live Editor Updates (Hot Reload)",
            params.enable_live_updates,
        );
    });
}
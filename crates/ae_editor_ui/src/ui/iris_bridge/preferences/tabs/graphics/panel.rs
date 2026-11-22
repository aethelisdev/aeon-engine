// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Graphics Preferences Tab Panel Builder
//!
//! Orchestrates the rendering of Shadows, Performance, Anti-Aliasing, Post-Processing,
//! Environment & Sky, and Procedural Clouds preference cards declaratively using [`UiScope`].

use super::environment::build_environment_card;
use super::performance::{build_aa_card, build_perf_card, build_post_processing_card};
use super::shadows::build_shadows_card;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;

/// Builds the complete Graphics preferences tab content declaratively using [`UiScope`].
pub fn build_graphics_tab(scope: &mut UiScope<'_>, graphics: &mut GraphicsSettings) {
    scope.section_heading(
        "Graphics Settings",
        "Configure render pipeline, shadows, post-processing, atmosphere, and cloud simulation.",
    );

    build_shadows_card(scope, graphics);
    build_perf_card(scope, graphics);
    build_aa_card(scope, graphics);
    build_post_processing_card(scope, graphics);
    build_environment_card(scope, graphics);
}
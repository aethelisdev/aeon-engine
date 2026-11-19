// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance & Post-Processing Settings Cards Builder
//!
//! Renders framerate limiting, hardware MSAA samples, and HDR bloom cards
//! declaratively using [`UiScope`].

use super::types::{FPS_OPTIONS, MSAA_OPTIONS};
use crate::ui::iris_bridge::preferences::tabs::PREF_COMBO_LAYOUT;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

const FPS_LABELS: [&str; 3] = ["60 FPS", "120 FPS", "Uncapped"];
const MSAA_LABELS: [&str; 3] = ["Off (1x)", "2x", "4x (Default)"];

/// Builds the Performance & Framerate card declaratively using [`UiScope`].
pub fn build_perf_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.collapsing_section(
        "graphics_perf",
        "⚡  Performance & Framerate",
        true,
        |body| {
            body.prop_combobox_mapped_styled(
                "Framerate Limit",
                FPS_OPTIONS
                    .iter()
                    .position(|&f| f == params.graphics_settings.fps_limit)
                    .unwrap_or(0),
                &FPS_LABELS,
                PREF_COMBO_LAYOUT,
                &mut params.graphics_settings.fps_limit,
                |idx| FPS_OPTIONS.get(idx).copied().unwrap_or(FPS_OPTIONS[0]),
            );
        },
    );
}

/// Builds the Anti-Aliasing (MSAA) card declaratively using [`UiScope`].
pub fn build_aa_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.collapsing_section("graphics_aa", "🔍  Anti-Aliasing (MSAA)", true, |body| {
        body.prop_combobox_mapped_styled(
            "MSAA Samples",
            match params.graphics_settings.msaa_samples {
                1 => 0,
                2 => 1,
                _ => 2,
            },
            &MSAA_LABELS,
            PREF_COMBO_LAYOUT,
            &mut params.graphics_settings.msaa_samples,
            |idx| MSAA_OPTIONS.get(idx).map_or(4, |&(samples, _)| samples),
        );
    });
}

/// Builds the HDR Bloom Post-Processing card declaratively using [`UiScope`].
pub fn build_post_processing_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.collapsing_section("graphics_pp", "✨  Post-Processing (Bloom)", true, |body| {
        body.property_checkbox("Enable Bloom", &mut params.graphics_settings.bloom_enabled);

        body.when(params.graphics_settings.bloom_enabled, |s| {
            s.property_slider(
                "Bloom Intensity",
                &mut params.graphics_settings.bloom_intensity,
                0.0,
                3.0,
                0.05,
            );
        });
    });
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance & Post-Processing Settings Cards Builder
//!
//! Renders framerate limiting, hardware MSAA samples, and HDR bloom cards
//! declaratively using [`UiScope`].

use super::types::{FPS_OPTIONS, MSAA_OPTIONS};
use crate::ui::iris_bridge::preferences::components::{pref_combobox, pref_section_card};
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Builds the Performance & Framerate card declaratively using [`UiScope`].
pub fn build_perf_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_perf");

    pref_section_card(
        scope,
        "graphics_perf",
        "⚡  Performance & Framerate",
        is_collapsed,
        |body| {
            let resp = pref_combobox(
                body,
                "Framerate Limit",
                params.graphics_settings.fps_limit.label(),
                &["60 FPS", "120 FPS", "Uncapped"],
            );
            if resp.changed()
                && let Some(idx) = resp.selected_index()
                && let Some(&fps) = FPS_OPTIONS.get(idx)
            {
                params.graphics_settings.fps_limit = fps;
            }
        },
    );
}

/// Builds the Anti-Aliasing (MSAA) card declaratively using [`UiScope`].
pub fn build_aa_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_aa");

    let msaa_label = match params.graphics_settings.msaa_samples {
        1 => "Off (1x)",
        2 => "2x",
        _ => "4x (Default)",
    };

    pref_section_card(
        scope,
        "graphics_aa",
        "🔍  Anti-Aliasing (MSAA)",
        is_collapsed,
        |body| {
            let resp = pref_combobox(
                body,
                "MSAA Samples",
                msaa_label,
                &["Off (1x)", "2x", "4x (Default)"],
            );
            if resp.changed()
                && let Some(idx) = resp.selected_index()
                && let Some(&(samples, _)) = MSAA_OPTIONS.get(idx)
            {
                params.graphics_settings.msaa_samples = samples;
            }
        },
    );
}

/// Builds the HDR Bloom Post-Processing card declaratively using [`UiScope`].
///
/// Binds `gs.bloom_enabled` and `gs.bloom_intensity` directly via declarative two-way property primitives.
pub fn build_post_processing_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_pp");
    let gs = &mut *params.graphics_settings;

    pref_section_card(
        scope,
        "graphics_pp",
        "✨  Post-Processing (Bloom)",
        is_collapsed,
        |body| {
            body.property_checkbox("Enable Bloom", &mut gs.bloom_enabled);

            if gs.bloom_enabled {
                body.property_slider("Bloom Intensity", &mut gs.bloom_intensity, 0.0, 3.0, 0.05);
            }
        },
    );
}
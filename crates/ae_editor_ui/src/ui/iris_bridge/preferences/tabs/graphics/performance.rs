// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Performance & Post-Processing Settings Cards Builder
//!
//! Renders framerate limiting, hardware MSAA samples, and HDR bloom cards
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{
    pref_dropdown_row, pref_section_card, pref_slider_row, pref_toggle_row,
};
use crate::ui::iris_bridge::preferences::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesToggleId,
};
use irisui::prelude::*;

/// Builds the Performance & Framerate card declaratively using [`UiScope`].
pub fn build_perf_card(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_perf");

    pref_section_card(
        scope,
        "graphics_perf",
        "⚡  Performance & Framerate",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_dropdown_row(
                body,
                PreferencesDropdownId::FpsLimit,
                "Framerate Limit",
                params.graphics_settings.fps_limit.label(),
                params.active_dropdown == Some(PreferencesDropdownId::FpsLimit),
                params.hovered_tag,
            );
        },
    );
}

/// Builds the Anti-Aliasing (MSAA) card declaratively using [`UiScope`].
pub fn build_aa_card(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
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
        params.hovered_tag,
        |body| {
            pref_dropdown_row(
                body,
                PreferencesDropdownId::MsaaSamples,
                "MSAA Samples",
                msaa_label,
                params.active_dropdown == Some(PreferencesDropdownId::MsaaSamples),
                params.hovered_tag,
            );
        },
    );
}

/// Builds the HDR Bloom Post-Processing card declaratively using [`UiScope`].
pub fn build_post_processing_card(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_pp");
    let gs = params.graphics_settings;

    pref_section_card(
        scope,
        "graphics_pp",
        "✨  Post-Processing (Bloom)",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_toggle_row(
                body,
                PreferencesToggleId::BloomEnabled,
                "Enable Bloom",
                gs.bloom_enabled,
                params.hovered_tag,
            );

            if gs.bloom_enabled {
                pref_slider_row(
                    body,
                    PreferencesSliderId::BloomIntensity,
                    "Bloom Intensity",
                    gs.bloom_intensity,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );
            }
        },
    );
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Environment & Sky Settings Card Builder
//!
//! Renders physical Rayleigh/Mie sky scattering, ozone absorption, sun parameters,
//! procedural clouds, and fog declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{
    pref_dropdown_row, pref_section_card, pref_slider_row, pref_toggle_row,
};
use crate::ui::iris_bridge::preferences::types::{
    PreferencesDropdownId, PreferencesParams, PreferencesSliderId, PreferencesToggleId,
};
use ae_renderer::graphics_settings::SkyQuality;
use irisui::prelude::*;

/// Builds the Environment & Sky configuration card declaratively using [`UiScope`].
pub fn build_environment_card(scope: &mut UiScope<'_>, params: &PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_env");
    let gs = params.graphics_settings;
    let is_advanced_sky = gs.sky_quality != SkyQuality::Low;

    pref_section_card(
        scope,
        "graphics_env",
        "⛅  Environment & Sky",
        is_collapsed,
        params.hovered_tag,
        |body| {
            pref_dropdown_row(
                body,
                PreferencesDropdownId::SkyQuality,
                "Sky Quality",
                gs.sky_quality.label(),
                params.active_dropdown == Some(PreferencesDropdownId::SkyQuality),
                params.hovered_tag,
            );

            pref_slider_row(
                body,
                PreferencesSliderId::SunPitch,
                "Sun Pitch",
                gs.sun_pitch,
                params.active_number_input,
                params.blink_caret,
                params.hovered_tag,
            );

            pref_slider_row(
                body,
                PreferencesSliderId::SunYaw,
                "Sun Yaw",
                gs.sun_yaw,
                params.active_number_input,
                params.blink_caret,
                params.hovered_tag,
            );

            if is_advanced_sky {
                pref_slider_row(
                    body,
                    PreferencesSliderId::AtmosphereDensity,
                    "Atmosphere Density",
                    gs.atmosphere_density,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::OzoneDensity,
                    "Ozone Absorption (Chappuis)",
                    gs.ozone_density,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::SunDiscSize,
                    "Sun Disc Size",
                    gs.sun_disc_size,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::SunGlowStrength,
                    "Sun Glow Strength",
                    gs.sun_glow_strength,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                // Procedural Clouds Sub-Header
                body.label_styled_passive(
                    "CloudsTitle",
                    "☁  Procedural Clouds",
                    12.0,
                    Color::rgba(0.0, 0.90, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(8.0, 0.0, 6.0, 0.0)),
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::CloudCoverage,
                    "Cloud Coverage",
                    gs.cloud_coverage,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::CloudDensity,
                    "Cloud Density",
                    gs.cloud_density,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::CloudSpeed,
                    "Wind Speed (Drift)",
                    gs.cloud_speed,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::CloudEvolution,
                    "Turbulence / Evolution",
                    gs.cloud_evolution,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );

                pref_slider_row(
                    body,
                    PreferencesSliderId::CloudAltitude,
                    "Cloud Base Altitude",
                    gs.cloud_altitude,
                    params.active_number_input,
                    params.blink_caret,
                    params.hovered_tag,
                );
            }

            // Atmospheric Depth Fog
            body.container_named(
                "FogContainer",
                Style::new()
                    .flex_col()
                    .margin_insets(Insets::new(8.0, 0.0, 0.0, 0.0)),
                |fog_col| {
                    pref_toggle_row(
                        fog_col,
                        PreferencesToggleId::FogEnabled,
                        "Enable Atmospheric Depth Fog",
                        gs.fog_enabled,
                        params.hovered_tag,
                    );

                    if gs.fog_enabled {
                        pref_slider_row(
                            fog_col,
                            PreferencesSliderId::FogDistance,
                            "Fog Distance",
                            gs.fog_distance,
                            params.active_number_input,
                            params.blink_caret,
                            params.hovered_tag,
                        );
                    }
                },
            );
        },
    );
}
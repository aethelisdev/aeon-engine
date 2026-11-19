// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Environment & Sky Settings Card Builder
//!
//! Renders physical Rayleigh/Mie sky scattering, ozone absorption, sun parameters,
//! procedural clouds, and atmospheric fog declaratively using [`UiScope`] and two-way property primitives.

use super::types::SKY_OPTIONS;
use crate::ui::iris_bridge::preferences::tabs::PREF_COMBO_LAYOUT;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use ae_renderer::graphics_settings::SkyQuality;
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PropertySliderOptions;

const SKY_LABELS: [&str; 3] = [
    "Low (Gradient)",
    "Medium (Fast HDR)",
    "High (Atmospheric 2.5D)",
];

/// Builds the Environment & Sky configuration card declaratively using [`UiScope`].
pub fn build_environment_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.collapsing_section("graphics_env", "⛅  Environment & Sky", true, |body| {
        body.prop_combobox_mapped_styled(
            "Sky Quality",
            SKY_OPTIONS
                .iter()
                .position(|&s| s == params.graphics_settings.sky_quality)
                .unwrap_or(2),
            &SKY_LABELS,
            PREF_COMBO_LAYOUT,
            &mut params.graphics_settings.sky_quality,
            |idx| SKY_OPTIONS.get(idx).copied().unwrap_or(SKY_OPTIONS[2]),
        );

        body.property_slider_mapped(
            "Sun Pitch",
            &mut params.graphics_settings.sun_pitch,
            f32::to_degrees,
            f32::to_radians,
            PropertySliderOptions::new(-180.0, 180.0, 0.5)
                .with_format("{:.1}°")
                .with_subtitle("Solar altitude angle relative to the horizon (-180° to +180°)"),
        );

        body.property_slider_mapped(
            "Sun Yaw",
            &mut params.graphics_settings.sun_yaw,
            f32::to_degrees,
            f32::to_radians,
            PropertySliderOptions::new(-180.0, 180.0, 0.5)
                .with_format("{:.1}°")
                .with_subtitle("Compass azimuth heading of the sun (-180° to +180°)"),
        );

        body.when(
            params.graphics_settings.sky_quality != SkyQuality::Low,
            |s| {
                s.property_slider(
                    "Atmosphere Density",
                    &mut params.graphics_settings.atmosphere_density,
                    0.0,
                    5.0,
                    0.05,
                );
                s.property_slider(
                    "Ozone Absorption (Chappuis)",
                    &mut params.graphics_settings.ozone_density,
                    0.0,
                    3.0,
                    0.02,
                );
                s.property_slider(
                    "Sun Disc Size",
                    &mut params.graphics_settings.sun_disc_size,
                    0.1,
                    5.0,
                    0.05,
                );
                s.property_slider(
                    "Sun Glow Strength",
                    &mut params.graphics_settings.sun_glow_strength,
                    0.0,
                    5.0,
                    0.05,
                );

                s.label_styled_passive(
                    "CloudsTitle",
                    "☁  Procedural Clouds",
                    12.0,
                    Color::rgba(0.0, 0.90, 1.0, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(8.0, 0.0, 6.0, 0.0)),
                );
                s.property_slider(
                    "Cloud Coverage",
                    &mut params.graphics_settings.cloud_coverage,
                    0.0,
                    1.0,
                    0.01,
                );
                s.property_slider(
                    "Cloud Density",
                    &mut params.graphics_settings.cloud_density,
                    0.1,
                    3.0,
                    0.02,
                );
                s.property_slider(
                    "Wind Speed (Drift)",
                    &mut params.graphics_settings.cloud_speed,
                    0.0,
                    5.0,
                    0.05,
                );
                s.property_slider(
                    "Turbulence / Evolution",
                    &mut params.graphics_settings.cloud_evolution,
                    0.0,
                    3.0,
                    0.02,
                );
                s.property_slider_with_options(
                    "Cloud Base Altitude",
                    &mut params.graphics_settings.cloud_altitude,
                    PropertySliderOptions::new(500.0, 5000.0, 25.0)
                        .with_format("{:.0} m")
                        .with_subtitle(
                            "Base altitude of the cloud layer in meters (500m to 5000m)",
                        ),
                );
            },
        );

        body.container_named(
            "FogContainer",
            Style::new()
                .flex_col()
                .margin_insets(Insets::new(8.0, 0.0, 0.0, 0.0)),
            |fog_col| {
                fog_col.property_checkbox(
                    "Enable Atmospheric Depth Fog",
                    &mut params.graphics_settings.fog_enabled,
                );

                fog_col.when(params.graphics_settings.fog_enabled, |s| {
                    s.property_slider(
                        "Fog Distance",
                        &mut params.graphics_settings.fog_distance,
                        100.0,
                        2000.0,
                        10.0,
                    );
                });
            },
        );
    });
}
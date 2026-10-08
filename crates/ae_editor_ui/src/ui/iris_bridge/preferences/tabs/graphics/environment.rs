// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Environment & Sky Settings Card Builder
//!
//! Renders physical Rayleigh/Mie sky scattering, ozone absorption, sun parameters,
//! procedural clouds, and atmospheric fog declaratively using [`UiScope`] and two-way property primitives.

use crate::ui::iris_bridge::preferences::components::{pref_dropdown_row, pref_section_card};
use crate::ui::iris_bridge::preferences::types::{PreferencesDropdownId, PreferencesParams};
use ae_renderer::graphics_settings::SkyQuality;
use irisui::prelude::*;

/// Builds the Environment & Sky configuration card declaratively using [`UiScope`].
///
/// Binds sun parameters, atmospheric absorption, procedural cloud simulation, and depth fog
/// directly to `gs` via immediate two-way property primitives without intermediary enum queues.
pub fn build_environment_card(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    let is_collapsed = params.collapsed_sections.contains("graphics_env");
    let active_dropdown = params.active_dropdown;
    let gs = &mut *params.graphics_settings;
    let is_advanced_sky = gs.sky_quality != SkyQuality::Low;

    pref_section_card(
        scope,
        "graphics_env",
        "⛅  Environment & Sky",
        is_collapsed,
        |body| {
            pref_dropdown_row(
                body,
                PreferencesDropdownId::SkyQuality,
                "Sky Quality",
                gs.sky_quality.label(),
                active_dropdown == Some(PreferencesDropdownId::SkyQuality),
            );

            let mut sun_pitch_deg = gs.sun_pitch.to_degrees();
            body.property_slider_with_options(
                "Sun Pitch",
                &mut sun_pitch_deg,
                PropertySliderOptions::new(-180.0, 180.0, 0.5)
                    .with_format("{:.1}°")
                    .with_subtitle("Solar altitude angle relative to the horizon (-180° to +180°)"),
            );
            if (sun_pitch_deg - gs.sun_pitch.to_degrees()).abs() > 1e-4 {
                gs.sun_pitch = sun_pitch_deg.to_radians();
            }

            let mut sun_yaw_deg = gs.sun_yaw.to_degrees();
            body.property_slider_with_options(
                "Sun Yaw",
                &mut sun_yaw_deg,
                PropertySliderOptions::new(-180.0, 180.0, 0.5)
                    .with_format("{:.1}°")
                    .with_subtitle("Compass azimuth heading of the sun (-180° to +180°)"),
            );
            if (sun_yaw_deg - gs.sun_yaw.to_degrees()).abs() > 1e-4 {
                gs.sun_yaw = sun_yaw_deg.to_radians();
            }

            if is_advanced_sky {
                body.property_slider(
                    "Atmosphere Density",
                    &mut gs.atmosphere_density,
                    0.0,
                    5.0,
                    0.05,
                );

                body.property_slider(
                    "Ozone Absorption (Chappuis)",
                    &mut gs.ozone_density,
                    0.0,
                    3.0,
                    0.02,
                );

                body.property_slider("Sun Disc Size", &mut gs.sun_disc_size, 0.1, 5.0, 0.05);

                body.property_slider(
                    "Sun Glow Strength",
                    &mut gs.sun_glow_strength,
                    0.0,
                    5.0,
                    0.05,
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

                body.property_slider("Cloud Coverage", &mut gs.cloud_coverage, 0.0, 1.0, 0.01);

                body.property_slider("Cloud Density", &mut gs.cloud_density, 0.1, 3.0, 0.02);

                body.property_slider("Wind Speed (Drift)", &mut gs.cloud_speed, 0.0, 5.0, 0.05);

                body.property_slider(
                    "Turbulence / Evolution",
                    &mut gs.cloud_evolution,
                    0.0,
                    3.0,
                    0.02,
                );

                body.property_slider_with_options(
                    "Cloud Base Altitude",
                    &mut gs.cloud_altitude,
                    PropertySliderOptions::new(500.0, 5000.0, 25.0)
                        .with_format("{:.0} m")
                        .with_subtitle(
                            "Base altitude of the cloud layer in meters (500m to 5000m)",
                        ),
                );
            }

            // Atmospheric Depth Fog
            body.container_named(
                "FogContainer",
                Style::new()
                    .flex_col()
                    .margin_insets(Insets::new(8.0, 0.0, 0.0, 0.0)),
                |fog_col| {
                    fog_col.property_checkbox("Enable Atmospheric Depth Fog", &mut gs.fog_enabled);

                    if gs.fog_enabled {
                        fog_col.property_slider(
                            "Fog Distance",
                            &mut gs.fog_distance,
                            100.0,
                            2000.0,
                            10.0,
                        );
                    }
                },
            );
        },
    );
}
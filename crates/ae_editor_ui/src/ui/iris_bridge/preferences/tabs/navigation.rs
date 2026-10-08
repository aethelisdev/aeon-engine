// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Navigation Preferences Tab
//!
//! Renders hardware-accelerated configuration cards for viewport camera flight speeds,
//! Shift boost multipliers, zoom curves, and orbit navigation declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::{pref_heading, pref_section_card};
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Builds the Navigation preferences tab content declaratively using [`UiScope`].
///
/// Binds camera base fly speed, Shift boost multiplier, and scroll zoom speed directly
/// to `params.editor_config` via immediate two-way property sliders.
pub fn build_navigation_tab(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    pref_heading(
        scope,
        "🧭  Navigation",
        "Configure camera movement speeds, zoom curves, and 3D viewport flight controls.",
    );

    let is_speeds_collapsed = params.collapsed_sections.contains("nav_speeds");
    let cfg = &mut *params.editor_config;

    pref_section_card(
        scope,
        "nav_speeds",
        "🚀  Camera Flight Speeds",
        is_speeds_collapsed,
        |body| {
            let base_speed_opts = PropertySliderOptions::new(0.5, 30.0, 0.2)
                .with_format("{:.1} m/s")
                .with_subtitle(
                    "Linear translation speed when flying through the 3D viewport holding Right Mouse Button and WASD.",
                );
            body.property_slider_with_options(
                "Base Fly Speed",
                &mut cfg.camera_base_speed,
                base_speed_opts,
            );

            let boost_opts = PropertySliderOptions::new(1.0, 10.0, 0.1)
                .with_format("{:.1}x")
                .with_subtitle(
                    "Speed multiplier applied to base camera translation when holding the Shift key.",
                );
            body.property_slider_with_options(
                "Shift Boost Multiplier",
                &mut cfg.camera_shift_multiplier,
                boost_opts,
            );

            let scroll_opts = PropertySliderOptions::new(0.2, 5.0, 0.05)
                .with_format("{:.2}x")
                .with_subtitle(
                    "Exponential zoom sensitivity multiplier when rotating the mouse scroll wheel.",
                );
            body.property_slider_with_options(
                "Scroll Zoom Speed",
                &mut cfg.camera_scroll_speed,
                scroll_opts,
            );
        },
    );

    let is_orbit_collapsed = params.collapsed_sections.contains("nav_orbit");
    pref_section_card(
        scope,
        "nav_orbit",
        "🔄  Orbit & Viewport Orientation",
        is_orbit_collapsed,
        |body| {
            body.container_named(
                "NavInfoCard",
                Style::new()
                    .flex_col()
                    .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                    .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                    .border_radius(6.0)
                    .padding_insets(Insets::new(10.0, 14.0, 10.0, 14.0))
                    .margin_insets(Insets::new(0.0, 0.0, 10.0, 0.0)),
                |card| {
                    card.label_styled_passive(
                        "NavOrbitTitle",
                        "🧭 Viewport Orbit Mode",
                        12.5,
                        Color::rgba(1.0, 1.0, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    );
                    card.label_styled_passive(
                        "NavOrbitPrimary",
                        "Turntable (Z-up aligned)",
                        11.5,
                        Color::rgba(0.0, 0.85, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    );
                    card.label_styled_passive(
                        "NavOrbitDesc",
                        "Maintains stable orbital rotation without unconstrained roll deviation.",
                        11.0,
                        Color::rgba(0.65, 0.68, 0.76, 1.0),
                        TextAlign::Left,
                        Style::new(),
                    );
                },
            );

            body.container_named(
                "NavZoomCard",
                Style::new()
                    .flex_col()
                    .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                    .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                    .border_radius(6.0)
                    .padding_insets(Insets::new(10.0, 14.0, 10.0, 14.0)),
                |card| {
                    card.label_styled_passive(
                        "NavZoomTitle",
                        "🔍 Zoom Acceleration",
                        12.5,
                        Color::rgba(1.0, 1.0, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    );
                    card.label_styled_passive(
                        "NavZoomPrimary",
                        "Smooth Exponential Distance Scaling",
                        11.5,
                        Color::rgba(0.0, 0.85, 1.0, 1.0),
                        TextAlign::Left,
                        Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    );
                    card.label_styled_passive(
                        "NavZoomDesc",
                        "Scales zoom step distance smoothly relative to focal target distance.",
                        11.0,
                        Color::rgba(0.65, 0.68, 0.76, 1.0),
                        TextAlign::Left,
                        Style::new(),
                    );
                },
            );
        },
    );
}
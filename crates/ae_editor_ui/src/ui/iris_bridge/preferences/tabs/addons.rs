// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Add-ons Preferences Tab
//!
//! Renders installed engine plugins, asset loaders, and runtime extensions declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::pref_heading;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Add-on plugin item descriptor.
struct AddonItem {
    name: &'static str,
    version: &'static str,
    status: &'static str,
    status_color: Color,
    description: &'static str,
}

const ENGINE_ADDONS: [AddonItem; 4] = [
    AddonItem {
        name: "glTF 2.0 / GLB Importer",
        version: "v2.0 Built-in",
        status: "Active",
        status_color: Color::rgba(0.20, 0.85, 0.45, 1.0),
        description: "Official Kronos glTF asset loading pipeline. Supports PBR materials, multi-channel textures, skeletal skins, and embedded binary buffers.",
    },
    AddonItem {
        name: "Iris UI Framework",
        version: "v1.0 Retained",
        status: "Active",
        status_color: Color::rgba(0.20, 0.85, 0.45, 1.0),
        description: "Hardware-accelerated Signed Distance Field (SDF) vector renderer and declarative UiScope hierarchy builder with zero CPU garbage generation.",
    },
    AddonItem {
        name: "Kira Audio Subsystem",
        version: "v0.9 DSP",
        status: "Active",
        status_color: Color::rgba(0.20, 0.85, 0.45, 1.0),
        description: "Low-latency spatial audio engine. Provides 3D sound attenuation, doppler pitch shifting, procedural synthesizers, and environmental reverb.",
    },
    AddonItem {
        name: "WGSL Shader Compiler & Hot-Reload",
        version: "v30.0 WGPU",
        status: "Active",
        status_color: Color::rgba(0.20, 0.85, 0.45, 1.0),
        description: "Asynchronous file-system watcher for WGSL compute and raster pipelines. Dynamically re-binds GPU pipelines on file modification with zero restart.",
    },
];

/// Builds the Add-ons preferences tab content declaratively using [`UiScope`].
pub fn build_addons_tab(scope: &mut UiScope<'_>, _params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "📦  Add-ons & Plugins",
        "Manage installed engine extensions, format importers, and runtime subsystems.",
    );

    for addon in &ENGINE_ADDONS {
        scope.container_named(
            "AddonCard",
            Style::new()
                .flex_col()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0)
                .padding_insets(Insets::new(12.0, 14.0, 12.0, 14.0))
                .margin_insets(Insets::new(0.0, 0.0, 12.0, 0.0)),
            |card| {
                card.container_named(
                    "AddonHeaderRow",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    |header| {
                        header.container_named(
                            "AddonTitleRow",
                            Style::new()
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .gap(8.0),
                            |left| {
                                left.label_styled_passive(
                                    "AddonName",
                                    addon.name,
                                    12.5,
                                    Color::rgba(1.0, 1.0, 1.0, 1.0),
                                    TextAlign::Left,
                                    Style::new(),
                                );
                                left.label_styled_passive(
                                    "AddonVersion",
                                    addon.version,
                                    11.0,
                                    Color::rgba(0.55, 0.58, 0.68, 1.0),
                                    TextAlign::Left,
                                    Style::new(),
                                );
                            },
                        );

                        header.container_named(
                            "AddonStatusBadge",
                            Style::new()
                                .background(Color::rgba(0.12, 0.28, 0.18, 0.80))
                                .border(1.0, Color::rgba(0.20, 0.85, 0.45, 0.35))
                                .border_radius(4.0)
                                .padding_insets(Insets::new(2.0, 8.0, 2.0, 8.0)),
                            |badge| {
                                badge.label_styled_passive(
                                    "AddonStatusText",
                                    addon.status,
                                    11.0,
                                    addon.status_color,
                                    TextAlign::Center,
                                    Style::new(),
                                );
                            },
                        );
                    },
                );

                card.label_styled_passive(
                    "AddonDesc",
                    addon.description,
                    11.0,
                    Color::rgba(0.65, 0.68, 0.76, 1.0),
                    TextAlign::Left,
                    Style::new(),
                );
            },
        );
    }
}
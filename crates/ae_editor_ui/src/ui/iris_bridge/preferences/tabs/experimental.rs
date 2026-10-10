// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Experimental Preferences Tab
//!
//! Renders preview cards for  engine features under active development
//! declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::components::pref_heading;
use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;

/// Experimental feature preview descriptor.
struct ExperimentalFeature {
    icon_title: &'static str,
    status_label: &'static str,
    status_color: Color,
    summary: &'static str,
    details_line1: &'static str,
    details_line2: &'static str,
}

const EXPERIMENTAL_FEATURES: [ExperimentalFeature; 3] = [
    ExperimentalFeature {
        icon_title: "⚡  GPU Hi-Z Occlusion Culling",
        status_label: "Active (Hardware Hi-Z)",
        status_color: Color::rgba(0.0, 0.85, 1.0, 1.0),
        summary: "Constructs hierarchical depth pyramids to discard occluded mesh bounding boxes.",
        details_line1: "Runs early in the frame before the forward pass. Drastically reduces",
        details_line2: "draw calls and vertex shading costs in dense indoor scenes.",
    },
    ExperimentalFeature {
        icon_title: "🌟  Compute Shader Skeletal Skinning",
        status_label: "Active (GPU Compute Dispatch)",
        status_color: Color::rgba(0.0, 0.85, 1.0, 1.0),
        summary: "Evaluates bone matrix vertex blending on GPU compute queues rather than CPU.",
        details_line1: "Frees CPU threads from manipulating thousands of vertices per frame",
        details_line2: "and feeds transformed buffers directly to the vertex pipeline.",
    },
    ExperimentalFeature {
        icon_title: "💎  Virtual Geometry (Mesh Cluster Streaming)",
        status_label: "Prototype (Research)",
        status_color: Color::rgba(0.95, 0.70, 0.20, 1.0),
        summary: "Splits ultra-high-poly models into 128-triangle clusters with continuous LOD.",
        details_line1: "Enables multi-million polygon CAD/film assets in real-time viewport",
        details_line2: "without manual LOD authoring or visible pop-in artifacts.",
    },
];

/// Builds the Experimental preferences tab content declaratively using [`UiScope`].
pub fn build_experimental_tab(scope: &mut UiScope<'_>, _params: &PreferencesParams<'_>) {
    pref_heading(
        scope,
        "🧪  Experimental Technologies",
        "Preview bleeding-edge engine features and next-generation graphics pipelines.",
    );

    for feature in &EXPERIMENTAL_FEATURES {
        scope.container_named(
            "ExperimentalCard",
            Style::new()
                .flex_col()
                .background(Color::rgba(0.09, 0.10, 0.14, 0.85))
                .border(1.0, Color::rgba(0.18, 0.20, 0.28, 0.90))
                .border_radius(6.0)
                .padding_insets(Insets::new(12.0, 14.0, 12.0, 14.0))
                .margin_insets(Insets::new(0.0, 0.0, 12.0, 0.0)),
            |card| {
                card.container_named(
                    "ExpHeaderRow",
                    Style::new()
                        .flex_row()
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::SpaceBetween)
                        .margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                    |header| {
                        header.label_styled_passive(
                            "ExpTitle",
                            feature.icon_title,
                            12.5,
                            Color::rgba(1.0, 1.0, 1.0, 1.0),
                            TextAlign::Left,
                            Style::new(),
                        );
                        header.label_styled_passive(
                            "ExpStatus",
                            feature.status_label,
                            11.5,
                            feature.status_color,
                            TextAlign::Right,
                            Style::new(),
                        );
                    },
                );

                card.label_styled_passive(
                    "ExpSummary",
                    feature.summary,
                    11.5,
                    Color::rgba(0.85, 0.88, 0.95, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 4.0, 0.0)),
                );

                card.label_styled_passive(
                    "ExpDetailsLine1",
                    feature.details_line1,
                    11.0,
                    Color::rgba(0.65, 0.68, 0.76, 1.0),
                    TextAlign::Left,
                    Style::new().margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
                );
                card.label_styled_passive(
                    "ExpDetailsLine2",
                    feature.details_line2,
                    11.0,
                    Color::rgba(0.65, 0.68, 0.76, 1.0),
                    TextAlign::Left,
                    Style::new(),
                );
            },
        );
    }
}
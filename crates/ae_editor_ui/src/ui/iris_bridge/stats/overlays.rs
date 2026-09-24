// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Viewport Overlays Declarative Card Builder
//!
//! Renders the wireframe mode and grid visibility interactive checkboxes
//! purely using [`UiScope`] and persistent semantic tags.
//!

use super::types::{STATS_TAG_TOGGLE_GRID, STATS_TAG_TOGGLE_WIREFRAME, StatsPanelParams};
use irisui::prelude::*;

/// Builds the Viewport Overlays section inside the declarative stats card.
///
/// Emits semantic interactive toggle controls tagged with [`STATS_TAG_TOGGLE_WIREFRAME`]
/// and [`STATS_TAG_TOGGLE_GRID`] for hardware hit-test routing.
pub fn build_viewport_overlays_content(scope: &mut UiScope<'_>, params: &StatsPanelParams<'_>) {
    let row_style = Style::new().flex_col().gap(6.0);

    scope.container(row_style, |s| {
        // 1. Wireframe Mode Checkbox
        build_declarative_checkbox(
            s,
            "🕸 Wireframe Mode (Edges)",
            params.wireframe_enabled,
            STATS_TAG_TOGGLE_WIREFRAME,
        );

        // 2. Show Grid Checkbox
        build_declarative_checkbox(
            s,
            "🔲 Show Grid",
            params.grid_enabled,
            STATS_TAG_TOGGLE_GRID,
        );
    });
}

/// Helper emitting an interactive declarative checkbox row with semantic tag and accessibility role.
fn build_declarative_checkbox(scope: &mut UiScope<'_>, label: &str, checked: bool, tag: u64) {
    let row_style = Style::new()
        .flex_row()
        .align_items(AlignItems::Center)
        .gap(8.0)
        .height(22.0)
        .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
        .border_radius(4.0)
        .background(Color::rgba(0.08, 0.09, 0.12, 0.50));

    scope.container_tagged(
        "CheckboxRow",
        row_style,
        WidgetRole::Checkbox,
        tag,
        |row_scope| {
            // Visual Checkbox Box
            let (box_bg, box_border) = if checked {
                (
                    Color::rgba(0.0, 0.55, 0.75, 1.0),
                    Color::rgba(0.0, 0.85, 1.0, 1.0),
                )
            } else {
                (
                    Color::rgba(0.10, 0.11, 0.15, 0.90),
                    Color::rgba(0.24, 0.28, 0.38, 0.70),
                )
            };

            let box_style = Style::new()
                .width(14.0)
                .height(14.0)
                .border_radius(3.0)
                .background(box_bg)
                .border(1.0, box_border)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center);

            row_scope.container_tagged(
                "CheckboxBox",
                box_style,
                WidgetRole::Checkbox,
                tag,
                |box_scope| {
                    if checked {
                        box_scope.label(
                            "✓",
                            10.0,
                            Color::rgba(1.0, 1.0, 1.0, 1.0),
                            TextAlign::Center,
                        );
                    }
                },
            );

            // Checkbox Label
            row_scope.text_colored(label, Color::rgba(0.85, 0.88, 0.95, 1.0));
        },
    );
}
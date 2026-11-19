// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Input Preferences Tab
//!
//! Renders hardware-accelerated configuration cards for mouse look sensitivity,
//! cursor behavior, and navigation control mappings declaratively using [`UiScope`].

use crate::ui::iris_bridge::preferences::types::PreferencesParams;
use irisui::prelude::*;
use irisui::widgets::declarative::scope::PropertySliderOptions;

const VIEWPORT_GESTURES: [(&str, &str, &str); 4] = [
    (
        "Fly Camera",
        "Right Mouse Button + WASD",
        "Move through scene in forward, backward, and strafe directions.",
    ),
    (
        "Orbit Camera",
        "Alt + Left Mouse Button",
        "Orbit smoothly around selected object or camera focal point.",
    ),
    (
        "Pan Camera",
        "Middle Mouse Button (Scroll Click)",
        "Translate camera horizontally and vertically on viewport plane.",
    ),
    (
        "Zoom to Target",
        "Mouse Scroll Wheel",
        "Step camera position along line of sight toward cursor target.",
    ),
];

/// Builds the Input preferences tab content declaratively using [`UiScope`].
pub fn build_input_tab(scope: &mut UiScope<'_>, params: &mut PreferencesParams<'_>) {
    scope.section_heading(
        "🖱  Input & Controls",
        "Configure mouse sensitivity, viewport interaction hotkeys, and pointer behaviors.",
    );

    scope.collapsing_section(
        "input_mouse",
        "🖱  Mouse & Cursor Sensitivity",
        true,
        |body| {
            body.property_slider_with_options(
                "Look Sensitivity",
                &mut params.editor_config.mouse_sensitivity,
                PropertySliderOptions::new(0.0005, 0.0300, 0.0002)
                    .with_format("{:.4}")
                    .with_subtitle(
                        "Sensitivity multiplier applied to mouse deltas when orbiting or looking around the 3D viewport.",
                    ),
            );
        },
    );

    scope.collapsing_section(
        "input_mapping",
        "🎮  Standard Viewport Gestures",
        true,
        |body| {
            for (action_name, hotkey, note) in VIEWPORT_GESTURES {
                body.container_named(
                    "InputGestureCard",
                    Style::new()
                        .flex_col()
                        .background(body.theme().card_bg)
                        .border(1.0, body.theme().card_border)
                        .border_radius(6.0)
                        .padding_insets(Insets::new(8.0, 12.0, 8.0, 12.0))
                        .margin_insets(Insets::new(0.0, 0.0, 8.0, 0.0)),
                    |card| {
                        card.container_named(
                            "InputGestureRow",
                            Style::new()
                                .flex_row()
                                .align_items(AlignItems::Center)
                                .justify_content(JustifyContent::SpaceBetween)
                                .margin_insets(Insets::new(0.0, 0.0, 2.0, 0.0)),
                            |row| {
                                row.label_styled_passive(
                                    "InputGestureName",
                                    action_name,
                                    12.0,
                                    Color::rgba(1.0, 1.0, 1.0, 1.0),
                                    TextAlign::Left,
                                    Style::new(),
                                );
                                row.label_styled_passive(
                                    "InputGestureKey",
                                    hotkey,
                                    11.5,
                                    Color::rgba(0.0, 0.85, 1.0, 1.0),
                                    TextAlign::Right,
                                    Style::new(),
                                );
                            },
                        );
                        card.label_styled_passive(
                            "InputGestureNote",
                            note,
                            11.0,
                            Color::rgba(0.65, 0.68, 0.76, 1.0),
                            TextAlign::Left,
                            Style::new(),
                        );
                    },
                );
            }
        },
    );
}
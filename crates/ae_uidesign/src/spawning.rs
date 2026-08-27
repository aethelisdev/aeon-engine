// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Data-driven UI Element and HUD Preset Spawning Engine.
//!

use crate::types::UiElementType;
use ae_core::ecs::*;

/// Spawns a canonical UI primitive or HUD preset entity into the ECS world.
pub fn spawn_ui_element(world: &mut hecs::World, ui_type: UiElementType) -> hecs::Entity {
    match ui_type {
        UiElementType::Panel => world.spawn((
            Name("UI Panel".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [300.0, 200.0],
                pivot: [0.5, 0.5],
                z_index: 0,
                alpha: 1.0,
                visible: true,
            },
            UiPanel::default(),
        )),
        UiElementType::ProgressBar => world.spawn((
            Name("UI Progress Bar".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [240.0, 24.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiProgressBar::default(),
        )),
        UiElementType::HealthBar => world.spawn((
            Name("Player Health Bar".to_string()),
            UiElement {
                anchor: UiAnchor::TopLeft,
                offset: [120.0, 36.0],
                size: [180.0, 16.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiProgressBar {
                min: 0.0,
                max: 100.0,
                value: 100.0,
                fill_color: [0.2, 0.85, 0.35, 1.0], // Neon Green
                background_color: [0.08, 0.10, 0.14, 0.85],
                border_color: [0.3, 0.4, 0.5, 0.8],
                corner_radius: 3.0,
            },
            PlayerHealthBarTag,
        )),
        UiElementType::Text => world.spawn((
            Name("UI Text".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [200.0, 30.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiText::new("Sample Text", 16.0),
        )),
        UiElementType::ScoreDisplay => world.spawn((
            Name("Score Display".to_string()),
            UiElement {
                anchor: UiAnchor::TopRight,
                offset: [-100.0, 36.0],
                size: [160.0, 24.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiText::new("SCORE: 00000", 16.0).with_color([1.0, 0.85, 0.2, 1.0]),
            ScoreDisplayTag,
        )),
        UiElementType::Button => world.spawn((
            Name("UI Button".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [140.0, 36.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiButton::default(),
        )),
        UiElementType::Image => world.spawn((
            Name("UI Image".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [64.0, 64.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiImage::default(),
        )),
        UiElementType::Slider => world.spawn((
            Name("UI Slider".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [160.0, 24.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiSlider::default(),
        )),
        UiElementType::Checkbox => world.spawn((
            Name("UI Checkbox".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [120.0, 24.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiCheckbox::default(),
        )),
        UiElementType::TextInput => world.spawn((
            Name("UI Text Input".to_string()),
            UiElement {
                anchor: UiAnchor::Center,
                offset: [0.0, 0.0],
                size: [180.0, 32.0],
                pivot: [0.5, 0.5],
                z_index: 10,
                alpha: 1.0,
                visible: true,
            },
            UiTextInput::default(),
        )),
    }
}
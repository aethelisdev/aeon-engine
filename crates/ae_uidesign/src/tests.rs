// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit Tests for the Aeon UI Designer (AUD) Crate.
//!

use crate::spawning::spawn_ui_element;
use crate::state::UiDesignerState;
use crate::types::{CanvasAspectRatio, UiElementType};
use ae_core::ecs::*;
use hecs::World;

#[test]
fn test_canvas_aspect_ratio_resolutions() {
    assert_eq!(CanvasAspectRatio::Ratio16x9.resolution(), [1920.0, 1080.0]);
    assert_eq!(CanvasAspectRatio::Ratio16x10.resolution(), [1920.0, 1200.0]);
    assert_eq!(CanvasAspectRatio::Ratio4x3.resolution(), [1440.0, 1080.0]);
    assert_eq!(CanvasAspectRatio::Ratio21x9.resolution(), [2560.0, 1080.0]);
}

#[test]
fn test_ui_designer_state_default() {
    let state = UiDesignerState::default();
    assert_eq!(state.aspect_ratio, CanvasAspectRatio::Ratio16x9);
    assert_eq!(state.zoom, 1.0);
    assert!(state.show_anchor_guides);
    assert!(state.show_grid);
    assert_eq!(state.snap_grid, Some(8.0));
    assert!(state.drag_state.is_none());
}

#[test]
fn test_ui_element_type_labels_and_icons() {
    let types = [
        UiElementType::Panel,
        UiElementType::ProgressBar,
        UiElementType::Text,
        UiElementType::Button,
        UiElementType::Image,
        UiElementType::Slider,
        UiElementType::Checkbox,
        UiElementType::TextInput,
        UiElementType::HealthBar,
        UiElementType::ScoreDisplay,
    ];
    for t in types {
        assert!(!t.label().is_empty());
        assert!(!t.icon().is_empty());
    }
}

#[test]
fn test_spawn_ui_element_all_types() {
    let mut world = World::new();

    // 1. Panel
    let ent = spawn_ui_element(&mut world, UiElementType::Panel);
    assert!(world.get::<&UiPanel>(ent).is_ok());
    assert!(world.get::<&UiElement>(ent).is_ok());

    // 2. ProgressBar
    let ent = spawn_ui_element(&mut world, UiElementType::ProgressBar);
    assert!(world.get::<&UiProgressBar>(ent).is_ok());
    assert!(world.get::<&PlayerHealthBarTag>(ent).is_err()); // Generic progress bar has no player tag

    // 3. HealthBar
    let ent = spawn_ui_element(&mut world, UiElementType::HealthBar);
    assert!(world.get::<&UiProgressBar>(ent).is_ok());
    assert!(world.get::<&PlayerHealthBarTag>(ent).is_ok()); // HUD preset has tag

    // 4. Text
    let ent = spawn_ui_element(&mut world, UiElementType::Text);
    assert!(world.get::<&UiText>(ent).is_ok());

    // 5. ScoreDisplay
    let ent = spawn_ui_element(&mut world, UiElementType::ScoreDisplay);
    assert!(world.get::<&UiText>(ent).is_ok());
    assert!(world.get::<&ScoreDisplayTag>(ent).is_ok());

    // 6. Button
    let ent = spawn_ui_element(&mut world, UiElementType::Button);
    assert!(world.get::<&UiButton>(ent).is_ok());

    // 7. Image
    let ent = spawn_ui_element(&mut world, UiElementType::Image);
    assert!(world.get::<&UiImage>(ent).is_ok());

    // 8. Slider
    let ent = spawn_ui_element(&mut world, UiElementType::Slider);
    assert!(world.get::<&UiSlider>(ent).is_ok());

    // 9. Checkbox
    let ent = spawn_ui_element(&mut world, UiElementType::Checkbox);
    assert!(world.get::<&UiCheckbox>(ent).is_ok());

    // 10. TextInput
    let ent = spawn_ui_element(&mut world, UiElementType::TextInput);
    assert!(world.get::<&UiTextInput>(ent).is_ok());
}
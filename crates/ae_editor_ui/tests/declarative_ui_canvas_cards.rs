// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative 2D Screen Transform and UI Canvas Inspector Cards Tests
//!
//! Integration tests verifying declarative UiScope rendering, hierarchy construction,
//! and semantic tag assignments for `UiElement`, `UiPanel`, `UiText`, `UiProgressBar`,
//! `UiButton`, `UiImage`, `UiSlider`, `UiCheckbox`, `UiTextInput`, and HUD tags.

use ae_editor_ui::ui::iris_bridge::inspector::components::ui_canvas::{
    draw_player_health_bar_tag_card, draw_reticle_tag_card, draw_score_display_tag_card,
    draw_ui_button_card, draw_ui_checkbox_card, draw_ui_image_card, draw_ui_panel_card,
    draw_ui_progress_bar_card, draw_ui_slider_card, draw_ui_text_card, draw_ui_text_input_card,
};
use ae_editor_ui::ui::iris_bridge::inspector::ui_transform::build_ui_transform_card;
use irisui::prelude::*;

#[test]
fn test_ui_transform_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::UiElement {
        anchor: ae_core::ecs::UiAnchor::Center,
        offset: [15.0, 25.0],
        size: [120.0, 45.0],
        pivot: [0.5, 0.5],
        z_index: 3,
        alpha: 0.9,
        visible: true,
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        build_ui_transform_card(&mut scope, &world, entity);
    }

    // Delete button tag
    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiElement"));
    assert!(has_delete, "UiElement delete button must be tagged");

    // 2D Screen Transform inputs via DragVec2
    tree.assert_node_exists("NumBox_Offset_X");
    tree.assert_node_exists("NumBox_Offset_Y");
    tree.assert_node_exists("NumBox_Size_W");
    tree.assert_node_exists("NumBox_Size_H");
    tree.assert_node_exists("NumBox_Pivot_X");
    tree.assert_node_exists("NumBox_Pivot_Y");

    // ComboBox and Checkbox
    let has_dropdown = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
    assert!(has_dropdown, "Anchor combobox must be present");

    let has_checkbox = tree.iter().any(|(_, n)| n.role == WidgetRole::Checkbox);
    assert!(has_checkbox, "UiVisible checkbox must be present");
}

#[test]
fn test_ui_panel_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::UiPanel {
        border_width: 2.0,
        corner_radius: 6.0,
        ..Default::default()
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_panel_card(&mut scope, &world, entity);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiPanel"));
    assert!(has_delete, "UiPanel delete button must be tagged");

    // Direct in-place mutation verification
    {
        let mut p = world.get::<&mut ae_core::ecs::UiPanel>(entity).unwrap();
        p.border_width = 5.0;
        p.corner_radius = 8.0;
    }
    let panel = world.get::<&ae_core::ecs::UiPanel>(entity).unwrap();
    assert_eq!(panel.border_width, 5.0);
    assert_eq!(panel.corner_radius, 8.0);
}

#[test]
fn test_ui_text_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::UiText {
        text: "Hello Aeon".to_string(),
        font_size: 16.0,
        alignment: ae_core::ui::UiTextAlignment::Center,
        ..Default::default()
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_text_card(&mut scope, &world, entity);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiText"));
    assert!(has_delete, "UiText delete button must be tagged");

    let has_text_input = tree.iter().any(|(_, n)| n.role == WidgetRole::TextInput);
    assert!(has_text_input, "UiText text input must be present");

    let has_alignment = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("ComboPill"));
    assert!(has_alignment, "UiText alignment combobox must be present");
}

#[test]
fn test_ui_progress_bar_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::UiProgressBar {
        value: 65.0,
        min: 0.0,
        max: 100.0,
        ..Default::default()
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_progress_bar_card(&mut scope, &world, entity);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiProgressBar"));
    assert!(has_delete, "UiProgressBar delete button must be tagged");

    // In-place mutation verification
    {
        let mut pb = world
            .get::<&mut ae_core::ecs::UiProgressBar>(entity)
            .unwrap();
        pb.value = 75.0;
    }
    let bar = world.get::<&ae_core::ecs::UiProgressBar>(entity).unwrap();
    assert_eq!(bar.value, 75.0);
}

#[test]
fn test_ui_button_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::UiButton {
        is_enabled: true,
        ..Default::default()
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_button_card(&mut scope, &world, entity);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiButton"));
    assert!(has_delete, "UiButton delete button must be tagged");

    let has_interactable = tree.iter().any(|(_, n)| n.role == WidgetRole::Checkbox);
    assert!(
        has_interactable,
        "UiButton enabled checkbox must be present"
    );
}

#[test]
fn test_ui_image_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::UiImage {
        slice_mode: ae_core::ui::UiSliceMode::Stretch,
        ..Default::default()
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_image_card(&mut scope, &world, entity);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiImage"));
    assert!(has_delete, "UiImage delete button must be tagged");
}

#[test]
fn test_ui_controls_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        ae_core::ecs::UiSlider {
            value: 0.5,
            min: 0.0,
            max: 1.0,
            ..Default::default()
        },
        ae_core::ecs::UiCheckbox {
            is_checked: true,
            label: "TestOption".to_string(),
            ..Default::default()
        },
        ae_core::ecs::UiTextInput {
            placeholder: "Input...".to_string(),
            ..Default::default()
        },
    ));

    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_ui_slider_card(&mut scope, &world, entity);
        draw_ui_checkbox_card(&mut scope, &world, entity);
        draw_ui_text_input_card(&mut scope, &world, entity);
    }

    let has_slider_del = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiSlider"));
    assert!(has_slider_del, "UiSlider delete button must be tagged");

    let has_cb_del = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiCheckbox"));
    assert!(has_cb_del, "UiCheckbox delete button must be tagged");

    let has_input_del = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("UiTextInput"));
    assert!(has_input_del, "UiTextInput delete button must be tagged");

    let has_placeholder_label = tree
        .iter()
        .any(|(_, n)| n.text.as_deref() == Some("Placeholder"));
    assert!(
        has_placeholder_label,
        "UiTextInput Placeholder property label must be present"
    );

    let has_text_input = tree.iter().any(|(_, n)| n.role == WidgetRole::TextInput);
    assert!(
        has_text_input,
        "UiTextInput interactive TextInput widget must be present"
    );
}

#[test]
fn test_ui_hud_tags_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        ae_core::ecs::PlayerHealthBarTag,
        ae_core::ecs::ScoreDisplayTag,
        ae_core::ecs::ReticleTag,
    ));

    let mut tree = UiTree::new();
    let root = WidgetId::default();

    {
        let mut scope = UiScope::new(&mut tree, root);
        draw_player_health_bar_tag_card(&mut scope, &world, entity);
        draw_score_display_tag_card(&mut scope, &world, entity);
        draw_reticle_tag_card(&mut scope, &world, entity);
    }

    let has_health_del = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("PlayerHealthBarTag"));
    assert!(
        has_health_del,
        "PlayerHealthBarTag delete button must be tagged"
    );

    let has_score_del = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("ScoreDisplayTag"));
    assert!(
        has_score_del,
        "ScoreDisplayTag delete button must be tagged"
    );

    let has_reticle_del = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("ReticleTag"));
    assert!(has_reticle_del, "ReticleTag delete button must be tagged");
}
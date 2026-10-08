// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative 2D Screen Transform and UI Canvas Inspector Cards Tests
//!
//! Integration tests verifying declarative UiScope rendering, hierarchy construction,
//! and semantic tag assignments for `UiElement`, `UiPanel`, `UiText`, `UiProgressBar`,
//! `UiButton`, `UiImage`, `UiSlider`, `UiCheckbox`, `UiTextInput`, and HUD tags.
//!

use ae_editor_ui::ui::iris_bridge::inspector::components::ui_canvas::{
    PlayerHealthBarTagHandler, ReticleTagHandler, ScoreDisplayTagHandler, UiButtonHandler,
    UiCheckboxHandler, UiImageHandler, UiPanelHandler, UiProgressBarHandler, UiSliderHandler,
    UiTextHandler, UiTextInputHandler,
};
use ae_editor_ui::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use ae_editor_ui::ui::iris_bridge::inspector::tags::{
    resolve_component_checkbox_tag, resolve_component_delete_tag, resolve_inspector_dropdown_tag,
    resolve_inspector_number_input_tag, resolve_inspector_text_input_tag,
};
use ae_editor_ui::ui::iris_bridge::inspector::types::{
    ComponentCheckboxId, InspectorDropdownId, InspectorNumberInputId, InspectorPanelParams,
    InspectorTextInputId,
};
use ae_editor_ui::ui::iris_bridge::inspector::ui_transform::build_ui_transform_card;
use irisui::prelude::*;

fn create_test_params<'a>(
    world: &'a hecs::World,
    entity: hecs::Entity,
    euler: &'a [f32; 3],
    swatches: &'a [[f32; 4]],
) -> InspectorPanelParams<'a> {
    InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world,
        selected_entity: Some(entity),
        inspector_euler: euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: swatches,
        cursor_pos: Point::new(0.0, 0.0),
        scroll_y: 0.0,
        active_dropdown: None,
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_number_input: None,
        active_text_input: None,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    }
}

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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        build_ui_transform_card(&mut scope, &mut ctx);
    }

    // Delete button tag
    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiElement"));
    assert!(has_delete, "UiElement delete button must be tagged");

    // Dropdown for Anchor
    let has_dropdown = tree
        .iter()
        .any(|(_, n)| resolve_inspector_dropdown_tag(n.tag) == Some(InspectorDropdownId::UiAnchor));
    assert!(has_dropdown, "UiAnchor dropdown must be tagged");

    // 8 Number inputs: Offset X, Y, Size W, H, Pivot X, Y, ZIndex, Alpha
    let expected_number_inputs = [
        InspectorNumberInputId::UiOffsetX,
        InspectorNumberInputId::UiOffsetY,
        InspectorNumberInputId::UiSizeW,
        InspectorNumberInputId::UiSizeH,
        InspectorNumberInputId::UiPivotX,
        InspectorNumberInputId::UiPivotY,
        InspectorNumberInputId::UiZIndex,
        InspectorNumberInputId::UiAlpha,
    ];
    for expected_id in expected_number_inputs {
        let found = tree.iter().any(|(_, n)| {
            resolve_inspector_number_input_tag(n.tag).is_some_and(|(id, _, _, _)| id == expected_id)
        });
        assert!(
            found,
            "Number input {:?} must be tagged in tree",
            expected_id
        );
    }

    // Visibility Checkbox
    let has_checkbox = tree.iter().any(|(_, n)| {
        resolve_component_checkbox_tag(n.tag) == Some(ComponentCheckboxId::UiVisible)
    });
    assert!(has_checkbox, "UiVisible checkbox must be tagged");
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        UiPanelHandler.render_card(&mut scope, &mut ctx);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiPanel"));
    assert!(has_delete, "UiPanel delete button must be tagged");

    let expected_inputs = [
        InspectorNumberInputId::UiBorderWidth,
        InspectorNumberInputId::UiCornerRadius,
    ];
    for expected_id in expected_inputs {
        let found = tree.iter().any(|(_, n)| {
            resolve_inspector_number_input_tag(n.tag).is_some_and(|(id, _, _, _)| id == expected_id)
        });
        assert!(
            found,
            "Number input {:?} must be tagged in tree",
            expected_id
        );
    }
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        UiTextHandler.render_card(&mut scope, &mut ctx);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiText"));
    assert!(has_delete, "UiText delete button must be tagged");

    let has_text_input = tree.iter().any(|(_, n)| {
        resolve_inspector_text_input_tag(n.tag) == Some(InspectorTextInputId::UiTextContent)
    });
    assert!(has_text_input, "UiTextContent text input must be tagged");

    let has_font_size = tree.iter().any(|(_, n)| {
        resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, _, _, _)| id == InspectorNumberInputId::UiFontSize)
    });
    assert!(has_font_size, "UiFontSize number input must be tagged");

    let has_alignment = tree.iter().any(|(_, n)| {
        resolve_inspector_dropdown_tag(n.tag) == Some(InspectorDropdownId::UiTextAlignment)
    });
    assert!(has_alignment, "UiTextAlignment dropdown must be tagged");
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        UiProgressBarHandler.render_card(&mut scope, &mut ctx);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiProgressBar"));
    assert!(has_delete, "UiProgressBar delete button must be tagged");
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        UiButtonHandler.render_card(&mut scope, &mut ctx);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiButton"));
    assert!(has_delete, "UiButton delete button must be tagged");

    let has_interactable = tree.iter().any(|(_, n)| {
        resolve_component_checkbox_tag(n.tag) == Some(ComponentCheckboxId::UiInteractable)
    });
    assert!(has_interactable, "UiInteractable checkbox must be tagged");
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        UiImageHandler.render_card(&mut scope, &mut ctx);
    }

    let has_delete = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiImage"));
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        UiSliderHandler.render_card(&mut scope, &mut ctx);
        UiCheckboxHandler.render_card(&mut scope, &mut ctx);
        UiTextInputHandler.render_card(&mut scope, &mut ctx);
    }

    let has_slider_del = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiSlider"));
    assert!(has_slider_del, "UiSlider delete button must be tagged");

    let has_cb_del = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiCheckbox"));
    assert!(has_cb_del, "UiCheckbox delete button must be tagged");

    let has_input_del = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("UiTextInput"));
    assert!(has_input_del, "UiTextInput delete button must be tagged");
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
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    {
        let mut scope = UiScope::new(&mut tree, root);
        PlayerHealthBarTagHandler.render_card(&mut scope, &mut ctx);
        ScoreDisplayTagHandler.render_card(&mut scope, &mut ctx);
        ReticleTagHandler.render_card(&mut scope, &mut ctx);
    }

    let has_health_del = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("PlayerHealthBarTag"));
    assert!(
        has_health_del,
        "PlayerHealthBarTag delete button must be tagged"
    );

    let has_score_del = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("ScoreDisplayTag"));
    assert!(
        has_score_del,
        "ScoreDisplayTag delete button must be tagged"
    );

    let has_reticle_del = tree
        .iter()
        .any(|(_, n)| resolve_component_delete_tag(n.tag) == Some("ReticleTag"));
    assert!(has_reticle_del, "ReticleTag delete button must be tagged");
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Rendering, LOD, and Hierarchy Inspector Cards Tests
//!
//! Integration tests verifying declarative UiScope rendering, hierarchy construction,
//! and target registrations for `LightHandler`, `ModelMeshHandler`, `LodGroupHandler`,
//! and `ParentHandler`.

use ae_editor_ui::ui::iris_bridge::inspector::components::hierarchy::ParentHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::lod::LodGroupHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::rendering::{
    LightHandler, ModelMeshHandler,
};
use ae_editor_ui::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use ae_editor_ui::ui::iris_bridge::inspector::types::InspectorPanelParams;
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
        active_submenu: None,
        is_add_menu_open: false,
        is_color_picker_open: false,
        active_rename_buffer: None,
        is_rename_all_selected: false,
        active_hex_buffer: None,
        inspector_hsv: [0.0, 0.0, 1.0],
        blink_caret: false,
    }
}

#[test]
fn test_light_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Light {
        position: [0.0, 5.0, 0.0],
        color: [1.0, 0.8, 0.6],
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = LightHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("Light"));
    assert!(has_delete, "Light delete button must be tagged");

    let numeric_pills = tree
        .iter()
        .filter(|(_, n)| n.name.as_deref() == Some("NumericDrag"))
        .count();
    assert_eq!(
        numeric_pills, 3,
        "Light card must emit 3 numeric pills (Light Offset Y, Color R, Color G)"
    );
}

#[test]
fn test_model_mesh_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::ModelId::default(),));
    let mut tree = UiTree::new();
    let root = WidgetId::default();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = ModelMeshHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("ModelId"));
    assert!(has_delete, "ModelId delete button must be tagged");

    let has_info = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("ModelAssetInfo")
            && n.text
                .as_deref()
                .is_some_and(|t| t.contains("Asset Handle"))
    });
    assert!(has_info);
}

#[test]
fn test_lod_group_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::LodGroup {
        threshold_1: 15.0,
        threshold_2: 35.0,
        lod_1: None,
        lod_2: None,
        ..Default::default()
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, entity, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(entity, &world, &params, 10.0, 20.0, 260.0);

    let handler = LodGroupHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_delete = tree
        .iter()
        .any(|(_, n)| n.tag == UiScope::action_tag("LodGroup"));
    assert!(has_delete, "LodGroup delete button must be tagged");

    let has_slots = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("LodSlotsInfo")
            && n.text
                .as_deref()
                .is_some_and(|t| t.contains("Slots: LOD0 (Active)"))
    });
    assert!(has_slots);

    let has_t1 = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("LodThresh1Lbl")
            && n.text.as_deref().is_some_and(|t| t.contains("15.0 m"))
    });
    assert!(has_t1);
}

#[test]
fn test_parent_card_declarative_structure() {
    let mut world = hecs::World::new();
    let parent = world.spawn((ae_core::ecs::Name("RootSceneNode".to_string()),));
    let child = world.spawn((ae_core::ecs::Parent(parent),));

    let mut tree = UiTree::new();
    let root = WidgetId::default();
    let euler = [0.0, 0.0, 0.0];
    let swatches = [];
    let params = create_test_params(&world, child, &euler, &swatches);

    let mut ctx = ComponentRenderContext::new(child, &world, &params, 10.0, 20.0, 260.0);

    let handler = ParentHandler;
    let mut scope = UiScope::new(&mut tree, root);
    handler.render_card(&mut scope, &mut ctx);

    let has_unparent = tree
        .iter()
        .any(|(_, n)| n.name.as_deref() == Some("UnparentBtn") && n.tag != 0);
    assert!(has_unparent, "Unparent button must be tagged");

    let has_parent_lbl = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("ParentLbl")
            && n.text
                .as_deref()
                .is_some_and(|t| t.contains("RootSceneNode"))
    });
    assert!(has_parent_lbl);
}
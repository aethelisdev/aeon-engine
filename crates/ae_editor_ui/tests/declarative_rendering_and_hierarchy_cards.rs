// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Rendering, LOD, and Hierarchy Inspector Cards Tests
//!
//! Integration tests verifying declarative UiScope rendering, hierarchy construction,
//! and target registrations for `LightHandler`, `ModelMeshHandler`, `LodGroupHandler`,
//! and `ParentHandler`.
//!

use ae_editor_ui::ui::iris_bridge::inspector::components::hierarchy::ParentHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::lod::LodGroupHandler;
use ae_editor_ui::ui::iris_bridge::inspector::components::rendering::{
    LightHandler, ModelMeshHandler,
};
use ae_editor_ui::ui::iris_bridge::inspector::registry::{
    ComponentInspectorHandler, ComponentRenderContext,
};
use ae_editor_ui::ui::iris_bridge::inspector::types::{
    InspectorNumberInputId, InspectorPanelParams,
};
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
        hovered_tag: None,
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

    let has_delete = tree.iter().any(|(_, n)| {
        ae_editor_ui::ui::iris_bridge::inspector::resolve_component_delete_tag(n.tag)
            == Some("Light")
    });
    assert!(has_delete, "Light delete button must be tagged");

    let has_offset_y = tree.iter().any(|(_, n)| {
        ae_editor_ui::ui::iris_bridge::inspector::resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::LightOffsetY)
    });
    assert!(has_offset_y, "LightOffsetY must be tagged");

    let has_color_r = tree.iter().any(|(_, n)| {
        ae_editor_ui::ui::iris_bridge::inspector::resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::LightColorR)
    });
    assert!(has_color_r, "LightColorR must be tagged");

    let has_color_g = tree.iter().any(|(_, n)| {
        ae_editor_ui::ui::iris_bridge::inspector::resolve_inspector_number_input_tag(n.tag)
            .is_some_and(|(id, ..)| id == InspectorNumberInputId::LightColorG)
    });
    assert!(has_color_g, "LightColorG must be tagged");
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

    let has_delete = tree.iter().any(|(_, n)| {
        ae_editor_ui::ui::iris_bridge::inspector::resolve_component_delete_tag(n.tag)
            == Some("ModelId")
    });
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

    let has_delete = tree.iter().any(|(_, n)| {
        ae_editor_ui::ui::iris_bridge::inspector::resolve_component_delete_tag(n.tag)
            == Some("LodGroup")
    });
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
        .any(|(_, n)| ae_editor_ui::ui::iris_bridge::inspector::resolve_unparent_tag(n.tag));
    assert!(has_unparent, "Unparent button must be tagged");

    let has_parent_lbl = tree.iter().any(|(_, n)| {
        n.name.as_deref() == Some("ParentLbl")
            && n.text
                .as_deref()
                .is_some_and(|t| t.contains("RootSceneNode"))
    });
    assert!(has_parent_lbl);
}
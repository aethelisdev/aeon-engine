// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Rendering, LOD, and Hierarchy Inspector Cards Tests
//!
//! Integration tests verifying declarative UiScope rendering, hierarchy construction,
//! and target registrations for `Light`, `ModelId`, `LodGroup`, and `Parent`.

use ae_editor_ui::ui::iris_bridge::inspector::components::hierarchy::draw_parent_card;
use ae_editor_ui::ui::iris_bridge::inspector::components::lod::draw_lod_group_card;
use ae_editor_ui::ui::iris_bridge::inspector::components::rendering::{
    draw_light_card, draw_model_mesh_card,
};
use irisui::prelude::*;

#[test]
fn test_light_card_declarative_structure() {
    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Light {
        position: [0.0, 5.0, 0.0],
        color: [1.0, 0.8, 0.6],
    },));
    let mut tree = UiTree::new();
    let root = WidgetId::default();

    let mut scope = UiScope::new(&mut tree, root);
    draw_light_card(&mut scope, &world, entity);

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

    let mut scope = UiScope::new(&mut tree, root);
    draw_model_mesh_card(&mut scope, &world, entity);

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

    let mut scope = UiScope::new(&mut tree, root);
    draw_lod_group_card(&mut scope, &world, entity);

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

    let mut scope = UiScope::new(&mut tree, root);
    draw_parent_card(&mut scope, &world, child);

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
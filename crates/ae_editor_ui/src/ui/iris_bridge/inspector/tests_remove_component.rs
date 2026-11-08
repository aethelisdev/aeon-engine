// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Inspector Component Removal Unit Tests
//!
//! Verifies that a click on a component card delete button travels through the real event
//! dispatcher, survives the retained subtree rebuild via its persistent semantic tag, and is
//! mapped exactly once by the declarative panel into `EngineUiAction::RemoveComponent`, with no
//! intermediate `InspectorAction` queued.

use super::*;
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{Name, Position, RigidBody, Rotation, Scale, UiElement};
use irisui::prelude::*;

fn test_params<'a>(
    world: &'a hecs::World,
    entity: hecs::Entity,
    euler: &'a [f32; 3],
) -> InspectorPanelParams<'a> {
    InspectorPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 320.0, 900.0),
        world,
        selected_entity: Some(entity),
        inspector_euler: euler,
        inspector_color_hex: "#ffffff",
        saved_swatches: &[],
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

/// Builds the panel, clicks the delete button of `component_name` through the dispatcher, then
/// rebuilds the panel (as the redraw scheduled by `resolve_inspector_tag_click` does) and
/// returns the emitted action together with any queued legacy `InspectorAction`s.
fn click_delete_and_rebuild(
    world: &hecs::World,
    entity: hecs::Entity,
    component_name: &str,
) -> Option<EngineUiAction> {
    let euler = [0.0, 0.0, 0.0];
    let params = test_params(world, entity, &euler);
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root node");
    let mut name = String::new();
    let mut swatches = Vec::new();
    let mut is_color = false;
    let mut is_add = false;
    let mut submenu = None;
    let mut session = super::types::InspectorSessionRefs {
        inspected_name: &mut name,
        saved_swatches: &mut swatches,
        is_color_picker_open: &mut is_color,
        is_add_menu_open: &mut is_add,
        active_submenu: &mut submenu,
    };

    assert!(
        build_inspector_panel(&mut UiScope::new(&mut tree, root), &params, &mut session,).is_none(),
        "No action may be emitted before any click"
    );
    let _ = LayoutEngine::new().compute_layout(&mut tree, Size::new(320.0, 900.0));

    let delete_tag = encode_component_delete_tag(component_name);
    let rect = tree
        .node_rect_tagged(delete_tag, Point::ZERO)
        .expect("Card delete button must carry the component delete tag");
    let point = Point::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        point,
        button: MouseButton::Left,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        point,
        button: MouseButton::Left,
    });
    assert_eq!(tree.event_state.frame_clicked_tag, Some(delete_tag));
    assert!(resolve_inspector_tag_click(delete_tag));

    tree.clear();
    let root2 = tree.create_root().expect("root2");
    build_inspector_panel(&mut UiScope::new(&mut tree, root2), &params, &mut session)
}

#[test]
fn test_registry_card_delete_click_emits_remove_component_directly() {
    let mut world = hecs::World::new();
    let entity = world.spawn((
        Name("Body".to_string()),
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Scale::one(),
        RigidBody::default(),
    ));

    let action = click_delete_and_rebuild(&world, entity, "RigidBody");

    assert!(
        matches!(action, Some(EngineUiAction::RemoveComponent(e, "RigidBody")) if e == entity),
        "Delete click must map directly to EngineUiAction::RemoveComponent(entity, \"RigidBody\")"
    );
}

#[test]
fn test_ui_transform_card_delete_click_emits_remove_component_directly() {
    let mut world = hecs::World::new();
    let entity = world.spawn((Name("Hud".to_string()), UiElement::default()));

    let action = click_delete_and_rebuild(&world, entity, ui_transform::UI_ELEMENT_COMPONENT_NAME);

    assert!(
        matches!(action, Some(EngineUiAction::RemoveComponent(e, name))
            if e == entity && name == ui_transform::UI_ELEMENT_COMPONENT_NAME),
        "2D Screen Transform delete click must map to RemoveComponent(entity, \"UiElement\")"
    );
}
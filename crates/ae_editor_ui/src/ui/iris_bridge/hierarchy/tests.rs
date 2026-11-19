// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Menu & Submenu Invariant Unit Tests
//!
//! Verifies static cascading Add Menu dropdowns, `UiTree` SSOT context menus,
//! declarative `TreeModel` row rendering, eye visibility toggles, and frustum culling.

use super::add_menu::{draw_hierarchy_add_menu, hierarchy_add_menu_entries};
use super::rows::SceneHierarchySource;
use super::types::{
    AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_PANEL_ROOT,
    HIERARCHY_TAG_SEARCH_INPUT, HIERARCHY_TAG_VIEWPORT, HierarchyPanelParams, make_eye_tag,
    make_row_tag,
};
use crate::ui::types::EngineUiAction;
use hecs::World;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

fn panel_params(world: &World, panel_rect: Rect, is_2d: bool) -> HierarchyPanelParams<'_> {
    HierarchyPanelParams {
        panel_rect,
        world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d,
        blink_caret: false,
        cursor_byte_idx: 0,
    }
}

fn build_add_menu(
    tree: &mut UiTree,
    parent_id: WidgetId,
    params: &HierarchyPanelParams<'_>,
    bus: &mut ActionBus<EngineUiAction>,
) -> Vec<Rect> {
    tree.open_popup(HIERARCHY_TAG_ADD_BUTTON.raw());
    let mut rects = Vec::new();
    if let Some(r) = draw_hierarchy_add_menu(&mut UiScope::new(tree, parent_id), params, bus) {
        rects.push(r);
    }
    for (_, node) in tree.iter() {
        if node.layer == UiLayer::Popup
            && node.role == WidgetRole::DropdownPopup
            && node
                .name
                .as_deref()
                .is_some_and(|nm| nm.ends_with("Submenu"))
        {
            rects.push(node.computed_rect);
        }
    }
    rects
}

#[test]
fn test_hierarchy_add_submenu_renders_text_without_self_occlusion() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    tree.set_active_submenu(
        irisui::widgets::declarative::hash_label("HierarchyAddMenu"),
        AddSubmenuId::Objects3D.tag().raw(),
    );

    let world = World::new();
    let params = panel_params(&world, Rect::new(0.0, 0.0, 300.0, 600.0), false);
    let mut bus = ActionBus::new();
    let menu_rects = build_add_menu(&mut tree, root_id, &params, &mut bus);

    assert!(menu_rects.len() >= 2);
    let active_dropdowns = [menu_rects[0], menu_rects[1]];
    let sections = irisui::IrisSurfaceHost::collect_text_sections_from_tree(
        &tree,
        &active_dropdowns,
        &[],
        &[],
    );
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    for expected in [
        "3D Objects",
        "UI & Canvas",
        "Cube",
        "Sphere",
        "Cylinder",
        "Capsule",
        "Torus",
        "Triangle",
    ] {
        assert!(
            rendered_texts.contains(&expected),
            "Item '{expected}' must be visible without self-occlusion"
        );
    }
}

#[test]
fn test_hierarchy_hud_presets_sub_submenu_cascading_and_spawning() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    let root_tag = irisui::widgets::declarative::hash_label("HierarchyAddMenu");
    tree.set_active_submenu(root_tag, AddSubmenuId::UiCanvas.tag().raw());
    tree.set_active_submenu(
        AddSubmenuId::UiCanvas.tag().raw(),
        AddSubmenuId::HudPresets.tag().raw(),
    );

    let world = World::new();
    let params = panel_params(&world, Rect::new(0.0, 0.0, 300.0, 600.0), false);
    let mut bus = ActionBus::new();
    let menu_rects = build_add_menu(&mut tree, root_id, &params, &mut bus);

    assert_eq!(menu_rects.len(), 3);
    let submenu_rect = menu_rects[1];
    let sub_submenu_rect = menu_rects[2];
    assert!(sub_submenu_rect.x >= submenu_rect.right());

    let active_dropdowns = [menu_rects[0], submenu_rect, sub_submenu_rect];
    let sections = irisui::IrisSurfaceHost::collect_text_sections_from_tree(
        &tree,
        &active_dropdowns,
        &[],
        &[],
    );
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();
    assert!(rendered_texts.contains(&"Health Bar (Player Tag)"));
    assert!(rendered_texts.contains(&"Score Display (Score Tag)"));

    let health_click = Point::new(sub_submenu_rect.x + 20.0, sub_submenu_rect.y + 10.0);
    let hit = tree
        .hit_test_target(health_click)
        .expect("Must hit HealthBar item");
    assert_eq!(hit.layer, UiLayer::Popup);
    assert_eq!(hit.role, WidgetRole::DropdownItem);
    assert_eq!(hit.tag, UiTag::from_str("Health Bar (Player Tag)").raw());
}

#[test]
fn test_hierarchy_add_menu_2d_mode_shows_2d_objects() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    tree.set_active_submenu(
        irisui::widgets::declarative::hash_label("HierarchyAddMenu"),
        AddSubmenuId::Objects2D.tag().raw(),
    );

    let world = World::new();
    let params = panel_params(&world, Rect::new(0.0, 0.0, 300.0, 600.0), true);
    let mut bus = ActionBus::new();
    let menu_rects = build_add_menu(&mut tree, root_id, &params, &mut bus);
    assert_eq!(menu_rects.len(), 2);

    let entries = hierarchy_add_menu_entries(true);
    let has_2d_branch = entries.iter().any(|entry| {
        matches!(
            entry,
            CascadingMenuEntry::Submenu { tag, children, .. }
            if *tag == AddSubmenuId::Objects2D.tag() && children.len() == 3
        )
    });
    assert!(has_2d_branch);
}

#[test]
fn test_hierarchy_row_context_menu_via_tree_ssot_and_action_bus() {
    let mut world = World::new();
    let entity = world.spawn((ae_core::ecs::Name("TestEntity".into()),));

    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    UiScope::new(&mut tree, root_id).finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));

    let mut model = TreeModel::new();
    let mut tree_state = TreeState::expanded_by_default();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );

    let params = panel_params(&world, Rect::new(0.0, 0.0, 300.0, 600.0), false);
    let mut bus = ActionBus::new();

    // Right click on row 0 opens UiTree context menu SSOT and selects entity
    let row_tag = make_row_tag(0);
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );
    let row_node_id = tree.find_node_by_tag(row_tag).expect("Row node exists");
    tree.event_state.frame_secondary_clicked_id = Some(row_node_id);
    tree.event_state.frame_secondary_clicked_tag = Some(row_tag);
    tree.event_state.frame_clicked_pos = Some(Point::new(120.0, 200.0));
    let _ = bus.drain();

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );
    assert!(tree.event_state.active_context_menu.is_some());

    // Find the actual tag assigned to "Toggle Visibility" inside ContextMenuPopup
    let ctx_popup_id = tree
        .event_state
        .active_context_menu
        .and_then(|m| m.container_id)
        .expect("ContextMenuPopup container exists");
    let vis_item_tag = tree
        .get(ctx_popup_id)
        .and_then(|p| p.children.get(1).copied())
        .and_then(|cid| tree.get(cid))
        .map(|n| n.tag)
        .expect("Second ContextMenuItem ('Toggle Visibility') exists");

    // Clicking "Toggle Visibility" emits ToggleVisibility(entity) directly to ActionBus
    tree.event_state.frame_secondary_clicked_id = None;
    tree.event_state.frame_secondary_clicked_tag = None;
    tree.event_state.frame_clicked_tag = Some(vis_item_tag);
    let _ = bus.drain();
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    let mut emitted = bus.drain();
    assert!(emitted.any(|a| matches!(a, EngineUiAction::ToggleVisibility(e) if e == entity)));
    assert!(tree.event_state.active_context_menu.is_none());
}

#[test]
fn test_hierarchy_eye_visibility_click_and_rebuild_invalidation() {
    use crate::ui::iris_bridge::icons::{ICON_EYE_CLOSED, ICON_EYE_OPEN};

    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Name("TestEntity".into()),));

    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");
    UiScope::new(&mut tree, root_id).finish_layout(Rect::new(0.0, 0.0, 1920.0, 1080.0));

    let panel_rect = Rect::new(0.0, 40.0, 300.0, 600.0);
    let params = panel_params(&world, panel_rect, false);

    let mut model = TreeModel::new();
    let mut tree_state = TreeState::expanded_by_default();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );
    let mut bus = ActionBus::new();

    tree.event_state.frame_clicked_tag = Some(make_eye_tag(0));
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );
    assert!(
        bus.drain()
            .any(|a| matches!(a, EngineUiAction::ToggleVisibility(e) if e == entity))
    );

    let mut eye_uv_opt = None;
    let mut eye_tag_opt = None;
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("EyeVisibilityButton") {
            eye_tag_opt = Some(node.tag);
        }
        if eye_uv_opt.is_none() && node.texture_uv == Some(ICON_EYE_OPEN) {
            eye_uv_opt = Some(ICON_EYE_OPEN);
        }
    });
    assert_eq!(eye_uv_opt, Some(ICON_EYE_OPEN));
    assert_eq!(eye_tag_opt, Some(make_eye_tag(0)));

    let _ = world.insert_one(entity, ae_core::ecs::Hidden);

    let mut new_tree = UiTree::new();
    let new_root = new_tree.create_root().expect("Root node must be created");
    let new_params = panel_params(&world, panel_rect, false);
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut new_tree, new_root),
        &new_params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    let mut closed_eye_uv_opt = None;
    new_tree.traverse_depth_first(new_root, &mut |_, node| {
        if node.texture_uv == Some(ICON_EYE_CLOSED) {
            closed_eye_uv_opt = Some(ICON_EYE_CLOSED);
        }
    });
    assert_eq!(closed_eye_uv_opt, Some(ICON_EYE_CLOSED));
}

#[test]
fn test_hierarchy_panel_declarative_scope_build() {
    let mut world = hecs::World::new();
    let _e1 = world.spawn((ae_core::ecs::Name("Entity Alpha".into()),));
    let _e2 = world.spawn((ae_core::ecs::Name("Entity Beta".into()),));

    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");

    let params = panel_params(&world, Rect::new(10.0, 10.0, 280.0, 500.0), false);
    let mut model = TreeModel::new();
    let mut tree_state = TreeState::expanded_by_default();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );
    let mut bus = ActionBus::new();

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );
    let max_scroll = tree.max_scroll(HIERARCHY_TAG_VIEWPORT.raw());
    assert_eq!(model.visible_count(), 2);
    assert!(max_scroll >= 0.0);

    let mut found_panel_root = false;
    let mut found_search_input = false;
    let mut found_add_btn = false;
    let mut found_delete_btn = false;
    let mut row_count = 0;

    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.tag == HIERARCHY_TAG_PANEL_ROOT.raw() {
            found_panel_root = true;
        } else if node.tag == HIERARCHY_TAG_SEARCH_INPUT.raw() {
            found_search_input = true;
        } else if node.tag == HIERARCHY_TAG_ADD_BUTTON.raw() {
            found_add_btn = true;
        } else if node.tag == HIERARCHY_TAG_DELETE_BUTTON.raw() {
            found_delete_btn = true;
        } else if node.tag == make_row_tag(0) || node.tag == make_row_tag(1) {
            row_count += 1;
        }
    });

    assert!(found_panel_root);
    assert!(found_search_input);
    assert!(found_add_btn);
    assert!(found_delete_btn);
    assert_eq!(row_count, 2);
}

#[test]
fn test_hierarchy_frustum_culling_limits_rendered_node_count() {
    let mut world = hecs::World::new();
    for i in 0..1000 {
        let _ = world.spawn((ae_core::ecs::Name(format!("Entity_{i}")),));
    }

    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");

    let params = panel_params(&world, Rect::new(0.0, 0.0, 300.0, 200.0), false);
    let mut model = TreeModel::new();
    let mut tree_state = TreeState::expanded_by_default();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );
    let mut bus = ActionBus::new();

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );
    let max_scroll = tree.max_scroll(HIERARCHY_TAG_VIEWPORT.raw());
    assert_eq!(model.visible_count(), 1000);
    assert!(max_scroll > 1000.0);

    let mut rendered_rows_count = 0;
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("HierarchyEntityRow") {
            rendered_rows_count += 1;
        }
    });
    assert!((5..=16).contains(&rendered_rows_count));
}

#[test]
fn test_hierarchy_header_and_row_hover_styling() {
    let mut world = hecs::World::new();
    let _ent = world.spawn((ae_core::ecs::Name("TestEntity".to_string()),));

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");

    let params = panel_params(&world, Rect::new(0.0, 0.0, 300.0, 400.0), false);
    let mut model = TreeModel::new();
    let mut tree_state = TreeState::expanded_by_default();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );
    let mut bus = ActionBus::new();

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    let (_, add_node) = tree
        .iter()
        .find(|(_, n)| n.tag == HIERARCHY_TAG_ADD_BUTTON.raw())
        .expect("Add button must exist");
    assert_eq!(
        add_node.style.hover_background,
        Some(Color::rgba(0.30, 0.35, 0.45, 0.98))
    );

    let row_tag = make_row_tag(0);
    let (_, row_node) = tree
        .iter()
        .find(|(_, n)| n.tag == row_tag)
        .expect("Entity row must exist");
    assert_eq!(
        row_node.style.hover_background,
        Some(Color::rgba(0.14, 0.18, 0.26, 0.65))
    );

    let add_center = Point::new(
        add_node.computed_rect.x + add_node.computed_rect.width * 0.5,
        add_node.computed_rect.y + add_node.computed_rect.height * 0.5,
    );
    assert_eq!(
        tree.hit_test_target(add_center).map(|h| h.tag),
        Some(HIERARCHY_TAG_ADD_BUTTON.raw())
    );

    let row_center = Point::new(
        row_node.computed_rect.x + row_node.computed_rect.width * 0.5,
        row_node.computed_rect.y + row_node.computed_rect.height * 0.5,
    );
    assert_eq!(
        tree.hit_test_target(row_center).map(|h| h.tag),
        Some(row_tag)
    );
}
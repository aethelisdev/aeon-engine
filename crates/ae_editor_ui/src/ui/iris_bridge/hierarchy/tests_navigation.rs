// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy TreeView Navigation and Multi-Selection Tests
//!
//! Verifies that Scene Hierarchy binds to Iris UI [`TreeModel`] and [`TreeState`],
//! handling foldout expansion/collapse, multi-selection modifiers, keyboard arrow navigation,
//! cached revision updates, and 10k entity frustum-culled scrolling.

use super::rows::SceneHierarchySource;
use super::types::{
    HIERARCHY_TAG_VIEWPORT, HierarchyPanelParams, HierarchyPanelState, make_foldout_tag,
    make_row_tag,
};
use crate::ui::types::EngineUiAction;
use hecs::World;
use irisui::prelude::*;
use irisui::widgets::tree_view::{
    TreeItemInfo, TreeSelectionModifiers, handle_tree_arrow_navigation, handle_tree_row_click,
};
use irisui::workbench::ActionBus;

#[test]
fn test_hierarchy_tree_connector_lines_and_foldout() {
    let mut tree = UiTree::new();
    let root_id = tree.create_root().expect("Root node must be created");

    let mut world = World::new();
    let parent_ent = world.spawn((ae_core::ecs::Name("Parent_Tower".to_string()),));
    let _child_a = world.spawn((
        ae_core::ecs::Name("Child_A".to_string()),
        ae_core::ecs::Parent(parent_ent),
    ));
    let _child_b = world.spawn((
        ae_core::ecs::Name("Child_B".to_string()),
        ae_core::ecs::Parent(parent_ent),
    ));

    let mut tree_state = TreeState::expanded_by_default();
    let mut model = TreeModel::new();
    let src = SceneHierarchySource(&world);

    // 1. Expanded by default
    model.sync(&src, &tree_state, 1, world.len() as usize, "");
    assert_eq!(model.visible_count(), 3);
    assert_eq!(model.rows()[0].depth, 0);
    assert!(model.rows()[0].has_children);
    assert!(model.rows()[0].is_expanded);
    assert_eq!(model.rows()[1].depth, 1);
    assert_eq!(model.rows()[2].depth, 1);

    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 400.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        blink_caret: false,
        cursor_byte_idx: 0,
    };

    let mut bus = ActionBus::new();
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    let mut found_foldout = false;
    let mut eye_x_coords = Vec::new();
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("TreeFoldoutButton") {
            found_foldout = true;
            assert_eq!(node.tag, make_foldout_tag(0));
        } else if node.name.as_deref() == Some("EyeVisibilityButton") {
            eye_x_coords.push(node.computed_rect.x);
        }
    });
    assert!(
        found_foldout,
        "Parent row must have a foldout chevron button"
    );
    assert_eq!(eye_x_coords.len(), 3);
    assert_eq!(eye_x_coords[0], eye_x_coords[1]);
    assert_eq!(eye_x_coords[1], eye_x_coords[2]);

    // 2. Collapse parent via tree_state.toggle_expanded & re-sync
    tree_state.toggle_expanded(&parent_ent);
    assert!(model.sync(&src, &tree_state, 1, world.len() as usize, ""));
    assert_eq!(model.visible_count(), 1);
    assert!(!model.rows()[0].is_expanded);

    // 3. Re-expand parent
    tree_state.toggle_expanded(&parent_ent);
    assert!(model.sync(&src, &tree_state, 1, world.len() as usize, ""));
    assert_eq!(model.visible_count(), 3);
    assert!(model.rows()[0].is_expanded);
}

#[test]
fn test_hierarchy_tree_multi_selection_ctrl_and_shift() {
    let mut state = HierarchyPanelState::default();
    let mut world = World::new();

    let ent1 = world.spawn((ae_core::ecs::Name("Entity_1".into()),));
    let _ent2 = world.spawn((ae_core::ecs::Name("Entity_2".into()),));
    let ent3 = world.spawn((ae_core::ecs::Name("Entity_3".into()),));

    state.model.sync(
        &SceneHierarchySource(&world),
        &state.tree_state,
        1,
        world.len() as usize,
        "",
    );
    assert_eq!(state.model.visible_count(), 3);

    let visible_order = state.model.visible_order();
    let resp1 = handle_tree_row_click(
        &mut state.tree_state,
        ent1,
        TreeSelectionModifiers::new(),
        Some(visible_order),
    );
    assert!(resp1.consumed && resp1.selection_changed);
    assert!(state.tree_state.is_selected(&ent1));
    assert_eq!(state.tree_state.selected_count(), 1);

    let resp2 = handle_tree_row_click(
        &mut state.tree_state,
        ent3,
        TreeSelectionModifiers::from_ctrl_shift(true, false),
        Some(visible_order),
    );
    assert!(resp2.consumed && resp2.selection_changed);
    assert!(state.tree_state.is_selected(&ent1));
    assert!(state.tree_state.is_selected(&ent3));
    assert_eq!(state.tree_state.selected_count(), 2);

    state.tree_state.select_single(ent1);
    let resp3 = handle_tree_row_click(
        &mut state.tree_state,
        ent3,
        TreeSelectionModifiers::from_ctrl_shift(false, true),
        Some(visible_order),
    );
    assert!(resp3.consumed && resp3.selection_changed);
    assert_eq!(state.tree_state.selected_count(), 3);
}

#[test]
fn test_hierarchy_keyboard_arrow_navigation() {
    let mut state = HierarchyPanelState::default();
    let mut world = World::new();

    let ent1 = world.spawn((ae_core::ecs::Name("Entity_1".into()),));
    let ent2 = world.spawn((ae_core::ecs::Name("Entity_2".into()),));

    state.model.sync(
        &SceneHierarchySource(&world),
        &state.tree_state,
        1,
        world.len() as usize,
        "",
    );
    assert_eq!(state.model.visible_count(), 2);

    let tree_items: Vec<TreeItemInfo<hecs::Entity>> = state
        .model
        .rows()
        .iter()
        .map(|r| TreeItemInfo::new(r.id, r.depth, r.has_children))
        .collect();

    let nav1 = handle_tree_arrow_navigation(
        &mut state.tree_state,
        &tree_items,
        KeyCode::ArrowDown,
        false,
    );
    assert!(nav1.consumed);
    assert_eq!(state.tree_state.primary_selected(), Some(&ent1));

    let nav2 = handle_tree_arrow_navigation(
        &mut state.tree_state,
        &tree_items,
        KeyCode::ArrowDown,
        false,
    );
    assert!(nav2.consumed);
    assert_eq!(state.tree_state.primary_selected(), Some(&ent2));

    let nav3 =
        handle_tree_arrow_navigation(&mut state.tree_state, &tree_items, KeyCode::ArrowUp, false);
    assert!(nav3.consumed);
    assert_eq!(state.tree_state.primary_selected(), Some(&ent1));
}

#[test]
fn test_hierarchy_declarative_row_selection_dispatch() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation must succeed");

    let mut world = World::new();
    let ent1 = world.spawn((ae_core::ecs::Name("Entity_A".into()),));
    let ent2 = world.spawn((ae_core::ecs::Name("Entity_B".into()),));

    let mut tree_state = TreeState::new();
    let mut model = TreeModel::new();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );
    let mut bus = ActionBus::new();

    tree.event_state.frame_clicked_tag = Some(make_row_tag(0));
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 400.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        blink_caret: false,
        cursor_byte_idx: 0,
    };

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    assert!(tree_state.is_selected(&ent1));
    assert_eq!(tree_state.selected_count(), 1);
    let actions: Vec<_> = bus.drain().collect();
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions[0], EngineUiAction::SelectEntity(Some(e)) if e == ent1));

    // Ctrl-click row 1
    tree.dispatch_event(&UiEvent::ModifiersChanged {
        ctrl_held: true,
        shift_held: false,
    });
    tree.event_state.frame_clicked_tag = Some(make_row_tag(1));

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    assert!(tree_state.is_selected(&ent1));
    assert!(tree_state.is_selected(&ent2));
    assert_eq!(tree_state.selected_count(), 2);
}

#[test]
fn test_hierarchy_10k_entities_virtual_scroll() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must be created");

    let mut world = World::new();
    for i in 0..10_000 {
        world.spawn((ae_core::ecs::Name(format!("Cube_{}", i)),));
    }

    let mut tree_state = TreeState::expanded_by_default();
    let mut model = TreeModel::new();
    model.sync(
        &SceneHierarchySource(&world),
        &tree_state,
        1,
        world.len() as usize,
        "",
    );

    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 800.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        blink_caret: false,
        cursor_byte_idx: 0,
    };
    let mut bus = ActionBus::new();

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    let sections_initial =
        irisui::IrisSurfaceHost::collect_text_sections_from_tree(&tree, &[], &[], &[]);
    let texts_initial: Vec<&str> = sections_initial.iter().map(|s| s.text.as_ref()).collect();
    assert!(texts_initial.contains(&"Cube_0"));
    assert!(texts_initial.contains(&"Cube_1"));

    let mut tree2 = UiTree::new();
    let root2 = tree2.create_root().expect("Root node must be created");
    tree2.set_viewport_size(HIERARCHY_TAG_VIEWPORT.raw(), Size::new(300.0, 748.0));
    tree2.set_scroll_offset(
        HIERARCHY_TAG_VIEWPORT.raw(),
        78.0 * super::rows::HIERARCHY_ROW_STRIDE,
    );

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree2, root2),
        &params,
        &model,
        &mut tree_state,
        &mut bus,
    );

    let sections = irisui::IrisSurfaceHost::collect_text_sections_from_tree(&tree2, &[], &[], &[]);
    let texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();
    assert!(texts.contains(&"Cube_78"));
    assert!(texts.contains(&"Cube_79"));
    assert!(texts.contains(&"Cube_100"));

    let mut layout_engine = irisui::layout::LayoutEngine::new();
    let mut text_system = irisui::text::TextSystem::new();
    text_system.measure_subtree_text(&mut tree2, root2);
    let taffy_res = layout_engine.compute_layout(&mut tree2, Size::new(300.0, 800.0));
    assert!(taffy_res.is_ok());

    let vp_id = tree2
        .find_node_by_tag(HIERARCHY_TAG_VIEWPORT.raw())
        .expect("Viewport exists");
    let vp_node = tree2.get(vp_id).unwrap();

    let top_spacer = tree2.get(vp_node.children[0]).unwrap();
    assert_eq!(
        top_spacer.computed_rect.height,
        76.0 * super::rows::HIERARCHY_ROW_STRIDE
    );

    let bottom_spacer = tree2.get(*vp_node.children.last().unwrap()).unwrap();
    assert!(bottom_spacer.computed_rect.height > 0.0);

    for &cid in &vp_node.children[1..vp_node.children.len() - 1] {
        let child = tree2.get(cid).unwrap();
        assert_eq!(child.computed_rect.height, 24.0);
    }
}
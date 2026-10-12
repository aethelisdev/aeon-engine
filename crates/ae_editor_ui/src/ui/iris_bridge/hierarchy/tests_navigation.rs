// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy TreeView Navigation and Multi-Selection Tests
//!
//! Verifies that Scene Hierarchy correctly binds to Iris UI [`TreeState`],
//! handling foldout expansion/collapse, multi-selection modifiers, and keyboard arrow navigation.
//!

use super::types::{HierarchyPanelParams, HierarchyPanelState, parse_foldout_tag};
use crate::ui::iris_bridge::hierarchy::rows::sync_hierarchy_rows;
use hecs::World;
use irisui::prelude::*;

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
    let mut rows_cache = Vec::new();

    // 1. Expanded by default
    sync_hierarchy_rows(&world, &tree_state, &mut rows_cache);
    assert_eq!(rows_cache.len(), 3);
    assert_eq!(rows_cache[0].depth, 0);
    assert!(rows_cache[0].has_children);
    assert!(rows_cache[0].is_expanded);
    assert_eq!(rows_cache[1].depth, 1);
    assert_eq!(rows_cache[2].depth, 1);

    // 2. Build panel with expanded parent
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 400.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        scroll_y: 0.0,
        active_submenu: None,
        active_sub_submenu: None,
        is_add_menu_open: false,
        active_context_menu: None,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        tree_state: &tree_state,
        is_scrollbar_dragging: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
    };

    let _ = super::panel::build_hierarchy_panel(&mut tree, root_id, &params, &mut rows_cache);

    // Verify foldout chevron widget exists and eye buttons maintain strictly identical X alignment across all rows
    let mut found_foldout = false;
    let mut eye_x_coords = Vec::new();
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("TreeFoldoutButton") {
            found_foldout = true;
            assert_eq!(parse_foldout_tag(node.tag), Some(0));
        } else if node.name.as_deref() == Some("EyeVisibilityButton") {
            eye_x_coords.push(node.computed_rect.x);
        }
    });
    assert!(
        found_foldout,
        "Parent row must have a foldout chevron button"
    );
    assert_eq!(eye_x_coords.len(), 3, "All 3 rows must render eye buttons");
    assert_eq!(
        eye_x_coords[0], eye_x_coords[1],
        "Parent and child eye buttons must have strictly identical X alignment"
    );
    assert_eq!(
        eye_x_coords[1], eye_x_coords[2],
        "Sibling child eye buttons must have strictly identical X alignment"
    );

    // 3. Collapse parent via handle_tree_foldout_click
    let resp = handle_tree_foldout_click(&mut tree_state, &parent_ent);
    assert!(resp.consumed);
    assert!(resp.layout_changed);
    rows_cache.clear();
    sync_hierarchy_rows(&world, &tree_state, &mut rows_cache);
    assert_eq!(rows_cache.len(), 1, "Collapsed parent hides its children");
    assert!(!rows_cache[0].is_expanded);

    // 4. Re-expand parent via handle_tree_foldout_click
    let resp2 = handle_tree_foldout_click(&mut tree_state, &parent_ent);
    assert!(resp2.consumed);
    assert!(resp2.layout_changed);
    rows_cache.clear();
    sync_hierarchy_rows(&world, &tree_state, &mut rows_cache);
    assert_eq!(rows_cache.len(), 3, "Expanded parent restores its children");
    assert!(rows_cache[0].is_expanded);
}

#[test]
fn test_hierarchy_tree_multi_selection_ctrl_and_shift() {
    let mut state = HierarchyPanelState::default();
    let mut world = World::new();

    let ent1 = world.spawn((ae_core::ecs::Name("Entity_1".into()),));
    let ent2 = world.spawn((ae_core::ecs::Name("Entity_2".into()),));
    let ent3 = world.spawn((ae_core::ecs::Name("Entity_3".into()),));

    sync_hierarchy_rows(&world, &state.tree_state, &mut state.rows_cache);
    assert_eq!(state.rows_cache.len(), 3);

    // 1. Initial single selection on row 0
    let visible_order = [ent1, ent2, ent3];
    let resp1 = handle_tree_row_click(
        &mut state.tree_state,
        ent1,
        TreeSelectionModifiers::new(),
        Some(&visible_order),
    );
    assert!(resp1.consumed);
    assert!(resp1.selection_changed);
    assert!(state.tree_state.is_selected(&ent1));
    assert_eq!(state.tree_state.selected_count(), 1);

    // 2. Ctrl+click on row 2 adds to selection
    let resp2 = handle_tree_row_click(
        &mut state.tree_state,
        ent3,
        TreeSelectionModifiers::from_ctrl_shift(true, false),
        Some(&visible_order),
    );
    assert!(resp2.consumed);
    assert!(resp2.selection_changed);
    assert!(state.tree_state.is_selected(&ent1));
    assert!(state.tree_state.is_selected(&ent3));
    assert_eq!(state.tree_state.selected_count(), 2);

    // 3. Shift+click from ent1 to ent3 selects range
    state.tree_state.select_single(ent1);
    let resp3 = handle_tree_row_click(
        &mut state.tree_state,
        ent3,
        TreeSelectionModifiers::from_ctrl_shift(false, true),
        Some(&visible_order),
    );
    assert!(resp3.consumed);
    assert!(resp3.selection_changed);
    assert_eq!(state.tree_state.selected_count(), 3);
    assert!(state.tree_state.is_selected(&ent1));
    assert!(state.tree_state.is_selected(&ent2));
    assert!(state.tree_state.is_selected(&ent3));
}

#[test]
fn test_hierarchy_keyboard_arrow_navigation() {
    let mut state = HierarchyPanelState::default();
    let mut world = World::new();

    let ent1 = world.spawn((ae_core::ecs::Name("Entity_1".into()),));
    let ent2 = world.spawn((ae_core::ecs::Name("Entity_2".into()),));

    sync_hierarchy_rows(&world, &state.tree_state, &mut state.rows_cache);
    assert_eq!(state.rows_cache.len(), 2);

    let tree_items: Vec<TreeItemInfo<hecs::Entity>> = state
        .rows_cache
        .iter()
        .map(|r| TreeItemInfo::new(r.entity, r.depth, r.has_children))
        .collect();

    // 1. ArrowDown navigation selects first item
    let nav1 = handle_tree_arrow_navigation(
        &mut state.tree_state,
        &tree_items,
        KeyCode::ArrowDown,
        false,
    );
    assert!(nav1.consumed);
    assert_eq!(state.tree_state.primary_selected(), Some(&ent1));

    // 2. Second ArrowDown moves to ent2
    let nav2 = handle_tree_arrow_navigation(
        &mut state.tree_state,
        &tree_items,
        KeyCode::ArrowDown,
        false,
    );
    assert!(nav2.consumed);
    assert_eq!(state.tree_state.primary_selected(), Some(&ent2));

    // 3. ArrowUp moves back to ent1
    let nav3 =
        handle_tree_arrow_navigation(&mut state.tree_state, &tree_items, KeyCode::ArrowUp, false);
    assert!(nav3.consumed);
    assert_eq!(state.tree_state.primary_selected(), Some(&ent1));
}
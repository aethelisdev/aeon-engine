// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy TreeView Navigation and Multi-Selection Tests
//!
//! Verifies that Scene Hierarchy correctly binds to Iris UI [`TreeState`],
//! handling foldout expansion/collapse, multi-selection modifiers, and keyboard arrow navigation.

use super::types::{HierarchyPanelParams, HierarchyPanelState, parse_foldout_tag};
use crate::ui::iris_bridge::hierarchy::rows::sync_hierarchy_rows;
use hecs::World;
use irisui::prelude::*;
use irisui::widgets::controls::FocusState;
use irisui::widgets::tree_view::{
    TreeItemInfo, TreeSelectionModifiers, handle_tree_arrow_navigation, handle_tree_row_click,
};

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
        active_submenu: None,
        active_sub_submenu: None,
        is_add_menu_open: false,
        active_context_menu: None,
        cursor_pos: Point::new(0.0, 0.0),
        blink_caret: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
        ctrl_held: false,
        shift_held: false,
    };

    let mut actions = Vec::new();
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root_id),
        &params,
        &mut rows_cache,
        &mut actions,
        &mut tree_state,
    );

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

    // 3. Collapse parent via tree_state.toggle_expanded
    tree_state.toggle_expanded(&parent_ent);
    rows_cache.clear();
    sync_hierarchy_rows(&world, &tree_state, &mut rows_cache);
    assert_eq!(rows_cache.len(), 1, "Collapsed parent hides its children");
    assert!(!rows_cache[0].is_expanded);

    // 4. Re-expand parent via tree_state.toggle_expanded
    tree_state.toggle_expanded(&parent_ent);
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

#[test]
fn test_hierarchy_declarative_row_selection_dispatch() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root creation must succeed");

    let mut world = World::new();
    let ent1 = world.spawn((ae_core::ecs::Name("Entity_A".into()),));
    let ent2 = world.spawn((ae_core::ecs::Name("Entity_B".into()),));

    let mut tree_state = TreeState::new();
    let mut rows_cache = Vec::new();
    let mut actions = Vec::new();

    // 1. Initial click on row 0 (ent1)
    tree.event_state.frame_clicked_tag = Some(super::types::make_row_tag(0));

    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 400.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        active_submenu: None,
        active_sub_submenu: None,
        is_add_menu_open: false,
        active_context_menu: None,
        cursor_pos: Point::new(100.0, 45.0),
        blink_caret: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
        ctrl_held: false,
        shift_held: false,
    };

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut rows_cache,
        &mut actions,
        &mut tree_state,
    );

    // Initial click selects ent1
    assert!(tree_state.is_selected(&ent1));
    assert_eq!(tree_state.selected_count(), 1);
    assert_eq!(
        actions.first(),
        Some(&super::types::HierarchyAction::SelectEntity(Some(ent1)))
    );

    // 2. Second click on row 1 (ent2) with Ctrl modifier
    actions.clear();
    tree.event_state.frame_clicked_tag = Some(super::types::make_row_tag(1));
    let params_ctrl = HierarchyPanelParams {
        ctrl_held: true,
        ..params
    };

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params_ctrl,
        &mut rows_cache,
        &mut actions,
        &mut tree_state,
    );

    // Both ent1 and ent2 are now selected in TreeState
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
    let mut rows_cache = Vec::new();

    let panel_rect = Rect::new(0.0, 0.0, 300.0, 800.0);
    let params = HierarchyPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        active_submenu: None,
        active_sub_submenu: None,
        is_add_menu_open: false,
        active_context_menu: None,
        cursor_pos: Point::new(100.0, 45.0),
        blink_caret: false,
        focus_state: &FocusState::new(),
        cursor_byte_idx: 0,
        ctrl_held: false,
        shift_held: false,
    };

    let mut actions = Vec::new();

    // 1. Initial build: verifies initial top slice (Cube_0..Cube_30) is collected
    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree, root),
        &params,
        &mut rows_cache,
        &mut actions,
        &mut tree_state,
    );

    let sections_initial =
        crate::ui::IrisEditorOverlay::collect_text_sections_from_tree(&tree, &[], &[], &[]);
    let texts_initial: Vec<&str> = sections_initial.iter().map(|s| s.text.as_ref()).collect();
    assert!(texts_initial.contains(&"Cube_0"));
    assert!(texts_initial.contains(&"Cube_1"));

    // 2. Scrolled state: simulate scrolling down to item 78
    let mut tree2 = UiTree::new();
    let root2 = tree2.create_root().expect("Root node must be created");
    tree2.set_viewport_size(
        super::types::HIERARCHY_TAG_VIEWPORT,
        Size::new(300.0, 748.0),
    );
    tree2.set_scroll_offset(
        super::types::HIERARCHY_TAG_VIEWPORT,
        78.0 * super::rows::HIERARCHY_ROW_STRIDE,
    );

    super::panel::build_hierarchy_panel(
        &mut UiScope::new(&mut tree2, root2),
        &params,
        &mut rows_cache,
        &mut actions,
        &mut tree_state,
    );

    let sections =
        crate::ui::IrisEditorOverlay::collect_text_sections_from_tree(&tree2, &[], &[], &[]);
    let texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();
    assert!(texts.contains(&"Cube_78"));
    assert!(texts.contains(&"Cube_79"));
    assert!(texts.contains(&"Cube_100"));

    // 3. Verify Taffy boundary layout does NOT flex-shrink virtual rows or spacers
    let mut layout_engine = irisui::layout::LayoutEngine::new();
    let mut text_system = irisui::text::TextSystem::new();
    text_system.measure_subtree_text(&mut tree2, root2);
    let taffy_res = layout_engine.compute_layout(&mut tree2, Size::new(300.0, 800.0));
    assert!(taffy_res.is_ok(), "Taffy layout computation must succeed");

    let vp_id = tree2
        .find_node_by_tag(super::types::HIERARCHY_TAG_VIEWPORT)
        .expect("Viewport exists");
    let vp_node = tree2.get(vp_id).unwrap();

    // Verify top spacer preserved its full height
    let top_spacer = tree2.get(vp_node.children[0]).unwrap();
    assert!(top_spacer.computed_rect.height > 0.0);
    assert_eq!(
        top_spacer.computed_rect.height,
        76.0 * super::rows::HIERARCHY_ROW_STRIDE
    );

    // Verify bottom spacer preserved its full height
    let bottom_spacer = tree2.get(*vp_node.children.last().unwrap()).unwrap();
    assert!(bottom_spacer.computed_rect.height > 0.0);

    // Verify all row children maintain full row height without shrinkage
    for &cid in &vp_node.children[1..vp_node.children.len() - 1] {
        let child = tree2.get(cid).unwrap();
        assert_eq!(
            child.computed_rect.height, 24.0,
            "Row height must remain exactly 24px and not collapse under Taffy flexbox"
        );
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Menu & Submenu Invariant Unit Tests
//!
//! Verifies that cascading Add Menu dropdowns, context menus, and declarative
//! tree rows assign correct semantic hardware tags and retain their text labels without occlusion.
//!

use super::add_menu::build_add_menu;
use super::types::{
    AddSubmenuId, HIERARCHY_TAG_ADD_BUTTON, HIERARCHY_TAG_DELETE_BUTTON, HIERARCHY_TAG_EYE_PREFIX,
    HIERARCHY_TAG_PANEL_ROOT, HIERARCHY_TAG_ROW_PREFIX, HIERARCHY_TAG_SEARCH_CLEAR,
    HIERARCHY_TAG_SEARCH_INPUT, HierarchyPanelParams, is_hierarchy_tag, make_eye_tag, make_row_tag,
    parse_eye_tag, parse_foldout_tag, parse_row_tag,
};
use crate::ui::iris_bridge::types::IrisEditorOverlay;
use hecs::World;
use irisui::prelude::*;
use std::collections::HashSet;

#[test]
fn test_hierarchy_add_submenu_renders_text_without_self_occlusion() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        scroll_y: 0.0,
        active_submenu: Some(AddSubmenuId::Objects3D),
        active_sub_submenu: None,
        is_add_menu_open: true,
        active_context_menu: None,
        cursor_pos: Point::new(150.0, 100.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let menu_rects = build_add_menu(&mut tree, root_id, &params);

    assert!(
        menu_rects.len() >= 2,
        "Add menu and submenu cards must be constructed"
    );
    let add_menu_rect = menu_rects[0];
    let submenu_rect = menu_rects[1];

    // Verify DropdownPopup semantic role is set on popup card nodes
    let mut found_submenu_popup_role = false;
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.layer == UiLayer::Popup && node.role == WidgetRole::DropdownPopup {
            found_submenu_popup_role = true;
        }
    });
    assert!(
        found_submenu_popup_role,
        "Popup card node must have WidgetRole::DropdownPopup"
    );

    // Collect text sections with both dropdown rects registered as active dropdowns
    let active_dropdowns = [add_menu_rect, submenu_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &active_dropdowns, &[], &[]);

    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    // Verify root menu items are rendered
    assert!(
        rendered_texts.contains(&"3D Objects"),
        "Root menu item '3D Objects' must be visible"
    );
    assert!(
        rendered_texts.contains(&"UI & Canvas"),
        "Root menu item 'UI & Canvas' must be visible"
    );

    // Verify submenu items are NOT occluded by active_submenu_rect
    assert!(
        rendered_texts.contains(&"Cube"),
        "Submenu item 'Cube' must be visible and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Sphere"),
        "Submenu item 'Sphere' must be visible and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Cylinder"),
        "Submenu item 'Cylinder' must be visible and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Capsule"),
        "Submenu item 'Capsule' must be visible and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Torus"),
        "Submenu item 'Torus' must be visible and not occluded"
    );
    assert!(
        rendered_texts.contains(&"Triangle"),
        "Submenu item 'Triangle' must be visible and not occluded"
    );
}

#[test]
fn test_hierarchy_ui_canvas_submenu_preserves_text_labels() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        scroll_y: 0.0,
        active_submenu: Some(AddSubmenuId::UiCanvas),
        active_sub_submenu: None,
        is_add_menu_open: true,
        active_context_menu: None,
        cursor_pos: Point::new(150.0, 100.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let menu_rects = build_add_menu(&mut tree, root_id, &params);

    assert!(menu_rects.len() >= 2);
    let add_menu_rect = menu_rects[0];
    let submenu_rect = menu_rects[1];

    let active_dropdowns = [add_menu_rect, submenu_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &active_dropdowns, &[], &[]);

    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"Panel / Canvas Box"),
        "Submenu item 'Panel / Canvas Box' must be visible"
    );
    assert!(
        rendered_texts.contains(&"HUD Presets"),
        "Submenu item 'HUD Presets' must be visible"
    );
}

#[test]
fn test_hierarchy_hud_presets_sub_submenu_cascading_and_spawning() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        scroll_y: 0.0,
        active_submenu: Some(AddSubmenuId::UiCanvas),
        active_sub_submenu: Some(AddSubmenuId::HudPresets),
        is_add_menu_open: true,
        active_context_menu: None,
        cursor_pos: Point::new(150.0, 100.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let menu_rects = build_add_menu(&mut tree, root_id, &params);

    assert_eq!(
        menu_rects.len(),
        3,
        "Root menu, UI Canvas submenu, and HUD Presets sub-submenu must exist"
    );
    let _add_menu_rect = menu_rects[0];
    let submenu_rect = menu_rects[1];
    let sub_submenu_rect = menu_rects[2];

    // Verify sub-submenu is positioned to the right of submenu card
    assert!(
        sub_submenu_rect.x >= submenu_rect.right(),
        "HUD Presets sub-submenu must cascade to the right of UI & Canvas card"
    );

    // Verify HUD Presets items (309: HealthBar, 310: ScoreDisplay) resolve actions
    assert_eq!(
        super::add_menu::get_hierarchy_add_menu_action(309),
        Some(super::types::HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::HealthBar
        ))
    );
    assert_eq!(
        super::add_menu::get_hierarchy_add_menu_action(310),
        Some(super::types::HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::ScoreDisplay
        ))
    );

    // Verify text labels render without self-occlusion in text collector
    let active_dropdowns = [menu_rects[0], submenu_rect, sub_submenu_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &active_dropdowns, &[], &[]);
    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"Health Bar (Player Tag)"),
        "Sub-submenu item 'Health Bar (Player Tag)' must be visible without self-occlusion"
    );
    assert!(
        rendered_texts.contains(&"Score Display (Score Tag)"),
        "Sub-submenu item 'Score Display (Score Tag)' must be visible without self-occlusion"
    );

    // Verify interactive click on Health Bar item dispatches SpawnUiElement action via hit_test_target
    let health_click = Point::new(sub_submenu_rect.x + 20.0, sub_submenu_rect.y + 10.0);
    let hit = tree
        .hit_test_target(health_click)
        .expect("Must hit HealthBar item");
    assert_eq!(hit.layer, UiLayer::Popup);
    assert_eq!(hit.role, WidgetRole::DropdownItem);
    assert_eq!(hit.tag, 309);
    assert_eq!(
        super::add_menu::get_hierarchy_add_menu_action(hit.tag),
        Some(super::types::HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::HealthBar
        ))
    );
}

#[test]
fn test_hierarchy_add_menu_dark_styling_and_popup_roles() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 28.0, 260.0, 600.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: false,
        scroll_y: 0.0,
        active_submenu: Some(AddSubmenuId::UiCanvas),
        active_sub_submenu: None,
        is_add_menu_open: true,
        active_context_menu: None,
        cursor_pos: Point::new(150.0, 100.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let menu_rects = build_add_menu(&mut tree, root_id, &params);

    // Verify popup cards exist and have DropdownPopup role
    let mut popup_cards = 0;
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.layer == UiLayer::Popup && node.role == WidgetRole::DropdownPopup {
            popup_cards += 1;
        }
    });
    assert!(
        popup_cards >= 2,
        "Root Add Menu and Submenu popup cards must exist"
    );

    assert!(menu_rects.len() >= 2);
}

#[test]
fn test_hierarchy_add_menu_2d_mode_shows_2d_objects() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: true,
        scroll_y: 0.0,
        active_submenu: Some(AddSubmenuId::Objects2D),
        active_sub_submenu: None,
        is_add_menu_open: true,
        active_context_menu: None,
        cursor_pos: Point::new(150.0, 100.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let menu_rects = build_add_menu(&mut tree, root_id, &params);

    assert_eq!(menu_rects.len(), 2);

    let items_2d = super::add_menu::get_hierarchy_add_menu_items(true);
    let branch_2d = items_2d
        .iter()
        .find(|i| i.tag == AddSubmenuId::Objects2D.to_tag())
        .expect("2D branch exists");
    let sub_items = branch_2d.submenu.as_ref().expect("2D submenu exists");

    assert!(
        sub_items.iter().any(|i| i.tag == 201),
        "Submenu must contain Sprite in 2D mode"
    );
    assert!(
        sub_items.iter().any(|i| i.tag == 202),
        "Submenu must contain Player Sprite in 2D mode"
    );
    assert!(
        sub_items.iter().any(|i| i.tag == 203),
        "Submenu must contain Empty 2D in 2D mode"
    );
}

#[test]
fn test_hierarchy_add_menu_click_submenu_item() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: None,
        search_query: "",
        is_editing: true,
        is_2d: true,
        scroll_y: 0.0,
        active_submenu: None,
        active_sub_submenu: None,
        is_add_menu_open: true,
        active_context_menu: None,
        cursor_pos: Point::new(150.0, 50.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let menu_rects = build_add_menu(&mut tree, root_id, &params);

    assert!(!menu_rects.is_empty());
    let root_rect = menu_rects[0];
    let click_pt = Point::new(root_rect.x + 15.0, root_rect.y + 10.0);

    let hit = tree
        .hit_test_target(click_pt)
        .expect("Must hit first menu item");
    assert_eq!(hit.layer, UiLayer::Popup);
    assert_eq!(hit.role, WidgetRole::DropdownItem);
    assert_eq!(
        AddSubmenuId::from_tag(hit.tag),
        Some(AddSubmenuId::Objects2D)
    );
}

#[test]
fn test_hierarchy_context_menu_builder_and_hit_testing() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }
    let _ = tree.set_root(root_id);

    let mut world = World::new();
    let entity = world.spawn(("TestEntity",));

    let params = HierarchyPanelParams {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        world: &world,
        selected_entity: Some(entity),
        search_query: "",
        is_editing: true,
        is_2d: false,
        scroll_y: 0.0,
        active_submenu: None,
        active_sub_submenu: None,
        is_add_menu_open: false,
        active_context_menu: Some((entity, Point::new(120.0, 200.0))),
        cursor_pos: Point::new(125.0, 210.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let card_rect_opt = super::context_menu::build_context_menu(&mut tree, root_id, &params);

    assert!(card_rect_opt.is_some());
    let card_rect = card_rect_opt.unwrap();
    assert_eq!(card_rect.x, 120.0);
    assert_eq!(card_rect.y, 200.0);

    // Hit test delete item (tag 0)
    let hit_del = tree
        .hit_test_target(Point::new(130.0, 212.0))
        .expect("Must hit Delete item");
    assert_eq!(hit_del.layer, UiLayer::Popup);
    assert_eq!(hit_del.role, WidgetRole::DropdownItem);
    assert_eq!(hit_del.tag, super::types::HIERARCHY_CTX_DELETE);

    // Hit test toggle visibility item (tag 1)
    let hit_vis = tree
        .hit_test_target(Point::new(130.0, 236.0))
        .expect("Must hit Visibility item");
    assert_eq!(hit_vis.layer, UiLayer::Popup);
    assert_eq!(hit_vis.role, WidgetRole::DropdownItem);
    assert_eq!(hit_vis.tag, super::types::HIERARCHY_CTX_VISIBILITY);
}

#[test]
fn test_hierarchy_eye_visibility_click_and_rebuild_invalidation() {
    use crate::ui::iris_bridge::icons::{ICON_EYE_CLOSED, ICON_EYE_OPEN};

    let mut world = hecs::World::new();
    let entity = world.spawn((ae_core::ecs::Name("TestEntity".into()),));

    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    if let Some(node) = tree.get_mut(root_id) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }
    let _ = tree.set_root(root_id);

    let mut rows_cache = Vec::new();

    let panel_rect = Rect::new(0.0, 40.0, 300.0, 600.0);
    let params = HierarchyPanelParams {
        panel_rect,
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
        cursor_pos: Point::new(10.0, 10.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    super::panel::build_hierarchy_panel(&mut tree, root_id, &params, &mut rows_cache);

    assert_eq!(rows_cache.len(), 1);
    assert_eq!(rows_cache[0].entity, entity);

    // Verify Eye Visibility Button exists in tree and uses ICON_EYE_OPEN initially
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
    assert_eq!(parse_eye_tag(make_eye_tag(0)), Some(0));

    // Execute toggle visibility on the entity
    assert!(world.get::<&ae_core::ecs::Hidden>(entity).is_err());
    let _ = world.insert_one(entity, ae_core::ecs::Hidden);
    assert!(world.get::<&ae_core::ecs::Hidden>(entity).is_ok());

    // Rebuilding hierarchy panel with mutated world must now reflect ICON_EYE_CLOSED
    let mut new_tree = UiTree::new();
    let new_root = new_tree.create_node();
    let _ = new_tree.set_root(new_root);
    let mut new_rows_cache = Vec::new();

    let new_params = HierarchyPanelParams {
        panel_rect,
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
        cursor_pos: Point::new(10.0, 10.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    super::panel::build_hierarchy_panel(&mut new_tree, new_root, &new_params, &mut new_rows_cache);

    let mut closed_eye_uv_opt = None;
    new_tree.traverse_depth_first(new_root, &mut |_, node| {
        if node.texture_uv == Some(ICON_EYE_CLOSED) {
            closed_eye_uv_opt = Some(ICON_EYE_CLOSED);
        }
    });
    assert_eq!(closed_eye_uv_opt, Some(ICON_EYE_CLOSED));
}

#[test]
fn test_hierarchy_semantic_tags_and_roundtrip() {
    // 1. Static Tag Constants
    assert!(is_hierarchy_tag(HIERARCHY_TAG_PANEL_ROOT));
    assert!(is_hierarchy_tag(HIERARCHY_TAG_SEARCH_INPUT));
    assert!(is_hierarchy_tag(HIERARCHY_TAG_SEARCH_CLEAR));
    assert!(is_hierarchy_tag(HIERARCHY_TAG_ADD_BUTTON));
    assert!(is_hierarchy_tag(HIERARCHY_TAG_DELETE_BUTTON));

    // Unrelated tag
    assert!(!is_hierarchy_tag(0x1234_5678_0000_0000));
    assert!(!is_hierarchy_tag(0));

    // 2. Row Tags Roundtrip
    for idx in [0, 1, 42, 1024, 0x00FF_FFFF] {
        let tag = make_row_tag(idx);
        assert!(is_hierarchy_tag(tag));
        assert_eq!(parse_row_tag(tag), Some(idx));
        assert_eq!(parse_eye_tag(tag), None);
    }

    // 3. Eye Tags Roundtrip
    for idx in [0, 1, 42, 1024, 0x00FF_FFFF] {
        let tag = make_eye_tag(idx);
        assert!(is_hierarchy_tag(tag));
        assert_eq!(parse_eye_tag(tag), Some(idx));
        assert_eq!(parse_row_tag(tag), None);
    }

    // Tag prefix isolation
    assert_ne!(HIERARCHY_TAG_ROW_PREFIX, HIERARCHY_TAG_EYE_PREFIX);
}

#[test]
fn test_hierarchy_panel_declarative_scope_build() {
    let mut world = hecs::World::new();
    let _e1 = world.spawn((ae_core::ecs::Name("Entity Alpha".into()),));
    let _e2 = world.spawn((ae_core::ecs::Name("Entity Beta".into()),));

    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let panel_rect = Rect::new(10.0, 10.0, 280.0, 500.0);
    let params = HierarchyPanelParams {
        panel_rect,
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
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let mut rows_cache = Vec::new();
    let max_scroll =
        super::panel::build_hierarchy_panel(&mut tree, root_id, &params, &mut rows_cache);

    assert_eq!(rows_cache.len(), 2);
    assert!(max_scroll >= 0.0);

    // Verify root container has HIERARCHY_TAG_PANEL_ROOT
    let mut found_panel_root = false;
    let mut found_search_input = false;
    let mut found_add_btn = false;
    let mut found_delete_btn = false;
    let mut row_count = 0;

    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.tag == HIERARCHY_TAG_PANEL_ROOT {
            found_panel_root = true;
        } else if node.tag == HIERARCHY_TAG_SEARCH_INPUT {
            found_search_input = true;
        } else if node.tag == HIERARCHY_TAG_ADD_BUTTON {
            found_add_btn = true;
        } else if node.tag == HIERARCHY_TAG_DELETE_BUTTON {
            found_delete_btn = true;
        } else if parse_row_tag(node.tag).is_some() {
            row_count += 1;
        }
    });

    assert!(found_panel_root, "Panel root tag must be present");
    assert!(found_search_input, "Search input tag must be present");
    assert!(found_add_btn, "Add button tag must be present");
    assert!(found_delete_btn, "Delete button tag must be present");
    assert_eq!(row_count, 2, "Both entity rows must be tagged");
}

#[test]
fn test_hierarchy_frustum_culling_limits_rendered_node_count() {
    let mut world = hecs::World::new();
    // Spawn 1,000 entities in the scene
    for i in 0..1000 {
        let _ = world.spawn((ae_core::ecs::Name(format!("Entity_{i}")),));
    }

    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    // Viewport height: 200.0 px (accommodates ~8 visible rows of 24px + 2px gap)
    let panel_rect = Rect::new(0.0, 0.0, 300.0, 200.0);
    let params = HierarchyPanelParams {
        panel_rect,
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
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &HashSet::new(),
        hovered_tag: None,
    };

    let mut rows_cache = Vec::new();
    let max_scroll =
        super::panel::build_hierarchy_panel(&mut tree, root_id, &params, &mut rows_cache);

    assert_eq!(
        rows_cache.len(),
        1000,
        "1,000 entities must be cached in rows_cache"
    );
    assert!(
        max_scroll > 1000.0,
        "Max scroll must reflect total 1,000 entities height"
    );

    // Count how many `HierarchyEntityRow` widgets were physically instantiated in UiTree
    let mut rendered_rows_count = 0;
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("HierarchyEntityRow") {
            rendered_rows_count += 1;
        }
    });

    // Frustum culling verification:
    // 1,000 entities are in the scene, but only visible rows (~8) + overscan buffer (~2-4)
    // should be present in the tree. Under no circumstances should 1,000 nodes be generated!
    assert!(
        rendered_rows_count <= 16,
        "UI Frustum Culling failure: expected <= 16 rendered rows for 200px viewport, got {rendered_rows_count}"
    );
    assert!(
        rendered_rows_count >= 5,
        "Expected at least visible rows to be rendered, got {rendered_rows_count}"
    );
}

#[test]
fn test_hierarchy_tree_connector_lines_and_foldout() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

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

    let mut collapsed = HashSet::new();
    let mut rows_cache = Vec::new();

    // 1. Expanded by default
    super::rows::sync_hierarchy_rows(&world, &collapsed, &mut rows_cache);
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
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &collapsed,
        hovered_tag: None,
    };

    let _ = super::panel::build_hierarchy_panel(&mut tree, root_id, &params, &mut rows_cache);

    // Verify foldout chevron widget exists and eye buttons maintain strictly identical X alignment across all rows
    let mut found_foldout = false;
    let mut eye_x_coords = Vec::new();
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("HierarchyFoldoutButton") {
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

    // 3. Collapse parent
    collapsed.insert(parent_ent);
    rows_cache.clear();
    super::rows::sync_hierarchy_rows(&world, &collapsed, &mut rows_cache);
    assert_eq!(rows_cache.len(), 1, "Collapsed parent hides its children");
    assert!(!rows_cache[0].is_expanded);

    // 4. Re-expand parent
    collapsed.remove(&parent_ent);
    rows_cache.clear();
    super::rows::sync_hierarchy_rows(&world, &collapsed, &mut rows_cache);
    assert_eq!(rows_cache.len(), 3, "Expanded parent restores its children");
    assert!(rows_cache[0].is_expanded);
}

#[test]
fn test_hierarchy_header_and_row_hover_styling() {
    let mut world = hecs::World::new();
    let _ent = world.spawn((ae_core::ecs::Name("TestEntity".to_string()),));

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");

    // 1. Test Add Button hover
    let params_add_hover = HierarchyPanelParams {
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
        cursor_pos: Point::new(10.0, 10.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &std::collections::HashSet::new(),
        hovered_tag: Some(HIERARCHY_TAG_ADD_BUTTON),
    };

    let mut rows_cache = Vec::new();
    super::panel::build_hierarchy_panel(&mut tree, root, &params_add_hover, &mut rows_cache);

    let (_, add_node) = tree
        .iter()
        .find(|(_, n)| n.tag == HIERARCHY_TAG_ADD_BUTTON)
        .expect("Add button must exist");

    assert_eq!(
        add_node.style.background_color,
        Color::rgba(0.30, 0.35, 0.45, 0.98),
        "Hovered Add button must have brightened hover background"
    );
    assert_eq!(
        add_node.style.border.color,
        Color::rgba(0.0, 0.85, 1.0, 0.85),
        "Hovered Add button must have cyan border highlight"
    );

    // 2. Test Entity Row hover
    let mut tree2 = UiTree::new();
    let root2 = tree2
        .create_root()
        .expect("Root node creation must succeed");
    let row_tag = make_row_tag(0);

    let params_row_hover = HierarchyPanelParams {
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
        cursor_pos: Point::new(10.0, 10.0),
        is_search_focused: false,
        blink_caret: false,
        collapsed_entities: &std::collections::HashSet::new(),
        hovered_tag: Some(row_tag),
    };

    let mut rows_cache2 = Vec::new();
    super::panel::build_hierarchy_panel(&mut tree2, root2, &params_row_hover, &mut rows_cache2);

    let (_, row_node) = tree2
        .iter()
        .find(|(_, n)| n.tag == row_tag)
        .expect("Entity row must exist");

    assert_eq!(
        row_node.style.background_color,
        Color::rgba(0.14, 0.18, 0.26, 0.65),
        "Hovered non-selected entity row must have subtle hover highlight background"
    );
    assert_eq!(
        row_node.style.border.color,
        Color::rgba(0.24, 0.32, 0.45, 0.50),
        "Hovered non-selected entity row must have subtle hover border"
    );

    // 3. Verify hit_test_target resolves semantic tags even when hit on child icons/labels
    let add_center = Point::new(
        add_node.computed_rect.x + add_node.computed_rect.width * 0.5,
        add_node.computed_rect.y + add_node.computed_rect.height * 0.5,
    );
    let hit_add = tree
        .hit_test_target(add_center)
        .expect("Add button must be hit");
    assert_eq!(hit_add.tag, HIERARCHY_TAG_ADD_BUTTON);

    let row_center = Point::new(
        row_node.computed_rect.x + row_node.computed_rect.width * 0.5,
        row_node.computed_rect.y + row_node.computed_rect.height * 0.5,
    );
    let hit_row = tree2.hit_test_target(row_center).expect("Row must be hit");
    assert_eq!(hit_row.tag, row_tag);
}
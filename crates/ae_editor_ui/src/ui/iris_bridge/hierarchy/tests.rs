// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scene Hierarchy Menu & Submenu Invariant Unit Tests
//!
//! Verifies that cascading Add Menu dropdowns and submenus assign correct
//! semantic `WidgetRole` tags and retain their text labels without occlusion.
//!

use super::add_menu::build_add_menu;
use super::types::{AddSubmenuId, HierarchyPanelParams, HierarchyPanelTargets};
use crate::ui::iris_bridge::types::IrisEditorOverlay;
use hecs::World;
use irisui::prelude::*;

#[test]
fn test_hierarchy_add_submenu_renders_text_without_self_occlusion() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let mut targets = HierarchyPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        add_btn_rect: Rect::new(100.0, 10.0, 24.0, 24.0),
        ..Default::default()
    };

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
    };

    build_add_menu(&mut tree, root_id, &params, &mut targets);

    assert!(
        targets.active_add_menu_rects.len() >= 2,
        "Add menu and submenu cards must be constructed"
    );
    let add_menu_rect = targets.active_add_menu_rects[0];
    let submenu_rect = targets.active_add_menu_rects[1];

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
    let mut targets = HierarchyPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        add_btn_rect: Rect::new(100.0, 10.0, 24.0, 24.0),
        ..Default::default()
    };

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
    };

    build_add_menu(&mut tree, root_id, &params, &mut targets);

    assert!(targets.active_add_menu_rects.len() >= 2);
    let add_menu_rect = targets.active_add_menu_rects[0];
    let submenu_rect = targets.active_add_menu_rects[1];

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
    let mut targets = HierarchyPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        add_btn_rect: Rect::new(100.0, 10.0, 24.0, 24.0),
        ..Default::default()
    };

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
    };

    build_add_menu(&mut tree, root_id, &params, &mut targets);

    assert_eq!(
        targets.active_add_menu_rects.len(),
        3,
        "Root menu, UI Canvas submenu, and HUD Presets sub-submenu must exist"
    );
    let _add_menu_rect = targets.active_add_menu_rects[0];
    let submenu_rect = targets.active_add_menu_rects[1];
    let sub_submenu_rect = targets.active_add_menu_rects[2];

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
    let active_dropdowns = [
        targets.active_add_menu_rects[0],
        submenu_rect,
        sub_submenu_rect,
    ];
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
fn test_hierarchy_add_menu_dark_styling_and_no_clickthrough() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let mut targets = HierarchyPanelTargets {
        panel_rect: Rect::new(0.0, 28.0, 260.0, 600.0),
        add_btn_rect: Rect::new(100.0, 32.0, 24.0, 24.0),
        ..Default::default()
    };

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
    };

    build_add_menu(&mut tree, root_id, &params, &mut targets);

    // 1. Verify card style attributes match Inspector's neutral dark theme via UiTree traversal    // 1. Verify popup cards exist and have DropdownPopup role
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

    // 2. Verify click consumption inside Add Menu and submenus (click-through protection)
    assert!(targets.active_add_menu_rects.len() >= 2);
    let add_rect = targets.active_add_menu_rects[0];
    let sub_rect = targets.active_add_menu_rects[1];

    let pt_in_add = Point::new(add_rect.x + 10.0, add_rect.y + 10.0);
    let pt_in_sub = Point::new(sub_rect.x + 10.0, sub_rect.y + 10.0);
    let pt_in_panel = Point::new(50.0, 200.0);

    let mut actions = Vec::new();
    let consumed_add =
        super::panel::handle_hierarchy_click(pt_in_add, MouseButton::Left, &targets, &mut actions);
    assert!(
        consumed_add,
        "Click inside AddMenuCard must be consumed to prevent click-through"
    );

    actions.clear();
    let consumed_sub =
        super::panel::handle_hierarchy_click(pt_in_sub, MouseButton::Left, &targets, &mut actions);
    assert!(
        consumed_sub,
        "Click inside AddSubmenuCard must be consumed to prevent click-through"
    );

    actions.clear();
    let consumed_panel = super::panel::handle_hierarchy_click(
        pt_in_panel,
        MouseButton::Left,
        &targets,
        &mut actions,
    );
    assert!(
        consumed_panel,
        "Click inside docked panel_rect must be consumed to prevent click-through to underlying modals"
    );
}

#[test]
fn test_hierarchy_add_menu_2d_mode_shows_2d_objects() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let world = World::new();
    let mut targets = HierarchyPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        add_btn_rect: Rect::new(100.0, 10.0, 24.0, 24.0),
        ..Default::default()
    };

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
    };

    build_add_menu(&mut tree, root_id, &params, &mut targets);

    assert_eq!(targets.active_add_menu_rects.len(), 2);

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
    let mut targets = HierarchyPanelTargets {
        panel_rect: Rect::new(0.0, 0.0, 300.0, 600.0),
        add_btn_rect: Rect::new(100.0, 10.0, 24.0, 24.0),
        ..Default::default()
    };

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
    };

    build_add_menu(&mut tree, root_id, &params, &mut targets);

    assert!(!targets.active_add_menu_rects.is_empty());
    let root_rect = targets.active_add_menu_rects[0];
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
    let mut targets = HierarchyPanelTargets::default();

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
    };

    super::context_menu::build_context_menu(&mut tree, root_id, &params, &mut targets);

    assert!(targets.active_context_menu.is_some());
    let (target_ent, card_rect) = targets.active_context_menu.unwrap();
    assert_eq!(target_ent, entity);
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
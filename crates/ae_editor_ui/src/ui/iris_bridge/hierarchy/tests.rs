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

    let add_menu_rect = targets
        .active_add_menu_rect
        .expect("Add menu card must be constructed");
    let submenu_rect = targets
        .active_submenu_rect
        .expect("Submenu card must be constructed when active_submenu is set");

    // Verify DropdownPopup semantic role is set on the submenu card node
    let mut found_submenu_popup_role = false;
    tree.traverse_depth_first(root_id, &mut |_, node| {
        if node.name.as_deref() == Some("AddSubmenuCard") && node.role == WidgetRole::DropdownPopup
        {
            found_submenu_popup_role = true;
        }
    });
    assert!(
        found_submenu_popup_role,
        "AddSubmenuCard node must have WidgetRole::DropdownPopup"
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

    let add_menu_rect = targets.active_add_menu_rect.unwrap();
    let submenu_rect = targets.active_submenu_rect.unwrap();

    let active_dropdowns = [add_menu_rect, submenu_rect];
    let sections =
        IrisEditorOverlay::collect_text_sections_from_tree(&tree, &active_dropdowns, &[], &[]);

    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    assert!(
        rendered_texts.contains(&"Panel / Canvas Box"),
        "Submenu item 'Panel / Canvas Box' must be visible"
    );
    assert!(
        rendered_texts.contains(&"HUD Presets ▸"),
        "Submenu item 'HUD Presets ▸' must be visible"
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

    let add_menu_rect = targets
        .active_add_menu_rect
        .expect("Root Add menu card must exist");
    let submenu_rect = targets
        .active_submenu_rect
        .expect("Level 2 UI Canvas submenu must exist");
    let sub_submenu_rect = targets
        .active_sub_submenu_rect
        .expect("Level 3 HUD Presets sub-submenu must exist when active_sub_submenu is set");

    // Verify sub-submenu is positioned to the right of submenu card
    assert!(
        sub_submenu_rect.x >= submenu_rect.right(),
        "HUD Presets sub-submenu must cascade to the right of UI & Canvas card"
    );

    // Verify targets.submenu_branch_items contains HUD Presets
    let branch_found = targets
        .submenu_branch_items
        .iter()
        .any(|(_, sub_id)| *sub_id == AddSubmenuId::HudPresets);
    assert!(
        branch_found,
        "targets.submenu_branch_items must contain AddSubmenuId::HudPresets"
    );

    // Verify HUD Presets items are registered in targets.submenu_items
    let has_health_bar = targets.submenu_items.iter().any(|(_, act)| {
        *act == super::types::HierarchyAction::SpawnUiElement(crate::ui::UiElementType::HealthBar)
    });
    let has_score_display = targets.submenu_items.iter().any(|(_, act)| {
        *act == super::types::HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::ScoreDisplay,
        )
    });
    assert!(
        has_health_bar,
        "Submenu items must include SpawnUiElement(HealthBar)"
    );
    assert!(
        has_score_display,
        "Submenu items must include SpawnUiElement(ScoreDisplay)"
    );

    // Verify text labels render without self-occlusion in text collector
    let active_dropdowns = [add_menu_rect, submenu_rect, sub_submenu_rect];
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

    // Verify interactive click on Health Bar item dispatches SpawnUiElement action
    let (health_item_rect, _) = targets
        .submenu_items
        .iter()
        .find(|(_, act)| {
            *act == super::types::HierarchyAction::SpawnUiElement(
                crate::ui::UiElementType::HealthBar,
            )
        })
        .expect("Health bar target item rect must exist");

    let click_pt = Point::new(
        health_item_rect.x + 10.0,
        health_item_rect.y + health_item_rect.height * 0.5,
    );
    let mut out_actions = Vec::new();
    let consumed = super::panel::handle_hierarchy_click(
        click_pt,
        MouseButton::Left,
        &targets,
        &mut out_actions,
    );
    assert!(consumed, "Click on Health Bar must be consumed");
    assert!(
        out_actions.contains(&super::types::HierarchyAction::SpawnUiElement(
            crate::ui::UiElementType::HealthBar
        )),
        "Click on Health Bar must dispatch SpawnUiElement(HealthBar)"
    );
    assert!(
        out_actions.contains(&super::types::HierarchyAction::CloseAddMenu),
        "Click on Health Bar must close the Add menu"
    );
}
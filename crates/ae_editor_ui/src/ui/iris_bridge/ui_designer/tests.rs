// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Visual UI Designer panel unit tests.
//!

use super::types::{
    UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN, UI_DESIGNER_TAG_ASPECT_BTN,
    UI_DESIGNER_TAG_GRID_BTN, UI_DESIGNER_TAG_SNAP_BTN, UI_DESIGNER_TAG_TOOLBAR,
    UI_DESIGNER_TAG_ZOOM_IN, UI_DESIGNER_TAG_ZOOM_OUT, UI_DESIGNER_TAG_ZOOM_RESET,
    encode_element_tag, is_ui_designer_tag,
};
use super::*;
use ae_core::ecs::{UiAnchor, UiButton, UiElement};
use irisui::prelude::*;

#[test]
fn test_ui_designer_tag_domain_check() {
    assert!(is_ui_designer_tag(UI_DESIGNER_TAG_TOOLBAR));
    assert!(is_ui_designer_tag(UI_DESIGNER_TAG_ASPECT_BTN));
    assert!(is_ui_designer_tag(encode_element_tag(0)));
    assert!(is_ui_designer_tag(encode_element_tag(999)));

    // Non-UI Designer tags must return false
    assert!(!is_ui_designer_tag(0));
    assert!(!is_ui_designer_tag(0x0010_0000_0000_0000));
    assert!(!is_ui_designer_tag(0x0050_0000_0000_0001));
}

#[test]
fn test_ui_designer_panel_build() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let world = hecs::World::new();
    let state = UiDesignerState::default();
    let panel_rect = Rect::new(50.0, 50.0, 800.0, 600.0);

    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        cursor_pos: Point::new(0.0, 0.0),
        state: &state,
        is_aspect_dropdown_open: false,
        is_add_menu_open: false,
        hovered_tag: None,
    };

    let mut contexts = Vec::new();
    let metrics = build_ui_designer_panel(&mut tree, root, &params, &mut contexts);

    assert_eq!(metrics.panel_rect, panel_rect);
    assert!(metrics.canvas_rect.width > 0.0);
    assert!(metrics.canvas_rect.height > 0.0);
    assert!(metrics.base_scale > 0.0);
    assert_eq!(contexts.len(), 0);

    // Verify layout was properly finalized across the entire subtree
    let root_node = tree.get(root).expect("Root node must exist");
    assert!(!root_node.children.is_empty(), "Root must have children");

    let panel_root_id = root_node.children[0];
    let panel_root = tree.get(panel_root_id).expect("Panel root must exist");
    assert_eq!(panel_root.computed_rect, panel_rect);

    // Check letterbox and toolbar non-zero geometry
    let mut found_toolbar = false;
    let mut found_letterbox = false;
    for &cid in &panel_root.children {
        if let Some(child) = tree.get(cid) {
            if child.name.as_deref() == Some("UiDesignerToolbar") {
                found_toolbar = true;
                assert_eq!(child.computed_rect.height, 34.0);
                assert_eq!(child.computed_rect.width, panel_rect.width);
            }
            if child.name.as_deref() == Some("UiDesignerLetterbox") {
                found_letterbox = true;
                assert!(child.computed_rect.width > 0.0);
                assert!(child.computed_rect.height > 0.0);
            }
        }
    }
    assert!(
        found_toolbar,
        "UiDesignerToolbar must exist in panel hierarchy"
    );
    assert!(
        found_letterbox,
        "UiDesignerLetterbox must exist in panel hierarchy"
    );
}

#[test]
fn test_ui_designer_canvas_projection() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let mut world = hecs::World::new();

    let ent = world.spawn((
        UiElement {
            anchor: UiAnchor::Center,
            offset: [12.0, 34.0],
            size: [200.0, 50.0],
            pivot: [0.5, 0.5],
            visible: true,
            z_index: 0,
            alpha: 1.0,
        },
        UiButton {
            text: "Start Game".to_string(),
            ..Default::default()
        },
    ));

    let state = UiDesignerState::default();
    let panel_rect = Rect::new(0.0, 0.0, 1024.0, 768.0);

    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: Some(ent),
        cursor_pos: Point::new(512.0, 384.0),
        state: &state,
        is_aspect_dropdown_open: false,
        is_add_menu_open: false,
        hovered_tag: None,
    };

    let mut contexts = Vec::new();
    let metrics = build_ui_designer_panel(&mut tree, root, &params, &mut contexts);

    assert_eq!(contexts.len(), 1);
    let found = &contexts[0];
    assert_eq!(found.entity, ent);
    assert_eq!(found.initial_offset, [12.0, 34.0]);
    assert!(metrics.canvas_rect.width > 0.0);
}

#[test]
fn test_ui_designer_click_hit_testing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let mut world = hecs::World::new();
    let ent = world.spawn((UiElement {
        anchor: UiAnchor::TopLeft,
        offset: [100.0, 100.0],
        size: [150.0, 40.0],
        pivot: [0.0, 0.0],
        visible: true,
        z_index: 0,
        alpha: 1.0,
    },));

    let state = UiDesignerState::default();
    let panel_rect = Rect::new(0.0, 0.0, 1000.0, 800.0);

    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        cursor_pos: Point::new(0.0, 0.0),
        state: &state,
        is_aspect_dropdown_open: false,
        is_add_menu_open: false,
        hovered_tag: None,
    };

    let mut contexts = Vec::new();
    let metrics = build_ui_designer_panel(&mut tree, root, &params, &mut contexts);
    assert_eq!(contexts.len(), 1);

    // Click on aspect ratio button via tag
    let aspect_hit = HitTargetInfo {
        id: root,
        name: Some("UiAspectBtn".to_string()),
        layer: UiLayer::Content,
        role: WidgetRole::Button,
        cursor: Some(WidgetCursor::Pointer),
        tag: UI_DESIGNER_TAG_ASPECT_BTN,
        rect: Rect::new(10.0, 10.0, 150.0, 26.0),
    };
    let click_res = handle_ui_designer_click(
        Point::new(20.0, 20.0),
        Some(&aspect_hit),
        &metrics,
        &contexts,
        false,
        false,
    );
    assert_eq!(
        click_res.action,
        Some(UiDesignerAction::ToggleAspectDropdown)
    );

    // Click on canvas element via element tag
    let elem_tag = encode_element_tag(0);
    let (_, elem_node) = tree
        .iter()
        .find(|(_, n)| n.tag == elem_tag)
        .expect("UiCanvasElement must exist in tree");
    let r = elem_node.computed_rect;
    let elem_center = Point::new(r.x + r.width * 0.5, r.y + r.height * 0.5);
    let real_hit = tree.hit_test_target(elem_center);
    assert!(
        real_hit.is_some(),
        "hit_test_target at {elem_center:?} returned None! computed_rect was {:?}",
        elem_node.computed_rect
    );
    let real_hit_target = real_hit.unwrap();
    assert_eq!(
        real_hit_target.tag, elem_tag,
        "Expected hit_target to be UiCanvasElement tag, got {:#x} ({:?})",
        real_hit_target.tag, real_hit_target.name
    );
    let elem_click = handle_ui_designer_click(
        elem_center,
        Some(&real_hit_target),
        &metrics,
        &contexts,
        false,
        false,
    );
    assert_eq!(
        elem_click.action,
        Some(UiDesignerAction::SelectEntity(Some(ent)))
    );
    assert!(elem_click.start_element_drag.is_some());

    // Test aspect ratio selection via popup HitTargetInfo
    let aspect_popup_hit = HitTargetInfo {
        id: root,
        name: Some("AspectOptionItem".to_string()),
        layer: UiLayer::Popup,
        role: WidgetRole::DropdownItem,
        cursor: None,
        tag: super::types::make_aspect_item_tag(1), // Ratio16x10
        rect: Rect::new(100.0, 100.0, 150.0, 24.0),
    };
    let aspect_popup_click = handle_ui_designer_click(
        Point::new(120.0, 110.0),
        Some(&aspect_popup_hit),
        &metrics,
        &contexts,
        true,
        false,
    );
    assert_eq!(
        aspect_popup_click.action,
        Some(UiDesignerAction::SetAspectRatio(
            CanvasAspectRatio::Ratio16x10
        ))
    );

    // Test add element selection via popup HitTargetInfo
    let add_hit = HitTargetInfo {
        id: root,
        name: Some("AddElementOption".to_string()),
        layer: UiLayer::Popup,
        role: WidgetRole::DropdownItem,
        cursor: None,
        tag: super::types::make_add_item_tag(3), // Button
        rect: Rect::new(100.0, 100.0, 150.0, 24.0),
    };
    let add_click = handle_ui_designer_click(
        Point::new(120.0, 110.0),
        Some(&add_hit),
        &metrics,
        &contexts,
        false,
        true,
    );
    assert_eq!(
        add_click.action,
        Some(UiDesignerAction::SpawnElement(UiElementType::Button))
    );
}

#[test]
fn test_ui_designer_element_drag() {
    let metrics = UiDesignerCanvasMetrics {
        panel_rect: Rect::new(0.0, 0.0, 800.0, 600.0),
        canvas_rect: Rect::new(100.0, 100.0, 600.0, 400.0),
        resolution: [1920.0, 1080.0],
        snap_grid: Some(16.0),
        current_zoom: 1.0,
        base_scale: 1.0,
    };

    let mut world = hecs::World::new();
    let ent = world.spawn(());

    let drag_state = UiDragState {
        entity: ent,
        anchor_origin: [0.0, 0.0],
        drag_start_mouse_canvas: [500.0, 500.0],
        initial_offset: [100.0, 100.0],
    };

    // Move cursor 100px to the right in canvas space
    let cursor_x = metrics.canvas_rect.x + (600.0 / 1920.0) * metrics.canvas_rect.width;
    let cursor_y = metrics.canvas_rect.y + (500.0 / 1080.0) * metrics.canvas_rect.height;

    let action = handle_ui_designer_drag(
        Point::new(cursor_x, cursor_y),
        [0.0, 0.0],
        Some(&drag_state),
        false,
        &metrics,
    );

    match action {
        Some(UiDesignerAction::UpdateElementOffset { entity, offset }) => {
            assert_eq!(entity, ent);
            assert!((offset[0] - 200.0).abs() < 17.0);
        }
        _ => panic!("Expected UpdateElementOffset action"),
    }
}

#[test]
fn test_ui_designer_toolbar_tags() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let world = hecs::World::new();
    let state = UiDesignerState::default();
    let panel_rect = Rect::new(0.0, 0.0, 1200.0, 800.0);

    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        cursor_pos: Point::new(0.0, 0.0),
        state: &state,
        is_aspect_dropdown_open: false,
        is_add_menu_open: false,
        hovered_tag: None,
    };

    let mut contexts = Vec::new();
    let _metrics = build_ui_designer_panel(&mut tree, root, &params, &mut contexts);

    // Verify all toolbar controls have been created with their designated 64-bit tags
    let required_tags = [
        UI_DESIGNER_TAG_ASPECT_BTN,
        UI_DESIGNER_TAG_ZOOM_OUT,
        UI_DESIGNER_TAG_ZOOM_RESET,
        UI_DESIGNER_TAG_ZOOM_IN,
        UI_DESIGNER_TAG_SNAP_BTN,
        UI_DESIGNER_TAG_ANCHORS_BTN,
        UI_DESIGNER_TAG_GRID_BTN,
        UI_DESIGNER_TAG_ADD_ELEMENT_BTN,
    ];

    for tag in required_tags {
        let found = tree.iter().any(|(_, node)| node.tag == tag);
        assert!(found, "Expected node with tag {tag:#x} to exist in UiTree");
    }
}

#[test]
fn test_ui_designer_element_click_not_occluded_by_selection_and_popup_anchored_correctly() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let mut world = hecs::World::new();
    let ent = world.spawn((
        ae_core::ecs::UiElement {
            anchor: UiAnchor::TopLeft,
            offset: [120.0, 36.0],
            size: [180.0, 16.0],
            pivot: [0.5, 0.5],
            visible: true,
            z_index: 10,
            alpha: 1.0,
        },
        ae_core::ecs::UiProgressBar {
            min: 0.0,
            max: 100.0,
            value: 100.0,
            fill_color: [0.2, 0.85, 0.35, 1.0],
            background_color: [0.08, 0.10, 0.14, 0.85],
            border_color: [0.3, 0.4, 0.5, 0.8],
            corner_radius: 3.0,
        },
    ));

    let state = UiDesignerState {
        show_anchor_guides: true,
        show_grid: true,
        ..Default::default()
    };
    let panel_rect = Rect::new(0.0, 0.0, 1200.0, 800.0);

    // Initial build with element selected and hovered
    let mut contexts = Vec::new();
    let params_selected = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: Some(ent),
        cursor_pos: Point::new(100.0, 100.0),
        state: &state,
        is_aspect_dropdown_open: false,
        is_add_menu_open: true,
        hovered_tag: None,
    };
    let metrics = build_ui_designer_panel(&mut tree, root, &params_selected, &mut contexts);

    // 1. Verify AddElementPopup is anchored to the Add Element button's computed rect
    let add_btn = tree
        .iter()
        .find(|(_, n)| n.tag == UI_DESIGNER_TAG_ADD_ELEMENT_BTN)
        .expect("Add element button must exist");
    let add_btn_rect = add_btn.1.computed_rect;

    let popup_node = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("AddElementPopup"))
        .expect("AddElementPopup must exist when open");
    let popup_rect = popup_node.1.computed_rect;

    // Must be positioned at left edge of button and just beneath it (not shifted way to panel right)
    assert_eq!(popup_rect.x, add_btn_rect.x);
    assert!((popup_rect.y - (add_btn_rect.y + add_btn_rect.height + 2.0)).abs() < 1.0);

    // 2. Verify hit-testing on the element's center succeeds and is not occluded by outlines, handles or guides
    let elem_tag = encode_element_tag(0);
    let (_, elem_node) = tree
        .iter()
        .find(|(_, n)| n.tag == elem_tag)
        .expect("UiCanvasElement must exist in tree");
    let r = elem_node.computed_rect;
    let elem_center = Point::new(r.x + r.width * 0.5, r.y + r.height * 0.5);

    let hit = tree.hit_test_target(elem_center);
    assert!(hit.is_some(), "Element center must register a hit");
    let hit_target = hit.unwrap();
    assert_eq!(
        hit_target.tag, elem_tag,
        "Selection outline or guide must not occlude UiCanvasElement hit target"
    );

    // 3. Verify handle_ui_designer_click prioritizes the element and initiates drag
    let click_res = handle_ui_designer_click(
        elem_center,
        Some(&hit_target),
        &metrics,
        &contexts,
        false,
        false,
    );
    assert_eq!(
        click_res.action,
        Some(UiDesignerAction::SelectEntity(Some(ent)))
    );
    assert!(click_res.start_element_drag.is_some());
    assert!(!click_res.start_canvas_pan);
}

#[test]
fn test_ui_designer_popup_items_hover_and_click_routing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let world = hecs::World::new();
    let state = UiDesignerState::default();

    let target_tag = super::types::make_add_item_tag(0); // First element type: Panel
    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        cursor_pos: Point::new(100.0, 100.0),
        state: &state,
        is_aspect_dropdown_open: false,
        is_add_menu_open: true,
        hovered_tag: Some(target_tag),
    };

    let mut contexts = Vec::new();
    let _metrics = build_ui_designer_panel(&mut tree, root, &params, &mut contexts);

    // Verify target dropdown item exists and has active hover styling
    let (_, item_node) = tree
        .iter()
        .find(|(_, n)| n.tag == target_tag)
        .expect("Dropdown item with target tag must exist");

    assert_eq!(
        item_node.style.background_color,
        Color::hex("#222634"),
        "Hovered dropdown item must have obsidian hover background"
    );

    // Verify click on hovered dropdown item produces SpawnElement action
    let hit_info = HitTargetInfo {
        id: WidgetId::default(),
        layer: UiLayer::Popup,
        role: WidgetRole::DropdownItem,
        cursor: None,
        tag: target_tag,
        rect: item_node.computed_rect,
        name: Some("DropdownItem".to_string()),
    };

    let click_res = handle_ui_designer_click(
        Point::new(10.0, 10.0),
        Some(&hit_info),
        &_metrics,
        &contexts,
        false,
        true,
    );

    assert_eq!(
        click_res.action,
        Some(UiDesignerAction::SpawnElement(
            super::types::UiElementType::Panel
        ))
    );
}
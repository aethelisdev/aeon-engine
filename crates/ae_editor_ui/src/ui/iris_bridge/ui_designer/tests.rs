// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Visual UI Designer panel unit tests.

use super::types::{
    UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN, UI_DESIGNER_TAG_ASPECT_BTN,
    UI_DESIGNER_TAG_GRID_BTN, UI_DESIGNER_TAG_SNAP_BTN, UI_DESIGNER_TAG_ZOOM_IN,
    UI_DESIGNER_TAG_ZOOM_OUT, UI_DESIGNER_TAG_ZOOM_RESET, encode_element_tag,
};
use super::*;
use ae_core::ecs::{UiAnchor, UiButton, UiElement};
use irisui::prelude::*;

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
    };

    let mut designer = UiDesignerPanelState::default();
    let metrics =
        build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);

    assert_eq!(metrics.panel_rect, panel_rect);
    assert!(metrics.canvas_rect.width > 0.0);
    assert!(metrics.canvas_rect.height > 0.0);
    assert!(metrics.base_scale > 0.0);
    assert_eq!(designer.drag_contexts.len(), 0);

    // Verify layout was properly finalized across the entire subtree
    let root_node = tree.get(root).expect("Root node must exist");
    assert!(!root_node.children.is_empty(), "Root must have children");

    let panel_root_id = root_node.children[0];
    let panel_root = tree.get(panel_root_id).expect("Panel root must exist");
    assert_eq!(panel_root.computed_rect, panel_rect);

    // Check letterbox and toolbar non-zero geometry
    assert_eq!(
        tree.node_named_in_subtree(panel_root_id, "UiDesignerToolbar")
            .expect("UiDesignerToolbar must exist in panel hierarchy")
            .computed_rect
            .height,
        34.0
    );
    assert_eq!(
        tree.node_named_in_subtree(panel_root_id, "UiDesignerToolbar")
            .expect("UiDesignerToolbar must exist in panel hierarchy")
            .computed_rect
            .width,
        panel_rect.width
    );
    assert!(
        tree.node_named_in_subtree(panel_root_id, "UiDesignerLetterbox")
            .expect("UiDesignerLetterbox must exist in panel hierarchy")
            .computed_rect
            .width
            > 0.0
    );
    assert!(
        tree.node_named_in_subtree(panel_root_id, "UiDesignerLetterbox")
            .expect("UiDesignerLetterbox must exist in panel hierarchy")
            .computed_rect
            .height
            > 0.0
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
    };

    let mut designer = UiDesignerPanelState::default();
    let metrics =
        build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);

    assert_eq!(designer.drag_contexts.len(), 1);
    let found = &designer.drag_contexts[0];
    assert_eq!(found.entity, ent);
    assert_eq!(found.initial_offset, [12.0, 34.0]);
    assert!(metrics.canvas_rect.width > 0.0);
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
    };

    let mut designer = UiDesignerPanelState::default();
    let _metrics =
        build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);

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

    // Initial build with element selected and Add Element popup open via scoped tree tag
    tree.open_popup(UI_DESIGNER_TAG_ADD_ELEMENT_BTN);
    let params_selected = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: Some(ent),
        cursor_pos: Point::new(100.0, 100.0),
        state: &state,
    };
    let mut designer = UiDesignerPanelState::default();
    let metrics = build_ui_designer_panel(
        &mut UiScope::new(&mut tree, root),
        &params_selected,
        &mut designer,
    );

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
    let _ = (metrics, ent);
}

#[test]
fn test_ui_designer_popup_items_hover_styling() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");

    let panel_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let world = hecs::World::new();
    let state = UiDesignerState::default();

    let target_tag = super::types::make_add_item_tag(0); // First element type: Panel
    tree.open_popup(UI_DESIGNER_TAG_ADD_ELEMENT_BTN);
    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        cursor_pos: Point::new(100.0, 100.0),
        state: &state,
    };

    let mut designer = UiDesignerPanelState::default();
    let _metrics =
        build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);

    // Verify target dropdown item exists and has active hover styling
    let (_, item_node) = tree
        .iter()
        .find(|(_, n)| n.tag == target_tag)
        .expect("Dropdown item with target tag must exist");

    assert_eq!(
        item_node.style.hover_background,
        Some(tree.theme().button_hover_bg),
        "Dropdown item must have theme button_hover_bg"
    );
}

#[test]
fn test_ui_designer_declarative_toolbar_dispatch() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let world = hecs::World::new();
    let state = UiDesignerState::default();
    let panel_rect = Rect::new(0.0, 0.0, 800.0, 600.0);

    let params = UiDesignerPanelParams {
        panel_rect,
        world: &world,
        selected_entity: None,
        cursor_pos: Point::new(0.0, 0.0),
        state: &state,
    };

    let mut designer = UiDesignerPanelState::default();

    // 1. Initial build: toolbar rendered
    build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);

    // 2. Click aspect button -> toggles exclusive popup tag in UiTree
    tree.event_state.frame_clicked_tag = Some(UI_DESIGNER_TAG_ASPECT_BTN);
    let _ = tree.clear_children(root);
    build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);
    assert!(tree.is_popup_tag_active(UI_DESIGNER_TAG_ASPECT_BTN));
    assert!(!tree.is_popup_tag_active(UI_DESIGNER_TAG_ADD_ELEMENT_BTN));

    // 3. Click Add Element button -> exclusively switches active popup tag
    tree.event_state.clear_frame_clicks();
    tree.event_state.frame_clicked_tag = Some(UI_DESIGNER_TAG_ADD_ELEMENT_BTN);
    let _ = tree.clear_children(root);
    build_ui_designer_panel(&mut UiScope::new(&mut tree, root), &params, &mut designer);
    assert!(!tree.is_popup_tag_active(UI_DESIGNER_TAG_ASPECT_BTN));
    assert!(tree.is_popup_tag_active(UI_DESIGNER_TAG_ADD_ELEMENT_BTN));

    // 4. Outside click dismisses popup autonomously
    assert!(tree.dismiss_popups_on_outside_click(Point::new(750.0, 550.0)));
    assert!(!tree.is_popup_tag_active(UI_DESIGNER_TAG_ADD_ELEMENT_BTN));

    designer.dispatch(UiDesignerAction::CycleGridSnap);
    let actions = designer.take_actions();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], UiDesignerAction::CycleGridSnap);
}
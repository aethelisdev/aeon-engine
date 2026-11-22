// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 2D Visual UI Designer panel unit tests.

use super::types::{
    UI_DESIGNER_TAG_ADD_ELEMENT_BTN, UI_DESIGNER_TAG_ANCHORS_BTN, UI_DESIGNER_TAG_ASPECT_BTN,
    UI_DESIGNER_TAG_GRID_BTN, UI_DESIGNER_TAG_SNAP_BTN, UI_DESIGNER_TAG_ZOOM_IN,
    UI_DESIGNER_TAG_ZOOM_OUT, UI_DESIGNER_TAG_ZOOM_RESET, encode_element_tag, make_add_item_tag,
};
use super::*;
use crate::ui::panel_layout::PanelId;
use crate::ui::types::EngineUiAction;
use ae_core::ecs::{UiAnchor, UiButton, UiElement};
use irisui::dock::DeferredNotifier;
use irisui::prelude::*;
use irisui::workbench::ActionBus;

#[test]
fn test_ui_designer_panel_build() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let world = hecs::World::new();
    let mut state = UiDesignerState::default();
    let panel_rect = Rect::new(50.0, 50.0, 800.0, 600.0);

    let mut designer = UiDesignerPanelState::default();
    let mut bus = ActionBus::new();
    let mut notifier = DeferredNotifier::new();

    let transform = {
        let mut scope = UiScope::new(&mut tree, root);
        let t = scope
            .panel("UiDesignerHost")
            .bounds(panel_rect)
            .show(|p| {
                build_ui_designer_panel(
                    p,
                    &world,
                    None,
                    &mut state,
                    &mut designer,
                    &mut bus,
                    &mut notifier,
                )
            })
            .inner;
        scope.finish_layout(panel_rect);
        t
    };

    assert!(transform.canvas_rect.width > 0.0);
    assert!(transform.canvas_rect.height > 0.0);
    assert!(transform.base_scale > 0.0);

    let toolbar_id = tree
        .find_node_by_name("UiDesignerToolbar")
        .expect("UiDesignerToolbar must exist");
    let toolbar = tree.get(toolbar_id).expect("toolbar node");
    assert_eq!(toolbar.computed_rect.height, 34.0);
    assert_eq!(toolbar.computed_rect.width, panel_rect.width);

    let letterbox_id = tree
        .find_node_by_name("UiDesignerLetterbox")
        .expect("UiDesignerLetterbox must exist");
    let letterbox = tree.get(letterbox_id).expect("letterbox node");
    assert!(letterbox.computed_rect.width > 0.0);
    assert!(letterbox.computed_rect.height > 0.0);
}

#[test]
fn test_ui_designer_canvas_projection_and_click_selection() {
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

    let mut state = UiDesignerState::default();
    let panel_rect = Rect::new(0.0, 0.0, 1024.0, 768.0);
    let mut designer = UiDesignerPanelState::default();
    let mut bus = ActionBus::new();
    let mut notifier = DeferredNotifier::new();

    let elem_tag = encode_element_tag(ent);
    tree.event_state.frame_clicked_tag = Some(elem_tag);

    let transform = {
        let mut scope = UiScope::new(&mut tree, root);
        let t = scope
            .panel("UiDesignerHost")
            .bounds(panel_rect)
            .show(|p| {
                build_ui_designer_panel(
                    p,
                    &world,
                    Some(ent),
                    &mut state,
                    &mut designer,
                    &mut bus,
                    &mut notifier,
                )
            })
            .inner;
        scope.finish_layout(panel_rect);
        t
    };

    assert!(transform.canvas_rect.width > 0.0);
    let actions: Vec<EngineUiAction> = bus.drain().collect();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineUiAction::SelectEntity(Some(e)) if *e == ent)),
        "Clicking UiCanvasElement must emit EngineUiAction::SelectEntity(Some(ent))"
    );
    assert!(notifier.pending_panels().contains(&PanelId::UiDesigner));
}

#[test]
fn test_ui_designer_element_drag_emits_modify_component() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let mut world = hecs::World::new();
    let ent = world.spawn((UiElement {
        anchor: UiAnchor::TopLeft,
        offset: [240.0, 160.0],
        size: [160.0, 48.0],
        pivot: [0.5, 0.5],
        visible: true,
        z_index: 0,
        alpha: 1.0,
    },));

    let mut state = UiDesignerState {
        snap_grid: Some(8.0),
        ..Default::default()
    };
    let panel_rect = Rect::new(0.0, 0.0, 1024.0, 768.0);
    let mut designer = UiDesignerPanelState::default();
    let mut bus = ActionBus::new();
    let mut notifier = DeferredNotifier::new();
    let elem_tag = encode_element_tag(ent);

    // Frame 1: initialize layout
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("UiDesignerHost").bounds(panel_rect).show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                Some(ent),
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });
        scope.finish_layout(panel_rect);
    }

    // Simulate active pointer drag in UiTree event_state
    let elem_rect = tree.tag_rects.get(&elem_tag).copied().expect("elem rect");
    let start = Point::new(
        elem_rect.x + elem_rect.width * 0.5,
        elem_rect.y + elem_rect.height * 0.5,
    );
    let end = Point::new(start.x + 40.0, start.y + 20.0);
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: start,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseMove { point: end });

    // Frame 2: panel build processes drag displacement into EngineUiAction::ModifyComponent
    let _ = tree.clear_children(root);
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("UiDesignerHost").bounds(panel_rect).show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                Some(ent),
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });
        scope.finish_layout(panel_rect);
    }

    let actions: Vec<EngineUiAction> = bus.drain().collect();
    let modified = actions.iter().find_map(|a| match a {
        EngineUiAction::ModifyComponent(e, name, bytes) if *e == ent && *name == "UiElement" => {
            postcard::from_bytes::<UiElement>(bytes).ok()
        }
        _ => None,
    });
    let updated = modified.expect("Dragging UiCanvasElement must emit ModifyComponent(UiElement)");
    assert!(updated.offset[0] > 240.0);
    assert!(updated.offset[1] > 160.0);
}

#[test]
fn test_ui_designer_toolbar_tags_and_direct_mutations() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let world = hecs::World::new();
    let mut state = UiDesignerState::default();
    let panel_rect = Rect::new(0.0, 0.0, 1200.0, 800.0);
    let mut designer = UiDesignerPanelState::default();
    let mut bus = ActionBus::new();
    let mut notifier = DeferredNotifier::new();

    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("UiDesignerHost").bounds(panel_rect).show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                None,
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });
        scope.finish_layout(panel_rect);
    }

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
        assert!(
            tree.iter().any(|(_, node)| node.tag == tag),
            "Expected node with tag {tag:#x} to exist in UiTree"
        );
    }

    // Click Zoom In -> increments zoom directly without bridge action enum
    tree.event_state.frame_clicked_tag = Some(UI_DESIGNER_TAG_ZOOM_IN);
    let _ = tree.clear_children(root);
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("UiDesignerHost").bounds(panel_rect).show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                None,
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });
    }
    assert!((state.zoom - 1.1).abs() < 1e-4);

    // Click Grid Snap -> cycles 8px -> 16px directly
    tree.event_state.frame_clicked_tag = Some(UI_DESIGNER_TAG_SNAP_BTN);
    let _ = tree.clear_children(root);
    {
        let mut scope = UiScope::new(&mut tree, root);
        scope.panel("UiDesignerHost").bounds(panel_rect).show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                None,
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });
    }
    assert_eq!(state.snap_grid, Some(16.0));
}

#[test]
fn test_ui_designer_element_click_not_occluded_and_add_popup_anchored() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation must succeed");
    let mut world = hecs::World::new();
    let ent = world.spawn((
        UiElement {
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

    let mut state = UiDesignerState {
        show_anchor_guides: true,
        show_grid: true,
        ..Default::default()
    };
    let panel_rect = Rect::new(260.0, 82.0, 1000.0, 700.0);
    let mut designer = UiDesignerPanelState::default();
    let mut bus = ActionBus::new();
    let mut notifier = DeferredNotifier::new();

    tree.open_popup(UI_DESIGNER_TAG_ADD_ELEMENT_BTN);
    // Warmup frame populates tag_rects for popup anchoring at docked offset (260, 82)
    UiScope::new(&mut tree, root)
        .panel("UiDesignerHost")
        .scrollable(false)
        .bounds(panel_rect)
        .show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                Some(ent),
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });

    let _ = tree.clear_children(root);
    tree.event_state.frame_clicked_tag = Some(make_add_item_tag(0));
    UiScope::new(&mut tree, root)
        .panel("UiDesignerHost")
        .scrollable(false)
        .bounds(panel_rect)
        .show(|p| {
            build_ui_designer_panel(
                p,
                &world,
                Some(ent),
                &mut state,
                &mut designer,
                &mut bus,
                &mut notifier,
            );
        });

    let add_btn = tree
        .iter()
        .find(|(_, n)| n.tag == UI_DESIGNER_TAG_ADD_ELEMENT_BTN)
        .expect("Add element button must exist")
        .1
        .computed_rect;
    let popup_rect = tree
        .iter()
        .find(|(_, n)| n.name.as_deref() == Some("AddElementPopup"))
        .expect("AddElementPopup must exist when open")
        .1
        .computed_rect;

    assert_eq!(popup_rect.x, add_btn.x);
    assert!((popup_rect.y - (add_btn.y + add_btn.height + 2.0)).abs() < 1.0);

    let elem_tag = encode_element_tag(ent);
    let r = tree
        .iter()
        .find(|(_, n)| n.tag == elem_tag)
        .expect("UiCanvasElement must exist in tree")
        .1
        .computed_rect;
    let hit = tree
        .hit_test_target(Point::new(r.x + r.width * 0.5, r.y + r.height * 0.5))
        .expect("Element center must register a hit");
    assert_eq!(hit.tag, elem_tag);

    let actions: Vec<EngineUiAction> = bus.drain().collect();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineUiAction::SpawnUiElement(UiElementType::Panel))),
        "Clicking AddElementPopup item must emit EngineUiAction::SpawnUiElement(Panel)"
    );
}
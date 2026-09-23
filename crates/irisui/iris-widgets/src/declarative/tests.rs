// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Subsystem Unit Tests

use crate::declarative::{UiScope, layout_subtree};
use iris_core::{CornerRadii, InteractionEvent, MouseButton, Point, Rect, UiTree, WidgetRole};

#[test]
fn test_declarative_scope_hierarchy() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let card_id = scope.card("Transform", |card| {
        card.row(|row| {
            row.text("Position");
            row.drag_value(101, "X: ", 0.0);
            row.drag_value(102, "Y: ", 1.0);
            row.drag_value(103, "Z: ", 2.0);
        });
        let _ = card.button("Reset");
    });

    assert!(tree.get(card_id).is_some());
    let root_node = tree.get(root).expect("Root node exists");
    assert_eq!(root_node.children.len(), 1);
    assert_eq!(root_node.children[0], card_id);

    let card_node = tree.get(card_id).expect("Card node exists");
    // Card has header container and body container
    assert_eq!(card_node.children.len(), 2);
}

#[test]
fn test_declarative_layout_subtree_rects() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let card_id = scope.card("Properties", |card| {
        card.heading("Section 1");
        card.text("Label A");
        let _ = card.button("Action");
    });

    let bounds = Rect::new(10.0, 20.0, 300.0, 400.0);
    layout_subtree(&mut tree, card_id, bounds);

    let card_node = tree.get(card_id).expect("Card node exists");
    assert_eq!(card_node.computed_rect, bounds);

    // Verify all child nodes received non-zero computed rects within bounds
    tree.traverse_depth_first(card_id, &mut |_id, node| {
        assert!(node.computed_rect.width > 0.0);
        assert!(node.computed_rect.height > 0.0);
        assert!(node.computed_rect.x >= bounds.x);
        assert!(node.computed_rect.y >= bounds.y);
    });
}

#[test]
fn test_declarative_button_click_interaction() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let btn_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Save");
        resp.id
    };

    let events = [(
        btn_id,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];

    let scope = UiScope::with_interactions(&mut tree, root, &events, Some(btn_id));
    let (clicked, hovered, _) = scope.check_interaction(btn_id);

    assert!(clicked);
    assert!(hovered);
}

#[test]
fn test_declarative_checkbox_toggle() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut is_active = false;
    let checkbox_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.checkbox("Enable Shadows", &mut is_active);
        resp.id
    };

    let events = [(
        checkbox_id,
        InteractionEvent::Click {
            button: MouseButton::Left,
        },
    )];

    {
        let scope = UiScope::with_interactions(&mut tree, root, &events, None);
        let (clicked, _, _) = scope.check_interaction(checkbox_id);
        if clicked {
            is_active = !is_active;
        }
    }

    assert!(is_active);
}

#[test]
fn test_declarative_drag_float_mutation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scalar = 5.0_f32;
    let float_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drag_float("Speed", &mut scalar, 0.1, 0.0, 100.0);
        resp.id
    };

    let events = [(
        float_id,
        InteractionEvent::Drag {
            delta: Point::new(10.0, 0.0),
        },
    )];

    {
        let scope = UiScope::with_interactions(&mut tree, root, &events, None);
        let (_, _, drag_delta) = scope.check_interaction(float_id);
        if let Some(delta) = drag_delta {
            scalar += delta.x * 0.1;
        }
    }

    assert!((scalar - 6.0).abs() < 1e-4);
}

#[test]
fn test_declarative_drag_vec3_mutation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut vec = [1.0_f32, 2.0_f32, 3.0_f32];
    let vec_resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.drag_vec3("Offset", &mut vec, 0.5)
    };

    assert!(tree.get(vec_resp.id).is_some());
    assert_eq!(vec, [1.0, 2.0, 3.0]);
}

#[test]
fn test_declarative_label_customization() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let label_id = scope.label(
        "Custom Title",
        20.0,
        iris_core::Color::rgba(0.0, 0.9, 1.0, 1.0),
        iris_core::TextAlign::Center,
    );

    let node = tree.get(label_id).expect("Label node exists");
    assert_eq!(node.font_size, 20.0);
    assert_eq!(node.text_align, iris_core::TextAlign::Center);
    assert_eq!(node.text.as_deref(), Some("Custom Title"));
    assert_eq!(node.line_height, 26.0);
}

#[test]
fn test_declarative_input_box_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let mut box_id = None;
    scope.column(|col| {
        let id = col.input_box("test_folder", "Enter name...", 50.0, true);
        box_id = Some(id);
    });

    let bounds = Rect::new(10.0, 20.0, 300.0, 100.0);
    scope.finish_layout(bounds);

    let id = box_id.expect("Input box created");
    let node = tree.get(id).expect("Input box exists");
    assert_eq!(node.computed_rect.width, 300.0);
    assert_eq!(node.computed_rect.height, 28.0);
    assert_eq!(node.children.len(), 2);

    let text_node = tree.get(node.children[0]).expect("Text child exists");
    assert_eq!(text_node.text.as_deref(), Some("test_folder"));
    assert_eq!(text_node.computed_rect.width, 300.0 - 16.0);

    let caret_node = tree.get(node.children[1]).expect("Caret child exists");
    assert_eq!(caret_node.computed_rect.width, 1.5);
    assert_eq!(caret_node.computed_rect.height, 16.0);
    assert_eq!(caret_node.computed_rect.x, 10.0 + 8.0 + 50.0 + 1.0);
}

#[test]
fn test_declarative_progress_bar_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let mut bar_id = None;
    scope.column(|col| {
        let id = col.progress_bar(0.75);
        bar_id = Some(id);
    });

    let bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
    scope.finish_layout(bounds);

    let id = bar_id.expect("Progress bar created");
    let track_node = tree.get(id).expect("Track exists");
    assert_eq!(track_node.computed_rect.width, 200.0);
    assert_eq!(track_node.computed_rect.height, 4.0);
    assert_eq!(track_node.children.len(), 1);

    let slug_node = tree.get(track_node.children[0]).expect("Slug exists");
    assert_eq!(slug_node.computed_rect.x, 0.0);
    assert_eq!(slug_node.computed_rect.width, 150.0);
    assert_eq!(slug_node.computed_rect.height, 4.0);
}

#[test]
fn test_declarative_indeterminate_progress_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    let mut bar_id = None;
    scope.column(|col| {
        let id = col.indeterminate_progress(0.5);
        bar_id = Some(id);
    });

    let bounds = Rect::new(50.0, 100.0, 200.0, 100.0);
    scope.finish_layout(bounds);

    let id = bar_id.expect("Indeterminate progress created");
    let track_node = tree.get(id).expect("Track exists");
    assert_eq!(track_node.computed_rect.x, 50.0);
    assert_eq!(track_node.computed_rect.width, 200.0);
    assert_eq!(track_node.computed_rect.height, 4.0);

    let slug_node = tree.get(track_node.children[0]).expect("Slug exists");
    assert_eq!(slug_node.computed_rect.y, 100.0);
    assert_eq!(slug_node.computed_rect.height, 4.0);
    // slug_w = clamp(200 * 0.28 = 56.0, 32.0, 96.0) = 56.0
    assert_eq!(slug_node.computed_rect.width, 56.0);
    // travel = 200 - 56 = 144; slug_x = 50 + 144 * 0.5 = 122.0
    assert_eq!(slug_node.computed_rect.x, 122.0);
}

#[test]
fn test_declarative_modal_hierarchy_and_tags() {
    use crate::modal::{MODAL_TAG_CANCEL, MODAL_TAG_CLOSE, MODAL_TAG_CONFIRM, MODAL_TAG_SCRIM};
    use iris_core::{Color, UiLayer, WidgetRole};

    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let screen_w = 1280.0;
    let screen_h = 720.0;
    let card_w = 400.0;
    let card_h = 200.0;

    let mut card_id_opt = None;
    let mut scope = UiScope::new(&mut tree, root);
    let scrim_id = scope.modal_scrim(Color::rgba(0.0, 0.0, 0.0, 0.6), |scrim| {
        let card_id = scrim.modal_card(card_w, card_h, |card| {
            card.row(|row| {
                row.text("Title");
                row.modal_close_button();
            });
            card.column(|col| {
                col.text("Content line");
            });
            card.row(|footer| {
                footer.modal_cancel_button("Cancel", 80.0);
                footer.modal_confirm_button("OK", 80.0);
            });
        });
        card_id_opt = Some(card_id);
        scrim.finish_layout(Rect::new(0.0, 0.0, screen_w, screen_h));
    });

    let scrim_node = tree.get(scrim_id).expect("Scrim node exists");
    assert_eq!(scrim_node.tag, MODAL_TAG_SCRIM);
    assert_eq!(scrim_node.role, WidgetRole::ModalWindow);
    assert_eq!(scrim_node.layer, UiLayer::Modal);
    assert_eq!(
        scrim_node.computed_rect,
        Rect::new(0.0, 0.0, screen_w, screen_h)
    );

    let card_id = card_id_opt.expect("Card created");
    let card_node = tree.get(card_id).expect("Card node exists");
    assert_eq!(card_node.role, WidgetRole::ModalWindow);
    assert_eq!(card_node.layer, UiLayer::Modal);
    assert_eq!(card_node.computed_rect.width, card_w);
    assert_eq!(card_node.computed_rect.height, card_h);
    assert_eq!(card_node.computed_rect.x, (screen_w - card_w) * 0.5);
    assert_eq!(card_node.computed_rect.y, (screen_h - card_h) * 0.5);

    // Verify close, cancel, and confirm tags are in the hierarchy
    let mut found_close = false;
    let mut found_confirm = false;
    let mut found_cancel = false;
    tree.traverse_depth_first(card_id, &mut |_id, node| {
        if node.tag == MODAL_TAG_CLOSE {
            found_close = true;
        }
        if node.tag == MODAL_TAG_CONFIRM {
            found_confirm = true;
        }
        if node.tag == MODAL_TAG_CANCEL {
            found_cancel = true;
        }
    });

    assert!(found_close, "Modal close tag must exist");
    assert!(found_confirm, "Modal confirm tag must exist");
    assert!(found_cancel, "Modal cancel tag must exist");
}

#[test]
fn test_toolbar_buttons_child_non_interactive_and_hit_test() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let (icon_btn_id, mode_btn_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let icon_resp = scope.toolbar_icon_button([0.0, 0.0, 1.0, 1.0], 1001, false);
        let mode_resp = scope.toolbar_mode_button(
            Some([0.0, 0.0, 1.0, 1.0]),
            "Perspective",
            1002,
            false,
            100.0,
            CornerRadii::all(4.0),
        );
        (icon_resp.id, mode_resp.id)
    };

    // 1. Verify that all children of ToolbarIconButton have interactive = false
    let icon_btn_node = tree.get(icon_btn_id).expect("Button node exists");
    assert!(
        icon_btn_node.interactive,
        "Button parent must be interactive"
    );
    assert_eq!(icon_btn_node.tag, 1001);
    for &child_id in &icon_btn_node.children {
        let child = tree.get(child_id).expect("Child node exists");
        assert!(
            !child.interactive,
            "Child '{}' of ToolbarIconButton must NOT be interactive",
            child.name.as_deref().unwrap_or("unknown")
        );
    }

    // 2. Verify that all children of ToolbarModeButton have interactive = false
    let mode_btn_node = tree.get(mode_btn_id).expect("Mode button exists");
    assert!(mode_btn_node.interactive, "Mode button must be interactive");
    assert_eq!(mode_btn_node.tag, 1002);
    for &child_id in &mode_btn_node.children {
        let child = tree.get(child_id).expect("Child node exists");
        assert!(
            !child.interactive,
            "Child '{}' of ToolbarModeButton must NOT be interactive",
            child.name.as_deref().unwrap_or("unknown")
        );
    }

    // 3. Test hit-testing directly over the center of the icon
    // Set parent rect to (10, 10, 32, 32) and child icon to (15, 15, 22, 22)
    if let Some(btn) = tree.get_mut(icon_btn_id) {
        btn.computed_rect = Rect::new(10.0, 10.0, 32.0, 32.0);
    }
    for &child_id in &tree.get(icon_btn_id).unwrap().children.clone() {
        if let Some(c) = tree.get_mut(child_id) {
            c.computed_rect = Rect::new(15.0, 15.0, 22.0, 22.0);
        }
    }

    // Center point (26, 26) falls directly on the child icon
    let hit = tree
        .hit_test_target(Point::new(26.0, 26.0))
        .expect("Must hit toolbar button");
    assert_eq!(
        hit.tag, 1001,
        "Click on child icon area must resolve to parent button tag"
    );
    assert_eq!(hit.role, WidgetRole::Button);
}
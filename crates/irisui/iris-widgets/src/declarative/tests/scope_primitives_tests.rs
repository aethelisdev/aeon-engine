// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Subsystem Unit Tests

use crate::declarative::{UiScope, layout_subtree};
use iris_core::{InteractionEvent, MouseButton, Point, Rect, UiTree};

#[test]
fn test_declarative_scope_hierarchy() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let card_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.card("Transform", None, None, |card| {
            card.row(|row| {
                row.text("Position");
                row.drag_value(101, "X: ", 0.0);
                row.drag_value(102, "Y: ", 1.0);
                row.drag_value(103, "Z: ", 2.0);
            });
            let _ = card.button("Reset");
        });
        resp.response.id
    };

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

    let card_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.card("Properties", None, None, |card| {
            card.heading("Section 1");
            card.text("Label A");
            let _ = card.button("Action");
        });
        resp.response.id
    };

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
    let vec_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.drag_vec3("Offset", &mut vec, 0.5);
        resp.id
    };

    assert!(tree.get(vec_id).is_some());
    assert_eq!(vec, [1.0, 2.0, 3.0]);
}

#[test]
fn test_declarative_label_customization() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.label_custom(
        "Custom Title",
        20.0,
        iris_core::Color::rgba(0.0, 0.9, 1.0, 1.0),
        iris_core::TextAlign::Center,
    );

    scope.label("Standard Label");
    scope.heading("Heading Title");

    let node = tree
        .find_node_by_text("Custom Title")
        .and_then(|id| tree.get(id))
        .expect("Custom label exists");
    assert_eq!(node.font_size, 20.0);
    assert_eq!(node.text_align, iris_core::TextAlign::Center);
    assert_eq!(node.text.as_deref(), Some("Custom Title"));
    assert_eq!(node.line_height, 26.0);

    let std_node = tree
        .find_node_by_text("Standard Label")
        .and_then(|id| tree.get(id))
        .expect("Standard label exists");
    assert_eq!(std_node.font_size, 11.5);
    assert_eq!(std_node.text_align, iris_core::TextAlign::Left);
    assert_eq!(std_node.text.as_deref(), Some("Standard Label"));

    let head_node = tree
        .find_node_by_text("Heading Title")
        .and_then(|id| tree.get(id))
        .expect("Heading exists");
    assert_eq!(head_node.font_size, 13.5);
}

#[test]
fn test_declarative_status_pill() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.status_pill(
        "GroundedStatusPill",
        "GroundedStatusText",
        "🛡 Grounded",
        iris_core::Color::rgba(0.2, 0.8, 0.4, 1.0),
    );

    assert!(tree.has_node_named("GroundedStatusPill"));
    assert!(tree.has_node_named("GroundedStatusText"));
    tree.assert_node_text("GroundedStatusText", "🛡 Grounded");
}

#[test]
fn test_declarative_labeled_status_and_label_desc_colored() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.labeled_status(
        "Status:",
        "▶ PLAYING",
        iris_core::Color::rgba(0.2, 0.85, 0.4, 1.0),
    );
    scope.label_desc_colored(
        "WarningDesc",
        "Armature not found",
        iris_core::Color::rgba(0.95, 0.75, 0.15, 1.0),
    );
    scope.labeled_status_desc(
        "StyledStatusRow",
        crate::declarative::scope::display::labels::LabeledStatusDescriptor::new(
            "State:",
            "PAUSED",
            iris_core::Color::rgba(0.9, 0.6, 0.1, 1.0),
        )
        .prefix_width(60.0)
        .font_size(12.0)
        .height(24.0),
    );
    scope.key_value_badge("Ping", "12ms", iris_core::Color::rgba(0.2, 0.8, 0.3, 1.0));

    assert!(tree.has_node_named("LabeledStatusRow"));
    assert!(tree.has_node_named("LabeledStatusPrefix"));
    assert!(tree.has_node_named("LabeledStatusValue"));

    assert!(tree.has_node_named("StyledStatusRow"));

    assert!(tree.has_node_named("WarningDesc"));
    tree.assert_node_text("WarningDesc", "Armature not found");
}

#[test]
fn test_declarative_key_value_and_number_formatting() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.key_value("Texture Format", "RGBA8 Unorm");
    scope.key_value_file_size("Texture Size", 2_500_000);
    scope.label_number(std::f64::consts::PI, 2);

    assert!(tree.has_node_named("KeyValueLabel"));
    assert!(tree.has_node_named("KeyValueContent"));
    tree.assert_node_text("KeyValueLabel", "Texture Format");
    tree.assert_node_text("KeyValueContent", "RGBA8 Unorm");
    assert!(tree.find_node_by_text("3.14").is_some());
}

#[test]
fn test_declarative_branch_combinators() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.branch(true, |s| s.label("BranchTrue"), |s| s.label("BranchFalse"));
    scope.branch_some(
        Some("ActiveItem"),
        |s, val| s.label(val),
        |s| s.label("NoItem"),
    );
    scope.branch_ok(
        Result::<&str, &str>::Ok("SuccessPayload"),
        |s, val| s.label(val),
        |s, err| s.label(err),
    );
    scope.branch_ok(
        Result::<&str, &str>::Err("FailurePayload"),
        |s, val| s.label(val),
        |s, err| s.label(err),
    );

    assert!(tree.find_node_by_text("BranchTrue").is_some());
    assert!(tree.find_node_by_text("BranchFalse").is_none());
    assert!(tree.find_node_by_text("ActiveItem").is_some());
    assert!(tree.find_node_by_text("SuccessPayload").is_some());
    assert!(tree.find_node_by_text("FailurePayload").is_some());
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
    assert_eq!(caret_node.computed_rect.x, 10.0 + 8.0 + 50.0);
}

#[test]
fn test_declarative_progress_bar_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.column(|col| {
        col.progress_bar(0.75);
    });

    let bounds = Rect::new(0.0, 0.0, 200.0, 100.0);
    scope.finish_layout(bounds);

    let id = tree
        .root()
        .and_then(|r| tree.find_node_by_role_in_subtree(r, iris_core::WidgetRole::ProgressBar))
        .expect("Progress bar created");
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
fn test_declarative_progress_bar_named() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.progress_bar_named(
        "HealthBarTrack",
        "HealthBarFill",
        200.0,
        10.0,
        0.5,
        iris_core::Color::RED,
    );

    assert!(
        tree.has_node_named("HealthBarTrack"),
        "HealthBarTrack must exist"
    );
    assert!(
        tree.has_node_named("HealthBarFill"),
        "HealthBarFill must exist"
    );
}

#[test]
fn test_declarative_indeterminate_progress_layout() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let mut scope = UiScope::new(&mut tree, root);
    scope.column(|col| {
        col.indeterminate_progress(0.5);
    });

    let bounds = Rect::new(50.0, 100.0, 200.0, 100.0);
    scope.finish_layout(bounds);

    let id = tree
        .root()
        .and_then(|r| tree.find_node_by_role_in_subtree(r, iris_core::WidgetRole::ProgressBar))
        .expect("Indeterminate progress created");
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

/// Verifies that [`UiScope::landing_indicator`] correctly compensates for parent container screen origins.
#[test]
fn test_declarative_landing_indicator_parent_origin_compensation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    let panel_id = tree.create_node();
    if let Some(panel) = tree.get_mut(panel_id) {
        panel.computed_rect = Rect::new(200.0, 50.0, 800.0, 600.0);
    }
    let _ = tree.add_child(root, panel_id);

    let center = Point::new(450.0, 300.0);
    {
        let mut scope = UiScope::new(&mut tree, panel_id);
        scope.landing_indicator(center, 18.0, 4.0, iris_core::Color::CYAN);
    }

    let panel_node = tree.get(panel_id).expect("Panel exists");
    assert_eq!(panel_node.children.len(), 2);

    let ring = tree.get(panel_node.children[0]).expect("Ring exists");
    // Local inset in style = center.x - origin.x - radius = 450 - 200 - 18 = 232.0
    assert_eq!(ring.style.inset_left, Some(232.0));
    assert_eq!(ring.style.inset_top, Some(300.0 - 50.0 - 18.0));
    // Measured screen rect = center.x - radius = 450 - 18 = 432.0
    assert_eq!(ring.computed_rect.x, 432.0);
    assert_eq!(ring.computed_rect.y, 282.0);

    let dot = tree.get(panel_node.children[1]).expect("Dot exists");
    assert_eq!(dot.style.inset_left, Some(450.0 - 200.0 - 4.0));
    assert_eq!(dot.style.inset_top, Some(300.0 - 50.0 - 4.0));
    assert_eq!(dot.computed_rect.x, 446.0);
    assert_eq!(dot.computed_rect.y, 296.0);
}
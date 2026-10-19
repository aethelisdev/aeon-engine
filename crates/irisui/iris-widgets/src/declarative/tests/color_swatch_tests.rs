// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Declarative Color Swatch Builder (`ColorSwatch`)
//!
//! Verifies builder construction, autonomous layout properties, interactive hover detection,
//! active border styling, and click tracking response.
//!

use crate::declarative::UiScope;
use crate::declarative::widget::ColorSwatch;
use iris_core::Color;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;

#[test]
fn test_declarative_color_swatch_builder_properties() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let test_color = Color::rgba(0.2, 0.4, 0.8, 1.0);
    let swatch_tag = 0xAA55_CC33_1122;

    let mut scope = UiScope::new(&mut tree, root);
    let resp = scope.add(
        ColorSwatch::new(test_color)
            .size(40.0, 20.0)
            .corner_radius(4.0)
            .border_width(1.5)
            .active(true)
            .tag(swatch_tag),
    );

    assert!(!resp.clicked());
    assert!(!resp.hovered());

    // Validate that the created node holds expected styling and tag
    let node = tree.get(resp.id).expect("Node must exist in tree");
    assert_eq!(node.tag, swatch_tag);
    assert_eq!(node.style.width, Some(40.0));
    assert_eq!(node.style.height, Some(20.0));
    assert_eq!(node.style.background_color, test_color);
    assert_eq!(node.style.border.width.top, 1.5);
    // Active state displays white border by default
    assert_eq!(node.style.border.color, Color::rgba(1.0, 1.0, 1.0, 0.95));
}

#[test]
fn test_declarative_color_swatch_interaction_flow() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node should be created");

    let test_color = Color::rgba(0.9, 0.1, 0.1, 1.0);
    let swatch_tag = 0xBEEF_CAFE_0001;

    let node_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            ColorSwatch::new(test_color)
                .size(38.0, 18.0)
                .tag(swatch_tag),
        );
        resp.id
    };

    // Assign computed geometry
    if let Some(node) = tree.get_mut(node_id) {
        node.computed_rect = Rect::new(10.0, 10.0, 38.0, 18.0);
    }

    let inside_pt = Point::new(20.0, 18.0);

    // Mouse move over swatch
    let move_res = tree.dispatch_event(&UiEvent::MouseMove { point: inside_pt });
    assert_eq!(move_res.target, Some(node_id));

    // Next frame render detects hover
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            ColorSwatch::new(test_color)
                .size(38.0, 18.0)
                .tag(swatch_tag),
        );
        assert!(resp.hovered());
        assert!(!resp.clicked());
    }

    // MouseDown and MouseUp over swatch
    let _ = tree.dispatch_event(&UiEvent::MouseDown {
        button: MouseButton::Left,
        point: inside_pt,
    });
    let _ = tree.dispatch_event(&UiEvent::MouseUp {
        button: MouseButton::Left,
        point: inside_pt,
    });

    // Next frame render reports clicked
    {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.add(
            ColorSwatch::new(test_color)
                .size(38.0, 18.0)
                .tag(swatch_tag),
        );
        assert!(resp.clicked());
    }
}
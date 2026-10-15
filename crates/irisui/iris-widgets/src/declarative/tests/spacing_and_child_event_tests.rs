// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unit Tests for Spacing, Separators, and Child Event Consumption
//!
//! Verifies:
//! 1. Declarative [`UiScope::spacing`] and [`UiScope::separator`] primitives.
//! 2. Child event consumption: Clicking a button inside an interactive container executes
//!    the child button's click handler and does NOT bubble to parent click or drag handlers.
//!

use crate::declarative::UiScope;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;
use iris_core::{Style, WidgetRole};

#[test]
fn test_declarative_spacing_and_separators() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    // 1. In a horizontal row, spacing(12.0) sets width = 12.0
    let (spacing_id, sep_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let mut sp_id = None;
        let mut s_id = None;
        scope.row(|row| {
            sp_id = Some(row.spacing(12.0));
            s_id = Some(row.separator());
        });
        (sp_id.unwrap(), s_id.unwrap())
    };

    let sp_node = tree.get(spacing_id).expect("Spacing node must exist");
    assert_eq!(sp_node.style.width, Some(12.0));
    assert!(!sp_node.interactive);

    let sep_node = tree.get(sep_id).expect("Separator node must exist");
    assert_eq!(sep_node.role, WidgetRole::Separator);
    assert_eq!(sep_node.style.height, Some(1.0));

    // 2. In a vertical column, spacing(20.0) sets height = 20.0
    let col_spacing_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let mut sp_id = None;
        scope.column(|col| {
            sp_id = Some(col.spacing(20.0));
        });
        sp_id.unwrap()
    };

    let col_sp_node = tree
        .get(col_spacing_id)
        .expect("Column spacing node must exist");
    assert_eq!(col_sp_node.style.height, Some(20.0));
}

#[test]
fn test_child_event_consumption_prevents_parent_click_or_drag() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    let tag_row = 0xAAAA_0001;
    let tag_btn = 0xBBBB_0001;

    let mut row_clicked = false;
    let mut row_dragged = false;
    let mut btn_clicked = false;

    // 1. Build an interactive container with a nested child button
    let (row_id, btn_id) = {
        let mut scope = UiScope::new(&mut tree, root);
        let row_resp = scope.container_tagged(
            "ClickableRow",
            Style::new().width(400.0).height(40.0),
            WidgetRole::Button,
            tag_row,
            |row_scope| row_scope.small_button_tagged("Action", tag_btn).id(),
        );
        (row_resp.response.id, row_resp.inner)
    };

    // Assign computed layout rectangles
    let row_rect = Rect::new(0.0, 0.0, 400.0, 40.0);
    let btn_rect = Rect::new(300.0, 5.0, 80.0, 30.0);

    if let Some(node) = tree.get_mut(row_id) {
        node.computed_rect = row_rect;
    }
    if let Some(node) = tree.get_mut(btn_id) {
        node.computed_rect = btn_rect;
    }

    // 2. Click squarely on the child button
    let click_pt = Point::new(320.0, 15.0);

    let down_res = tree.dispatch_event(&UiEvent::MouseDown {
        point: click_pt,
        button: MouseButton::Left,
    });
    assert!(
        down_res.consumed,
        "Click on button must be consumed by dispatch engine"
    );
    assert_eq!(
        down_res.target,
        Some(btn_id),
        "Hit target must be the child button, not the parent container"
    );

    let up_res = tree.dispatch_event(&UiEvent::MouseUp {
        point: click_pt,
        button: MouseButton::Left,
    });
    assert!(
        up_res.consumed,
        "Mouse release must be consumed by child button"
    );

    // 3. Re-evaluate responses in a declarative scope cycle
    {
        let scope = UiScope::new(&mut tree, root);
        scope
            .response(row_id)
            .on_click(|| row_clicked = true)
            .on_any_click(|| row_clicked = true)
            .on_drag(|| row_dragged = true);

        scope.response(btn_id).on_click(|| btn_clicked = true);
    }

    // 4. Assert: Child button clicked, parent NEVER clicked and NEVER dragged!
    assert!(
        btn_clicked,
        "Child button must execute on_click when clicked"
    );
    assert!(
        !row_clicked,
        "Parent container must NOT receive click when child button was clicked"
    );
    assert!(
        !row_dragged,
        "Parent container must NOT receive drag when child button was clicked"
    );
}

#[test]
fn test_small_button_with_icon_renders_and_responds() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node creation failed");

    let tag_btn = 0xCCCC_0001;
    let icon_uv = [0.0, 0.0, 1.0, 1.0];

    let btn_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.small_button_with_icon_tagged(icon_uv, "Spawn", tag_btn);
        resp.id()
    };

    let btn_node = tree.get(btn_id).expect("Button node must exist");
    assert_eq!(btn_node.role, WidgetRole::Button);
    assert_eq!(btn_node.tag, tag_btn);
    assert_eq!(
        btn_node.children.len(),
        2,
        "Button must contain icon and text label children"
    );

    let icon_child = tree
        .get(btn_node.children[0])
        .expect("Icon child must exist");
    assert_eq!(icon_child.texture_uv, Some(icon_uv));

    let text_child = tree
        .get(btn_node.children[1])
        .expect("Text child must exist");
    assert_eq!(text_child.text.as_deref(), Some("Spawn"));
}
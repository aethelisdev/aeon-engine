// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Floating Drag Preview Widget Unit Tests
//!
//! Verifies builder lifecycle, elevated [`UiLayer::Tooltip`] assignment, non-interactive
//! pass-through semantics, and child badge pill layout.

use crate::declarative::scope::core::UiScope;
use crate::declarative::widget::{
    DEFAULT_DRAG_PREVIEW_OFFSET, FloatingDragPreview, FloatingDragPreviewStyle,
};
use iris_core::{Color, Point, UiLayer, UiTree, WidgetRole};

#[test]
fn test_floating_drag_preview_defaults() {
    let _preview = FloatingDragPreview::new("Character_Mesh.glb");
    let style = FloatingDragPreviewStyle::default();

    assert_eq!(style.offset, DEFAULT_DRAG_PREVIEW_OFFSET);
    assert_eq!(style.font_size, 13.0);
    assert_eq!(style.border_width, 1.0);
    assert_eq!(style.text_color, Color::WHITE);
}

#[test]
fn test_floating_drag_preview_hierarchy_and_layer() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root node must be created");

    let cursor = Point::new(250.0, 180.0);
    {
        let mut scope = UiScope::new(&mut tree, root);
        let preview = FloatingDragPreview::new("Character_Mesh.glb")
            .with_cursor(cursor)
            .with_badge("Mesh", Color::from_u8(0, 200, 255, 255))
            .with_sub_label("3.4 MB");

        scope.floating_drag_preview(preview);
    }

    // Verify preview card attached to root
    let root_node = tree.get(root).expect("Root node exists");
    assert_eq!(root_node.children.len(), 1);

    let card_id = root_node.children[0];
    let card = tree.get(card_id).expect("Preview card node exists");

    // 1. Verify elevation to UiLayer::Tooltip
    assert_eq!(card.layer, UiLayer::Tooltip);

    // 2. Verify non-interactive pass-through semantics
    assert!(!card.interactive);
    assert_eq!(card.role, WidgetRole::Tooltip);
    assert_eq!(card.name.as_deref(), Some("FloatingDragPreview"));

    // 3. Verify absolute position equals cursor + offset
    let expected_pos = Point::new(
        cursor.x + DEFAULT_DRAG_PREVIEW_OFFSET.x,
        cursor.y + DEFAULT_DRAG_PREVIEW_OFFSET.y,
    );
    assert_eq!(card.style.inset_left, Some(expected_pos.x));
    assert_eq!(card.style.inset_top, Some(expected_pos.y));

    // 4. Verify child elements: Badge Pill + Title Label + Sub Label
    assert_eq!(card.children.len(), 3);

    let badge_pill = tree.get(card.children[0]).expect("Badge pill exists");
    assert_eq!(badge_pill.name.as_deref(), Some("DragBadgePill"));
    assert!(!badge_pill.children.is_empty());

    let badge_label = tree
        .get(badge_pill.children[0])
        .expect("Badge label exists");
    assert_eq!(badge_label.text.as_deref(), Some("Mesh"));

    let title_label = tree.get(card.children[1]).expect("Title label exists");
    assert_eq!(title_label.text.as_deref(), Some("Character_Mesh.glb"));

    let sub_label = tree.get(card.children[2]).expect("Sub label exists");
    assert_eq!(sub_label.text.as_deref(), Some("3.4 MB"));
}
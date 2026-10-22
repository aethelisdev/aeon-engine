// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Comprehensive unit tests for UI tree hierarchy, hit testing, and cursor evaluation.

use super::arena::UiTree;
use crate::geometry::{Point, Rect};
use crate::node::{UiLayer, WidgetCursor, WidgetRole};

#[test]
fn test_hit_test_layered_priority() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
    }

    let content_btn = tree.create_node();
    if let Some(node) = tree.get_mut(content_btn) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 50.0);
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, content_btn).unwrap();

    let popup_item = tree.create_node();
    if let Some(node) = tree.get_mut(popup_item) {
        // Popup overlaps content button directly
        node.computed_rect = Rect::new(150.0, 100.0, 100.0, 100.0);
        node.layer = UiLayer::Popup;
    }
    tree.add_child(root, popup_item).unwrap();

    // Hit point inside both popup and content button: Popup MUST win!
    let hit = tree.hit_test_layered(Point::new(160.0, 120.0));
    assert_eq!(hit, Some(popup_item));

    // Hit point inside content button only
    let hit2 = tree.hit_test_layered(Point::new(110.0, 120.0));
    assert_eq!(hit2, Some(content_btn));
}

#[test]
fn test_hit_test_layered_modal_blocking() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
    }

    let bg_btn = tree.create_node();
    if let Some(node) = tree.get_mut(bg_btn) {
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 50.0);
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, bg_btn).unwrap();

    let modal_dlg = tree.create_node();
    if let Some(node) = tree.get_mut(modal_dlg) {
        node.computed_rect = Rect::new(300.0, 200.0, 400.0, 300.0);
        node.layer = UiLayer::Modal;
    }
    tree.add_child(root, modal_dlg).unwrap();

    assert!(tree.is_modal_active());

    // Inside modal: hits modal
    let hit_modal = tree.hit_test_layered(Point::new(350.0, 250.0));
    assert_eq!(hit_modal, Some(modal_dlg));

    // Outside modal over bg_btn: blocked by active modal!
    let hit_outside = tree.hit_test_layered(Point::new(60.0, 60.0));
    assert_eq!(hit_outside, None);
}

#[test]
fn test_hit_test_target_metadata_extraction() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    }

    let dropdown_item = tree.create_node();
    if let Some(node) = tree.get_mut(dropdown_item) {
        node.computed_rect = Rect::new(200.0, 150.0, 120.0, 24.0);
        node.role = WidgetRole::DropdownItem;
        node.layer = UiLayer::Popup;
        node.tag = 2; // Option index 2
        node.name = Some("Fps120Option".to_string());
    }
    tree.add_child(root, dropdown_item).unwrap();

    let hit_target = tree
        .hit_test_target(Point::new(250.0, 160.0))
        .expect("Dropdown item must be hit");

    assert_eq!(hit_target.id, dropdown_item);
    assert_eq!(hit_target.layer, UiLayer::Popup);
    assert_eq!(hit_target.role, WidgetRole::DropdownItem);
    assert_eq!(hit_target.tag, 2);
    assert_eq!(hit_target.name.as_deref(), Some("Fps120Option"));
    assert_eq!(hit_target.rect, Rect::new(200.0, 150.0, 120.0, 24.0));
    assert_eq!(hit_target.cursor, Some(WidgetCursor::Pointer));
}

#[test]
fn test_tree_cursor_at_layered_priority() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // 1. Text input in Content layer
    let input_id = tree.create_node();
    if let Some(node) = tree.get_mut(input_id) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 30.0);
        node.role = WidgetRole::TextInput;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, input_id).unwrap();

    // 2. Numeric drag pill in Content layer
    let num_id = tree.create_node();
    if let Some(node) = tree.get_mut(num_id) {
        node.computed_rect = Rect::new(320.0, 100.0, 80.0, 30.0);
        node.role = WidgetRole::NumericInput;
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, num_id).unwrap();

    // 3. Button in Content layer with explicit Grab cursor override
    let grab_btn = tree.create_node();
    if let Some(node) = tree.get_mut(grab_btn) {
        node.computed_rect = Rect::new(420.0, 100.0, 80.0, 30.0);
        node.role = WidgetRole::Button;
        node.cursor = Some(WidgetCursor::Grab);
        node.layer = UiLayer::Content;
    }
    tree.add_child(root, grab_btn).unwrap();

    // Test resolved cursors
    assert_eq!(tree.cursor_at(Point::new(150.0, 115.0)), WidgetCursor::Text);
    assert_eq!(
        tree.cursor_at(Point::new(350.0, 115.0)),
        WidgetCursor::Default
    );
    assert_eq!(tree.cursor_at(Point::new(450.0, 115.0)), WidgetCursor::Grab);
    assert_eq!(
        tree.cursor_at(Point::new(10.0, 10.0)),
        WidgetCursor::Default
    );
}

#[test]
fn test_hit_test_target_resolves_ancestor_tag_for_untagged_children() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // Tagged container (e.g. Button or Row)
    let parent_box = tree.create_node();
    if let Some(node) = tree.get_mut(parent_box) {
        node.computed_rect = Rect::new(100.0, 100.0, 200.0, 40.0);
        node.role = WidgetRole::Button;
        node.layer = UiLayer::Content;
        node.tag = 0x4849_4552_0000_0003; // Semantic tag
        node.interactive = true;
    }
    tree.add_child(root, parent_box).unwrap();

    // Untagged child (e.g. label or icon inside the button)
    let child_label = tree.create_node();
    if let Some(node) = tree.get_mut(child_label) {
        node.computed_rect = Rect::new(110.0, 110.0, 80.0, 20.0);
        node.role = WidgetRole::Default;
        node.layer = UiLayer::Content;
        node.tag = 0; // Untagged
        node.interactive = true;
    }
    tree.add_child(parent_box, child_label).unwrap();

    // Hit testing directly on the untagged child label must resolve the parent's semantic tag
    let hit = tree
        .hit_test_target(Point::new(120.0, 120.0))
        .expect("Child label must be hit");
    assert_eq!(hit.id, child_label);
    assert_eq!(hit.tag, 0x4849_4552_0000_0003);
}

#[test]
fn test_wants_pointer_at_evaluation() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    let button = tree.create_node();
    if let Some(node) = tree.get_mut(button) {
        node.computed_rect = Rect::new(50.0, 50.0, 100.0, 30.0);
        node.role = WidgetRole::Button;
        node.layer = UiLayer::Content;
        node.tag = 42;
    }
    tree.add_child(root, button).unwrap();

    // Over button -> wants pointer
    assert!(tree.wants_pointer_at(Point::new(60.0, 60.0)));
    // Empty root background -> does not want pointer
    assert!(!tree.wants_pointer_at(Point::new(10.0, 10.0)));
}

#[test]
fn test_hit_test_elevated_overlay_scissor_decoupling() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    // Parent container with clip_children = true restricting bounds to (0, 0, 100, 100)
    let parent = tree.create_node();
    if let Some(node) = tree.get_mut(parent) {
        node.computed_rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        node.style.clip_children = true;
    }
    tree.add_child(root, parent).unwrap();

    // Cascading submenu elevated overlay placed outside parent's scissor boundary
    let elevated_submenu = tree.create_node();
    if let Some(node) = tree.get_mut(elevated_submenu) {
        node.computed_rect = Rect::new(120.0, 40.0, 100.0, 100.0);
        node.role = WidgetRole::DropdownPopup;
        node.layer = UiLayer::Popup;
        node.interactive = true;
    }
    tree.add_child(parent, elevated_submenu).unwrap();

    // Point inside elevated submenu outside parent's bounds: Scissor must NOT clip elevated overlay
    let hit = tree.hit_test_layered(Point::new(130.0, 50.0));
    assert_eq!(
        hit,
        Some(elevated_submenu),
        "Elevated overlay must break out of parent scissor boundaries during hit-testing"
    );
}

#[test]
fn test_wants_pointer_at_with_open_popup() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();
    if let Some(node) = tree.get_mut(root) {
        node.computed_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    }

    assert!(!tree.wants_pointer_at(Point::new(20.0, 20.0)));

    // Open a menu dropdown
    tree.set_active_menu(Some(42));
    assert!(
        tree.wants_pointer_at(Point::new(20.0, 20.0)),
        "Tree must capture pointer when a menu popup is open"
    );

    // Dismiss popups
    tree.close_all_popups();
    assert!(!tree.wants_pointer_at(Point::new(20.0, 20.0)));
}

#[test]
fn test_invalidation_boundary_isolation_and_propagation() {
    use crate::dirty::DirtyFlags;

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let panel_a = tree.create_node();
    tree.set_boundary(panel_a, true);
    assert!(tree.is_boundary(panel_a));
    tree.add_child(root, panel_a).unwrap();

    let container = tree.create_node();
    tree.add_child(panel_a, container).unwrap();

    let button = tree.create_node();
    tree.add_child(container, button).unwrap();

    let panel_b = tree.create_node();
    tree.set_boundary(panel_b, true);
    tree.add_child(root, panel_b).unwrap();

    // Clear all dirty flags initially to simulate a settled layout pass
    for (_, node) in tree.nodes.iter_mut() {
        node.dirty = DirtyFlags::empty();
    }
    tree.clear_dirty_boundaries();

    assert!(!tree.has_dirty_boundaries());
    assert!(!tree.has_dirty_nodes(DirtyFlags::LAYOUT));

    // Mark button inside panel_a dirty: Propagation MUST stop at panel_a
    let captured_boundary = tree.mark_dirty(button, DirtyFlags::LAYOUT);
    assert_eq!(
        captured_boundary,
        Some(panel_a),
        "Dirty propagation must halt at the nearest boundary"
    );

    // Button, container, and panel_a must be dirty
    assert!(tree.get(button).unwrap().dirty.contains(DirtyFlags::LAYOUT));
    assert!(
        tree.get(container)
            .unwrap()
            .dirty
            .contains(DirtyFlags::LAYOUT)
    );
    assert!(
        tree.get(panel_a)
            .unwrap()
            .dirty
            .contains(DirtyFlags::LAYOUT)
    );

    // Root and panel_b must remain completely clean!
    assert!(
        tree.get(root).unwrap().dirty.is_empty(),
        "Root node must remain clean when dirty is contained in boundary"
    );
    assert!(
        tree.get(panel_b).unwrap().dirty.is_empty(),
        "Sibling panel must remain clean"
    );

    // dirty_boundaries set must contain panel_a
    assert_eq!(tree.dirty_boundaries().len(), 1);
    assert!(tree.dirty_boundaries().contains(&panel_a));
}

#[test]
fn test_invalidation_boundary_nested_boundaries() {
    use crate::dirty::DirtyFlags;

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let outer_panel = tree.create_node();
    tree.set_boundary(outer_panel, true);
    tree.add_child(root, outer_panel).unwrap();

    let inner_scroll_area = tree.create_node();
    tree.set_boundary(inner_scroll_area, true);
    tree.add_child(outer_panel, inner_scroll_area).unwrap();

    let item = tree.create_node();
    tree.add_child(inner_scroll_area, item).unwrap();

    for (_, node) in tree.nodes.iter_mut() {
        node.dirty = DirtyFlags::empty();
    }
    tree.clear_dirty_boundaries();

    // Invalidation inside inner_scroll_area must halt at inner_scroll_area, NOT outer_panel
    let boundary = tree.mark_dirty(item, DirtyFlags::LAYOUT);
    assert_eq!(boundary, Some(inner_scroll_area));
    assert!(tree.dirty_boundaries().contains(&inner_scroll_area));
    assert!(!tree.dirty_boundaries().contains(&outer_panel));
    assert!(tree.get(outer_panel).unwrap().dirty.is_empty());
}

#[test]
fn test_invalidation_boundary_without_boundary_propagates_to_root() {
    use crate::dirty::DirtyFlags;

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let uncontained_node = tree.create_node();
    tree.add_child(root, uncontained_node).unwrap();

    for (_, node) in tree.nodes.iter_mut() {
        node.dirty = DirtyFlags::empty();
    }
    tree.clear_dirty_boundaries();

    let boundary = tree.mark_dirty(uncontained_node, DirtyFlags::LAYOUT);
    assert_eq!(
        boundary, None,
        "Uncontained node must propagate to root without boundary"
    );
    assert!(tree.get(root).unwrap().dirty.contains(DirtyFlags::LAYOUT));
    assert!(tree.dirty_boundaries().is_empty());
}

#[test]
fn test_find_invalidation_boundary() {
    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    let boundary = tree.create_node();
    tree.set_boundary(boundary, true);
    tree.add_child(root, boundary).unwrap();

    let child = tree.create_node();
    tree.add_child(boundary, child).unwrap();

    let grandchild = tree.create_node();
    tree.add_child(child, grandchild).unwrap();

    assert_eq!(tree.find_invalidation_boundary(grandchild), Some(boundary));
    assert_eq!(tree.find_invalidation_boundary(child), Some(boundary));
    assert_eq!(tree.find_invalidation_boundary(boundary), Some(boundary));
    assert_eq!(tree.find_invalidation_boundary(root), None);
}

#[test]
fn test_tree_clear_children_preserves_parent_and_cleans_descendants() {
    use crate::canvas::CanvasPrimitive;
    use crate::color::Color;
    use crate::dirty::DirtyFlags;
    use crate::style::Style;

    let mut tree = UiTree::new();
    let root = tree.create_root().unwrap();

    // Create a panel boundary node
    let panel = tree.create_node();
    tree.set_boundary(panel, true);
    if let Some(node) = tree.get_mut(panel) {
        node.computed_rect = Rect::new(10.0, 20.0, 300.0, 400.0);
        node.set_style(Style::new().width(300.0).height(400.0));
    }
    tree.add_child(root, panel).unwrap();

    // Create descendants under panel
    let child1 = tree.create_node();
    tree.add_child(panel, child1).unwrap();

    let child2 = tree.create_node();
    tree.set_boundary(child2, true);
    tree.dirty_boundaries.insert(child2);
    tree.add_child(panel, child2).unwrap();

    let grandchild = tree.create_node();
    tree.add_child(child1, grandchild).unwrap();

    // Register canvas primitive on child1
    tree.add_canvas_primitive(
        child1,
        CanvasPrimitive::Rect {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::WHITE,
            corner_radius: 0.0,
        },
    );
    assert!(tree.canvas_primitives(child1).is_some());

    // Setup event state pointing to descendants
    tree.event_state.hovered = Some(child1);
    tree.event_state.focused = Some(grandchild);
    tree.event_state.pointer_capture = Some(child2);

    // Initial assertions: 5 total nodes (root, panel, child1, child2, grandchild)
    assert_eq!(tree.len(), 5);
    assert_eq!(tree.get(panel).unwrap().children.len(), 2);
    assert!(tree.dirty_boundaries.contains(&child2));

    // Clear children of panel
    let clear_res = tree.clear_children(panel);
    assert!(clear_res.is_ok());

    // 1. Parent node is strictly preserved
    let panel_node = tree.get(panel).expect("Parent node must be preserved");
    assert!(
        panel_node.children.is_empty(),
        "Parent children must be empty"
    );
    assert!(panel_node.is_boundary, "Boundary flag must be preserved");
    assert_eq!(panel_node.style.width, Some(300.0));
    assert_eq!(panel_node.style.height, Some(400.0));
    assert_eq!(
        panel_node.computed_rect,
        Rect::new(10.0, 20.0, 300.0, 400.0)
    );
    assert!(
        panel_node
            .dirty
            .contains(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT)
    );

    // 2. All descendants removed from arena
    assert!(tree.get(child1).is_none());
    assert!(tree.get(child2).is_none());
    assert!(tree.get(grandchild).is_none());
    assert_eq!(tree.len(), 2, "Only root and panel remain");

    // 3. Subsidiary structures purged
    assert!(!tree.dirty_boundaries.contains(&child2));
    assert!(tree.canvas_primitives(child1).is_none());

    // 4. Event state references untracked
    assert_eq!(tree.event_state.hovered, None);
    assert_eq!(tree.event_state.focused, None);
    assert_eq!(tree.event_state.pointer_capture, None);

    // 5. Test error on nonexistent node
    let dummy_id = tree.create_node();
    tree.remove_node(dummy_id).unwrap();
    assert!(tree.clear_children(dummy_id).is_err());
}
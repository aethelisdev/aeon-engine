// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative ScrollArea and Virtualized List Unit Tests

use crate::declarative::scope::UiScope;
use crate::scroll_area::style::ScrollAreaStyle;
use crate::scroll_area::types::VirtualScrollConfig;
use iris_core::geometry::{Point, Rect};
use iris_core::style::Style;
use iris_core::tree::UiTree;

#[test]
fn test_scroll_area_virtual_culling_slice() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let vp = Rect::new(0.0, 0.0, 200.0, 400.0);
    let total_items = 10_000;
    let item_stride = 20.0; // 10,000 * 20 = 200,000 px content height
    let scroll_y = 1_000.0; // 50 items scrolled down

    let mut rendered_indices = Vec::new();

    let (_frame, slice) = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.virtual_scroll_area_bounded(
            "TestVirtualOutliner",
            vp,
            scroll_y,
            total_items,
            item_stride,
            |_s, idx, rect| {
                rendered_indices.push(idx);
                assert_eq!(
                    rect.width,
                    vp.width
                        - ScrollAreaStyle::default().thickness
                        - ScrollAreaStyle::default().inset * 2.0
                );
            },
        )
    };

    // 10,000 items in collection, but only ~24 items rendered in UI tree!
    assert!(slice.visible_count < 30);
    assert_eq!(rendered_indices.len(), slice.visible_count);
    assert_eq!(rendered_indices[0], slice.start_idx);
    assert_eq!(*rendered_indices.last().unwrap(), slice.end_idx - 1);
}

#[test]
fn test_declarative_scroll_area_scope() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let vp = Rect::new(10.0, 10.0, 300.0, 400.0);
    let content_height = 800.0;
    let scroll_y = 100.0;

    let frame = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.scroll_area_bounded("MyScrollContainer", vp, scroll_y, content_height, |s| {
            s.text("Header item inside scroll");
            s.text("Second item inside scroll");
        })
    };

    // Viewport container was created with clip_children
    let container_node = tree.get(frame.container_id).unwrap();
    assert!(container_node.style.clip_children);
    assert_eq!(container_node.computed_rect, vp);
    assert!(container_node.children.len() >= 2);

    // Scrollbar was created because content_height 800 > viewport 400
    assert!(frame.scrollbar.is_some());
    let geom = frame.scrollbar.unwrap();
    assert_eq!(geom.track_rect.y, vp.y + ScrollAreaStyle::default().inset);
}

#[test]
fn test_declarative_scroll_area_fits_no_scrollbar() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let vp = Rect::new(0.0, 0.0, 200.0, 500.0);
    let content_height = 300.0; // Content fits in viewport

    let frame = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.scroll_area_bounded("FittingContainer", vp, 0.0, content_height, |s| {
            s.text("Fits completely");
        })
    };

    // No scrollbar indicator when content fits
    assert!(frame.scrollbar.is_none());
    assert_eq!(frame.max_scroll_y, 0.0);
}

#[test]
fn test_autonomous_virtual_scroll_area_height_resolution_and_slicing() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let test_tag = 0x9001;
    // Mock a previously computed viewport size in the tree
    tree.set_viewport_size(test_tag, iris_core::Size::new(240.0, 300.0));
    // Mock a scroll position of 100.0 px (5 items down with 20px item height)
    tree.set_scroll_offset(test_tag, 100.0);

    let mut rendered_indices = Vec::new();
    let config = VirtualScrollConfig::fixed(1000, 20.0).with_scrollbar_tags(0x9002, 0x9003);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.virtual_scroll_area("AutonomousVirtualList", test_tag, config, |_s, idx| {
            rendered_indices.push(idx);
        })
    };

    assert_eq!(resp.viewport_height, 300.0);
    assert_eq!(resp.current_scroll_y, 100.0);
    assert_eq!(resp.content_height, 20_000.0);
    assert_eq!(resp.max_scroll_y, 20_000.0 - 300.0);
    assert!(!rendered_indices.is_empty());
    assert!(rendered_indices.contains(&5)); // item at y=100 is visible
    assert_eq!(resp.visible_start, rendered_indices[0]);
    assert_eq!(resp.visible_end, *rendered_indices.last().unwrap() + 1);

    // Verify scrollbar was emitted autonomously
    let thumb_node = tree.find_node_by_tag(0x9003);
    assert!(
        thumb_node.is_some(),
        "autonomous scrollbar thumb should be emitted"
    );
}

#[test]
fn test_autonomous_height_resolution_parent_minus_siblings() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    // Set root panel rect to 600px height
    if let Some(r) = tree.get_mut(root) {
        r.computed_rect = Rect::new(0.0, 0.0, 400.0, 600.0);
    }

    let resolved_h = {
        let mut scope = UiScope::new(&mut tree, root);
        // Add a fixed 40px toolbar sibling
        scope.container(Style::new().height(40.0), |_s| {});
        // Add a 10px spacing sibling
        scope.container(Style::new().height(10.0), |_s| {});
        // Resolve available height for next child with unrecorded tag
        scope.resolve_available_height(0x9999)
    };

    // 600 - 40 - 10 = 550.0
    assert_eq!(resolved_h, 550.0);
}

#[test]
fn test_autonomous_scroll_area_measures_content_and_emits_scrollbar() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let test_tag = 0x9010;
    let track_tag = 0x9011;
    let thumb_tag = 0x9012;

    tree.set_viewport_size(test_tag, iris_core::Size::new(300.0, 200.0));

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.scroll_area(
            "AutonomousContentScroll",
            test_tag,
            Some((track_tag, thumb_tag)),
            |vp| {
                // 5 children of 80px each = 400px content
                for _ in 0..5 {
                    vp.container(Style::new().height(80.0), |_s| {});
                }
            },
        )
    };

    assert_eq!(resp.viewport_height, 200.0);
    assert_eq!(resp.content_height, 400.0);
    assert_eq!(resp.max_scroll_y, 200.0);

    let thumb_node = tree.find_node_by_tag(thumb_tag);
    assert!(
        thumb_node.is_some(),
        "autonomous scrollbar thumb should be emitted for overflowing content"
    );
}

#[test]
fn test_autonomous_visible_range() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let test_tag = 0x9020;
    tree.set_viewport_size(test_tag, iris_core::Size::new(200.0, 300.0));
    tree.set_scroll_offset(test_tag, 100.0);

    let mut scope = UiScope::new(&mut tree, root);
    let resp = scope.scroll_area("Container", test_tag, None, |s| {
        // 20px item height, 100 items
        let range = s.visible_range(20.0, 100);
        assert!(range.start > 0);
        assert!(range.end <= 100);
        assert!(range.contains(&5)); // 5 * 20 = 100px
        range
    });

    assert_eq!(resp.viewport_height, 300.0);
}

#[test]
fn test_scroll_area_auto_derives_scrollbar_tags_when_none() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let test_tag: u64 = 0x9040;
    let expected_track_tag = test_tag.wrapping_add(0x1000);
    let expected_thumb_tag = test_tag.wrapping_add(0x2000);

    tree.set_viewport_size(test_tag, iris_core::Size::new(300.0, 200.0));

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.scroll_area(
            "AutoDerivedScroll",
            test_tag,
            None, // Passing None must auto-derive scrollbar tags
            |vp| {
                for _ in 0..5 {
                    vp.container(Style::new().height(80.0), |_s| {});
                }
            },
        )
    };

    assert_eq!(resp.max_scroll_y, 200.0);
    assert!(
        tree.find_node_by_tag(expected_track_tag).is_some(),
        "Track node must be emitted with auto-derived tag"
    );
    assert!(
        tree.find_node_by_tag(expected_thumb_tag).is_some(),
        "Thumb node must be emitted with auto-derived tag"
    );
}

#[test]
fn test_virtual_scroll_area_auto_derives_scrollbar_tags() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let test_tag: u64 = 0x9050;
    let expected_track_tag = test_tag.wrapping_add(0x1000);
    let expected_thumb_tag = test_tag.wrapping_add(0x2000);

    tree.set_viewport_size(test_tag, iris_core::Size::new(240.0, 300.0));

    // Default config has scrollbar_tags: None
    let config = VirtualScrollConfig::fixed(100, 20.0);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.virtual_scroll_area("AutoVirtualList", test_tag, config, |_s, _idx| {})
    };

    assert!(resp.max_scroll_y > 0.0);
    assert!(
        tree.find_node_by_tag(expected_track_tag).is_some(),
        "Virtual scroll track must be emitted with auto-derived tag"
    );
    assert!(
        tree.find_node_by_tag(expected_thumb_tag).is_some(),
        "Virtual scroll thumb must be emitted with auto-derived tag"
    );
}

#[test]
fn test_scroll_area_auto_scrollbar_wheel_and_drag_interaction() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("root must exist");

    let test_tag: u64 = 0x9060;
    let expected_track_tag = test_tag.wrapping_add(0x1000);
    let expected_thumb_tag = test_tag.wrapping_add(0x2000);

    tree.set_viewport_size(test_tag, iris_core::Size::new(300.0, 200.0));

    let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);
    if let Some(r) = tree.get_mut(root) {
        r.computed_rect = bounds;
        r.set_style(Style::new().flex_col().height(200.0).width(300.0));
    }

    let _resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope.scroll_area("InteractionScroll", test_tag, None, |vp| {
            for _ in 0..10 {
                vp.container(Style::new().height(50.0), |_s| {});
            }
        })
    };
    crate::declarative::layout_subtree(&mut tree, root, bounds);

    // 1. Mouse wheel scroll
    let wheel_evt = iris_core::UiEvent::MouseScroll {
        delta_x: 0.0,
        delta_y: -40.0,
        point: Point::new(50.0, 50.0),
    };
    let disp = tree.dispatch_event(&wheel_evt);
    assert!(disp.consumed, "Mouse wheel must be consumed by scroll area");
    assert_eq!(tree.scroll_offset(test_tag), 40.0);

    // 2. Thumb drag interaction
    assert!(tree.find_node_by_tag(expected_track_tag).is_some());
    let thumb_id = tree
        .find_node_by_tag(expected_thumb_tag)
        .expect("Thumb exists");
    let thumb_rect = tree.get(thumb_id).unwrap().computed_rect;
    let thumb_center = Point::new(thumb_rect.x + 2.0, thumb_rect.y + 10.0);

    // Mouse down on thumb
    let down_evt = iris_core::UiEvent::MouseDown {
        button: iris_core::MouseButton::Left,
        point: thumb_center,
    };
    let disp_down = tree.dispatch_event(&down_evt);
    assert!(disp_down.consumed);
    assert_eq!(tree.event_state.pointer_capture, Some(thumb_id));

    // Drag down by 20px
    let drag_evt = iris_core::UiEvent::MouseMove {
        point: Point::new(thumb_center.x, thumb_center.y + 20.0),
    };
    let disp_drag = tree.dispatch_event(&drag_evt);
    assert!(disp_drag.consumed);
    assert!(tree.scroll_offset(test_tag) > 40.0);
}
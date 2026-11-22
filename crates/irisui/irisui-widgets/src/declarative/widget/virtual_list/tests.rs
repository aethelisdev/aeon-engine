// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for [`VirtualList`] and [`VirtualListBuilder`].

use crate::declarative::scope::UiScope;
use irisui_core::color::Color;
use irisui_core::node::WidgetRole;
use irisui_core::tree::UiTree;

/// Verifies Blueprint requirement:
/// In a virtual list with 1.000.000 items, the total allocated UiTree node count MUST NOT exceed 35.
#[test]
fn test_virtual_list_one_million_items_node_count_limit() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(1_000_000, 24.0)
            .visible_height(600.0)
            .scroll_y(1200.0)
            .render_item(|item_scope, idx| {
                item_scope.label(format!("Row #{}", idx));
            })
    };

    assert_eq!(resp.first_index(), 48);
    assert_eq!(resp.last_index(), 77);
    assert_eq!(resp.visible_count(), 29);
    assert_eq!(resp.top_spacer(), 48.0 * 24.0);
    assert_eq!(resp.total_height(), 24_000_000.0);

    let total_nodes = tree.len();
    assert_eq!(total_nodes, 33);
    assert!(
        total_nodes <= 35,
        "Total allocated nodes in UiTree ({total_nodes}) exceeded the 35 node ceiling!"
    );
}

/// Verifies top boundary ($S_y = 0.0$):
/// Top spacer is 0px so no top spacer node is allocated; only bottom spacer exists.
#[test]
fn test_virtual_list_top_boundary_no_top_spacer() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(100_000, 24.0)
            .visible_height(600.0)
            .scroll_y(0.0)
            .render_item(|item_scope, idx| {
                item_scope.label(format!("Row #{}", idx));
            })
    };

    assert_eq!(resp.first_index(), 0);
    assert_eq!(resp.last_index(), 27);
    assert_eq!(resp.visible_count(), 27);
    assert_eq!(resp.top_spacer(), 0.0);
    assert!(resp.bottom_spacer() > 0.0);
    assert_eq!(tree.len(), 30);
}

/// Verifies bottom boundary:
/// Bottom spacer is 0px so no bottom spacer node is allocated; only top spacer exists.
#[test]
fn test_virtual_list_bottom_boundary_no_bottom_spacer() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(100_000, 24.0)
            .visible_height(600.0)
            .scroll_y(2_399_400.0)
            .render_item(|item_scope, idx| {
                item_scope.label(format!("Row #{}", idx));
            })
    };

    assert_eq!(resp.first_index(), 99_973);
    assert_eq!(resp.last_index(), 100_000);
    assert_eq!(resp.visible_count(), 27);
    assert!(resp.top_spacer() > 0.0);
    assert_eq!(resp.bottom_spacer(), 0.0);
    assert_eq!(tree.len(), 30);
}

/// Verifies empty collection ($N = 0$):
/// Zero spacers and zero items emitted; only the viewport node is created.
#[test]
fn test_virtual_list_empty_collection() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(0, 24.0)
            .visible_height(600.0)
            .scroll_y(0.0)
            .render_item(|item_scope, idx| {
                item_scope.label(format!("Row #{}", idx));
            })
    };

    assert_eq!(resp.first_index(), 0);
    assert_eq!(resp.last_index(), 0);
    assert_eq!(resp.visible_count(), 0);
    assert_eq!(resp.top_spacer(), 0.0);
    assert_eq!(resp.bottom_spacer(), 0.0);
    assert_eq!(resp.total_height(), 0.0);
    assert_eq!(tree.len(), 2);
}

/// Verifies builder configuration chaining, styling, tagging, and persistent scroll state.
#[test]
fn test_virtual_list_builder_fluent_chaining_and_tagging() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let tag = 0xABCD_1234_u64;
    tree.set_scroll_offset(tag, 240.0);

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(1000, 20.0)
            .visible_height(400.0)
            .tag(tag)
            .name("TestConsoleList")
            .background(Color::rgba(0.1, 0.1, 0.1, 1.0))
            .overscan(3)
            .show(|item_scope, idx| {
                item_scope.label(format!("Log #{idx}"));
            })
    };

    assert_eq!(resp.slice.first_index, 9);
    assert_eq!(resp.slice.last_index, 35);
    assert_eq!(resp.visible_count(), 26);
    assert_eq!(resp.max_scroll_y, 19_600.0);

    let node = tree.get(resp.container_id).expect("Viewport node exists");
    assert_eq!(node.name.as_deref(), Some("TestConsoleList"));
    assert_eq!(node.tag, tag);
    assert_eq!(node.role, WidgetRole::ScrollAreaViewport);
    assert_eq!(node.style.background_color, Color::rgba(0.1, 0.1, 0.1, 1.0));
}

/// Verifies that `auto_scroll(&mut bool)` pins scroll position to `f32::MAX` when true.
#[test]
fn test_virtual_list_auto_scroll_pins_to_bottom() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let mut auto_scroll = true;
    let tag = 0x5555;

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(100, 20.0)
            .visible_height(200.0)
            .tag(tag)
            .auto_scroll(&mut auto_scroll)
            .render_item(|s, idx| {
                s.label(format!("{idx}"));
            })
    };

    assert!(auto_scroll);
    assert_eq!(resp.last_index(), 100);
    assert_eq!(resp.first_index(), 88);
    assert_eq!(resp.visible_count(), 12);
    assert_eq!(tree.scroll_offset(tag), 1800.0);
}

/// Verifies that auto-scroll properly clamps and renders all items when total content fits within viewport.
#[test]
fn test_virtual_list_auto_scroll_renders_all_items_when_content_fits_viewport() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let mut auto_scroll = true;
    let tag = 0x6666;
    let mut rendered_count = 0;

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(4, 26.0)
            .visible_height(300.0)
            .tag(tag)
            .auto_scroll(&mut auto_scroll)
            .render_item(|s, idx| {
                rendered_count += 1;
                s.label(format!("Item {idx}"));
            })
    };

    assert!(auto_scroll);
    assert_eq!(rendered_count, 4, "All 4 items must be rendered");
    assert_eq!(resp.first_index(), 0);
    assert_eq!(resp.last_index(), 4);
    assert_eq!(resp.visible_count(), 4);
    assert_eq!(
        tree.scroll_offset(tag),
        0.0,
        "Clamped scroll offset must be 0.0"
    );
}

/// Verifies that `auto_scroll(&mut bool)` detects upward scrolling on the list's tag and sets `auto_scroll = false`.
#[test]
fn test_virtual_list_auto_scroll_disabled_on_scroll_up() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let tag = 0x5555;
    tree.event_state.frame_scrolled_tag = Some(tag);
    tree.event_state.frame_scroll_delta = Some((0.0, 15.0));

    let mut auto_scroll = true;
    let mut scrolled_up_called = false;
    let mut scrolled_down_called = false;

    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(100, 20.0)
            .visible_height(200.0)
            .tag(tag)
            .scroll_y(500.0)
            .on_scroll_up(|| {
                scrolled_up_called = true;
            })
            .on_scroll_down(|| {
                scrolled_down_called = true;
            })
            .auto_scroll(&mut auto_scroll)
            .render_item(|s, idx| {
                s.label(format!("{idx}"));
            })
    };

    assert!(
        !auto_scroll,
        "auto_scroll must be disabled automatically when an upward scroll occurs"
    );
    assert!(scrolled_up_called, "on_scroll_up callback must be invoked");
    assert!(
        !scrolled_down_called,
        "on_scroll_down must not be invoked on scroll up"
    );
    assert_eq!(tree.scroll_offset(tag), 500.0);
    assert!(resp.last_index() < 100);
}

/// Verifies that `empty_notice(&'static str)` emits an empty notice when `item_count == 0`.
#[test]
fn test_virtual_list_empty_notice_renders_when_item_count_zero() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let mut items_rendered = 0;
    let resp = {
        let mut scope = UiScope::new(&mut tree, root);
        scope
            .virtual_list(0, 20.0)
            .visible_height(200.0)
            .name("EmptyVirtualList")
            .empty_notice("No entries found.")
            .render_item(|_, _| {
                items_rendered += 1;
            })
    };

    assert_eq!(items_rendered, 0);
    assert_eq!(resp.visible_count(), 0);
    assert_eq!(resp.total_height(), 0.0);

    let vp_node = tree.get(resp.container_id).expect("viewport node");
    assert_eq!(vp_node.children.len(), 1);
    let notice_node = tree
        .get(vp_node.children[0])
        .expect("notice container node");
    assert_eq!(notice_node.children.len(), 1);
    let label_node = tree
        .get(notice_node.children[0])
        .expect("notice label node");
    assert_eq!(label_node.text.as_deref(), Some("No entries found."));
}
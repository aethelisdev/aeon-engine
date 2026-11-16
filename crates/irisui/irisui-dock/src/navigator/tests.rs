// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::layout::ComputedDockLayout;
use crate::state::DockState;
use crate::tree::{DockNode, DockTree, SplitDirection};

#[test]
fn test_dock_navigator_geometry_and_hit_testing() {
    let content_rect = Rect::new(100.0, 100.0, 400.0, 300.0);
    let style = DockNavigatorStyle::default();
    let geometry = DockNavigatorGeometry::from_content_rect_with_style(content_rect, &style);

    // Center should be at (300, 250) with 40x40 size
    assert_eq!(geometry.center_button.x, 300.0 - 20.0);
    assert_eq!(geometry.center_button.y, 250.0 - 20.0);
    assert_eq!(geometry.center_button.width, 40.0);
    assert_eq!(geometry.center_button.height, 40.0);

    // Hit testing center
    assert_eq!(
        geometry.hit_test(Point::new(300.0, 250.0)),
        Some(DropZone::Center)
    );

    // Hit testing left button (x = 280 - 40 - 4 = 236)
    assert_eq!(
        geometry.hit_test(Point::new(250.0, 250.0)),
        Some(DropZone::Left)
    );

    // Hit testing right button (x = 320 + 4 = 324)
    assert_eq!(
        geometry.hit_test(Point::new(340.0, 250.0)),
        Some(DropZone::Right)
    );

    // Hit testing top button (y = 230 - 40 - 4 = 186)
    assert_eq!(
        geometry.hit_test(Point::new(300.0, 200.0)),
        Some(DropZone::Top)
    );

    // Hit testing bottom button (y = 270 + 4 = 274)
    assert_eq!(
        geometry.hit_test(Point::new(300.0, 290.0)),
        Some(DropZone::Bottom)
    );

    // Hit outside cross
    assert_eq!(geometry.hit_test(Point::new(110.0, 110.0)), None);
}

#[test]
fn test_dock_navigator_nodes_construction() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let content_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let style = DockNavigatorStyle::default();
    let geometry = DockNavigatorGeometry::from_content_rect_with_style(content_rect, &style);

    build_dock_navigator_nodes(&mut tree, root_id, &geometry, Some(DropZone::Left), &style);

    let root_node = tree.get(root_id).expect("Root node exists");
    // Root must have 5 child buttons
    assert_eq!(root_node.children.len(), 5);

    // Verify Left button active styling
    let left_button_id = root_node.children[1];
    let left_button = tree.get(left_button_id).expect("Left button exists");
    assert_eq!(left_button.style.background_color, style.hover_bg);

    // Left button must contain inner window, header rim, active fill, and 4 dashed segments
    assert_eq!(left_button.children.len(), 7);
}

#[test]
fn test_degenerate_content_rect_safety() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let degenerate_rect = Rect::new(0.0, 0.0, 20.0, 20.0);
    let style = DockNavigatorStyle::default();
    let geometry = DockNavigatorGeometry::from_content_rect_with_style(degenerate_rect, &style);

    build_dock_navigator_nodes(&mut tree, root_id, &geometry, None, &style);

    let root_node = tree.get(root_id).expect("Root node exists");
    assert_eq!(root_node.children.len(), 0);

    assert_eq!(
        hit_test_navigator(degenerate_rect, Point::new(10.0, 10.0), &style),
        None
    );
}

#[test]
fn test_floating_tab_badge_construction() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let badge_id = build_floating_tab_badge(
        &mut tree,
        root_id,
        FloatingTabBadgeParams {
            cursor_pos: Point::new(150.0, 200.0),
            title: "Hierarchy",
            icon: Some("🏗️"),
        },
    );

    let badge_node = tree.get(badge_id).expect("Badge node exists");
    assert_eq!(badge_node.computed_rect.x, 150.0 + 12.0);
    assert_eq!(badge_node.computed_rect.y, 200.0 + 12.0);
    assert_eq!(badge_node.children.len(), 2); // label + close
}

#[test]
fn test_drop_preview_node_construction() {
    let mut tree = UiTree::new();
    let root_id = tree.create_node();
    let _ = tree.set_root(root_id);

    let content_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
    let style = DockNavigatorStyle::default();
    let preview_id =
        build_drop_preview_node(&mut tree, root_id, content_rect, DropZone::Left, &style);

    let preview_node = tree.get(preview_id).expect("Preview node exists");
    assert_eq!(preview_node.computed_rect.width, 400.0);
    assert_eq!(preview_node.computed_rect.height, 600.0);
}

#[test]
fn test_navigator_coordinates_to_dock_tree_split_actions() {
    // Tests each 5-way compass drop zone: Center, Left, Right, Top, Bottom
    // Verifies that cursor over the compass button correctly splits the dock tree leaf
    for (zone, expected_is_split, expected_direction) in [
        (DropZone::Center, false, None),
        (DropZone::Left, true, Some(SplitDirection::Horizontal)),
        (DropZone::Right, true, Some(SplitDirection::Horizontal)),
        (DropZone::Top, true, Some(SplitDirection::Vertical)),
        (DropZone::Bottom, true, Some(SplitDirection::Vertical)),
    ] {
        let mut tree = DockTree::new();
        let leaf = tree.create_leaf(vec!["Tab1", "Tab2"]);
        tree.set_root(leaf);
        let mut state = DockState::new(tree);

        let content_rect = Rect::new(0.0, 0.0, 800.0, 600.0);
        let geometry = DockNavigatorGeometry::from_content_rect_default(content_rect);
        let button_rect = geometry.get_button_rect(zone);
        let hover_point = Point::new(
            button_rect.x + button_rect.width * 0.5,
            button_rect.y + button_rect.height * 0.5,
        );

        // Verify hit test identifies the target zone directly
        assert_eq!(
            geometry.hit_test(hover_point),
            Some(zone),
            "Hover point must resolve to zone {zone:?}"
        );

        // Initiate tab drag with second tab (index 1), keeping Tab1 in the leaf
        state
            .start_tab_drag(leaf, 1, Point::new(50.0, 50.0), content_rect)
            .expect("Tab drag starts");

        // Build mock layout with single leaf containing remaining Tab1
        let layout = ComputedDockLayout {
            leaves: vec![crate::layout::LeafLayoutInfo {
                node_id: leaf,
                rect: content_rect,
                content_rect,
                tab_bar_rect: Rect::new(0.0, 0.0, 800.0, 26.0),
                tabs: vec!["Tab1"],
                active_tab: 0,
                tab_bar_layout: None,
                is_maximized: false,
            }],
            splitters: Vec::new(),
        };

        // Update tab drag with cursor directly over the compass button
        state.update_tab_drag(hover_point, &layout);

        let active_drag = state.active_drag.as_ref().expect("Drag must be active");
        let (target_leaf, hit_zone, preview) = active_drag
            .hover_target
            .expect("Hover target must be detected");
        assert_eq!(target_leaf, leaf);
        assert_eq!(hit_zone, zone);
        assert_eq!(
            preview,
            calculate_drop_preview_rect(content_rect, zone),
            "Preview rectangle must match zone specification"
        );

        // Commit drop
        state.drop_tab().expect("Drop tab succeeds");
        assert!(state.active_drag.is_none());

        // Verify resulting dock tree hierarchy
        if expected_is_split {
            let root_id = state.tree.root().expect("Root exists");
            let root_node = state.tree.get(root_id).expect("Root node exists");
            match root_node {
                DockNode::Split {
                    direction,
                    first: _,
                    second: _,
                    ratio,
                } => {
                    assert_eq!(Some(*direction), expected_direction);
                    assert_eq!(*ratio, 0.5);
                    // Both split children must exist and contain the respective tabs
                    assert!(state.tree.find_tab(&"Tab1").is_some());
                    assert!(state.tree.find_tab(&"Tab2").is_some());
                }
                _ => panic!("Expected Split node for zone {zone:?}"),
            }
        } else {
            // Center drop zone keeps single leaf containing both tabs
            let root_id = state.tree.root().expect("Root exists");
            let root_node = state.tree.get(root_id).expect("Root node exists");
            match root_node {
                DockNode::Leaf { tabs, .. } => {
                    assert_eq!(tabs.len(), 2);
                    assert!(tabs.contains(&"Tab1"));
                    assert!(tabs.contains(&"Tab2"));
                }
                _ => panic!("Expected Leaf node for Center drop zone"),
            }
        }
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit test suite for `DockTree` split-tree lifecycle, tab searching, and pruning.

use super::*;
use crate::drag_drop::DropZone;

#[test]
fn test_split_ordered_and_dock_tab() {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec!["Tab A"]);
    tree.set_root(root);

    // 1. Dock Tab B to the Left of Tab A
    let left_leaf = tree
        .dock_tab(root, DropZone::Left, "Tab B")
        .expect("Dock left succeeds");

    // Root must now be a split container
    let split_node = tree.get(root).expect("Root exists");
    match split_node {
        DockNode::Split {
            direction,
            first,
            second,
            ..
        } => {
            assert_eq!(*direction, SplitDirection::Horizontal);
            assert_eq!(*first, left_leaf);

            // Check left leaf has Tab B
            let left_node = tree.get(*first).expect("Left leaf exists");
            if let DockNode::Leaf { tabs, .. } = left_node {
                assert_eq!(tabs, &["Tab B"]);
            } else {
                panic!("Expected leaf");
            }

            // Check right leaf has Tab A
            let right_node = tree.get(*second).expect("Right leaf exists");
            if let DockNode::Leaf { tabs, .. } = right_node {
                assert_eq!(tabs, &["Tab A"]);
            } else {
                panic!("Expected leaf");
            }
        }
        _ => panic!("Expected split node at root"),
    }

    // 2. Dock Tab C to the Top of Tab B
    let top_leaf = tree
        .dock_tab(left_leaf, DropZone::Top, "Tab C")
        .expect("Dock top succeeds");

    let left_split = tree.get(left_leaf).expect("Left split exists");
    match left_split {
        DockNode::Split {
            direction,
            first,
            second,
            ..
        } => {
            assert_eq!(*direction, SplitDirection::Vertical);
            assert_eq!(*first, top_leaf);

            // First child (top) has Tab C
            if let DockNode::Leaf { tabs, .. } = tree.get(*first).unwrap() {
                assert_eq!(tabs, &["Tab C"]);
            }
            // Second child (bottom) has Tab B
            if let DockNode::Leaf { tabs, .. } = tree.get(*second).unwrap() {
                assert_eq!(tabs, &["Tab B"]);
            }
        }
        _ => panic!("Expected vertical split"),
    }

    // 3. Dock Tab D into Center of top_leaf (append)
    let center_res = tree
        .dock_tab(top_leaf, DropZone::Center, "Tab D")
        .expect("Dock center succeeds");
    assert_eq!(center_res, top_leaf);

    if let DockNode::Leaf { tabs, active_tab } = tree.get(top_leaf).unwrap() {
        assert_eq!(tabs, &["Tab C", "Tab D"]);
        assert_eq!(*active_tab, 1);
    }
}

#[test]
fn test_find_tab_and_focus_and_move() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec!["Tab 1", "Tab 2", "Tab 3"]);
    tree.set_root(leaf);
    tree.set_focused_leaf(Some(leaf));

    assert_eq!(tree.focused_leaf(), Some(leaf));
    assert_eq!(tree.find_tab(&"Tab 2"), Some((leaf, 1)));
    assert_eq!(tree.find_tab(&"NonExistent"), None);

    // Move Tab 1 to index 2 within same leaf
    tree.move_tab(leaf, 0, leaf, 2).expect("Move tab succeeds");
    if let DockNode::Leaf { tabs, active_tab } = tree.get(leaf).unwrap() {
        assert_eq!(tabs, &["Tab 2", "Tab 3", "Tab 1"]);
        assert_eq!(*active_tab, 2);
    }

    // Push to focused leaf
    let (target, idx) = tree
        .push_to_focused_leaf("Tab 4")
        .expect("Push to focused succeeds");
    assert_eq!(target, leaf);
    assert_eq!(idx, 3);

    // Directional splits
    let right_leaf = tree
        .split_right(leaf, "Tab 5")
        .expect("Split right succeeds");
    assert_eq!(tree.find_tab(&"Tab 5"), Some((right_leaf, 0)));

    let above_leaf = tree
        .split_above(right_leaf, "Tab 6")
        .expect("Split above succeeds");
    assert_eq!(tree.find_tab(&"Tab 6"), Some((above_leaf, 0)));

    // Move Tab 5 across leaves into above_leaf
    let (tab5_leaf, tab5_idx) = tree.find_tab(&"Tab 5").expect("Tab 5 exists");
    tree.move_tab(tab5_leaf, tab5_idx, above_leaf, 0)
        .expect("Move across leaves succeeds");
    if let DockNode::Leaf { tabs, .. } = tree.get(above_leaf).unwrap() {
        assert_eq!(tabs, &["Tab 5", "Tab 6"]);
    }

    // Collapse empty leaves prunes tab5_leaf (now empty) and cleans focused_leaf
    tree.set_focused_leaf(Some(tab5_leaf));
    tree.collapse_empty_leaves();
    assert_eq!(tree.focused_leaf(), None);

    // Bulk close tests
    let test_leaf = tree.create_leaf(vec!["A", "B", "C", "D"]);
    let closed_right = tree.close_tabs_to_right(test_leaf, 1).unwrap();
    assert_eq!(closed_right, vec!["C", "D"]);
    let closed_others = tree.close_other_tabs(test_leaf, 1).unwrap();
    assert_eq!(closed_others, vec!["A"]);
}
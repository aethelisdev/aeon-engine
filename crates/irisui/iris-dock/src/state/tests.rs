// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::*;
use crate::floating::FloatingWindowClampBounds;

#[test]
fn test_splitter_drag_and_min_pane_size_clamping() {
    let mut tree = DockTree::new();
    let leaf1 = tree.create_leaf(vec!["Tab A"]);
    let (_first, _second) = tree
        .split(leaf1, SplitDirection::Horizontal, 0.5, vec!["Tab B"])
        .expect("Split succeeds");
    tree.set_root(leaf1);

    let mut state = DockState::new(tree).with_min_pane_size(100.0);
    assert_eq!(state.min_pane_size, 100.0);

    // Start drag on split node leaf1 with total dimension 1000px
    state.start_splitter_drag(leaf1, SplitDirection::Horizontal, 500.0, 1000.0);

    // Drag all the way to 50px (cursor = 50.0). delta = -450 -> ratio would be 0.05, but min_pane_size = 100 -> min_ratio = 0.10
    state.update_splitter_drag(50.0);
    if let Some(DockNode::Split { ratio, .. }) = state.tree.get(leaf1) {
        assert!(*ratio >= 0.10);
    }

    // Drag all the way to 950px. max_ratio = 0.90
    state.update_splitter_drag(950.0);
    if let Some(DockNode::Split { ratio, .. }) = state.tree.get(leaf1) {
        assert!(*ratio <= 0.90);
    }

    state.end_splitter_drag();
    assert!(state.active_splitter.is_none());

    // Reset splitter to 0.5
    state.reset_splitter(leaf1).expect("Reset succeeds");
    if let Some(DockNode::Split { ratio, .. }) = state.tree.get(leaf1) {
        assert_eq!(*ratio, 0.5);
    }
}

#[test]
fn test_dock_state_maximization() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec!["Tab"]);
    tree.set_root(leaf);
    let mut state = DockState::new(tree);

    assert!(!state.is_maximized());
    state.maximize_leaf(leaf);
    assert!(state.is_maximized());
    assert_eq!(state.maximized_leaf, Some(leaf));

    state.restore();
    assert!(!state.is_maximized());

    state.toggle_maximize(leaf);
    assert!(state.is_maximized());
    state.toggle_maximize(leaf);
    assert!(!state.is_maximized());
}

#[test]
fn test_floating_window_detach_and_dock_back() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec!["Tab 1", "Tab 2"]);
    tree.set_root(leaf);
    let mut state = DockState::new(tree);

    let win_id = state
        .detach_tab_to_floating(
            leaf,
            1,
            "Detached Tab",
            Point::new(200.0, 200.0),
            Point::new(300.0, 200.0),
        )
        .expect("Detach succeeds");

    assert_eq!(state.floating_windows.len(), 1);
    assert_eq!(state.floating_windows[0].id, win_id);

    // Dock floating window back into main tree
    state
        .dock_floating_window(win_id, leaf, DropZone::Right)
        .expect("Dock back succeeds");
    assert_eq!(state.floating_windows.len(), 0);

    // Tree now has 2 tabs again
    assert!(state.tree.find_tab(&"Tab 1").is_some());
    assert!(state.tree.find_tab(&"Tab 2").is_some());
}

#[test]
fn test_start_floating_tab_drag() {
    let mut tree = DockTree::new();
    let leaf = tree.create_leaf(vec!["Tab 1", "Tab 2"]);
    tree.set_root(leaf);
    let mut state = DockState::new(tree);

    let win_id = state
        .detach_tab_to_floating(
            leaf,
            1,
            "Tab 2",
            Point::new(200.0, 200.0),
            Point::new(300.0, 200.0),
        )
        .expect("Detach succeeds");

    assert_eq!(state.floating_windows.len(), 1);
    let float_root = state.floating_windows[0].tree.root().unwrap();

    // Initiate drag from floating window
    state
        .start_floating_tab_drag(win_id, float_root, 0, Point::new(250.0, 250.0))
        .expect("Start floating tab drag succeeds");

    // Window with 0 tabs should be pruned
    assert_eq!(state.floating_windows.len(), 0);
    assert!(state.active_drag.is_some());
    assert_eq!(state.active_drag.as_ref().unwrap().tab_data, "Tab 2");

    // Drop into main tree
    state.active_drag.as_mut().unwrap().hover_target = Some((leaf, DropZone::Center, Rect::ZERO));
    state.drop_tab().expect("Drop into tree succeeds");
    assert!(state.active_drag.is_none());
    assert!(state.tree.find_tab(&"Tab 1").is_some());
    assert!(state.tree.find_tab(&"Tab 2").is_some());
}

#[test]
fn test_sync_floating_window_rects_and_clamping() {
    let mut state = DockState::<&str>::default();
    let mut rect_buffer = Vec::with_capacity(4);

    state.sync_floating_window_rects(&mut rect_buffer);
    assert!(rect_buffer.is_empty());

    let win = FloatingWindow::new(
        1,
        "Floating Win",
        Rect::new(-500.0, -100.0, 300.0, 400.0),
        vec!["Tab Float"],
    );
    state.floating_windows.push(win);

    state.sync_floating_window_rects(&mut rect_buffer);
    assert_eq!(rect_buffer.len(), 1);
    assert_eq!(rect_buffer[0], Rect::new(-500.0, -100.0, 300.0, 400.0));

    // Clamp floating windows within 1920x1080 bounds
    let bounds = FloatingWindowClampBounds::new(1920.0, 1080.0)
        .with_min_y(32.0)
        .with_bottom_inset(24.0);
    state.clamp_floating_windows(&bounds);

    assert_eq!(state.floating_windows[0].rect.y, 32.0);
    assert_eq!(
        state.floating_windows[0].rect.x,
        60.0 - state.floating_windows[0].rect.width
    );

    // Tab membership check without allocation
    assert!(state.is_tab_in_floating_window(&"Tab Float"));
    assert!(!state.is_tab_in_floating_window(&"Nonexistent Tab"));

    // Content rect resolution
    let content_rect = state.find_floating_tab_content_rect(&"Tab Float", 28.0);
    assert!(content_rect.is_some());
    let rect = content_rect.unwrap();
    assert_eq!(rect.y, state.floating_windows[0].rect.y + 28.0);
}

#[test]
fn test_open_or_focus_floating_tab() {
    let mut tree = DockTree::new();
    let root = tree.create_leaf(vec!["Tab A"]);
    tree.set_root(root);
    let mut state = DockState::new(tree);

    assert_eq!(state.floating_windows.len(), 0);

    // 1. Open new floating window
    let rect1 = Rect::new(100.0, 100.0, 400.0, 300.0);
    let id1 = state.open_or_focus_floating_tab("Tab Preferences", "Preferences", rect1);
    assert_eq!(id1, 1);
    assert_eq!(state.floating_windows.len(), 1);
    assert!(state.is_tab_in_floating_window(&"Tab Preferences"));

    // 2. Open another floating window
    let rect2 = Rect::new(200.0, 200.0, 500.0, 400.0);
    let id2 = state.open_or_focus_floating_tab("Tab Inspector", "Inspector", rect2);
    assert_eq!(id2, 2);
    assert_eq!(state.floating_windows.len(), 2);
    // id2 is top-most
    assert_eq!(state.floating_windows.last().unwrap().id, 2);

    // 3. Re-requesting Tab Preferences should bring window 1 to front without creating duplicate
    let id1_refocus =
        state.open_or_focus_floating_tab("Tab Preferences", "Preferences", Rect::ZERO);
    assert_eq!(id1_refocus, 1);
    assert_eq!(state.floating_windows.len(), 2);
    // id1 is now brought to top-most
    assert_eq!(state.floating_windows.last().unwrap().id, 1);
}
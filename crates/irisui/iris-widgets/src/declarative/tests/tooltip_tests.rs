// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::declarative::{TooltipExt, UiScope};
use iris_core::{Point, UiTree};

#[test]
fn test_declarative_tooltip_attaches_and_renders_on_hover() {
    let mut tree = UiTree::new();
    let root = tree.create_root().expect("Root should be created");

    // 1. Build a button with a tooltip
    let btn_id = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Save File");
        resp.tooltip(&mut scope, "Saves current project (Ctrl+S)");
        resp.id
    };

    // Before hover timer triggers, no tooltip node exists under root
    assert_eq!(tree.get(root).unwrap().children.len(), 1);

    // Simulate hover timer expiration for this button
    let now = std::time::Instant::now();
    let delay = std::time::Duration::from_millis(400);
    tree.event_state.active_tooltip = Some(iris_core::dispatch::TooltipState {
        target_id: btn_id,
        target_tag: tree.get(btn_id).unwrap().tag,
        anchor_pos: Point::new(50.0, 50.0),
        hover_start: now - delay - std::time::Duration::from_millis(50),
        delay,
        is_open: true,
    });

    // 2. Re-render UI frame with active tooltip
    let _ = {
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.button("Save File");
        resp.tooltip(&mut scope, "Saves current project (Ctrl+S)");
        resp.id
    };

    // Now a tooltip overlay card node exists on UiLayer::Tooltip
    let root_node = tree.get(root).unwrap();
    let tooltip_node_id = root_node.children.iter().copied().find(|&id| {
        tree.get(id)
            .is_some_and(|n| n.layer == iris_core::UiLayer::Tooltip)
    });
    assert!(
        tooltip_node_id.is_some(),
        "Tooltip node must be attached to root on UiLayer::Tooltip"
    );
}
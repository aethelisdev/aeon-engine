// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests verifying native dock drag overlays, hybrid proportional shrinking, and overflow chevrons.

use super::builder::build_native_dock;
use super::overflow::build_native_dock_overflow_menu;
use super::overlays::build_native_dock_drag_overlays;
use super::types::*;
use crate::ui::iris_bridge::icons::ICON_CHEVRON_DOWN;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use irisui::dock::DockNodeId;
use irisui::prelude::*;

#[test]
fn test_build_native_dock_drag_overlays_renders_nodes() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);

    let mut layout_state = PanelLayoutState::default();
    let source_leaf = layout_state.dock_state.tree.find_first_leaf().unwrap();

    // Start dragging the first tab in the default layout
    let _ = layout_state.dock_state.start_tab_drag(
        source_leaf,
        0,
        Point::new(200.0, 200.0),
        Rect::new(0.0, 0.0, 400.0, 400.0),
    );

    let workspace = Rect::new(0.0, 0.0, 1920.0, 1080.0);
    build_native_dock_drag_overlays(&mut tree, root, &layout_state, workspace);

    let root_node = tree.get(root).expect("Root exists");
    assert!(
        !root_node.children.is_empty(),
        "Overlay nodes must be created during active drag"
    );
}

fn setup_test_layout() -> (PanelLayoutState, DockNodeId) {
    let mut ds = irisui::dock::DockState::default();
    let leaf = ds.tree.create_leaf(vec![PanelId::Viewport]);
    ds.tree.set_root(leaf);
    (PanelLayoutState { dock_state: ds }, leaf)
}

#[test]
fn test_hybrid_tab_proportional_shrink() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let (mut layout_state, leaf) = setup_test_layout();

    // Add multiple panels to the leaf so natural widths exceed pane width
    for panel in [PanelId::Console, PanelId::MaterialEditor, PanelId::Assets] {
        let _ = layout_state.dock_state.tree.add_tab(leaf, panel);
    }

    // A pane width of 350px can hold 4 panels shrunk proportionally (4 * 68px = 272px <= 350px)
    let workspace = Rect::new(0.0, 0.0, 350.0, 600.0);
    let frame = build_native_dock(
        &mut tree,
        root,
        &layout_state,
        workspace,
        Point::new(0.0, 0.0),
        false,
    );

    assert_eq!(frame.tab_targets.len(), 4);
    assert!(
        frame.chevron_targets.is_empty(),
        "No chevron when tabs fit proportionally"
    );
    for target in &frame.tab_targets {
        assert!(
            target.rect.right() <= 350.1,
            "Tab must not bleed beyond boundary"
        );
        assert!(
            target.rect.width >= MIN_SHRUNK_TAB_WIDTH - 0.1,
            "Width >= minimum"
        );
    }
}

#[test]
fn test_hybrid_tab_overflow_chevron() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let (mut layout_state, leaf) = setup_test_layout();

    for panel in [
        PanelId::Console,
        PanelId::MaterialEditor,
        PanelId::Assets,
        PanelId::Stats,
        PanelId::AnimationTimeline,
    ] {
        let _ = layout_state.dock_state.tree.add_tab(leaf, panel);
    }

    let workspace = Rect::new(0.0, 0.0, 200.0, 600.0);
    let mut frame = build_native_dock(
        &mut tree,
        root,
        &layout_state,
        workspace,
        Point::new(0.0, 0.0),
        false,
    );

    assert!(
        !frame.chevron_targets.is_empty(),
        "Chevron button must be created on overflow"
    );
    let chevron = frame.chevron_targets[0];
    assert_eq!(chevron.rect.width, CHEVRON_WIDTH);
    for target in &frame.tab_targets {
        assert!(
            target.rect.right() <= 200.0 - CHEVRON_WIDTH + 0.1,
            "Tab must not overlap chevron or bleed"
        );
    }

    build_native_dock_overflow_menu(
        &mut tree,
        root,
        NativeDockOverflowMenuParams {
            leaf_id: chevron.leaf,
            anchor_rect: chevron.rect,
            layout_state: &layout_state,
            cursor_pos: Point::new(0.0, 0.0),
            is_cursor_occluded: false,
        },
        &mut frame,
    );

    assert!(frame.active_overflow_rect.is_some());
    assert_eq!(frame.overflow_item_targets.len(), 6);
}

#[test]
fn test_tab_text_label_does_not_overlap_close_button() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let (mut layout_state, leaf) = setup_test_layout();

    for panel in [
        PanelId::AnimationTimeline,
        PanelId::Inspector,
        PanelId::Hierarchy,
    ] {
        let _ = layout_state.dock_state.tree.add_tab(leaf, panel);
    }

    // Set active tab to AnimationTimeline (has long text and close button)
    let _ = layout_state.dock_state.tree.set_active_tab(leaf, 1);

    let workspace = Rect::new(0.0, 0.0, 300.0, 600.0);
    let frame = build_native_dock(
        &mut tree,
        root,
        &layout_state,
        workspace,
        Point::new(0.0, 0.0),
        false,
    );

    // Collect all label nodes and verify strict non-overlapping boundary with close buttons
    for close_target in &frame.close_targets {
        // Find corresponding tab target
        let tab_target = frame
            .tab_targets
            .iter()
            .find(|t| t.leaf == close_target.leaf && t.tab_index == close_target.tab_index)
            .expect("Matching tab target exists");

        // The close button must sit inside the right side of the tab
        assert!(close_target.rect.right() <= tab_target.rect.right());

        // Find label nodes within this tab's bounds using depth-first traversal
        let mut labels_found = 0;
        tree.traverse_depth_first(root, &mut |_id, node| {
            if node.name.as_deref() == Some("IrisDockTabLabel")
                && node.computed_rect.x >= tab_target.rect.x
                && node.computed_rect.x < close_target.rect.x
            {
                labels_found += 1;
                assert!(
                    node.computed_rect.right() <= close_target.rect.x,
                    "Label right edge ({}) must not overlap close button x ({})",
                    node.computed_rect.right(),
                    close_target.rect.x
                );
                assert!(
                    node.style.clip_children,
                    "Label node must have clip_children enabled for hardware scissor clipping"
                );
            }
        });
        assert!(labels_found > 0, "Tab label must exist for active tab");
    }
}

#[test]
fn test_dock_tab_chevron_renders_atlas_icon() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let (mut layout_state, leaf) = setup_test_layout();

    for panel in [
        PanelId::Console,
        PanelId::MaterialEditor,
        PanelId::Assets,
        PanelId::Stats,
        PanelId::AnimationTimeline,
    ] {
        let _ = layout_state.dock_state.tree.add_tab(leaf, panel);
    }

    let workspace = Rect::new(0.0, 0.0, 200.0, 600.0);
    let frame = build_native_dock(
        &mut tree,
        root,
        &layout_state,
        workspace,
        Point::new(0.0, 0.0),
        false,
    );

    assert!(!frame.chevron_targets.is_empty());

    let mut chevron_icon_found = false;
    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.name.as_deref() == Some("IrisDockTabChevronIcon") {
            chevron_icon_found = true;
            assert_eq!(
                node.texture_uv,
                Some(ICON_CHEVRON_DOWN),
                "Chevron icon must sample from ICON_CHEVRON_DOWN layer in editor_atlas"
            );
        }
    });

    assert!(
        chevron_icon_found,
        "IrisDockTabChevronIcon node must be created"
    );
}

#[test]
fn test_dock_overflow_menu_hierarchy_and_roles() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let (mut layout_state, leaf) = setup_test_layout();

    for panel in [PanelId::Console, PanelId::Assets, PanelId::Stats] {
        let _ = layout_state.dock_state.tree.add_tab(leaf, panel);
    }

    let workspace = Rect::new(0.0, 0.0, 150.0, 600.0);
    let mut frame = build_native_dock(
        &mut tree,
        root,
        &layout_state,
        workspace,
        Point::new(0.0, 0.0),
        false,
    );

    assert!(!frame.chevron_targets.is_empty());
    let chevron = frame.chevron_targets[0];

    build_native_dock_overflow_menu(
        &mut tree,
        root,
        NativeDockOverflowMenuParams {
            leaf_id: chevron.leaf,
            anchor_rect: chevron.rect,
            layout_state: &layout_state,
            cursor_pos: Point::new(0.0, 0.0),
            is_cursor_occluded: false,
        },
        &mut frame,
    );

    let mut popup_found = false;
    let mut item_count = 0;

    tree.traverse_depth_first(root, &mut |_id, node| {
        if node.name.as_deref() == Some("IrisDockOverflowMenu") {
            popup_found = true;
            assert_eq!(
                node.role,
                WidgetRole::DropdownPopup,
                "Menu card must have DropdownPopup role"
            );
        }
        if node.name.as_deref() == Some("IrisDockOverflowItem") {
            item_count += 1;
            assert_eq!(
                node.role,
                WidgetRole::DropdownItem,
                "Item node must have DropdownItem role"
            );
        }
    });

    assert!(popup_found, "Overflow menu popup container must exist");
    assert_eq!(
        item_count, 4,
        "All 4 tabs must be represented as DropdownItem widgets"
    );
}

#[test]
fn test_dock_overflow_menu_occludes_background_text() {
    let mut tree = UiTree::new();
    let root = tree.create_node();
    let _ = tree.set_root(root);
    let (mut layout_state, leaf) = setup_test_layout();

    for panel in [PanelId::Console, PanelId::Assets, PanelId::Stats] {
        let _ = layout_state.dock_state.tree.add_tab(leaf, panel);
    }

    let workspace = Rect::new(0.0, 0.0, 150.0, 600.0);
    let mut frame = build_native_dock(
        &mut tree,
        root,
        &layout_state,
        workspace,
        Point::new(0.0, 0.0),
        false,
    );

    let chevron = frame.chevron_targets[0];
    build_native_dock_overflow_menu(
        &mut tree,
        root,
        NativeDockOverflowMenuParams {
            leaf_id: chevron.leaf,
            anchor_rect: chevron.rect,
            layout_state: &layout_state,
            cursor_pos: Point::new(0.0, 0.0),
            is_cursor_occluded: false,
        },
        &mut frame,
    );

    let overflow_rect = frame
        .active_overflow_rect
        .expect("Overflow rect must exist");

    // Background panel text node placed entirely inside the bounds of the active overflow popup
    let bg_text_id = tree.create_node();
    if let Some(node) = tree.get_mut(bg_text_id) {
        node.set_name("BackgroundPanelText");
        node.set_text("Console Auto-Scroll Log Entry");
        node.computed_rect = Rect::new(overflow_rect.x + 10.0, overflow_rect.y + 10.0, 100.0, 18.0);
        node.set_text_properties(12.0, 16.0, Color::WHITE, TextAlign::Left);
    }
    let _ = tree.add_child(root, bg_text_id);

    let active_dropdowns = [overflow_rect];
    let sections = crate::ui::iris_bridge::IrisEditorOverlay::collect_text_sections_from_tree(
        &tree,
        &active_dropdowns,
        &[],
        &[],
    );

    let rendered_texts: Vec<&str> = sections.iter().map(|s| s.text.as_ref()).collect();

    // Background panel text must be completely occluded by the active overflow menu rect
    assert!(
        !rendered_texts
            .iter()
            .any(|t| t.contains("Console Auto-Scroll Log Entry")),
        "Background panel text must be suppressed by the active dock overflow dropdown"
    );

    // Overflow menu items must NOT be occluded
    assert!(
        rendered_texts.iter().any(|t| t.contains("Console")),
        "Overflow menu items must not be occluded by their own menu rect"
    );
}
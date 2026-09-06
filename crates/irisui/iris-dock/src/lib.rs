// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Docking Engine (`iris-dock`)
//!
//! Generational Binary Split-Tree Docking, multi-tab management,
//! draggable splitter dividers, 5-way drop zones, and Serde layout persistence.
//!
//! Adheres strictly to a zero-unsafe policy (`#![forbid(unsafe_code)]`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod context_menu;
pub mod drag_drop;
pub mod floating;
pub mod layout;
pub mod navigator;
pub mod state;
pub mod style;
pub mod tab_bar;
pub mod tab_viewer;
pub mod tree;
pub mod viewport;

pub use context_menu::{TabContextMenuAction, TabContextMenuState};
pub use drag_drop::{
    DockDragState, DropZone, calculate_drop_preview_rect, calculate_drop_zone,
    calculate_leaf_half_drop_zone, calculate_screen_drop_zone,
};
pub use floating::FloatingWindow;
pub use layout::{
    ComputedDockLayout, ComputedFloatingLayout, DockLayoutOptions, LeafLayoutInfo,
    SplitterLayoutInfo, compute_dock_layout, compute_dock_layout_advanced,
    compute_dock_layout_with_options, compute_dock_layout_with_viewer, compute_floating_layouts,
};
pub use navigator::{
    DockNavigatorGeometry, DockNavigatorStyle, FloatingTabBadgeParams, build_dock_navigator_nodes,
    build_drop_preview_node, build_floating_tab_badge, hit_test_navigator,
};
pub use state::{ActiveSplitterDrag, DockState};
pub use style::DockStyle;
pub use tab_bar::{
    TabBarLayoutInfo, TabLayoutInfo, calculate_tab_reorder_index, compute_tab_bar_layout,
};
pub use tab_viewer::{SimpleTabViewer, TabViewer};
pub use tree::{DockError, DockNode, DockNodeId, DockTree, SplitDirection};
pub use viewport::{FloatingDockWindow, FloatingWindowId, MultiViewportManager};

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::{Point, Rect};

    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    enum PanelKind {
        Hierarchy,
        Viewport,
        Inspector,
        Assets,
    }

    #[test]
    fn test_dock_tree_split_and_tabs() {
        let mut tree = DockTree::new();
        let root_leaf = tree.create_leaf(vec![PanelKind::Hierarchy]);
        tree.set_root(root_leaf);

        let (first, second) = tree
            .split(
                root_leaf,
                SplitDirection::Horizontal,
                0.3,
                vec![PanelKind::Inspector],
            )
            .expect("Split must succeed");

        assert!(tree.get(first).is_some());
        assert!(tree.get(second).is_some());

        tree.add_tab(second, PanelKind::Assets)
            .expect("Adding tab must succeed");

        let leaf = tree.get(second).expect("Second leaf exists");
        if let DockNode::Leaf { tabs, active_tab } = leaf {
            assert_eq!(tabs.len(), 2);
            assert_eq!(*active_tab, 1);
        } else {
            panic!("Expected leaf node");
        }
    }

    #[test]
    fn test_dock_layout_computation() {
        let mut tree = DockTree::new();
        let root_leaf = tree.create_leaf(vec![PanelKind::Viewport]);
        tree.set_root(root_leaf);

        let _ = tree.split(
            root_leaf,
            SplitDirection::Horizontal,
            0.5,
            vec![PanelKind::Inspector],
        );

        let available = Rect::new(0.0, 0.0, 1000.0, 600.0);
        let layout = compute_dock_layout(&tree, available, 4.0, 26.0);

        assert_eq!(layout.leaves.len(), 2);
        assert_eq!(layout.splitters.len(), 1);
        assert_eq!(layout.splitters[0].rect.width, 4.0);
        assert_eq!(layout.splitters[0].rect.height, 600.0);
    }

    #[test]
    fn test_5way_drop_zone_calculation() {
        let rect = Rect::new(100.0, 100.0, 400.0, 400.0);

        // Center (inside 50% core)
        assert_eq!(
            calculate_drop_zone(rect, Point::new(300.0, 300.0)),
            Some(DropZone::Center)
        );

        // Left (<25% margin)
        assert_eq!(
            calculate_drop_zone(rect, Point::new(150.0, 300.0)),
            Some(DropZone::Left)
        );

        // Right (>75% margin)
        assert_eq!(
            calculate_drop_zone(rect, Point::new(450.0, 300.0)),
            Some(DropZone::Right)
        );

        // Top (<25% margin)
        assert_eq!(
            calculate_drop_zone(rect, Point::new(300.0, 150.0)),
            Some(DropZone::Top)
        );

        // Bottom (>75% margin)
        assert_eq!(
            calculate_drop_zone(rect, Point::new(300.0, 450.0)),
            Some(DropZone::Bottom)
        );

        // Outside rect
        assert_eq!(calculate_drop_zone(rect, Point::new(50.0, 50.0)), None);
    }

    #[test]
    fn test_dock_state_serialization_roundtrip() {
        let mut tree = DockTree::new();
        let leaf = tree.create_leaf(vec![PanelKind::Hierarchy, PanelKind::Viewport]);
        tree.set_root(leaf);

        let state = DockState::new(tree);
        let json = state.to_json().expect("Serialization must succeed");
        let restored: DockState<PanelKind> =
            DockState::from_json(&json).expect("Deserialization must succeed");

        assert!(restored.tree.root().is_some());
    }

    #[test]
    fn test_multi_viewport_detaching_and_docking_back() {
        let mut manager = MultiViewportManager::new();
        let main_root = manager.main_tree.create_leaf(vec![PanelKind::Viewport]);
        manager.main_tree.set_root(main_root);

        // Detach Inspector to an independent floating OS window
        let float_id = manager.detach_to_floating_window(
            "Inspector",
            (100, 100),
            (400, 600),
            vec![PanelKind::Inspector],
        );

        assert_eq!(manager.floating_windows.len(), 1);
        assert!(manager.floating_windows.contains_key(&float_id));

        // Dock the floating window back into main tree (Center tab)
        let res = manager.dock_floating_window_back(float_id, main_root, DropZone::Center);
        assert!(res.is_ok());
        assert_eq!(manager.floating_windows.len(), 0);

        let root_node = manager.main_tree.get(main_root).unwrap();
        if let DockNode::Leaf { tabs, .. } = root_node {
            assert_eq!(tabs.len(), 2);
            assert_eq!(tabs[0], PanelKind::Viewport);
            assert_eq!(tabs[1], PanelKind::Inspector);
        } else {
            panic!("Expected leaf node");
        }
    }
}
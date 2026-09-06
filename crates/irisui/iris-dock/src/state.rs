// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! High-level dock controller coordinating splitters, drag-drop targets, and persistence.

use crate::context_menu::TabContextMenuState;
use crate::drag_drop::{DockDragState, DropZone, calculate_drop_preview_rect, calculate_drop_zone};
use crate::floating::FloatingWindow;
use crate::layout::ComputedDockLayout;
use crate::tree::{DockError, DockNode, DockNodeId, DockTree, SplitDirection};
use iris_core::{Point, Rect};
use serde::{Deserialize, Serialize};

/// Active state while dragging a splitter divider to resize adjacent panes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveSplitterDrag {
    /// Target split container node identifier.
    pub node_id: DockNodeId,
    /// Partition axis.
    pub direction: SplitDirection,
    /// Initial cursor coordinate along the division axis.
    pub start_cursor: f32,
    /// Initial division ratio before the drag started.
    pub start_ratio: f32,
    /// Total available dimension across both split children.
    pub total_dimension: f32,
}

/// Central controller managing docking tree mutations, dragging interactions, and layout serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct DockState<T> {
    /// Underlying binary split-tree.
    pub tree: DockTree<T>,
    /// Minimum allowable pane dimension in logical pixels along the partition axis.
    #[serde(default = "default_min_pane_size")]
    pub min_pane_size: f32,
    /// Active tab drag-and-drop state, if a tab is currently being moved.
    #[serde(skip)]
    pub active_drag: Option<DockDragState<T>>,
    /// Active splitter drag state, if a divider is currently being resized.
    #[serde(skip)]
    pub active_splitter: Option<ActiveSplitterDrag>,
    /// Identifier of a leaf temporarily maximized to fill the entire dock area.
    #[serde(default)]
    pub maximized_leaf: Option<DockNodeId>,
    /// Collection of independent floating windows detached from the main dock hierarchy.
    #[serde(skip)]
    pub floating_windows: Vec<FloatingWindow<T>>,
    /// Whether to automatically collapse single-tab bars to 0 height.
    #[serde(default)]
    pub auto_hide_single_tab_bar: bool,
    /// Active tab context menu state, if a right-click menu is currently open.
    #[serde(skip)]
    pub active_context_menu: Option<TabContextMenuState>,
}

fn default_min_pane_size() -> f32 {
    60.0
}

impl<T> Default for DockState<T> {
    fn default() -> Self {
        Self::new(DockTree::new())
    }
}

impl<T> DockState<T> {
    /// Creates a new `DockState` wrapping the specified docking tree with default minimum pane size (60.0 px).
    pub fn new(tree: DockTree<T>) -> Self {
        Self {
            tree,
            min_pane_size: default_min_pane_size(),
            active_drag: None,
            active_splitter: None,
            maximized_leaf: None,
            floating_windows: Vec::new(),
            auto_hide_single_tab_bar: false,
            active_context_menu: None,
        }
    }

    /// Sets the minimum allowable pane dimension in logical pixels.
    pub fn with_min_pane_size(mut self, min_size: f32) -> Self {
        self.min_pane_size = min_size.max(10.0);
        self
    }

    /// Initiates a splitter resize drag operation on the specified split node.
    pub fn start_splitter_drag(
        &mut self,
        node_id: DockNodeId,
        direction: SplitDirection,
        start_cursor: f32,
        total_dimension: f32,
    ) {
        if let Some(DockNode::Split { ratio, .. }) = self.tree.get(node_id) {
            self.active_splitter = Some(ActiveSplitterDrag {
                node_id,
                direction,
                start_cursor,
                start_ratio: *ratio,
                total_dimension: total_dimension.max(1.0),
            });
        }
    }

    /// Updates the division ratio of the actively dragged splitter based on new cursor position.
    /// Clamps the ratio mathematically so that neither child pane shrinks below `min_pane_size`.
    pub fn update_splitter_drag(&mut self, current_cursor: f32) {
        if let Some(drag) = self.active_splitter {
            let delta = current_cursor - drag.start_cursor;
            let ratio_delta = delta / drag.total_dimension;

            let min_ratio = (self.min_pane_size / drag.total_dimension).clamp(0.01, 0.45);
            let max_ratio = (1.0 - min_ratio).max(min_ratio);

            let new_ratio = (drag.start_ratio + ratio_delta).clamp(min_ratio, max_ratio);
            let _ = self.tree.set_split_ratio(drag.node_id, new_ratio);
        }
    }

    /// Resets the specified splitter partition ratio back to an even 50/50 balance.
    /// Typically triggered by double-clicking on a divider line.
    pub fn reset_splitter(&mut self, node_id: DockNodeId) -> Result<(), DockError> {
        self.tree.set_split_ratio(node_id, 0.5)
    }

    /// Finalizes the active splitter drag operation.
    pub fn end_splitter_drag(&mut self) {
        self.active_splitter = None;
    }

    /// Detaches a tab from a leaf node and initiates a drag-and-drop operation.
    /// Stores the source leaf rectangle so detached floating windows preserve their exact
    /// dimensions from the docked workspace.
    pub fn start_tab_drag(
        &mut self,
        source_leaf: DockNodeId,
        tab_idx: usize,
        cursor: Point,
        source_rect: Rect,
    ) -> Result<(), DockError> {
        let tab_data = self.tree.remove_tab(source_leaf, tab_idx)?;
        self.active_drag = Some(DockDragState {
            source_leaf,
            source_tab_index: tab_idx,
            tab_data,
            cursor_pos: cursor,
            source_rect,
            hover_target: None,
            tab_reorder_target: None,
        });
        Ok(())
    }

    /// Detaches a tab from a floating window and initiates a drag-and-drop operation into the dock tree.
    /// If the floating window becomes empty, it is automatically removed.
    pub fn start_floating_tab_drag(
        &mut self,
        window_id: u64,
        source_leaf: DockNodeId,
        tab_idx: usize,
        cursor: Point,
    ) -> Result<(), DockError> {
        let win = self
            .floating_windows
            .iter_mut()
            .find(|w| w.id == window_id)
            .ok_or(DockError::NodeNotFound)?;
        let source_rect = win.rect;
        let tab_data = win.tree.remove_tab(source_leaf, tab_idx)?;
        win.tree.collapse_empty_leaves();
        let is_empty = win.tree.root().is_none()
            || win.tree.iter().all(|(_, n)| match n {
                DockNode::Leaf { tabs, .. } => tabs.is_empty(),
                DockNode::Split { .. } => false,
            });
        if is_empty {
            self.floating_windows.retain(|w| w.id != window_id);
        }
        let fallback_leaf = self.tree.find_first_leaf().unwrap_or(source_leaf);
        self.active_drag = Some(DockDragState {
            source_leaf: fallback_leaf,
            source_tab_index: 0,
            tab_data,
            cursor_pos: cursor,
            source_rect,
            hover_target: None,
            tab_reorder_target: None,
        });
        Ok(())
    }

    /// Updates the drag position and recomputes the active hover drop zone preview or tab reorder target.
    pub fn update_tab_drag(&mut self, cursor: Point, layout: &ComputedDockLayout<T>) {
        if let Some(ref mut drag) = self.active_drag {
            drag.cursor_pos = cursor;
            drag.hover_target = None;
            drag.tab_reorder_target = None;

            // 1. Check if hovering over any leaf's tab bar for tab reordering / insertion
            for leaf in &layout.leaves {
                if let Some(ref tab_bar) = leaf.tab_bar_layout {
                    if let Some(insert_idx) =
                        crate::tab_bar::calculate_tab_reorder_index(tab_bar, cursor)
                    {
                        drag.tab_reorder_target = Some((leaf.node_id, insert_idx));
                        return;
                    }
                } else if leaf.tab_bar_rect.contains_point(cursor) {
                    drag.tab_reorder_target = Some((leaf.node_id, leaf.tabs.len()));
                    return;
                }
            }

            // 2. Check 5-way leaf content drop zones
            for leaf in &layout.leaves {
                if let Some(zone) = calculate_drop_zone(leaf.content_rect, cursor) {
                    let preview_rect = calculate_drop_preview_rect(leaf.content_rect, zone);
                    drag.hover_target = Some((leaf.node_id, zone, preview_rect));
                    return;
                }
            }
        }
    }

    /// Drops the actively dragged tab into its target drop zone, tab strip, or restores it to its source.
    pub fn drop_tab(&mut self) -> Result<(), DockError> {
        let Some(drag) = self.active_drag.take() else {
            return Ok(());
        };

        // 1. Dropped on a tab bar strip to reorder or insert at index
        if let Some((target_leaf, target_idx)) = drag.tab_reorder_target {
            let _ = self.tree.add_tab(target_leaf, drag.tab_data);
            let current_len = match self.tree.get(target_leaf) {
                Some(DockNode::Leaf { tabs, .. }) => tabs.len(),
                _ => 0,
            };
            if current_len > 0 {
                let _ = self
                    .tree
                    .move_tab(target_leaf, current_len - 1, target_leaf, target_idx);
            }
        } else if let Some((target_leaf, zone, _)) = drag.hover_target {
            match zone {
                DropZone::Center => {
                    self.tree.add_tab(target_leaf, drag.tab_data)?;
                }
                DropZone::Left | DropZone::Right | DropZone::Top | DropZone::Bottom => {
                    self.tree.dock_tab(target_leaf, zone, drag.tab_data)?;
                }
                DropZone::ScreenLeft
                | DropZone::ScreenRight
                | DropZone::ScreenTop
                | DropZone::ScreenBottom => {
                    self.tree.dock_root(zone, drag.tab_data)?;
                }
            }
        } else {
            // Restore back to source leaf if dropped in empty space
            let _ = self.tree.add_tab(drag.source_leaf, drag.tab_data);
        }

        self.tree.collapse_empty_leaves();
        Ok(())
    }

    /// Drops the actively dragged tab into its target drop zone, tab strip, or detaches it as a floating window.
    /// If dropped outside any docking drop zones or tab strips, detaches into an independent floating window.
    pub fn drop_tab_or_float(&mut self, default_size: Point) -> Result<Option<u64>, DockError> {
        let Some(drag) = self.active_drag.take() else {
            return Ok(None);
        };

        if let Some((target_leaf, target_idx)) = drag.tab_reorder_target {
            let _ = self.tree.add_tab(target_leaf, drag.tab_data);
            let current_len = match self.tree.get(target_leaf) {
                Some(DockNode::Leaf { tabs, .. }) => tabs.len(),
                _ => 0,
            };
            if current_len > 0 {
                let _ = self
                    .tree
                    .move_tab(target_leaf, current_len - 1, target_leaf, target_idx);
            }
            self.tree.collapse_empty_leaves();
            Ok(None)
        } else if let Some((target_leaf, zone, _)) = drag.hover_target {
            match zone {
                DropZone::Center => {
                    self.tree.add_tab(target_leaf, drag.tab_data)?;
                }
                DropZone::Left | DropZone::Right | DropZone::Top | DropZone::Bottom => {
                    self.tree.dock_tab(target_leaf, zone, drag.tab_data)?;
                }
                DropZone::ScreenLeft
                | DropZone::ScreenRight
                | DropZone::ScreenTop
                | DropZone::ScreenBottom => {
                    self.tree.dock_root(zone, drag.tab_data)?;
                }
            }
            self.tree.collapse_empty_leaves();
            Ok(None)
        } else {
            // Detach as floating window preserving source pane dimensions
            let next_id = self
                .floating_windows
                .iter()
                .map(|w| w.id)
                .max()
                .unwrap_or(0)
                + 1;
            let target_w = if drag.source_rect.width > 50.0 {
                drag.source_rect.width
            } else {
                default_size.x.max(260.0)
            };
            let target_h = if drag.source_rect.height > 50.0 {
                drag.source_rect.height
            } else {
                default_size.y.max(180.0)
            };
            let rect = Rect::new(
                (drag.cursor_pos.x - 80.0).max(10.0),
                (drag.cursor_pos.y - 14.0).max(10.0),
                target_w,
                target_h,
            );
            let window = FloatingWindow::new(next_id, String::new(), rect, vec![drag.tab_data]);
            self.floating_windows.push(window);
            self.tree.collapse_empty_leaves();
            Ok(Some(next_id))
        }
    }

    /// Maximizes the specified leaf to temporarily occupy the entire dock area.
    pub fn maximize_leaf(&mut self, leaf_id: DockNodeId) {
        if self.tree.get(leaf_id).is_some() {
            self.maximized_leaf = Some(leaf_id);
        }
    }

    /// Restores the normal multi-pane layout from maximized state.
    pub fn restore(&mut self) {
        self.maximized_leaf = None;
    }

    /// Returns `true` if any leaf is currently maximized.
    pub fn is_maximized(&self) -> bool {
        self.maximized_leaf.is_some()
    }

    /// Toggles the maximization state of the specified leaf.
    pub fn toggle_maximize(&mut self, leaf_id: DockNodeId) {
        if self.maximized_leaf == Some(leaf_id) {
            self.maximized_leaf = None;
        } else {
            self.maximize_leaf(leaf_id);
        }
    }

    /// Configures whether single-tab bars should be collapsed to 0 height.
    pub fn set_auto_hide_single_tab_bar(&mut self, auto_hide: bool) {
        self.auto_hide_single_tab_bar = auto_hide;
    }

    /// Detaches a tab from the main dock tree and converts it into an independent floating window.
    /// Returns the unique identifier of the created floating window.
    pub fn detach_tab_to_floating(
        &mut self,
        leaf: DockNodeId,
        tab_idx: usize,
        title: impl Into<String>,
        position: Point,
        size: Point,
    ) -> Result<u64, DockError> {
        let tab = self.tree.remove_tab(leaf, tab_idx)?;
        self.tree.collapse_empty_leaves();
        if self.maximized_leaf == Some(leaf) && self.tree.get(leaf).is_none() {
            self.maximized_leaf = None;
        }

        let next_id = self
            .floating_windows
            .iter()
            .map(|w| w.id)
            .max()
            .unwrap_or(0)
            + 1;
        let rect = Rect::new(position.x, position.y, size.x.max(120.0), size.y.max(80.0));
        let window = FloatingWindow::new(next_id, title, rect, vec![tab]);
        self.floating_windows.push(window);
        Ok(next_id)
    }

    /// Closes and removes the specified floating window by ID.
    pub fn close_floating_window(&mut self, window_id: u64) -> Option<FloatingWindow<T>> {
        if let Some(pos) = self.floating_windows.iter().position(|w| w.id == window_id) {
            Some(self.floating_windows.remove(pos))
        } else {
            None
        }
    }

    /// Docks all tabs from a floating window back into the main tree at the specified target leaf and zone.
    pub fn dock_floating_window(
        &mut self,
        window_id: u64,
        target_leaf: DockNodeId,
        zone: DropZone,
    ) -> Result<(), DockError> {
        let Some(pos) = self.floating_windows.iter().position(|w| w.id == window_id) else {
            return Err(DockError::NodeNotFound);
        };
        let mut window = self.floating_windows.remove(pos);

        let mut tabs = Vec::new();
        if let Some(root) = window.tree.root() {
            collect_all_tabs_recursive(&mut window.tree, root, &mut tabs);
        }

        if tabs.is_empty() {
            return Ok(());
        }

        // Validate that target_leaf is indeed a Leaf node; if not, safely fallback to first leaf
        let resolved_leaf = if matches!(self.tree.get(target_leaf), Some(DockNode::Leaf { .. })) {
            target_leaf
        } else if let Some(first_leaf) = self.tree.find_first_leaf() {
            first_leaf
        } else {
            // Tree is entirely empty; create root leaf directly
            let new_leaf = self.tree.create_leaf(tabs);
            self.tree.set_root(new_leaf);
            return Ok(());
        };

        for tab in tabs {
            self.tree.dock_tab(resolved_leaf, zone, tab)?;
        }

        self.tree.collapse_empty_leaves();
        Ok(())
    }

    /// Serializes the docking state to a JSON string for layout persistence.
    pub fn to_json(&self) -> Result<String, serde_json::Error>
    where
        T: Serialize,
    {
        serde_json::to_string_pretty(self)
    }

    /// Deserializes a docking state from a JSON string.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error>
    where
        for<'de> T: Deserialize<'de>,
    {
        serde_json::from_str(json_str)
    }
}

fn collect_all_tabs_recursive<T>(tree: &mut DockTree<T>, node_id: DockNodeId, out: &mut Vec<T>) {
    if let Some(node) = tree.get_mut(node_id) {
        match node {
            DockNode::Leaf { tabs, .. } => {
                out.append(tabs);
            }
            DockNode::Split { first, second, .. } => {
                let f = *first;
                let s = *second;
                collect_all_tabs_recursive(tree, f, out);
                collect_all_tabs_recursive(tree, s, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        state.active_drag.as_mut().unwrap().hover_target =
            Some((leaf, DropZone::Center, Rect::ZERO));
        state.drop_tab().expect("Drop into tree succeeds");
        assert!(state.active_drag.is_none());
        assert!(state.tree.find_tab(&"Tab 1").is_some());
        assert!(state.tree.find_tab(&"Tab 2").is_some());
    }
}
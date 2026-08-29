// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! High-level dock controller coordinating splitters, drag-drop targets, and persistence.

use crate::drag_drop::{DockDragState, DropZone, calculate_drop_preview_rect, calculate_drop_zone};
use crate::layout::ComputedDockLayout;
use crate::tree::{DockError, DockNode, DockNodeId, DockTree, SplitDirection};
use iris_core::Point;
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
    /// Active tab drag-and-drop state, if a tab is currently being moved.
    #[serde(skip)]
    pub active_drag: Option<DockDragState<T>>,
    /// Active splitter drag state, if a divider is currently being resized.
    #[serde(skip)]
    pub active_splitter: Option<ActiveSplitterDrag>,
}

impl<T> Default for DockState<T> {
    fn default() -> Self {
        Self::new(DockTree::new())
    }
}

impl<T> DockState<T> {
    /// Creates a new `DockState` wrapping the specified docking tree.
    pub fn new(tree: DockTree<T>) -> Self {
        Self {
            tree,
            active_drag: None,
            active_splitter: None,
        }
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
    pub fn update_splitter_drag(&mut self, current_cursor: f32) {
        if let Some(drag) = self.active_splitter {
            let delta = current_cursor - drag.start_cursor;
            let ratio_delta = delta / drag.total_dimension;
            let new_ratio = (drag.start_ratio + ratio_delta).clamp(0.05, 0.95);
            let _ = self.tree.set_split_ratio(drag.node_id, new_ratio);
        }
    }

    /// Finalizes the active splitter drag operation.
    pub fn end_splitter_drag(&mut self) {
        self.active_splitter = None;
    }

    /// Detaches a tab from a leaf node and initiates a drag-and-drop operation.
    pub fn start_tab_drag(
        &mut self,
        source_leaf: DockNodeId,
        tab_idx: usize,
        cursor: Point,
    ) -> Result<(), DockError> {
        let tab_data = self.tree.remove_tab(source_leaf, tab_idx)?;
        self.active_drag = Some(DockDragState {
            source_leaf,
            source_tab_index: tab_idx,
            tab_data,
            cursor_pos: cursor,
            hover_target: None,
        });
        Ok(())
    }

    /// Updates the drag position and recomputes the active hover drop zone preview.
    pub fn update_tab_drag(&mut self, cursor: Point, layout: &ComputedDockLayout<T>) {
        if let Some(ref mut drag) = self.active_drag {
            drag.cursor_pos = cursor;
            drag.hover_target = None;

            for leaf in &layout.leaves {
                if let Some(zone) = calculate_drop_zone(leaf.content_rect, cursor) {
                    let preview_rect = calculate_drop_preview_rect(leaf.content_rect, zone);
                    drag.hover_target = Some((leaf.node_id, zone, preview_rect));
                    break;
                }
            }
        }
    }

    /// Drops the actively dragged tab into its target drop zone or restores it to its source.
    pub fn drop_tab(&mut self) -> Result<(), DockError> {
        let Some(drag) = self.active_drag.take() else {
            return Ok(());
        };

        if let Some((target_leaf, zone, _)) = drag.hover_target {
            match zone {
                DropZone::Center => {
                    self.tree.add_tab(target_leaf, drag.tab_data)?;
                }
                DropZone::Left => {
                    self.tree.split(
                        target_leaf,
                        SplitDirection::Horizontal,
                        0.5,
                        vec![drag.tab_data],
                    )?;
                }
                DropZone::Right => {
                    let (first, second) = self.tree.split(
                        target_leaf,
                        SplitDirection::Horizontal,
                        0.5,
                        vec![drag.tab_data],
                    )?;
                    // Swap children so new tab is placed on the right
                    if let Some(DockNode::Split {
                        first: f,
                        second: s,
                        ..
                    }) = self.tree.get_mut(target_leaf)
                    {
                        *f = second;
                        *s = first;
                    }
                }
                DropZone::Top => {
                    self.tree.split(
                        target_leaf,
                        SplitDirection::Vertical,
                        0.5,
                        vec![drag.tab_data],
                    )?;
                }
                DropZone::Bottom => {
                    let (first, second) = self.tree.split(
                        target_leaf,
                        SplitDirection::Vertical,
                        0.5,
                        vec![drag.tab_data],
                    )?;
                    // Swap children so new tab is placed on the bottom
                    if let Some(DockNode::Split {
                        first: f,
                        second: s,
                        ..
                    }) = self.tree.get_mut(target_leaf)
                    {
                        *f = second;
                        *s = first;
                    }
                }
            }
        } else {
            // Restore back to source leaf if dropped in empty space
            let _ = self.tree.add_tab(drag.source_leaf, drag.tab_data);
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
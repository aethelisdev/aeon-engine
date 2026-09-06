// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generational Binary Split-Tree data structure for panel docking and multi-tab layouts.

use serde::{Deserialize, Serialize};
use slotmap::{SlotMap, new_key_type};
use thiserror::Error;

new_key_type! {
    /// Generational key indexing a node within the docking hierarchy.
    pub struct DockNodeId;
}

/// Errors returned during docking tree operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DockError {
    /// Specified dock node identifier was not found in the arena.
    #[error("Dock node was not found in tree arena")]
    NodeNotFound,
    /// Specified operation required a leaf node, but a split container was provided.
    #[error("Expected a leaf node containing tabs, found a split container")]
    ExpectedLeafNode,
    /// Specified operation required a split container, but a leaf node was provided.
    #[error("Expected a split container, found a leaf node")]
    ExpectedSplitNode,
    /// Tab index was out of bounds for the target leaf.
    #[error("Tab index out of bounds")]
    IndexOutOfBounds,
}

/// Direction along which a dock node is partitioned into two child regions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SplitDirection {
    /// Divides the region horizontally (left and right sub-regions).
    Horizontal,
    /// Divides the region vertically (top and bottom sub-regions).
    Vertical,
}

/// A node within the binary split-tree, representing either a container split or a tab leaf.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DockNode<T> {
    /// Container node partitioning available space between two children.
    Split {
        /// Partition axis (horizontal or vertical).
        direction: SplitDirection,
        /// Fraction of dimension allocated to the first child in range `[0.05, 0.95]`.
        ratio: f32,
        /// Identifier of the first (left or top) child node.
        first: DockNodeId,
        /// Identifier of the second (right or bottom) child node.
        second: DockNodeId,
    },
    /// Terminal leaf node hosting one or more tabbed panels.
    Leaf {
        /// List of tabs currently hosted inside this leaf.
        tabs: Vec<T>,
        /// Index of the currently visible and active tab.
        active_tab: usize,
    },
}

/// Safe, arena-based hierarchical tree governing panel splits, tabs, and layout persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockTree<T> {
    /// Generational storage arena containing all active docking nodes.
    nodes: SlotMap<DockNodeId, DockNode<T>>,
    /// Root node identifier of the docking tree, if present.
    root: Option<DockNodeId>,
    /// Identifier of the currently focused leaf node, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    focused_leaf: Option<DockNodeId>,
}

impl<T> Default for DockTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DockTree<T> {
    /// Constructs a new, empty docking tree.
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
            focused_leaf: None,
        }
    }

    /// Returns the root node key of the tree, if set.
    #[inline]
    pub fn root(&self) -> Option<DockNodeId> {
        self.root
    }

    /// Sets the specified node as the root of the tree.
    #[inline]
    pub fn set_root(&mut self, root_id: DockNodeId) {
        self.root = Some(root_id);
    }

    /// Returns the key of the currently focused leaf node, if set.
    #[inline]
    pub fn focused_leaf(&self) -> Option<DockNodeId> {
        self.focused_leaf
    }

    /// Updates or clears the currently focused leaf node key.
    #[inline]
    pub fn set_focused_leaf(&mut self, leaf_id: Option<DockNodeId>) {
        self.focused_leaf = leaf_id;
    }

    /// Retrieves an immutable reference to a dock node by its key.
    #[inline]
    pub fn get(&self, id: DockNodeId) -> Option<&DockNode<T>> {
        self.nodes.get(id)
    }

    /// Retrieves a mutable reference to a dock node by its key.
    #[inline]
    pub fn get_mut(&mut self, id: DockNodeId) -> Option<&mut DockNode<T>> {
        self.nodes.get_mut(id)
    }

    /// Returns an iterator over all node pairs in the docking tree.
    pub fn iter(&self) -> impl Iterator<Item = (DockNodeId, &DockNode<T>)> {
        self.nodes.iter()
    }

    /// Extracts all hosted tabs across all leaves in the tree.
    pub fn all_tabs(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut tabs = Vec::new();
        for node in self.nodes.values() {
            if let DockNode::Leaf {
                tabs: leaf_tabs, ..
            } = node
            {
                tabs.extend(leaf_tabs.clone());
            }
        }
        tabs
    }

    /// Creates a new terminal leaf node hosting the provided tabs.
    pub fn create_leaf(&mut self, tabs: Vec<T>) -> DockNodeId {
        self.nodes.insert(DockNode::Leaf {
            tabs,
            active_tab: 0,
        })
    }

    /// Partitions an existing leaf node into a split container holding two leaves.
    /// Placed tabs default to the second child (right or bottom) for backwards compatibility.
    pub fn split(
        &mut self,
        target_leaf: DockNodeId,
        direction: SplitDirection,
        ratio: f32,
        new_tabs: Vec<T>,
    ) -> Result<(DockNodeId, DockNodeId), DockError> {
        self.split_ordered(target_leaf, direction, ratio, new_tabs, false)
    }

    /// Partitions an existing leaf node into a split container with explicit child ordering.
    /// When `new_is_first` is `true`, `new_tabs` are placed in the first child (left or top)
    /// and existing tabs in the second child (right or bottom).
    pub fn split_ordered(
        &mut self,
        target_leaf: DockNodeId,
        direction: SplitDirection,
        ratio: f32,
        new_tabs: Vec<T>,
        new_is_first: bool,
    ) -> Result<(DockNodeId, DockNodeId), DockError> {
        let (existing_tabs, existing_active) = match self.nodes.get_mut(target_leaf) {
            Some(DockNode::Leaf { tabs, active_tab }) => (std::mem::take(tabs), *active_tab),
            Some(DockNode::Split { .. }) => return Err(DockError::ExpectedLeafNode),
            None => return Err(DockError::NodeNotFound),
        };

        let (first_id, second_id) = if new_is_first {
            let first = self.nodes.insert(DockNode::Leaf {
                tabs: new_tabs,
                active_tab: 0,
            });
            let second = self.nodes.insert(DockNode::Leaf {
                tabs: existing_tabs,
                active_tab: existing_active,
            });
            (first, second)
        } else {
            let first = self.nodes.insert(DockNode::Leaf {
                tabs: existing_tabs,
                active_tab: existing_active,
            });
            let second = self.nodes.insert(DockNode::Leaf {
                tabs: new_tabs,
                active_tab: 0,
            });
            (first, second)
        };

        if let Some(node) = self.nodes.get_mut(target_leaf) {
            *node = DockNode::Split {
                direction,
                ratio: ratio.clamp(0.05, 0.95),
                first: first_id,
                second: second_id,
            };
        }

        Ok((first_id, second_id))
    }

    /// Partitions an existing leaf according to a specified drop zone, inserting the tab safely.
    /// - [`crate::drag_drop::DropZone::Center`]: Appends `tab` into the target leaf without partitioning.
    /// - [`crate::drag_drop::DropZone::Left`]: Splits horizontally with ratio 0.5; `tab` is placed in the left child.
    /// - [`crate::drag_drop::DropZone::Right`]: Splits horizontally with ratio 0.5; `tab` is placed in the right child.
    /// - [`crate::drag_drop::DropZone::Top`]: Splits vertically with ratio 0.5; `tab` is placed in the top child.
    /// - [`crate::drag_drop::DropZone::Bottom`]: Splits vertically with ratio 0.5; `tab` is placed in the bottom child.
    /// Returns the ID of the leaf hosting the newly docked tab.
    pub fn dock_tab(
        &mut self,
        target_leaf: DockNodeId,
        zone: crate::drag_drop::DropZone,
        tab: T,
    ) -> Result<DockNodeId, DockError> {
        match zone {
            crate::drag_drop::DropZone::Center => {
                self.add_tab(target_leaf, tab)?;
                Ok(target_leaf)
            }
            crate::drag_drop::DropZone::Left => {
                let (first, _second) = self.split_ordered(
                    target_leaf,
                    SplitDirection::Horizontal,
                    0.5,
                    vec![tab],
                    true,
                )?;
                Ok(first)
            }
            crate::drag_drop::DropZone::Right => {
                let (_first, second) = self.split_ordered(
                    target_leaf,
                    SplitDirection::Horizontal,
                    0.5,
                    vec![tab],
                    false,
                )?;
                Ok(second)
            }
            crate::drag_drop::DropZone::Top => {
                let (first, _second) = self.split_ordered(
                    target_leaf,
                    SplitDirection::Vertical,
                    0.5,
                    vec![tab],
                    true,
                )?;
                Ok(first)
            }
            crate::drag_drop::DropZone::Bottom => {
                let (_first, second) = self.split_ordered(
                    target_leaf,
                    SplitDirection::Vertical,
                    0.5,
                    vec![tab],
                    false,
                )?;
                Ok(second)
            }
            crate::drag_drop::DropZone::ScreenLeft
            | crate::drag_drop::DropZone::ScreenRight
            | crate::drag_drop::DropZone::ScreenTop
            | crate::drag_drop::DropZone::ScreenBottom => self.dock_root(zone, tab),
        }
    }

    /// Partitions the entire root tree along an outer window edge, placing the tab in the new outer region.
    pub fn dock_root(
        &mut self,
        zone: crate::drag_drop::DropZone,
        tab: T,
    ) -> Result<DockNodeId, DockError> {
        let Some(current_root) = self.root else {
            let new_leaf = self.create_leaf(vec![tab]);
            self.root = Some(new_leaf);
            return Ok(new_leaf);
        };

        let (direction, new_is_first) = match zone {
            crate::drag_drop::DropZone::ScreenLeft => (SplitDirection::Horizontal, true),
            crate::drag_drop::DropZone::ScreenRight => (SplitDirection::Horizontal, false),
            crate::drag_drop::DropZone::ScreenTop => (SplitDirection::Vertical, true),
            crate::drag_drop::DropZone::ScreenBottom => (SplitDirection::Vertical, false),
            _ => return Err(DockError::ExpectedSplitNode),
        };

        let new_leaf = self.create_leaf(vec![tab]);
        let (first, second) = if new_is_first {
            (new_leaf, current_root)
        } else {
            (current_root, new_leaf)
        };

        let new_root = self.nodes.insert(DockNode::Split {
            direction,
            ratio: 0.5,
            first,
            second,
        });
        self.root = Some(new_root);
        Ok(new_leaf)
    }

    /// Appends a new tab into the specified leaf node.
    pub fn add_tab(&mut self, target_leaf: DockNodeId, tab: T) -> Result<usize, DockError> {
        let node = self
            .nodes
            .get_mut(target_leaf)
            .ok_or(DockError::NodeNotFound)?;
        match node {
            DockNode::Leaf { tabs, active_tab } => {
                tabs.push(tab);
                *active_tab = tabs.len() - 1;
                Ok(*active_tab)
            }
            DockNode::Split { .. } => Err(DockError::ExpectedLeafNode),
        }
    }

    /// Removes a tab at the specified index from a leaf node.
    pub fn remove_tab(&mut self, target_leaf: DockNodeId, tab_idx: usize) -> Result<T, DockError> {
        let node = self
            .nodes
            .get_mut(target_leaf)
            .ok_or(DockError::NodeNotFound)?;
        match node {
            DockNode::Leaf { tabs, active_tab } => {
                if tab_idx >= tabs.len() {
                    return Err(DockError::IndexOutOfBounds);
                }
                let removed = tabs.remove(tab_idx);
                if *active_tab >= tabs.len() && !tabs.is_empty() {
                    *active_tab = tabs.len() - 1;
                }
                Ok(removed)
            }
            DockNode::Split { .. } => Err(DockError::ExpectedLeafNode),
        }
    }

    /// Updates the active tab index of a leaf node.
    pub fn set_active_tab(
        &mut self,
        target_leaf: DockNodeId,
        tab_idx: usize,
    ) -> Result<(), DockError> {
        let node = self
            .nodes
            .get_mut(target_leaf)
            .ok_or(DockError::NodeNotFound)?;
        match node {
            DockNode::Leaf { tabs, active_tab } => {
                if tab_idx < tabs.len() {
                    *active_tab = tab_idx;
                    Ok(())
                } else {
                    Err(DockError::IndexOutOfBounds)
                }
            }
            DockNode::Split { .. } => Err(DockError::ExpectedLeafNode),
        }
    }

    /// Searches the tree for the specified tab by value equality.
    /// Returns the host [`DockNodeId`] and tab index within that leaf if found.
    pub fn find_tab(&self, tab: &T) -> Option<(DockNodeId, usize)>
    where
        T: PartialEq,
    {
        for (id, node) in self.nodes.iter() {
            if let DockNode::Leaf { tabs, .. } = node
                && let Some(idx) = tabs.iter().position(|t| t == tab)
            {
                return Some((id, idx));
            }
        }
        None
    }

    /// Appends a tab into the currently focused leaf, or the first available leaf in the tree.
    /// If the tree is entirely empty, a new root leaf is created hosting the tab.
    /// Returns the target [`DockNodeId`] and the inserted tab index.
    pub fn push_to_focused_leaf(&mut self, tab: T) -> Result<(DockNodeId, usize), DockError> {
        if let Some(focused) = self.focused_leaf
            && let Some(DockNode::Leaf { .. }) = self.nodes.get(focused)
        {
            let idx = self.add_tab(focused, tab)?;
            return Ok((focused, idx));
        }

        for (id, node) in self.nodes.iter() {
            if let DockNode::Leaf { .. } = node {
                let idx = self.add_tab(id, tab)?;
                self.focused_leaf = Some(id);
                return Ok((id, idx));
            }
        }

        let new_leaf = self.create_leaf(vec![tab]);
        self.root = Some(new_leaf);
        self.focused_leaf = Some(new_leaf);
        Ok((new_leaf, 0))
    }

    /// Returns the identifier of the first leaf node found in the tree, if any exists.
    /// Useful for safely targeting dock operations when canonical partner leaves are missing,
    /// preventing invalid attempts to dock into non-leaf split nodes.
    pub fn find_first_leaf(&self) -> Option<DockNodeId> {
        for (id, node) in self.nodes.iter() {
            if let DockNode::Leaf { .. } = node {
                return Some(id);
            }
        }
        None
    }

    /// Splits an existing leaf horizontally, placing the new tab in the left sub-pane.
    pub fn split_left(&mut self, target_leaf: DockNodeId, tab: T) -> Result<DockNodeId, DockError> {
        self.dock_tab(target_leaf, crate::drag_drop::DropZone::Left, tab)
    }

    /// Splits an existing leaf horizontally, placing the new tab in the right sub-pane.
    pub fn split_right(
        &mut self,
        target_leaf: DockNodeId,
        tab: T,
    ) -> Result<DockNodeId, DockError> {
        self.dock_tab(target_leaf, crate::drag_drop::DropZone::Right, tab)
    }

    /// Splits an existing leaf vertically, placing the new tab in the top sub-pane.
    pub fn split_above(
        &mut self,
        target_leaf: DockNodeId,
        tab: T,
    ) -> Result<DockNodeId, DockError> {
        self.dock_tab(target_leaf, crate::drag_drop::DropZone::Top, tab)
    }

    /// Splits an existing leaf vertically, placing the new tab in the bottom sub-pane.
    pub fn split_below(
        &mut self,
        target_leaf: DockNodeId,
        tab: T,
    ) -> Result<DockNodeId, DockError> {
        self.dock_tab(target_leaf, crate::drag_drop::DropZone::Bottom, tab)
    }

    /// Moves a tab from a specific position to a new target index, either within the same leaf or across leaves.
    /// Clamps destination index if it exceeds the target leaf's tab count.
    pub fn move_tab(
        &mut self,
        from_leaf: DockNodeId,
        from_idx: usize,
        to_leaf: DockNodeId,
        to_idx: usize,
    ) -> Result<(), DockError> {
        if from_leaf == to_leaf {
            let node = self
                .nodes
                .get_mut(from_leaf)
                .ok_or(DockError::NodeNotFound)?;
            if let DockNode::Leaf { tabs, active_tab } = node {
                if from_idx >= tabs.len() {
                    return Err(DockError::IndexOutOfBounds);
                }
                let tab = tabs.remove(from_idx);
                let target_idx = to_idx.min(tabs.len());
                tabs.insert(target_idx, tab);
                *active_tab = target_idx;
                Ok(())
            } else {
                Err(DockError::ExpectedLeafNode)
            }
        } else {
            let tab = self.remove_tab(from_leaf, from_idx)?;
            let target_node = self.nodes.get_mut(to_leaf).ok_or(DockError::NodeNotFound)?;
            if let DockNode::Leaf { tabs, active_tab } = target_node {
                let target_idx = to_idx.min(tabs.len());
                tabs.insert(target_idx, tab);
                *active_tab = target_idx;
                Ok(())
            } else {
                let _ = self.add_tab(from_leaf, tab);
                Err(DockError::ExpectedLeafNode)
            }
        }
    }

    /// Closes all tabs in the specified leaf except the tab at `keep_idx`.
    /// Returns the vector of closed tabs, or an error if the node or index is invalid.
    pub fn close_other_tabs(
        &mut self,
        leaf: DockNodeId,
        keep_idx: usize,
    ) -> Result<Vec<T>, DockError> {
        let node = self.nodes.get_mut(leaf).ok_or(DockError::NodeNotFound)?;
        if let DockNode::Leaf { tabs, active_tab } = node {
            if keep_idx >= tabs.len() {
                return Err(DockError::IndexOutOfBounds);
            }
            let kept = tabs.remove(keep_idx);
            let removed = std::mem::replace(tabs, vec![kept]);
            *active_tab = 0;
            Ok(removed)
        } else {
            Err(DockError::ExpectedLeafNode)
        }
    }

    /// Closes all tabs located to the right of `from_idx` in the specified leaf.
    /// Returns the vector of closed tabs, or an error if the node or index is invalid.
    pub fn close_tabs_to_right(
        &mut self,
        leaf: DockNodeId,
        from_idx: usize,
    ) -> Result<Vec<T>, DockError> {
        let node = self.nodes.get_mut(leaf).ok_or(DockError::NodeNotFound)?;
        if let DockNode::Leaf { tabs, active_tab } = node {
            if from_idx >= tabs.len() {
                return Err(DockError::IndexOutOfBounds);
            }
            let removed = tabs.split_off(from_idx + 1);
            if *active_tab >= tabs.len() {
                *active_tab = tabs.len().saturating_sub(1);
            }
            Ok(removed)
        } else {
            Err(DockError::ExpectedLeafNode)
        }
    }

    /// Updates the division ratio of a split container node.
    pub fn set_split_ratio(&mut self, split_node: DockNodeId, ratio: f32) -> Result<(), DockError> {
        let node = self
            .nodes
            .get_mut(split_node)
            .ok_or(DockError::NodeNotFound)?;
        match node {
            DockNode::Split { ratio: r, .. } => {
                *r = ratio.clamp(0.05, 0.95);
                Ok(())
            }
            DockNode::Leaf { .. } => Err(DockError::ExpectedSplitNode),
        }
    }

    /// Prunes empty leaves and collapses redundant split parent containers.
    pub fn collapse_empty_leaves(&mut self) {
        if let Some(root_id) = self.root {
            self.root = self.collapse_recursive(root_id);
        }
        if let Some(focused) = self.focused_leaf
            && !self.nodes.contains_key(focused)
        {
            self.focused_leaf = None;
        }
    }

    fn collapse_recursive(&mut self, current_id: DockNodeId) -> Option<DockNodeId> {
        let node = self.nodes.get(current_id)?;
        match *node {
            DockNode::Leaf { ref tabs, .. } => {
                if tabs.is_empty() {
                    self.nodes.remove(current_id);
                    None
                } else {
                    Some(current_id)
                }
            }
            DockNode::Split {
                direction: _,
                ratio: _,
                first,
                second,
            } => {
                let new_first = self.collapse_recursive(first);
                let new_second = self.collapse_recursive(second);

                match (new_first, new_second) {
                    (Some(f), Some(s)) => {
                        if let Some(DockNode::Split {
                            first: ref_f,
                            second: ref_s,
                            ..
                        }) = self.nodes.get_mut(current_id)
                        {
                            *ref_f = f;
                            *ref_s = s;
                        }
                        Some(current_id)
                    }
                    (Some(f), None) => {
                        self.nodes.remove(current_id);
                        Some(f)
                    }
                    (None, Some(s)) => {
                        self.nodes.remove(current_id);
                        Some(s)
                    }
                    (None, None) => {
                        self.nodes.remove(current_id);
                        None
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tree_tests.rs"]
mod tests;
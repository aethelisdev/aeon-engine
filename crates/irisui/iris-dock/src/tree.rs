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
    pub fn split(
        &mut self,
        target_leaf: DockNodeId,
        direction: SplitDirection,
        ratio: f32,
        new_tabs: Vec<T>,
    ) -> Result<(DockNodeId, DockNodeId), DockError> {
        let (existing_tabs, existing_active) = match self.nodes.get_mut(target_leaf) {
            Some(DockNode::Leaf { tabs, active_tab }) => (std::mem::take(tabs), *active_tab),
            Some(DockNode::Split { .. }) => return Err(DockError::ExpectedLeafNode),
            None => return Err(DockError::NodeNotFound),
        };

        let first_id = self.nodes.insert(DockNode::Leaf {
            tabs: existing_tabs,
            active_tab: existing_active,
        });

        let second_id = self.nodes.insert(DockNode::Leaf {
            tabs: new_tabs,
            active_tab: 0,
        });

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
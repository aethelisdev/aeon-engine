// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Docking Layout Topology Specification
//!
//! Constructs hierarchical [`DockTree`] and [`DockState`] instances directly from immutable
//! declarative tree descriptions (`DockLayoutNode`), eliminating imperative node splitting chains
//! in consumer applications.

use crate::area::splitter_drag::{MAX_SPLIT_RATIO, MIN_SPLIT_RATIO};
use crate::state::DockState;
use crate::tree::{DockNode, DockNodeId, DockTree, SplitDirection};

/// Immutable declarative specification of a binary split docking layout tree.
///
/// Build layouts cleanly via [`DockLayoutNode::tabs`], [`DockLayoutNode::split_horizontal`],
/// and [`DockLayoutNode::split_vertical`], then materialize them with [`DockState::from_layout`]
/// or [`DockTree::from_layout`].
#[derive(Debug, Clone, PartialEq)]
pub enum DockLayoutNode<T> {
    /// Leaf pane hosting an ordered tab sequence with `active_tab` selected initially.
    Leaf {
        /// Ordered tabs belonging to this pane.
        tabs: Vec<T>,
        /// Zero-based index of the initially active tab.
        active_tab: usize,
    },
    /// Binary container dividing space between `first` (left/top) and `second` (right/bottom).
    Split {
        /// Axis dividing the two child subtrees.
        direction: SplitDirection,
        /// Fraction allocated to `first`, clamped into `[0.10, 0.90]`.
        ratio: f32,
        /// First child region (left for horizontal, top for vertical).
        first: Box<Self>,
        /// Second child region (right for horizontal, bottom for vertical).
        second: Box<Self>,
    },
}

impl<T> DockLayoutNode<T> {
    /// Constructs a leaf pane holding `tabs` with the first tab active.
    #[inline]
    #[must_use]
    pub fn tabs(tabs: impl IntoIterator<Item = T>) -> Self {
        Self::Leaf {
            tabs: tabs.into_iter().collect(),
            active_tab: 0,
        }
    }

    /// Constructs a leaf pane holding `tabs` with `active_tab` selected.
    #[inline]
    #[must_use]
    pub fn tabs_with_active(tabs: impl IntoIterator<Item = T>, active_tab: usize) -> Self {
        let tabs: Vec<T> = tabs.into_iter().collect();
        let clamped_active = if tabs.is_empty() {
            0
        } else {
            active_tab.min(tabs.len() - 1)
        };
        Self::Leaf {
            tabs,
            active_tab: clamped_active,
        }
    }

    /// Constructs a horizontal split (`first` on left, `second` on right) at `ratio`.
    #[inline]
    #[must_use]
    pub fn split_horizontal(ratio: f32, first: Self, second: Self) -> Self {
        Self::Split {
            direction: SplitDirection::Horizontal,
            ratio,
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    /// Constructs a vertical split (`first` on top, `second` on bottom) at `ratio`.
    #[inline]
    #[must_use]
    pub fn split_vertical(ratio: f32, first: Self, second: Self) -> Self {
        Self::Split {
            direction: SplitDirection::Vertical,
            ratio,
            first: Box::new(first),
            second: Box::new(second),
        }
    }
}

impl<T> DockTree<T> {
    /// Materializes a [`DockTree`] from a declarative [`DockLayoutNode`] tree without
    /// imperative multi-step splitting.
    #[must_use]
    pub fn from_layout(spec: DockLayoutNode<T>) -> Self {
        let mut tree = Self::new();
        let root_id = insert_layout_node(&mut tree, spec);
        tree.set_root(root_id);
        tree
    }

    /// Inserts an entire declarative [`DockNode`] directly, returning its allocated [`DockNodeId`].
    pub(crate) fn insert_raw_node(&mut self, node: DockNode<T>) -> DockNodeId {
        self.insert_node_internal(node)
    }
}

fn insert_layout_node<T>(tree: &mut DockTree<T>, spec: DockLayoutNode<T>) -> DockNodeId {
    match spec {
        DockLayoutNode::Leaf { tabs, active_tab } => {
            let active_tab = if tabs_len_check(&tabs) {
                active_tab.min(tabs.len() - 1)
            } else {
                0
            };
            tree.insert_raw_node(DockNode::Leaf { tabs, active_tab })
        }
        DockLayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let first_id = insert_layout_node(tree, *first);
            let second_id = insert_layout_node(tree, *second);
            tree.insert_raw_node(DockNode::Split {
                direction,
                ratio: ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO),
                first: first_id,
                second: second_id,
            })
        }
    }
}

#[inline]
fn tabs_len_check<T>(tabs: &[T]) -> bool {
    !tabs.is_empty()
}

impl<T> DockState<T> {
    /// Constructs a [`DockState`] directly from a declarative [`DockLayoutNode`] tree.
    #[inline]
    #[must_use]
    pub fn from_layout(spec: DockLayoutNode<T>) -> Self {
        Self::new(DockTree::from_layout(spec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Tab {
        Viewport,
        Hierarchy,
        Inspector,
        Assets,
    }

    #[test]
    fn test_dock_state_from_declarative_layout_preserves_hierarchy_and_ratios() {
        let state = DockState::from_layout(DockLayoutNode::split_horizontal(
            0.14,
            DockLayoutNode::tabs([Tab::Hierarchy]),
            DockLayoutNode::split_horizontal(
                0.82,
                DockLayoutNode::split_vertical(
                    0.80,
                    DockLayoutNode::tabs([Tab::Viewport]),
                    DockLayoutNode::tabs([Tab::Assets]),
                ),
                DockLayoutNode::tabs([Tab::Inspector]),
            ),
        ))
        .with_min_pane_size(60.0);

        assert!(state.contains_tab(&Tab::Viewport));
        assert!(state.contains_tab(&Tab::Hierarchy));
        assert!(state.contains_tab(&Tab::Inspector));
        assert!(state.contains_tab(&Tab::Assets));
        assert_eq!(state.min_pane_size, 60.0);

        let root_id = state.tree.root().expect("root node");
        match state.tree.get(root_id).expect("root") {
            DockNode::Split {
                direction, ratio, ..
            } => {
                assert_eq!(*direction, SplitDirection::Horizontal);
                assert!((*ratio - 0.14).abs() < 1e-6);
            }
            DockNode::Leaf { .. } => panic!("expected root split"),
        }
    }
}
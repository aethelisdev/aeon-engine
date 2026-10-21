// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Invalidation boundary and dirty flag tracking mechanisms for the UI tree.
//!
//! Enables fine-grained sub-tree layout and paint caching, halting dirty propagation
//! at designated boundary nodes and eliminating full-tree recomputations on interactive frames.

use crate::dirty::DirtyFlags;
use crate::id::WidgetId;
use std::collections::HashSet;

use super::arena::UiTree;

impl UiTree {
    /// Marks dirty flags on a specific node and propagates the invalidation up the ancestor chain.
    ///
    /// # Invalidation Boundary Contract
    /// - If this node or any ancestor has `is_boundary == true`, the upward invalidation propagation
    ///   halts immediately at the nearest enclosing boundary. That boundary node is marked dirty and
    ///   recorded in `dirty_boundaries`, isolating layout recalculation to that specific subtree.
    /// - If no enclosing boundary exists, the invalidation propagates all the way up to the tree root,
    ///   requiring full tree layout on the next frame.
    ///
    /// Returns `Some(boundary_id)` if captured by an invalidation boundary, or `None` if propagation reached root.
    pub fn mark_dirty(&mut self, id: WidgetId, flags: DirtyFlags) -> Option<WidgetId> {
        let target = self.nodes.get_mut(id)?;
        target.mark_dirty(flags);

        if target.is_boundary {
            self.dirty_boundaries.insert(id);
            return Some(id);
        }

        let mut curr_parent = target.parent;
        while let Some(parent_id) = curr_parent {
            if let Some(parent_node) = self.nodes.get_mut(parent_id) {
                parent_node.mark_dirty(flags);
                if parent_node.is_boundary {
                    self.dirty_boundaries.insert(parent_id);
                    return Some(parent_id);
                }
                curr_parent = parent_node.parent;
            } else {
                break;
            }
        }

        None
    }

    /// Recursively marks dirty flags on a node and all of its descendants with zero heap allocation.
    ///
    /// If `id` or any of its descendants is an invalidation boundary, it is recorded in `dirty_boundaries`.
    pub fn mark_dirty_subtree(&mut self, id: WidgetId, flags: DirtyFlags) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.mark_dirty(flags);
            if node.is_boundary {
                self.dirty_boundaries.insert(id);
            }
        }
        let child_count = self.nodes.get(id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            let child_id = self.nodes[id].children[i];
            self.mark_dirty_subtree(child_id, flags);
        }
    }

    /// Returns the nearest enclosing invalidation boundary for a given node, if any.
    ///
    /// Traverses up the parent chain starting from `id`. Returns `Some(boundary_id)` if either
    /// `id` itself or any of its ancestors has `is_boundary == true`.
    #[must_use]
    pub fn find_invalidation_boundary(&self, id: WidgetId) -> Option<WidgetId> {
        let mut curr = Some(id);
        while let Some(curr_id) = curr {
            if let Some(node) = self.nodes.get(curr_id) {
                if node.is_boundary {
                    return Some(curr_id);
                }
                curr = node.parent;
            } else {
                break;
            }
        }
        None
    }

    /// Configures whether the specified node acts as an invalidation boundary for sub-tree layout caching.
    pub fn set_boundary(&mut self, id: WidgetId, is_boundary: bool) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.set_boundary(is_boundary);
            if !is_boundary {
                self.dirty_boundaries.remove(&id);
            }
        }
    }

    /// Returns `true` if the specified node is configured as an invalidation boundary.
    #[must_use]
    pub fn is_boundary(&self, id: WidgetId) -> bool {
        self.nodes.get(id).is_some_and(|n| n.is_boundary)
    }

    /// Returns a reference to the set of invalidation boundaries currently flagged as dirty.
    #[inline]
    #[must_use]
    pub fn dirty_boundaries(&self) -> &HashSet<WidgetId> {
        &self.dirty_boundaries
    }

    /// Clears all recorded dirty invalidation boundary markers.
    #[inline]
    pub fn clear_dirty_boundaries(&mut self) {
        self.dirty_boundaries.clear();
    }

    /// Returns `true` if any invalidation boundaries are currently marked dirty.
    #[inline]
    #[must_use]
    pub fn has_dirty_boundaries(&self) -> bool {
        !self.dirty_boundaries.is_empty()
    }

    /// Checks if any node in the tree currently has any of the specified dirty flags set.
    #[must_use]
    pub fn has_dirty_nodes(&self, flags: DirtyFlags) -> bool {
        self.nodes.values().any(|n| n.dirty.intersects(flags))
    }
}
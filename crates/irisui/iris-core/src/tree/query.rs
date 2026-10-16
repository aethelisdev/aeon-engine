// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Node search, subtree traversal, and spatial rectangle queries for the UI tree.

use crate::geometry::{Point, Rect};
use crate::id::WidgetId;
use crate::node::WidgetNode;

use super::arena::UiTree;

impl UiTree {
    /// Searches the tree for the first widget node possessing the specified semantic tag.
    ///
    /// Performs an $O(N)$ linear scan over the active node arena and returns the first
    /// matching generational [`WidgetId`].
    ///
    /// # Arguments
    /// * `tag` - 64-bit persistent semantic interaction tag to locate.
    ///
    /// Returns `Some(WidgetId)` if found, or `None` if `tag == 0` or no matching node exists.
    #[inline]
    #[must_use]
    pub fn find_node_by_tag(&self, tag: u64) -> Option<WidgetId> {
        if tag == 0 {
            return None;
        }
        for (id, node) in &self.nodes {
            if node.tag == tag {
                return Some(id);
            }
        }
        None
    }

    /// Returns a shared reference to the first node matching the specified name without requiring an intermediate ID.
    ///
    /// # Arguments
    /// * `name` - Exact string name assigned to the target widget node.
    #[inline]
    #[must_use]
    pub fn node_named(&self, name: &str) -> Option<&WidgetNode> {
        self.find_node_by_name(name).and_then(|id| self.get(id))
    }

    /// Returns a shared reference to the first node matching the specified semantic tag without requiring an intermediate ID.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag assigned to the target widget node.
    #[inline]
    #[must_use]
    pub fn node_tagged(&self, tag: u64) -> Option<&WidgetNode> {
        self.find_node_by_tag(tag).and_then(|id| self.get(id))
    }

    /// Returns a shared reference to the first node within a subtree matching the specified name.
    ///
    /// # Arguments
    /// * `root_id` - Root container node where subtree traversal begins.
    /// * `name` - Exact string name assigned to the target descendant node.
    #[inline]
    #[must_use]
    pub fn node_named_in_subtree(&self, root_id: WidgetId, name: &str) -> Option<&WidgetNode> {
        self.find_node_by_name_in_subtree(root_id, name)
            .and_then(|id| self.get(id))
    }

    /// Returns the computed bounding rectangle of the widget node tagged with `tag`,
    /// translated by an optional root coordinate offset (such as a parent panel position).
    ///
    /// Eliminates linear tree searches and manual coordinate math when anchoring floating popups.
    ///
    /// # Arguments
    /// * `tag` - 64-bit semantic tag assigned to the target widget node.
    /// * `offset` - Translation offset vector applied to the local computed bounds.
    ///
    /// # Returns
    /// `Some(Rect)` if a node matching `tag` exists, or `None` otherwise.
    #[inline]
    #[must_use]
    pub fn node_rect_tagged(&self, tag: u64, offset: Point) -> Option<Rect> {
        self.node_tagged(tag).map(|node| {
            Rect::new(
                offset.x + node.computed_rect.x,
                offset.y + node.computed_rect.y,
                node.computed_rect.width,
                node.computed_rect.height,
            )
        })
    }

    /// Returns the computed bounding rectangle of the widget node with the given `name`,
    /// translated by an optional root coordinate offset (such as a parent panel position).
    ///
    /// Eliminates linear tree searches and manual coordinate math when anchoring floating popups.
    ///
    /// # Arguments
    /// * `name` - Exact string name assigned to the widget node.
    /// * `offset` - Translation offset vector applied to the local computed bounds.
    ///
    /// # Returns
    /// `Some(Rect)` if a node matching `name` exists, or `None` otherwise.
    #[inline]
    #[must_use]
    pub fn node_rect_named(&self, name: &str, offset: Point) -> Option<Rect> {
        self.node_named(name).map(|node| {
            Rect::new(
                offset.x + node.computed_rect.x,
                offset.y + node.computed_rect.y,
                node.computed_rect.width,
                node.computed_rect.height,
            )
        })
    }

    /// Finds the first widget identifier in the tree matching the specified debug or semantic name.
    ///
    /// # Arguments
    /// * `name` - Exact string name assigned to the widget node during scope creation.
    ///
    /// # Returns
    /// `Some(WidgetId)` if a node with matching name exists, or `None` otherwise.
    #[inline]
    #[must_use]
    pub fn find_node_by_name(&self, name: &str) -> Option<WidgetId> {
        for (id, node) in &self.nodes {
            if node.name.as_deref() == Some(name) {
                return Some(id);
            }
        }
        None
    }

    /// Finds the first widget identifier within a specific subtree matching the specified name.
    ///
    /// # Arguments
    /// * `root_id` - Root container node where subtree traversal begins.
    /// * `name` - Exact string name assigned to the target descendant node.
    ///
    /// # Returns
    /// `Some(WidgetId)` if a matching descendant node is found, or `None` otherwise.
    #[must_use]
    pub fn find_node_by_name_in_subtree(&self, root_id: WidgetId, name: &str) -> Option<WidgetId> {
        let mut target = None;
        self.traverse_depth_first(root_id, &mut |id, node| {
            if target.is_none() && node.name.as_deref() == Some(name) {
                target = Some(id);
            }
        });
        target
    }

    /// Finds the first widget identifier within a specific subtree matching the specified semantic tag.
    ///
    /// # Arguments
    /// * `root_id` - Root container node where subtree traversal begins.
    /// * `tag` - 64-bit semantic tag assigned to the target descendant node.
    ///
    /// # Returns
    /// `Some(WidgetId)` if a matching descendant node with non-zero tag is found, or `None` otherwise.
    #[must_use]
    pub fn find_node_by_tag_in_subtree(&self, root_id: WidgetId, tag: u64) -> Option<WidgetId> {
        if tag == 0 {
            return None;
        }
        let mut target = None;
        self.traverse_depth_first(root_id, &mut |id, node| {
            if target.is_none() && node.tag == tag {
                target = Some(id);
            }
        });
        target
    }

    /// Checks whether any widget node within the subtree matches the specified predicate.
    ///
    /// # Arguments
    /// * `root_id` - Root container node where subtree traversal begins.
    /// * `predicate` - Boolean evaluation function executed on each visited node.
    ///
    /// # Returns
    /// `true` if at least one descendant satisfies the predicate, `false` otherwise.
    #[must_use]
    pub fn any_in_subtree<F>(&self, root_id: WidgetId, mut predicate: F) -> bool
    where
        F: FnMut(&WidgetNode) -> bool,
    {
        let mut found = false;
        self.traverse_depth_first(root_id, &mut |_id, node| {
            if !found && predicate(node) {
                found = true;
            }
        });
        found
    }

    /// Traverses the subtree starting from `root_id` in depth-first order.
    pub fn traverse_depth_first<'a, F>(&'a self, root_id: WidgetId, visitor: &mut F)
    where
        F: FnMut(WidgetId, &'a WidgetNode),
    {
        if let Some(node) = self.nodes.get(root_id) {
            visitor(root_id, node);
            for &child_id in &node.children {
                self.traverse_depth_first(child_id, visitor);
            }
        }
    }

    /// Traverses and mutates the subtree starting from `root_id` in depth-first order.
    pub fn traverse_depth_first_mut<F>(&mut self, root_id: WidgetId, visitor: &mut F)
    where
        F: FnMut(WidgetId, &mut WidgetNode),
    {
        if let Some(node) = self.nodes.get_mut(root_id) {
            visitor(root_id, node);
        }
        let child_count = self.nodes.get(root_id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            if let Some(child_id) = self
                .nodes
                .get(root_id)
                .and_then(|n| n.children.get(i).copied())
            {
                self.traverse_depth_first_mut(child_id, visitor);
            }
        }
    }

    /// Checks if `potential_descendant` is a descendant of `ancestor`.
    ///
    /// Climbs the parent pointer chain from `potential_descendant` towards the root.
    /// Returns `true` if `ancestor` is encountered at any level, `false` otherwise.
    pub fn is_descendant_of(&self, potential_descendant: WidgetId, ancestor: WidgetId) -> bool {
        let mut current = Some(potential_descendant);
        while let Some(curr_id) = current {
            if curr_id == ancestor {
                return true;
            }
            current = self.nodes.get(curr_id).and_then(|n| n.parent);
        }
        false
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generational arena-based UI tree manager guaranteeing 100% safe memory operations.

use crate::dirty::DirtyFlags;
use crate::error::IrisCoreError;
use crate::geometry::Point;
use crate::id::WidgetId;
use crate::node::WidgetNode;
use slotmap::SlotMap;

/// The central hierarchical arena storing all UI nodes.
/// `UiTree` manages generational keys, ensures parent-child invariant consistency,
/// prevents circular references, and coordinates traversal and dirty caching.
#[derive(Debug, Clone, Default)]
pub struct UiTree {
    /// Generational arena containing all active nodes.
    nodes: SlotMap<WidgetId, WidgetNode>,
    /// The root widget node ID of the tree.
    root: Option<WidgetId>,
    /// Aggregate dirty bitmask across all nodes, enabling O(1) change queries.
    dirty_mask: DirtyFlags,
}

impl UiTree {
    /// Creates a new, empty UI tree.
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
            dirty_mask: DirtyFlags::empty(),
        }
    }

    /// Clears all nodes from the tree and resets the root pointer.
    #[inline]
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.dirty_mask = DirtyFlags::empty();
    }

    /// Returns the root node key of the tree, if set.
    #[inline]
    pub fn root(&self) -> Option<WidgetId> {
        self.root
    }

    /// Returns the total number of active nodes in the arena.
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the tree contains no active nodes.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Allocates a new node in the arena and returns its unique generational key.
    pub fn create_node(&mut self) -> WidgetId {
        self.dirty_mask.insert(DirtyFlags::ALL);
        self.nodes.insert_with_key(WidgetNode::new)
    }

    /// Creates and assigns the root node of the tree.
    /// # Errors
    /// Returns `IrisCoreError::RootAlreadyExists` if a root node has already been created.
    pub fn create_root(&mut self) -> Result<WidgetId, IrisCoreError> {
        if let Some(existing_root) = self.root {
            return Err(IrisCoreError::RootAlreadyExists(existing_root));
        }
        let root_id = self.create_node();
        self.root = Some(root_id);
        Ok(root_id)
    }

    /// Sets an existing node as the root node of the tree.
    pub fn set_root(&mut self, id: WidgetId) -> Result<(), IrisCoreError> {
        if !self.nodes.contains_key(id) {
            return Err(IrisCoreError::NodeNotFound(id));
        }
        self.root = Some(id);
        Ok(())
    }

    /// Retrieves an immutable reference to a widget node.
    #[inline]
    pub fn get(&self, id: WidgetId) -> Option<&WidgetNode> {
        self.nodes.get(id)
    }

    /// Retrieves a mutable reference to a widget node.
    #[inline]
    pub fn get_mut(&mut self, id: WidgetId) -> Option<&mut WidgetNode> {
        self.dirty_mask.insert(DirtyFlags::ALL);
        self.nodes.get_mut(id)
    }

    /// Attaches a child node to a parent node.
    /// # Errors
    /// Returns `IrisCoreError::NodeNotFound` if either the parent or child node key does not exist.
    /// Returns `IrisCoreError::CircularHierarchy` if attaching the child would produce a cyclical loop.
    pub fn add_child(&mut self, parent: WidgetId, child: WidgetId) -> Result<(), IrisCoreError> {
        if !self.nodes.contains_key(parent) {
            return Err(IrisCoreError::NodeNotFound(parent));
        }
        if !self.nodes.contains_key(child) {
            return Err(IrisCoreError::NodeNotFound(child));
        }
        if parent == child || self.is_descendant_of(parent, child) {
            return Err(IrisCoreError::CircularHierarchy { child, parent });
        }

        // If the child already has a different parent, detach it first
        if let Some(prev_parent_id) = self.nodes[child].parent
            && prev_parent_id != parent
            && let Some(prev_parent) = self.nodes.get_mut(prev_parent_id)
        {
            prev_parent.children.retain(|&c| c != child);
            prev_parent.mark_dirty(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        }

        self.nodes[child].parent = Some(parent);
        if !self.nodes[parent].children.contains(&child) {
            self.nodes[parent].children.push(child);
        }

        self.dirty_mask
            .insert(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        self.nodes[parent].mark_dirty(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        self.nodes[child].mark_dirty(DirtyFlags::LAYOUT | DirtyFlags::TRANSFORM);
        Ok(())
    }

    /// Detaches a child node from its parent without removing it from the arena.
    pub fn remove_child(&mut self, parent: WidgetId, child: WidgetId) -> Result<(), IrisCoreError> {
        self.dirty_mask
            .insert(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        let parent_node = self
            .nodes
            .get_mut(parent)
            .ok_or(IrisCoreError::NodeNotFound(parent))?;
        parent_node.children.retain(|&c| c != child);
        parent_node.mark_dirty(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);

        if let Some(child_node) = self.nodes.get_mut(child)
            && child_node.parent == Some(parent)
        {
            child_node.parent = None;
            child_node.mark_dirty(DirtyFlags::LAYOUT | DirtyFlags::TRANSFORM);
        }
        Ok(())
    }

    /// Recursively removes all child descendants of a parent node while preserving the parent itself.
    /// Marks `CHILDREN | LAYOUT` dirty flags on the parent.
    /// # Errors
    /// Returns `IrisCoreError::NodeNotFound` if the parent node does not exist in the arena.
    pub fn clear_children(&mut self, parent: WidgetId) -> Result<(), IrisCoreError> {
        self.dirty_mask
            .insert(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        let Some(parent_node) = self.nodes.get_mut(parent) else {
            return Err(IrisCoreError::NodeNotFound(parent));
        };
        let children = std::mem::take(&mut parent_node.children);
        parent_node.mark_dirty(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);

        for child in children {
            let mut to_remove = Vec::new();
            self.collect_subtree(child, &mut to_remove);
            for node_id in to_remove {
                self.nodes.remove(node_id);
            }
        }
        Ok(())
    }

    /// Returns `true` if the arena contains the given node handle.
    #[inline]
    pub fn contains_node(&self, id: WidgetId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Recursively removes a node and all of its descendants from the arena.
    pub fn remove_node(&mut self, id: WidgetId) -> Result<(), IrisCoreError> {
        if !self.nodes.contains_key(id) {
            return Err(IrisCoreError::NodeNotFound(id));
        }

        self.dirty_mask
            .insert(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);

        // Detach from parent
        if let Some(parent_id) = self.nodes[id].parent
            && let Some(parent) = self.nodes.get_mut(parent_id)
        {
            parent.children.retain(|&c| c != id);
            parent.mark_dirty(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        }

        // If removing the root, clear root pointer
        if self.root == Some(id) {
            self.root = None;
        }

        // Collect all subtree IDs for removal
        let mut to_remove = Vec::new();
        self.collect_subtree(id, &mut to_remove);

        for node_id in to_remove {
            self.nodes.remove(node_id);
        }

        Ok(())
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

    /// Recursively marks dirty flags on a node and all of its descendants with zero heap allocation.
    pub fn mark_dirty_subtree(&mut self, id: WidgetId, flags: DirtyFlags) {
        self.dirty_mask.insert(flags);
        if let Some(node) = self.nodes.get_mut(id) {
            node.mark_dirty(flags);
        }
        let child_count = self.nodes.get(id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            let child_id = self.nodes[id].children[i];
            self.mark_dirty_subtree(child_id, flags);
        }
    }

    /// Sets dirty flags on a specific node and updates the tree-level dirty mask in O(1) time.
    #[inline]
    pub fn mark_node_dirty(&mut self, id: WidgetId, flags: DirtyFlags) {
        self.dirty_mask.insert(flags);
        if let Some(node) = self.nodes.get_mut(id) {
            node.mark_dirty(flags);
        }
    }

    /// Checks if any node in the tree currently has any of the specified dirty flags set.
    /// Executes in $O(1)$ time by evaluating the aggregate bitmask without traversing arena nodes.
    #[inline]
    pub fn has_dirty_nodes(&self, flags: DirtyFlags) -> bool {
        self.dirty_mask.intersects(flags)
    }

    /// Checks if the specified node or any of its descendants in the subtree has any of the given dirty flags set.
    /// Short-circuits in $O(1)$ time if the aggregate tree bitmask contains none of the specified flags.
    pub fn is_subtree_dirty(&self, id: WidgetId, flags: DirtyFlags) -> bool {
        if !self.dirty_mask.intersects(flags) {
            return false;
        }
        let Some(node) = self.nodes.get(id) else {
            return false;
        };
        if node.dirty.intersects(flags) {
            return true;
        }
        for &child in &node.children {
            if self.is_subtree_dirty(child, flags) {
                return true;
            }
        }
        false
    }

    /// Clears the specified dirty flags across all active nodes in the tree arena.
    /// If no nodes in the tree have the specified flags set, this method returns
    /// immediately in $O(1)$ time without traversing or mutating any nodes.
    #[inline]
    pub fn clear_all_dirty(&mut self, flags: DirtyFlags) {
        if !self.dirty_mask.intersects(flags) {
            return;
        }
        self.dirty_mask.remove(flags);
        for node in self.nodes.values_mut() {
            node.clear_dirty(flags);
        }
    }

    /// Clears the specified dirty flags on the target node and all its descendants in the subtree.
    /// Short-circuits in $O(1)$ time if the aggregate tree bitmask contains none of the specified flags.
    pub fn clear_dirty_subtree(&mut self, id: WidgetId, flags: DirtyFlags) {
        if !self.dirty_mask.intersects(flags) {
            return;
        }
        if let Some(node) = self.nodes.get_mut(id) {
            node.clear_dirty(flags);
        }
        let child_count = self.nodes.get(id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            let child_id = self.nodes[id].children[i];
            self.clear_dirty_subtree(child_id, flags);
        }
    }

    /// Marks the specified dirty flags on all active nodes in the tree arena.
    /// Used during full viewport resizes or configuration changes requiring
    /// universal layout or paint re-evaluation.
    pub fn mark_all_dirty(&mut self, flags: DirtyFlags) {
        self.dirty_mask.insert(flags);
        for node in self.nodes.values_mut() {
            node.mark_dirty(flags);
        }
    }

    /// Performs screen-space hit testing to find the deepest interactive node under `point`.
    pub fn hit_test(&self, point: Point) -> Option<WidgetId> {
        let root_id = self.root?;
        self.hit_test_recursive(root_id, point)
    }

    fn hit_test_recursive(&self, current_id: WidgetId, point: Point) -> Option<WidgetId> {
        let node = self.nodes.get(current_id)?;
        if !node.visible || !node.computed_rect.contains_point(point) {
            return None;
        }

        // Iterate children in reverse order (top-most z-order first)
        for &child_id in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(child_id, point) {
                return Some(hit);
            }
        }

        if node.interactive {
            Some(current_id)
        } else {
            None
        }
    }

    /// Checks if `potential_descendant` is a descendant of `ancestor`.
    fn is_descendant_of(&self, potential_descendant: WidgetId, ancestor: WidgetId) -> bool {
        let mut current = Some(potential_descendant);
        while let Some(curr_id) = current {
            if curr_id == ancestor {
                return true;
            }
            current = self.nodes.get(curr_id).and_then(|n| n.parent);
        }
        false
    }

    /// Helper to recursively collect all descendant keys in a subtree.
    fn collect_subtree(&self, id: WidgetId, list: &mut Vec<WidgetId>) {
        list.push(id);
        if let Some(node) = self.nodes.get(id) {
            for &child_id in &node.children {
                self.collect_subtree(child_id, list);
            }
        }
    }
}
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
}

impl UiTree {
    /// Creates a new, empty UI tree.
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
        }
    }

    /// Clears all nodes from the tree and resets the root pointer.
    #[inline]
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
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

        self.nodes[parent].mark_dirty(DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        self.nodes[child].mark_dirty(DirtyFlags::LAYOUT | DirtyFlags::TRANSFORM);
        Ok(())
    }

    /// Detaches a child node from its parent without removing it from the arena.
    pub fn remove_child(&mut self, parent: WidgetId, child: WidgetId) -> Result<(), IrisCoreError> {
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

    /// Recursively removes a node and all of its descendants from the arena.
    pub fn remove_node(&mut self, id: WidgetId) -> Result<(), IrisCoreError> {
        if !self.nodes.contains_key(id) {
            return Err(IrisCoreError::NodeNotFound(id));
        }

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
        if let Some(node) = self.nodes.get_mut(id) {
            node.mark_dirty(flags);
        }
        let child_count = self.nodes.get(id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            let child_id = self.nodes[id].children[i];
            self.mark_dirty_subtree(child_id, flags);
        }
    }

    /// Checks if any node in the tree currently has any of the specified dirty flags set.
    pub fn has_dirty_nodes(&self, flags: DirtyFlags) -> bool {
        self.nodes.values().any(|n| n.dirty.intersects(flags))
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
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generational node arena and core lifecycle management for the UI tree.

use crate::dirty::DirtyFlags;
use crate::dispatch::EventState;
use crate::error::IrisCoreError;
use crate::geometry::{Rect, Size};
use crate::id::WidgetId;
use crate::node::WidgetNode;
use slotmap::SlotMap;
use std::collections::HashMap;

/// The central hierarchical arena storing all UI nodes.
///
/// `UiTree` manages generational keys, ensures parent-child invariant consistency,
/// prevents circular references, and coordinates traversal and dirty caching.
#[derive(Debug, Clone, Default)]
pub struct UiTree {
    /// Generational arena containing all active nodes.
    pub(crate) nodes: SlotMap<WidgetId, WidgetNode>,
    /// The root widget node ID of the tree.
    pub(crate) root: Option<WidgetId>,
    /// Persistent user interaction and event dispatching state.
    pub event_state: EventState,
    /// Persistent state map tracking open/closed states for collapsing headers and accordions, keyed by 64-bit tag.
    pub collapsing_state: HashMap<u64, bool>,
    /// Persistent state map tracking vertical scroll positions, keyed by 64-bit semantic tag.
    pub scroll_offsets: HashMap<u64, f32>,
    /// Persistent state map tracking resolved viewport sizes from layout passes, keyed by 64-bit semantic tag.
    pub viewport_sizes: HashMap<u64, Size>,
    /// Persistent state map tracking maximum vertical scroll extents, keyed by 64-bit semantic tag.
    pub max_scrolls: HashMap<u64, f32>,
    /// Persistent 64-bit semantic tag identifying the currently opened menu bar dropdown menu, if any.
    pub active_menu_bar_tag: Option<u64>,
    /// Persistent 64-bit semantic tag identifying the currently opened combobox dropdown menu, if any.
    pub active_combobox_tag: Option<u64>,
    /// Persistent map tracking active cascading submenus (parent menu tag -> active child submenu tag).
    pub active_submenus: HashMap<u64, u64>,
    /// Persistent state map tracking computed bounding rectangles across tree rebuilds, keyed by 64-bit semantic tag.
    pub tag_rects: HashMap<u64, Rect>,
    /// Set of widget IDs acting as invalidation boundaries that are currently marked dirty.
    pub dirty_boundaries: std::collections::HashSet<WidgetId>,
    /// Volatile immediate-mode canvas drawing primitives registered for high-frequency widgets.
    pub volatile_canvas_commands: HashMap<WidgetId, Vec<crate::canvas::CanvasPrimitive>>,
    /// Active visual theme tokens governing surface colors, text, and metrics.
    pub theme: crate::theme::Theme,
}

impl UiTree {
    /// Creates a new, empty UI tree with default dark theme tokens.
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
            event_state: EventState::new(),
            collapsing_state: HashMap::new(),
            scroll_offsets: HashMap::new(),
            viewport_sizes: HashMap::new(),
            max_scrolls: HashMap::new(),
            active_menu_bar_tag: None,
            active_combobox_tag: None,
            active_submenus: HashMap::new(),
            tag_rects: HashMap::new(),
            dirty_boundaries: std::collections::HashSet::new(),
            volatile_canvas_commands: HashMap::new(),
            theme: crate::theme::Theme::default(),
        }
    }

    /// Clears all nodes from the tree and resets the root pointer.
    ///
    /// Preserves confirmed frame interaction state on [`EventState`] so that subsequent
    /// declarative scopes constructed during the current frame can observe clicks, keypresses,
    /// and drags without state dropping.
    #[inline]
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.dirty_boundaries.clear();
        self.volatile_canvas_commands.clear();
    }

    /// Clears transient frame click and gesture records recorded for the current frame.
    #[inline]
    pub fn clear_frame_clicks(&mut self) {
        self.event_state.clear_frame_clicks();
    }

    /// Returns `true` if any confirmed clicks, double clicks, secondary clicks, or drag gestures
    /// are currently pending consumption within the active frame.
    #[inline]
    #[must_use]
    pub fn has_frame_interactions(&self) -> bool {
        self.event_state.has_frame_interactions()
    }

    /// Returns `true` if this UI tree requires an immediate declarative rebuild and render.
    ///
    /// Evaluates whether the tree has pending interactions (clicks, double-clicks, secondary clicks,
    /// drags), active open cascading menus, autonomous context menus, or invalidating dirty flags.
    #[inline]
    #[must_use]
    pub fn needs_rebuild(&self) -> bool {
        self.event_state.has_frame_interactions()
            || self.has_open_popup()
            || self.has_dirty_nodes(DirtyFlags::LAYOUT | DirtyFlags::PAINT)
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

    /// Returns an iterator over all allocated `(WidgetId, &WidgetNode)` pairs in the arena.
    #[inline]
    pub fn iter(&self) -> slotmap::basic::Iter<'_, WidgetId, WidgetNode> {
        self.nodes.iter()
    }

    /// Allocates a new node in the arena and returns its unique generational key.
    pub fn create_node(&mut self) -> WidgetId {
        self.nodes.insert_with_key(WidgetNode::new)
    }

    /// Creates and assigns the root node of the tree.
    ///
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
    ///
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
            self.dirty_boundaries.remove(&node_id);
            self.volatile_canvas_commands.remove(&node_id);
            self.event_state.untrack_widget(node_id);
            self.nodes.remove(node_id);
        }

        Ok(())
    }

    /// Recursively removes all descendant child nodes of the specified parent node while preserving the parent itself.
    ///
    /// Preserves the parent node's identity, invalidation boundary status (`is_boundary`), layout style,
    /// and pre-computed boundary coordinates, while deep-removing all children and sub-descendants from the arena.
    /// Safely purges descendant entries from [`Self::dirty_boundaries`], [`Self::volatile_canvas_commands`],
    /// and [`Self::event_state`] to eliminate stale references, and flags the parent node with [`DirtyFlags::CHILDREN`]
    /// and [`DirtyFlags::LAYOUT`].
    ///
    /// # Errors
    /// Returns [`IrisCoreError::NodeNotFound`] if the specified `parent_id` is not present in the tree.
    pub fn clear_children(&mut self, parent_id: WidgetId) -> Result<(), IrisCoreError> {
        let parent_node = self
            .nodes
            .get_mut(parent_id)
            .ok_or(IrisCoreError::NodeNotFound(parent_id))?;

        let children = std::mem::take(&mut parent_node.children);
        let mut to_remove = Vec::new();
        for child_id in children {
            self.collect_subtree(child_id, &mut to_remove);
        }

        for node_id in to_remove {
            self.dirty_boundaries.remove(&node_id);
            self.volatile_canvas_commands.remove(&node_id);
            self.event_state.untrack_widget(node_id);
            self.nodes.remove(node_id);
        }

        self.mark_dirty(parent_id, DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        Ok(())
    }

    /// Recursively collects all widget node IDs within the subtree rooted at `id`, including `id` itself.
    ///
    /// Performs a pre-order depth-first traversal of the widget hierarchy, appending all visited
    /// keys into the provided `list` buffer with zero intermediate heap allocations.
    ///
    /// # Parameters
    /// - `id`: The root node key of the subtree to collect.
    /// - `list`: Destination vector receiving the collected widget IDs.
    pub fn collect_subtree(&self, id: WidgetId, list: &mut Vec<WidgetId>) {
        list.push(id);
        if let Some(node) = self.nodes.get(id) {
            for &child_id in &node.children {
                self.collect_subtree(child_id, list);
            }
        }
    }

    /// Appends an immediate volatile canvas primitive to the command list of the given widget node.
    #[inline]
    pub fn add_canvas_primitive(
        &mut self,
        id: WidgetId,
        primitive: crate::canvas::CanvasPrimitive,
    ) {
        self.volatile_canvas_commands
            .entry(id)
            .or_default()
            .push(primitive);
    }

    /// Returns the volatile canvas primitives registered for the given widget node, if any.
    #[inline]
    #[must_use]
    pub fn canvas_primitives(&self, id: WidgetId) -> Option<&[crate::canvas::CanvasPrimitive]> {
        self.volatile_canvas_commands
            .get(&id)
            .map(|cmds| cmds.as_slice())
    }

    /// Clears all volatile canvas commands for a specific widget node.
    #[inline]
    pub fn clear_canvas_primitives_for(&mut self, id: WidgetId) {
        self.volatile_canvas_commands.remove(&id);
    }

    /// Clears all volatile canvas commands across the entire tree.
    #[inline]
    pub fn clear_all_canvas_primitives(&mut self) {
        self.volatile_canvas_commands.clear();
    }
}
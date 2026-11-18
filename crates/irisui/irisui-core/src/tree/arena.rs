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
    /// Persistent set of 64-bit semantic tags identifying active open popup menus or custom dropdowns.
    pub active_popup_tags: std::collections::HashSet<u64>,
    /// Persistent map tracking active cascading submenus (parent menu tag -> active child submenu tag).
    pub active_submenus: HashMap<u64, u64>,
    /// Persistent state map tracking active modal dialogs, keyed by 64-bit semantic tag.
    pub modal_state: HashMap<u64, bool>,
    /// Persistent state map tracking computed bounding rectangles across tree rebuilds, keyed by 64-bit semantic tag.
    pub tag_rects: HashMap<u64, Rect>,
    /// Set of widget IDs acting as invalidation boundaries that are currently marked dirty.
    pub dirty_boundaries: std::collections::HashSet<WidgetId>,
    /// Volatile immediate-mode canvas drawing primitives registered for high-frequency widgets.
    pub volatile_canvas_commands: HashMap<WidgetId, Vec<crate::canvas::CanvasPrimitive>>,
    /// Semantic tags of live/volatile widgets that support layout-bypassing in-place updates.
    pub live_node_tags: std::collections::HashSet<u64>,
    /// Accumulated `(needs_text_prepare, needs_quad_reupload)` flags from live widget updates.
    pub pending_live_refresh: (bool, bool),
    /// Flag raised when an existing static node is newly promoted to `live_node_tags`,
    /// signaling a one-time static text layer re-collection so duplicate text is cleared.
    pub pending_static_text_sync: bool,
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
            active_popup_tags: std::collections::HashSet::new(),
            active_submenus: HashMap::new(),
            modal_state: HashMap::new(),
            tag_rects: HashMap::new(),
            dirty_boundaries: std::collections::HashSet::new(),
            volatile_canvas_commands: HashMap::new(),
            live_node_tags: std::collections::HashSet::new(),
            pending_live_refresh: (false, false),
            pending_static_text_sync: false,
            theme: crate::theme::Theme::default(),
        }
    }

    /// Creates a new UI tree pre-allocating arena node storage for `capacity` widgets.
    ///
    /// Eliminates vector reallocations during initial shell or subtree synthesis.
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: SlotMap::with_capacity_and_key(capacity),
            root: None,
            event_state: EventState::new(),
            collapsing_state: HashMap::new(),
            scroll_offsets: HashMap::new(),
            viewport_sizes: HashMap::new(),
            max_scrolls: HashMap::new(),
            active_menu_bar_tag: None,
            active_combobox_tag: None,
            active_popup_tags: std::collections::HashSet::new(),
            active_submenus: HashMap::new(),
            modal_state: HashMap::new(),
            tag_rects: HashMap::new(),
            dirty_boundaries: std::collections::HashSet::new(),
            volatile_canvas_commands: HashMap::new(),
            live_node_tags: std::collections::HashSet::new(),
            pending_live_refresh: (false, false),
            pending_static_text_sync: false,
            theme: crate::theme::Theme::default(),
        }
    }

    /// Returns the currently allocated node capacity of the generational arena.
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Reserves capacity for at least `additional` more nodes to be inserted in the arena.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
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
        self.live_node_tags.clear();
        self.pending_live_refresh = (false, false);
        self.pending_static_text_sync = false;
        self.event_state.clear_arena_ids();
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

    /// Allocates a new node in the arena initialized with the specified UI stacking layer.
    ///
    /// # Arguments
    /// * `layer` - Stacking context layer defining drawing order and hit-test priority.
    pub fn create_layer_node(&mut self, layer: crate::node::UiLayer) -> WidgetId {
        self.nodes.insert_with_key(|id| {
            let mut node = WidgetNode::new(id);
            node.layer = layer;
            node
        })
    }

    /// Creates and attaches an elevated layer portal container node to the specified parent.
    ///
    /// Portal nodes isolate top-level dialogs, popups, and floating palettes from their parent's
    /// layout constraints and coordinate clipping boundaries.
    ///
    /// # Arguments
    /// * `parent_id` - Target ancestor node to host the portal container.
    /// * `layer` - Elevated destination stacking layer.
    ///
    /// # Errors
    /// Returns `IrisCoreError::NodeNotFound` if `parent_id` does not exist in the arena.
    pub fn attach_layer_portal(
        &mut self,
        parent_id: WidgetId,
        layer: crate::node::UiLayer,
    ) -> Result<WidgetId, IrisCoreError> {
        if !self.nodes.contains_key(parent_id) {
            return Err(IrisCoreError::NodeNotFound(parent_id));
        }
        let portal_id = self.create_layer_node(layer);
        self.add_child(parent_id, portal_id)?;
        Ok(portal_id)
    }

    /// Explicitly updates the stacking context layer of an existing node in the arena.
    ///
    /// # Arguments
    /// * `id` - Unique widget identifier.
    /// * `layer` - New stacking layer.
    ///
    /// # Errors
    /// Returns `IrisCoreError::NodeNotFound` if the specified node is missing.
    pub fn set_node_layer(
        &mut self,
        id: WidgetId,
        layer: crate::node::UiLayer,
    ) -> Result<(), IrisCoreError> {
        let node = self
            .nodes
            .get_mut(id)
            .ok_or(IrisCoreError::NodeNotFound(id))?;
        node.layer = layer;
        Ok(())
    }

    /// Recursively sets the stacking layer of a root node and all its descendants.
    ///
    /// # Arguments
    /// * `root_id` - Subtree root node identifier.
    /// * `layer` - Target stacking layer to propagate.
    ///
    /// # Errors
    /// Returns `IrisCoreError::NodeNotFound` if `root_id` does not exist.
    pub fn set_layer_recursive(
        &mut self,
        root_id: WidgetId,
        layer: crate::node::UiLayer,
    ) -> Result<(), IrisCoreError> {
        if !self.nodes.contains_key(root_id) {
            return Err(IrisCoreError::NodeNotFound(root_id));
        }
        let mut stack = vec![root_id];
        while let Some(current_id) = stack.pop() {
            if let Some(node) = self.nodes.get_mut(current_id) {
                node.layer = layer;
                stack.extend(node.children.iter().copied());
            }
        }
        Ok(())
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
            if let Some(removed) = self.nodes.remove(node_id) {
                self.live_node_tags.remove(&removed.tag);
            }
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
            if let Some(removed) = self.nodes.remove(node_id) {
                self.live_node_tags.remove(&removed.tag);
            }
        }

        self.mark_dirty(parent_id, DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        Ok(())
    }

    /// Detaches `node_id` from its parent and removes it along with all descendants from the arena.
    ///
    /// Safely purges descendant entries from [`Self::dirty_boundaries`],
    /// [`Self::volatile_canvas_commands`], [`Self::event_state`], and [`Self::live_node_tags`].
    pub fn remove_subtree(&mut self, node_id: WidgetId) -> bool {
        let Some(parent_opt) = self.nodes.get(node_id).map(|n| n.parent) else {
            return false;
        };
        if let Some(pid) = parent_opt
            && let Some(parent) = self.nodes.get_mut(pid)
        {
            parent.children.retain(|&id| id != node_id);
        }
        let mut to_remove = Vec::new();
        self.collect_subtree(node_id, &mut to_remove);
        for id in to_remove {
            self.dirty_boundaries.remove(&id);
            self.volatile_canvas_commands.remove(&id);
            self.event_state.untrack_widget(id);
            if let Some(removed) = self.nodes.remove(id) {
                self.live_node_tags.remove(&removed.tag);
            }
        }
        if let Some(pid) = parent_opt {
            self.mark_dirty(pid, DirtyFlags::CHILDREN | DirtyFlags::LAYOUT);
        }
        true
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

    /// Removes and returns the volatile canvas commands for a specific widget node, if any.
    ///
    /// Used by retained live canvas updaters to compare previous primitives against newly emitted
    /// primitives and avoid marking unchanged canvases dirty.
    #[inline]
    pub fn take_canvas_primitives_for(
        &mut self,
        id: WidgetId,
    ) -> Option<Vec<crate::canvas::CanvasPrimitive>> {
        self.volatile_canvas_commands.remove(&id)
    }

    /// Clears all volatile canvas commands across the entire tree.
    #[inline]
    pub fn clear_all_canvas_primitives(&mut self) {
        self.volatile_canvas_commands.clear();
    }

    /// Returns whether the autonomous numeric undo/redo stack is enabled.
    #[inline]
    #[must_use]
    pub const fn is_numeric_undo_enabled(&self) -> bool {
        self.event_state.numeric_undo.is_enabled()
    }

    /// Enables or disables the autonomous numeric undo/redo stack across all numeric controls.
    ///
    /// When set to `false`, dragging and inline expression submissions will not record undo transitions,
    /// and `Ctrl+Z` / `Ctrl+Y` shortcuts will be ignored for numeric widgets.
    #[inline]
    pub fn set_numeric_undo_enabled(&mut self, enabled: bool) {
        self.event_state.numeric_undo.set_enabled(enabled);
    }

    /// Registers `tag` as a live/volatile node supporting layout-bypassing in-place refreshes.
    #[inline]
    pub fn register_live_tag(&mut self, tag: u64) {
        self.live_node_tags.insert(tag);
    }

    /// Returns `true` when at least one live/volatile node tag is currently registered in the tree.
    #[inline]
    #[must_use]
    pub fn has_live_nodes(&self) -> bool {
        !self.live_node_tags.is_empty()
    }

    /// Returns `true` when `tag` is registered as a live/volatile node in the tree.
    #[inline]
    #[must_use]
    pub fn is_live_tag(&self, tag: u64) -> bool {
        self.live_node_tags.contains(&tag)
    }

    /// Records that an in-place live update mutated text content requiring glyph shaping.
    #[inline]
    pub fn record_live_text_update(&mut self) {
        self.pending_live_refresh.0 = true;
    }

    /// Records that an in-place live update mutated visual primitives requiring quad re-upload.
    #[inline]
    pub fn record_live_quad_update(&mut self) {
        self.pending_live_refresh.1 = true;
    }

    /// Atomically returns and clears accumulated `(needs_text_prepare, needs_quad_reupload)` flags.
    #[inline]
    #[must_use]
    pub fn take_live_refresh_flags(&mut self) -> (bool, bool) {
        std::mem::take(&mut self.pending_live_refresh)
    }

    /// Atomically returns and clears whether a newly promoted live tag requires a one-time
    /// static text layer refresh.
    #[inline]
    #[must_use]
    pub fn take_static_text_sync_needed(&mut self) -> bool {
        std::mem::replace(&mut self.pending_static_text_sync, false)
    }

    /// Updates the retained text of the node tagged `tag` in place without mutating layout.
    ///
    /// Marks `DirtyFlags::PAINT | DirtyFlags::TEXT` and records live text + quad refresh flags
    /// only when the text actually changed. Returns `true` when the text was updated.
    pub fn update_live_text_by_tag(&mut self, tag: u64, text: &str) -> bool {
        let Some(id) = self.node_tagged(tag).map(|n| n.id) else {
            return false;
        };
        if tag != 0 && self.live_node_tags.insert(tag) {
            self.pending_static_text_sync = true;
        }
        let Some(node) = self.nodes.get_mut(id) else {
            return false;
        };
        if node.text.as_deref() == Some(text) {
            return false;
        }
        if let Some(existing) = &mut node.text {
            existing.clear();
            existing.push_str(text);
        } else {
            node.text = Some(text.to_owned());
        }
        self.mark_dirty(id, DirtyFlags::PAINT | DirtyFlags::TEXT);
        self.record_live_text_update();
        self.record_live_quad_update();
        true
    }
}
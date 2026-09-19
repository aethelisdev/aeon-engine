// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Generational arena-based UI tree manager guaranteeing 100% safe memory operations.

use crate::dirty::DirtyFlags;
use crate::error::IrisCoreError;
use crate::geometry::{Point, Rect};
use crate::id::WidgetId;
use crate::node::{UiLayer, WidgetNode, WidgetRole};
use slotmap::SlotMap;

/// The central hierarchical arena storing all UI nodes.
///
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

    /// Returns the effective stacking layer for a given node, taking parent layer inheritance into account.
    pub fn effective_layer(&self, mut current_id: WidgetId) -> UiLayer {
        let mut max_layer = UiLayer::Background;
        while let Some(node) = self.nodes.get(current_id) {
            if node.layer > max_layer {
                max_layer = node.layer;
            }
            match node.parent {
                Some(parent_id) => current_id = parent_id,
                None => break,
            }
        }
        if max_layer == UiLayer::Background {
            UiLayer::Content
        } else {
            max_layer
        }
    }

    /// Checks if any visible modal window currently exists within the tree.
    ///
    /// When true, interaction with underlying background or docked panels is blocked.
    pub fn is_modal_active(&self) -> bool {
        self.nodes
            .values()
            .any(|n| n.visible && n.layer == UiLayer::Modal)
    }

    /// Performs layered screen-space hit testing, prioritizing higher `UiLayer` stacking contexts.
    ///
    /// Traversal priority:
    /// 1. `UiLayer::Tooltip`
    /// 2. `UiLayer::Popup`
    /// 3. `UiLayer::Modal`
    /// 4. `UiLayer::Floating`
    /// 5. `UiLayer::Content`
    /// 6. `UiLayer::Background`
    ///
    /// If an active `UiLayer::Modal` is visible on screen, hits on lower layers
    /// (`Floating`, `Content`, `Background`) are blocked with zero heap allocations.
    pub fn hit_test_layered(&self, point: Point) -> Option<WidgetId> {
        let root_id = self.root?;
        let mut layer_hits = [None; 6];
        let mut has_active_modal = false;

        self.hit_test_layered_recursive(
            root_id,
            point,
            UiLayer::Background,
            None,
            &mut layer_hits,
            &mut has_active_modal,
        );

        // Tooltip (index 5)
        if let Some(hit) = layer_hits[UiLayer::Tooltip.index()] {
            return Some(hit);
        }
        // Popup (index 4)
        if let Some(hit) = layer_hits[UiLayer::Popup.index()] {
            return Some(hit);
        }
        // Modal (index 3)
        if let Some(hit) = layer_hits[UiLayer::Modal.index()] {
            return Some(hit);
        }

        // If an active modal is displayed on screen, lower layers cannot receive events
        if has_active_modal {
            return None;
        }

        // Floating (index 2)
        if let Some(hit) = layer_hits[UiLayer::Floating.index()] {
            return Some(hit);
        }
        // Content (index 1)
        if let Some(hit) = layer_hits[UiLayer::Content.index()] {
            return Some(hit);
        }
        // Background (index 0)
        layer_hits[UiLayer::Background.index()]
    }

    fn hit_test_layered_recursive(
        &self,
        current_id: WidgetId,
        point: Point,
        inherited_layer: UiLayer,
        clip_rect: Option<Rect>,
        layer_hits: &mut [Option<WidgetId>; 6],
        has_active_modal: &mut bool,
    ) {
        let Some(node) = self.nodes.get(current_id) else {
            return;
        };
        if !node.visible {
            return;
        }

        let effective_layer = if node.layer > inherited_layer {
            node.layer
        } else {
            inherited_layer
        };

        if effective_layer == UiLayer::Modal {
            *has_active_modal = true;
        }

        // Elevated overlay layers break out of parent scissor boundaries
        let effective_clip = if effective_layer > inherited_layer {
            None
        } else {
            clip_rect
        };

        let child_clip = if node.style.clip_children {
            match effective_clip {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            effective_clip
        };

        // Traverse children in reverse order (top-most sibling first)
        for &child_id in node.children.iter().rev() {
            self.hit_test_layered_recursive(
                child_id,
                point,
                effective_layer,
                child_clip,
                layer_hits,
                has_active_modal,
            );
        }

        let is_within_clip = effective_clip.is_none_or(|c| c.contains_point(point));
        if node.interactive && is_within_clip && node.computed_rect.contains_point(point) {
            let idx = effective_layer.index();
            if layer_hits[idx].is_none() {
                layer_hits[idx] = Some(current_id);
            }
        }
    }

    /// Returns the highest `UiLayer` present under the given screen-space point.
    pub fn layer_at(&self, point: Point) -> Option<UiLayer> {
        let root_id = self.root?;
        let mut highest: Option<UiLayer> = None;
        self.find_layer_at_recursive(root_id, point, UiLayer::Background, None, &mut highest);
        highest
    }

    fn find_layer_at_recursive(
        &self,
        current_id: WidgetId,
        point: Point,
        inherited_layer: UiLayer,
        clip_rect: Option<Rect>,
        highest: &mut Option<UiLayer>,
    ) {
        let Some(node) = self.nodes.get(current_id) else {
            return;
        };
        if !node.visible {
            return;
        }
        let effective_layer = if node.layer > inherited_layer {
            node.layer
        } else {
            inherited_layer
        };

        let effective_clip = if effective_layer > inherited_layer {
            None
        } else {
            clip_rect
        };

        let child_clip = if node.style.clip_children {
            match effective_clip {
                Some(existing) => Some(existing.intersect(node.computed_rect)),
                None => Some(node.computed_rect),
            }
        } else {
            effective_clip
        };

        let is_within_clip = effective_clip.is_none_or(|c| c.contains_point(point));
        if is_within_clip && node.computed_rect.contains_point(point) {
            *highest = match *highest {
                Some(prev) => Some(prev.max(effective_layer)),
                None => Some(effective_layer),
            };
        }

        for &child_id in &node.children {
            self.find_layer_at_recursive(child_id, point, effective_layer, child_clip, highest);
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

    /// Performs layered hit-testing and returns detailed metadata for the hit widget.
    ///
    /// Evaluates the highest priority [`UiLayer`] under `point` via [`UiTree::hit_test_layered`],
    /// then extracts the node's properties, role, effective layer, and tag with zero heap allocations for numeric queries.
    pub fn hit_test_target(&self, point: Point) -> Option<HitTargetInfo> {
        let hit_id = self.hit_test_layered(point)?;
        let node = self.nodes.get(hit_id)?;
        let layer = self.effective_layer(hit_id);
        Some(HitTargetInfo {
            id: hit_id,
            layer,
            role: node.role,
            tag: node.tag,
            rect: node.computed_rect,
            name: node.name.clone(),
        })
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

/// Comprehensive metadata and interaction properties of a hit-tested widget.
///
/// Returned by [`UiTree::hit_test_target`] to supply the host application with
/// the widget's identifier, effective stacking layer, functional role, user-defined tag,
/// layout boundary rectangle, and optional debug name in a single query.
#[derive(Debug, Clone, PartialEq)]
pub struct HitTargetInfo {
    /// Generational unique identifier of the hit widget.
    pub id: WidgetId,
    /// Effective stacking context layer computed from hierarchy inheritance.
    pub layer: UiLayer,
    /// Semantic functional role assigned to the widget.
    pub role: WidgetRole,
    /// User-defined numeric tag or action identifier associated with the widget.
    pub tag: u64,
    /// Absolute computed screen-space rectangle of the widget.
    pub rect: Rect,
    /// Optional debug name of the widget node.
    pub name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Rect;

    #[test]
    fn test_hit_test_layered_priority() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        }

        let content_btn = tree.create_node();
        if let Some(node) = tree.get_mut(content_btn) {
            node.computed_rect = Rect::new(100.0, 100.0, 200.0, 50.0);
            node.layer = UiLayer::Content;
        }
        tree.add_child(root, content_btn).unwrap();

        let popup_item = tree.create_node();
        if let Some(node) = tree.get_mut(popup_item) {
            // Popup overlaps content button directly
            node.computed_rect = Rect::new(150.0, 100.0, 100.0, 100.0);
            node.layer = UiLayer::Popup;
        }
        tree.add_child(root, popup_item).unwrap();

        // Hit point inside both popup and content button: Popup MUST win!
        let hit = tree.hit_test_layered(Point::new(160.0, 120.0));
        assert_eq!(hit, Some(popup_item));

        // Hit point inside content button only
        let hit2 = tree.hit_test_layered(Point::new(110.0, 120.0));
        assert_eq!(hit2, Some(content_btn));
    }

    #[test]
    fn test_hit_test_layered_modal_blocking() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1000.0, 1000.0);
        }

        let bg_btn = tree.create_node();
        if let Some(node) = tree.get_mut(bg_btn) {
            node.computed_rect = Rect::new(50.0, 50.0, 100.0, 50.0);
            node.layer = UiLayer::Content;
        }
        tree.add_child(root, bg_btn).unwrap();

        let modal_dlg = tree.create_node();
        if let Some(node) = tree.get_mut(modal_dlg) {
            node.computed_rect = Rect::new(300.0, 200.0, 400.0, 300.0);
            node.layer = UiLayer::Modal;
        }
        tree.add_child(root, modal_dlg).unwrap();

        assert!(tree.is_modal_active());

        // Inside modal: hits modal
        let hit_modal = tree.hit_test_layered(Point::new(350.0, 250.0));
        assert_eq!(hit_modal, Some(modal_dlg));

        // Outside modal over bg_btn: blocked by active modal!
        let hit_outside = tree.hit_test_layered(Point::new(60.0, 60.0));
        assert_eq!(hit_outside, None);
    }

    #[test]
    fn test_hit_test_target_metadata_extraction() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        if let Some(node) = tree.get_mut(root) {
            node.computed_rect = Rect::new(0.0, 0.0, 1920.0, 1080.0);
        }

        let dropdown_item = tree.create_node();
        if let Some(node) = tree.get_mut(dropdown_item) {
            node.computed_rect = Rect::new(200.0, 150.0, 120.0, 24.0);
            node.role = WidgetRole::DropdownItem;
            node.layer = UiLayer::Popup;
            node.tag = 2; // Option index 2
            node.name = Some("Fps120Option".to_string());
        }
        tree.add_child(root, dropdown_item).unwrap();

        let hit_target = tree
            .hit_test_target(Point::new(250.0, 160.0))
            .expect("Dropdown item must be hit");

        assert_eq!(hit_target.id, dropdown_item);
        assert_eq!(hit_target.layer, UiLayer::Popup);
        assert_eq!(hit_target.role, WidgetRole::DropdownItem);
        assert_eq!(hit_target.tag, 2);
        assert_eq!(hit_target.name.as_deref(), Some("Fps120Option"));
        assert_eq!(hit_target.rect, Rect::new(200.0, 150.0, 120.0, 24.0));
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! The central layout engine synchronizing `UiTree` with Taffy's flexbox/grid representation.

use crate::error::LayoutError;
use iris_core::{
    AlignItems, DirtyFlags, FlexDirection, Insets, JustifyContent, Rect, Size, UiTree, WidgetId,
    WidgetRole,
};
use std::collections::HashMap;
use taffy::prelude as tf;

/// The layout manager that synchronizes the `UiTree` with Taffy's internal representation.
pub struct LayoutEngine {
    /// Underlying Taffy layout tree.
    taffy: tf::TaffyTree<()>,
    /// Map from `WidgetId` to Taffy's `NodeId`.
    node_map: HashMap<WidgetId, tf::NodeId>,
    /// Inverse map from Taffy's `NodeId` to `WidgetId`.
    id_map: HashMap<tf::NodeId, WidgetId>,
    /// Last measured available size.
    last_available_size: Option<Size>,
}

impl Default for LayoutEngine {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    /// Creates a new layout engine instance.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            taffy: tf::TaffyTree::new(),
            node_map: HashMap::new(),
            id_map: HashMap::new(),
            last_available_size: None,
        }
    }

    /// Clears internal Taffy state and node mappings when the UI tree is reset.
    #[inline]
    pub fn clear(&mut self) {
        self.taffy.clear();
        self.node_map.clear();
        self.id_map.clear();
        self.last_available_size = None;
    }

    /// Recursively untracks and removes all descendant layout nodes of the specified parent from Taffy.
    ///
    /// Preserves the parent node itself in the layout map, while detaching all children in Taffy
    /// via [`taffy::TaffyTree::set_children`] and purging all descendants from [`self.node_map`]
    /// and [`self.id_map`]. This prevents dangling layout nodes and memory leaks when a panel subtree
    /// is invalidated and reconstructed in-place.
    ///
    /// # Robustness & Ordering Invariant
    /// Traverses and purges descendant nodes directly from both Taffy's internal hierarchy and
    /// the [`UiTree`] structure. This ensures that even if [`UiTree::clear_children`] was called
    /// before `untrack_subtree` (clearing `node.children` in the arena), all descendant Taffy nodes
    /// are still properly discovered and unmapped without memory leaks or dangling layout nodes.
    ///
    /// # Performance
    /// Operates in $O(M)$ time proportional only to the number of descendant nodes in the target subtree,
    /// leaving sibling panels and chrome structures completely untouched.
    pub fn untrack_subtree(&mut self, tree: &UiTree, parent_id: WidgetId) {
        let parent_taffy = self.node_map.get(&parent_id).copied();

        let mut taffy_descendants = Vec::new();
        if let Some(p_taffy) = parent_taffy {
            self.collect_taffy_descendants(p_taffy, &mut taffy_descendants);
            let _ = self.taffy.set_children(p_taffy, &[]);
        }

        // Also collect any descendants recorded in the UiTree (if clear_children was not yet called)
        let mut tree_descendants = Vec::new();
        if let Some(parent_node) = tree.get(parent_id) {
            for &child_id in &parent_node.children {
                tree.collect_subtree(child_id, &mut tree_descendants);
            }
        }

        // Untrack all Taffy nodes identified via Taffy tree traversal
        for taffy_node in taffy_descendants {
            if let Some(widget_id) = self.id_map.remove(&taffy_node) {
                self.node_map.remove(&widget_id);
            }
            let _ = self.taffy.remove(taffy_node);
        }

        // Untrack any remaining nodes identified via UiTree traversal
        for descendant_id in tree_descendants {
            if let Some(taffy_node) = self.node_map.remove(&descendant_id) {
                self.id_map.remove(&taffy_node);
                let _ = self.taffy.remove(taffy_node);
            }
        }
    }

    /// Recursively collects all descendant Taffy node IDs under `parent` in depth-first order.
    fn collect_taffy_descendants(&self, parent: tf::NodeId, list: &mut Vec<tf::NodeId>) {
        if let Ok(children) = self.taffy.children(parent) {
            for child in children {
                list.push(child);
                self.collect_taffy_descendants(child, list);
            }
        }
    }

    /// Returns `true` if the specified widget ID is currently tracked in the layout engine.
    #[inline]
    #[must_use]
    pub fn is_tracked(&self, id: WidgetId) -> bool {
        self.node_map.contains_key(&id)
    }

    /// Returns the total number of tracked layout nodes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.node_map.len()
    }

    /// Returns `true` if the layout engine contains no tracked nodes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.node_map.is_empty()
    }

    /// Computes layout for the entire tree given the viewport/screen available size.
    ///
    /// # Errors
    /// Returns `LayoutError` if Taffy fails to compute layout or if the root node is missing.
    pub fn compute_layout(
        &mut self,
        tree: &mut UiTree,
        available_size: Size,
    ) -> Result<(), LayoutError> {
        let Some(root_id) = tree.root() else {
            self.clear();
            return Ok(());
        };

        let is_same_size = self.last_available_size == Some(available_size);
        let root_is_dirty = tree.get(root_id).is_some_and(|n| {
            n.dirty
                .intersects(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN)
        });

        // Fast path 1: If screen size hasn't changed, root is clean, and NO nodes are marked layout dirty,
        // layout computation is a pure no-op (0 allocations, 0 Taffy calls).
        if is_same_size
            && !root_is_dirty
            && !tree.has_dirty_nodes(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN)
        {
            tree.clear_dirty_boundaries();
            return Ok(());
        }

        // Fast path 2: Invalidation Boundaries - if window size matches, root is clean,
        // and isolated boundary subtrees are dirty, recalculate only those subtrees!
        if is_same_size && !root_is_dirty && !tree.dirty_boundaries.is_empty() {
            let mut stack_buf = [WidgetId::default(); 8];
            let count = tree.dirty_boundaries.len();
            let mut all_ok = true;

            if count <= 8 {
                for (i, &id) in tree.dirty_boundaries.iter().enumerate() {
                    stack_buf[i] = id;
                }
                for id in stack_buf.iter().take(count) {
                    if self.compute_boundary_subtree(tree, *id).is_err() {
                        all_ok = false;
                        break;
                    }
                }
            } else {
                let boundaries: Vec<WidgetId> = tree.dirty_boundaries.iter().copied().collect();
                for id in boundaries {
                    if self.compute_boundary_subtree(tree, id).is_err() {
                        all_ok = false;
                        break;
                    }
                }
            }

            if all_ok && !tree.has_dirty_nodes(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN) {
                tree.clear_dirty_boundaries();
                return Ok(());
            }
        }

        self.last_available_size = Some(available_size);
        tree.clear_dirty_boundaries();

        // Synchronize Taffy tree structure with UiTree
        self.sync_tree(tree, root_id)?;

        let root_taffy_node = *self
            .node_map
            .get(&root_id)
            .ok_or_else(|| LayoutError::Taffy("Root node missing in layout map".into()))?;

        // Run Taffy layout computation
        let available_space = tf::Size {
            width: tf::AvailableSpace::Definite(available_size.width),
            height: tf::AvailableSpace::Definite(available_size.height),
        };

        self.taffy
            .compute_layout(root_taffy_node, available_space)
            .map_err(|e| LayoutError::Taffy(format!("{:?}", e)))?;

        // Write back computed coordinates to UiTree nodes recursively without allocating temporary vectors
        self.apply_layout(tree, root_id, 0.0, 0.0);

        Ok(())
    }

    /// Computes layout exclusively for an isolated invalidation boundary subtree.
    ///
    /// When interactive widgets (sliders, text inputs, hover buttons) change within a panel,
    /// this function recalculates layout for that boundary's subtree using its established
    /// screen coordinates and dimensions without invalidating or traversing the rest of the UI tree.
    ///
    /// The boundary container's absolute screen position `(boundary_rect.x, boundary_rect.y)`
    /// is an external invariant established by the surrounding layout context and is strictly
    /// preserved across partial rebuilds. Recursive layout updates are applied exclusively
    /// to the boundary's descendant children relative to the boundary container's screen origin.
    ///
    /// # Performance
    /// Recalculates only the affected subtree in $O(M)$ time (where $M \ll N$ is the number of nodes
    /// in the boundary) and preserves cached layout coordinates for all sibling panels, achieving
    /// near **0.00 ms** CPU layout cost during continuous dragging or hovering.
    ///
    /// # Errors
    /// Returns [`LayoutError::Taffy`] if the boundary node does not exist in the tree or has non-positive dimensions.
    pub fn compute_boundary_subtree(
        &mut self,
        tree: &mut UiTree,
        boundary_id: WidgetId,
    ) -> Result<(), LayoutError> {
        let Some(boundary_node) = tree.get(boundary_id) else {
            return Err(LayoutError::Taffy(format!(
                "Boundary node {:?} not found in tree",
                boundary_id
            )));
        };

        let boundary_rect = boundary_node.computed_rect;
        if boundary_rect.width <= 0.0 || boundary_rect.height <= 0.0 {
            return Err(LayoutError::Taffy(format!(
                "Boundary node {:?} has non-positive dimensions ({:?}); full layout required",
                boundary_id, boundary_rect
            )));
        }

        // Synchronize boundary subtree with Taffy
        let boundary_taffy_node = self.sync_tree(tree, boundary_id)?;

        let available_space = tf::Size {
            width: tf::AvailableSpace::Definite(boundary_rect.width),
            height: tf::AvailableSpace::Definite(boundary_rect.height),
        };

        self.taffy
            .compute_layout(boundary_taffy_node, available_space)
            .map_err(|e| LayoutError::Taffy(format!("{:?}", e)))?;

        // 1. Maintain boundary container's authoritative screen coordinates and update size if resolved by Taffy
        let boundary_size = self
            .taffy
            .layout(boundary_taffy_node)
            .map_or(boundary_rect.size(), |l| {
                Size::new(l.size.width, l.size.height)
            });

        let updated_boundary_rect = Rect::new(
            boundary_rect.x,
            boundary_rect.y,
            boundary_size.width,
            boundary_size.height,
        );

        if let Some(node) = tree.get_mut(boundary_id) {
            node.computed_rect = updated_boundary_rect;
            node.clear_dirty(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN);
            let tag = node.tag;
            if tag != 0 {
                tree.set_tag_rect(tag, updated_boundary_rect);
            }
        }
        tree.dirty_boundaries.remove(&boundary_id);

        // 2. Recursively apply layout to children relative to boundary container's screen origin
        let scroll_offset_y = tree
            .get(boundary_id)
            .map_or(0.0, |n| n.style.scroll_offset_y);
        let child_parent_y = boundary_rect.y - scroll_offset_y;
        let child_parent_x = boundary_rect.x;

        let child_count = tree.get(boundary_id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            if let Some(child_id) = tree
                .get(boundary_id)
                .and_then(|n| n.children.get(i).copied())
            {
                self.apply_layout(tree, child_id, child_parent_x, child_parent_y);
            }
        }

        Ok(())
    }

    /// Synchronizes the hierarchical structure and styles between `UiTree` and Taffy.
    fn sync_tree(
        &mut self,
        tree: &UiTree,
        current_id: WidgetId,
    ) -> Result<tf::NodeId, LayoutError> {
        let Some(node) = tree.get(current_id) else {
            return Err(LayoutError::Taffy(format!(
                "Node {:?} not found",
                current_id
            )));
        };

        let is_dirty = node
            .dirty
            .intersects(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN);

        let taffy_node = if let Some(&existing) = self.node_map.get(&current_id) {
            if is_dirty {
                let taffy_style = Self::convert_style(node);
                let _ = self.taffy.set_style(existing, taffy_style);
            }
            existing
        } else {
            let taffy_style = Self::convert_style(node);
            let created = self
                .taffy
                .new_leaf(taffy_style)
                .map_err(|e| LayoutError::Taffy(format!("{:?}", e)))?;
            self.node_map.insert(current_id, created);
            self.id_map.insert(created, current_id);
            created
        };

        if is_dirty || node.children.is_empty() {
            let mut child_taffy_nodes = Vec::with_capacity(node.children.len());
            for &child_id in &node.children {
                let child_taffy = self.sync_tree(tree, child_id)?;
                child_taffy_nodes.push(child_taffy);
            }

            self.taffy
                .set_children(taffy_node, &child_taffy_nodes)
                .map_err(|e| LayoutError::Taffy(format!("{:?}", e)))?;
        } else {
            for &child_id in &node.children {
                self.sync_tree(tree, child_id)?;
            }
        }

        Ok(taffy_node)
    }

    /// Recursively applies computed layout offsets and sizes back into the `UiTree` with zero allocation.
    fn apply_layout(&self, tree: &mut UiTree, current_id: WidgetId, parent_x: f32, parent_y: f32) {
        let Some(&taffy_node) = self.node_map.get(&current_id) else {
            return;
        };

        let Ok(layout) = self.taffy.layout(taffy_node) else {
            return;
        };

        let abs_x = parent_x + layout.location.x;
        let abs_y = parent_y + layout.location.y;
        let rect = Rect::new(abs_x, abs_y, layout.size.width, layout.size.height);

        if let Some(node) = tree.get_mut(current_id) {
            node.computed_rect = rect;
            node.clear_dirty(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN);
            let tag = node.tag;
            if tag != 0 {
                tree.set_tag_rect(tag, rect);
            }
        }

        let scroll_offset_y = tree
            .get(current_id)
            .map_or(0.0, |n| n.style.scroll_offset_y);
        let child_parent_y = abs_y - scroll_offset_y;

        let child_count = tree.get(current_id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            if let Some(child_id) = tree
                .get(current_id)
                .and_then(|n| n.children.get(i).copied())
            {
                self.apply_layout(tree, child_id, abs_x, child_parent_y);
            }
        }
    }

    /// Converts Iris `WidgetNode` styling and content dimensions to Taffy `Style`.
    fn convert_style(node: &iris_core::WidgetNode) -> tf::Style {
        let style = &node.style;
        let mut tf_style = tf::Style {
            display: tf::Display::Flex,
            position: match style.position {
                iris_core::Position::Relative => tf::Position::Relative,
                iris_core::Position::Absolute => tf::Position::Absolute,
            },
            inset: tf::Rect {
                top: style.inset_top.map_or_else(
                    tf::LengthPercentageAuto::auto,
                    tf::LengthPercentageAuto::length,
                ),
                right: style.inset_right.map_or_else(
                    tf::LengthPercentageAuto::auto,
                    tf::LengthPercentageAuto::length,
                ),
                bottom: style.inset_bottom.map_or_else(
                    tf::LengthPercentageAuto::auto,
                    tf::LengthPercentageAuto::length,
                ),
                left: style.inset_left.map_or_else(
                    tf::LengthPercentageAuto::auto,
                    tf::LengthPercentageAuto::length,
                ),
            },
            flex_direction: match style.flex_direction {
                FlexDirection::Row => tf::FlexDirection::Row,
                FlexDirection::Column => tf::FlexDirection::Column,
                FlexDirection::RowReverse => tf::FlexDirection::RowReverse,
                FlexDirection::ColumnReverse => tf::FlexDirection::ColumnReverse,
            },
            flex_wrap: match style.flex_wrap {
                iris_core::FlexWrap::NoWrap => tf::FlexWrap::NoWrap,
                iris_core::FlexWrap::Wrap => tf::FlexWrap::Wrap,
                iris_core::FlexWrap::WrapReverse => tf::FlexWrap::WrapReverse,
            },
            align_items: match style.align_items {
                AlignItems::FlexStart => tf::AlignItems::FLEX_START,
                AlignItems::FlexEnd => tf::AlignItems::FLEX_END,
                AlignItems::Center => tf::AlignItems::CENTER,
                AlignItems::Stretch => tf::AlignItems::STRETCH,
            },
            justify_content: match style.justify_content {
                JustifyContent::FlexStart => tf::JustifyContent::FLEX_START,
                JustifyContent::FlexEnd => tf::JustifyContent::FLEX_END,
                JustifyContent::Center => tf::JustifyContent::CENTER,
                JustifyContent::SpaceBetween => tf::JustifyContent::SPACE_BETWEEN,
                JustifyContent::SpaceAround => tf::JustifyContent::SPACE_AROUND,
                JustifyContent::SpaceEvenly => tf::JustifyContent::SPACE_EVENLY,
            },
            gap: tf::Size {
                width: tf::LengthPercentage::length(style.gap),
                height: tf::LengthPercentage::length(style.gap),
            },
            padding: Self::convert_insets(&style.padding),
            margin: Self::convert_insets_margin(&style.margin),
            flex_grow: style.flex_grow,
            flex_shrink: style.flex_shrink,
            ..Default::default()
        };

        let pad_h = style.padding.left
            + style.padding.right
            + style.border.width.left
            + style.border.width.right;
        let pad_v = style.padding.top
            + style.padding.bottom
            + style.border.width.top
            + style.border.width.bottom;

        if let Some(w) = style.width {
            tf_style.size.width = tf::Dimension::length(w);
        } else if node.content_size.width > 0.0 && node.role != WidgetRole::ScrollAreaViewport {
            let total_w = node.content_size.width + pad_h;
            tf_style.min_size.width = tf::LengthPercentageAuto::length(total_w);
            tf_style.size.width = tf::Dimension::length(total_w);
        }

        if let Some(h) = style.height {
            tf_style.size.height = tf::Dimension::length(h);
        } else if node.content_size.height > 0.0 && node.role != WidgetRole::ScrollAreaViewport {
            let total_h = node.content_size.height + pad_v;
            tf_style.min_size.height = tf::LengthPercentageAuto::length(total_h);
            tf_style.size.height = tf::Dimension::length(total_h);
        }

        if node.role == WidgetRole::ScrollAreaViewport && style.min_height.is_none() {
            tf_style.min_size.height = tf::LengthPercentageAuto::length(0.0);
        }

        if let Some(min_w) = style.min_width {
            tf_style.min_size.width = tf::LengthPercentageAuto::length(min_w);
        }
        if let Some(min_h) = style.min_height {
            tf_style.min_size.height = tf::LengthPercentageAuto::length(min_h);
        }

        if let Some(max_w) = style.max_width {
            tf_style.max_size.width = tf::LengthPercentageAuto::length(max_w);
        }
        if let Some(max_h) = style.max_height {
            tf_style.max_size.height = tf::LengthPercentageAuto::length(max_h);
        }

        tf_style
    }

    /// Converts inner padding insets into Taffy length percentage rects.
    fn convert_insets(insets: &Insets) -> tf::Rect<tf::LengthPercentage> {
        tf::Rect {
            top: tf::LengthPercentage::length(insets.top),
            right: tf::LengthPercentage::length(insets.right),
            bottom: tf::LengthPercentage::length(insets.bottom),
            left: tf::LengthPercentage::length(insets.left),
        }
    }

    /// Converts outer margin insets into Taffy auto length percentage rects.
    fn convert_insets_margin(insets: &Insets) -> tf::Rect<tf::LengthPercentageAuto> {
        tf::Rect {
            top: tf::LengthPercentageAuto::length(insets.top),
            right: tf::LengthPercentageAuto::length(insets.right),
            bottom: tf::LengthPercentageAuto::length(insets.bottom),
            left: tf::LengthPercentageAuto::length(insets.left),
        }
    }
}
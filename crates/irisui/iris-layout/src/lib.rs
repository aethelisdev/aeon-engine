// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Layout Engine (`iris-layout`)
//!
//! Adapts the Taffy Flexbox & CSS Grid layout engine to the Retained-Mode `UiTree`,
//! providing dirty-state caching for zero-overhead layout recalculations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use iris_core::{
    AlignItems, DirtyFlags, FlexDirection, Insets, JustifyContent, Rect, Size, UiTree, WidgetId,
};
use std::collections::HashMap;
use taffy::prelude as tf;
use thiserror::Error;

/// Layout computation errors.
#[derive(Debug, Error)]
pub enum LayoutError {
    /// Taffy layout engine error occurred.
    #[error("Taffy layout error: {0}")]
    Taffy(String),
}

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

    /// Computes layout for the entire UI tree if layout dirty flags are set, leveraging
    /// invalidation boundary caching when changes are isolated to individual panels.
    ///
    /// # Fast Paths
    /// 1. **Zero-Dirty Skip**: If no nodes have `LAYOUT` or `CHILDREN` dirty flags set and the available
    ///    dimensions match the previous frame, returns immediately in near **0.00 ms** with zero traversal.
    /// 2. **Invalidation Boundary Sub-Tree Resolution**: If the container size is unchanged, the root
    ///    layout is clean, and dirty nodes are strictly contained within registered [`UiTree::dirty_boundaries`],
    ///    only those isolated subtrees are recalculated in-place. Sibling panels and higher-level chrome
    ///    are preserved from cache without Taffy recomputations.
    /// 3. **Full Re-layout**: If the window size changed or non-boundary nodes are dirty, synchronizes the
    ///    complete hierarchy and computes global layout.
    ///
    /// # Errors
    /// Returns [`LayoutError::Taffy`] if internal layout resolution encounters invalid constraints.
    pub fn compute_layout(
        &mut self,
        tree: &mut UiTree,
        available_size: Size,
    ) -> Result<(), LayoutError> {
        let Some(root_id) = tree.root() else {
            return Ok(());
        };

        // Fast path 1: If layout is completely clean and size matches, return immediately (0.00ms)
        if self.last_available_size == Some(available_size)
            && !tree.has_dirty_nodes(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN)
        {
            return Ok(());
        }

        let is_same_size = self.last_available_size == Some(available_size);
        let root_is_dirty = tree.get(root_id).is_none_or(|n| {
            n.dirty
                .intersects(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN)
        });

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

        let parent_x = boundary_rect.x;
        let parent_y = boundary_rect.y;

        // Synchronize boundary subtree with Taffy
        let boundary_taffy_node = self.sync_tree(tree, boundary_id)?;

        let available_space = tf::Size {
            width: tf::AvailableSpace::Definite(boundary_rect.width),
            height: tf::AvailableSpace::Definite(boundary_rect.height),
        };

        self.taffy
            .compute_layout(boundary_taffy_node, available_space)
            .map_err(|e| LayoutError::Taffy(format!("{:?}", e)))?;

        // Apply updated coordinates to the boundary subtree
        self.apply_layout(tree, boundary_id, parent_x, parent_y);

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
            align_items: Some(match style.align_items {
                AlignItems::FlexStart => tf::AlignItems::FLEX_START,
                AlignItems::FlexEnd => tf::AlignItems::FLEX_END,
                AlignItems::Center => tf::AlignItems::CENTER,
                AlignItems::Stretch => tf::AlignItems::STRETCH,
            }),
            justify_content: Some(match style.justify_content {
                JustifyContent::FlexStart => tf::JustifyContent::FLEX_START,
                JustifyContent::FlexEnd => tf::JustifyContent::FLEX_END,
                JustifyContent::Center => tf::JustifyContent::CENTER,
                JustifyContent::SpaceBetween => tf::JustifyContent::SPACE_BETWEEN,
                JustifyContent::SpaceAround => tf::JustifyContent::SPACE_AROUND,
                JustifyContent::SpaceEvenly => tf::JustifyContent::SPACE_EVENLY,
            }),
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
        } else if node.content_size.width > 0.0 {
            let total_w = node.content_size.width + pad_h;
            tf_style.min_size.width = tf::LengthPercentageAuto::length(total_w);
            tf_style.size.width = tf::Dimension::length(total_w);
        }

        if let Some(h) = style.height {
            tf_style.size.height = tf::Dimension::length(h);
        } else if node.content_size.height > 0.0 {
            let total_h = node.content_size.height + pad_v;
            tf_style.min_size.height = tf::LengthPercentageAuto::length(total_h);
            tf_style.size.height = tf::Dimension::length(total_h);
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

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::Style;

    #[test]
    fn test_layout_computation_simple_column() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let child1 = tree.create_node();
        let child2 = tree.create_node();

        if let Some(node) = tree.get_mut(root) {
            node.set_style(Style::new().flex_col().padding(10.0).gap(5.0));
        }

        if let Some(node) = tree.get_mut(child1) {
            node.set_style(Style::new().width(100.0).height(40.0));
        }

        if let Some(node) = tree.get_mut(child2) {
            node.set_style(Style::new().width(100.0).height(60.0));
        }

        assert!(tree.add_child(root, child1).is_ok());
        assert!(tree.add_child(root, child2).is_ok());

        let mut engine = LayoutEngine::new();
        let result = engine.compute_layout(&mut tree, Size::new(800.0, 600.0));
        assert!(result.is_ok());

        let c1_rect = tree.get(child1).unwrap().computed_rect;
        let c2_rect = tree.get(child2).unwrap().computed_rect;

        // child1 starts at padding (10, 10)
        assert_eq!(c1_rect.x, 10.0);
        assert_eq!(c1_rect.y, 10.0);
        assert_eq!(c1_rect.width, 100.0);
        assert_eq!(c1_rect.height, 40.0);

        // child2 starts after child1 + gap (10 + 40 + 5 = 55)
        assert_eq!(c2_rect.x, 10.0);
        assert_eq!(c2_rect.y, 55.0);
        assert_eq!(c2_rect.width, 100.0);
        assert_eq!(c2_rect.height, 60.0);
    }

    #[test]
    fn test_layout_computation_with_scroll_offset_y() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();
        let child = tree.create_node();

        if let Some(node) = tree.get_mut(root) {
            let mut style = Style::new().flex_col().padding(0.0);
            style.scroll_offset_y = 25.0;
            node.set_style(style);
        }

        if let Some(node) = tree.get_mut(child) {
            node.set_style(Style::new().width(100.0).height(40.0));
        }

        assert!(tree.add_child(root, child).is_ok());

        let mut engine = LayoutEngine::new();
        assert!(
            engine
                .compute_layout(&mut tree, Size::new(800.0, 600.0))
                .is_ok()
        );

        let c_rect = tree.get(child).unwrap().computed_rect;
        // child starts at y = 0.0 - scroll_offset_y (25.0) = -25.0
        assert_eq!(c_rect.y, -25.0);
    }

    #[test]
    fn test_layout_computation_invalidation_boundary_isolation() {
        let mut tree = UiTree::new();
        let root = tree.create_root().unwrap();

        // Root row holding Panel A and Panel B
        if let Some(node) = tree.get_mut(root) {
            node.set_style(Style::new().flex_row().width(800.0).height(600.0));
        }

        // Panel A (Boundary)
        let panel_a = tree.create_node();
        tree.set_boundary(panel_a, true);
        if let Some(node) = tree.get_mut(panel_a) {
            node.set_style(Style::new().flex_col().width(300.0).height(600.0));
        }
        tree.add_child(root, panel_a).unwrap();

        let a_child1 = tree.create_node();
        if let Some(node) = tree.get_mut(a_child1) {
            node.set_style(Style::new().width(200.0).height(50.0));
        }
        tree.add_child(panel_a, a_child1).unwrap();

        let a_child2 = tree.create_node();
        if let Some(node) = tree.get_mut(a_child2) {
            node.set_style(Style::new().width(200.0).height(50.0));
        }
        tree.add_child(panel_a, a_child2).unwrap();

        // Panel B (Boundary)
        let panel_b = tree.create_node();
        tree.set_boundary(panel_b, true);
        if let Some(node) = tree.get_mut(panel_b) {
            node.set_style(Style::new().flex_col().width(500.0).height(600.0));
        }
        tree.add_child(root, panel_b).unwrap();

        let b_child = tree.create_node();
        if let Some(node) = tree.get_mut(b_child) {
            node.set_style(Style::new().width(400.0).height(100.0));
        }
        tree.add_child(panel_b, b_child).unwrap();

        let mut engine = LayoutEngine::new();
        assert!(
            engine
                .compute_layout(&mut tree, Size::new(800.0, 600.0))
                .is_ok()
        );

        // Verify initial layout
        assert_eq!(tree.get(panel_a).unwrap().computed_rect.x, 0.0);
        assert_eq!(tree.get(panel_b).unwrap().computed_rect.x, 300.0);
        assert_eq!(tree.get(a_child1).unwrap().computed_rect.y, 0.0);
        assert_eq!(tree.get(a_child2).unwrap().computed_rect.y, 50.0);
        assert_eq!(tree.get(b_child).unwrap().computed_rect.x, 300.0);

        // Mutate a_child1 inside Panel A and mark dirty
        if let Some(node) = tree.get_mut(a_child1) {
            node.set_style(Style::new().width(200.0).height(90.0));
        }
        let boundary = tree.mark_dirty(a_child1, DirtyFlags::LAYOUT);
        assert_eq!(boundary, Some(panel_a));

        // Root and Panel B must be completely clean of layout flags
        assert!(
            !tree
                .get(root)
                .unwrap()
                .dirty
                .intersects(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN)
        );
        assert!(
            !tree
                .get(panel_b)
                .unwrap()
                .dirty
                .intersects(DirtyFlags::LAYOUT | DirtyFlags::CHILDREN)
        );
        assert_eq!(tree.dirty_boundaries().len(), 1);

        // Execute incremental boundary layout
        assert!(
            engine
                .compute_layout(&mut tree, Size::new(800.0, 600.0))
                .is_ok()
        );

        // Verify that a_child2 moved down to y = 90.0
        assert_eq!(tree.get(a_child1).unwrap().computed_rect.height, 90.0);
        assert_eq!(tree.get(a_child2).unwrap().computed_rect.y, 90.0);

        // Verify that Panel B and b_child positions were preserved untouched
        assert_eq!(tree.get(panel_b).unwrap().computed_rect.x, 300.0);
        assert_eq!(tree.get(b_child).unwrap().computed_rect.x, 300.0);

        // dirty_boundaries set must now be clean
        assert!(!tree.has_dirty_boundaries());
        assert!(!tree.has_dirty_nodes(DirtyFlags::LAYOUT));
    }
}
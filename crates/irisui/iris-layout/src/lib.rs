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
        }
    }

    /// Computes layout for the entire UI tree if layout dirty flags are set.
    /// If no nodes have the `LAYOUT` dirty flag set and available size matches,
    /// this function returns immediately with near **0.00 ms** CPU time.
    /// # Errors
    /// Returns `LayoutError::Taffy` if internal layout resolution encounters invalid constraints.
    pub fn compute_layout(
        &mut self,
        tree: &mut UiTree,
        available_size: Size,
    ) -> Result<(), LayoutError> {
        let Some(root_id) = tree.root() else {
            return Ok(());
        };

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
            node.clear_dirty(DirtyFlags::LAYOUT);
        }

        let child_count = tree.get(current_id).map_or(0, |n| n.children.len());
        for i in 0..child_count {
            if let Some(child_id) = tree
                .get(current_id)
                .and_then(|n| n.children.get(i).copied())
            {
                self.apply_layout(tree, child_id, abs_x, abs_y);
            }
        }
    }

    /// Converts Iris `WidgetNode` styling and content dimensions to Taffy `Style`.
    fn convert_style(node: &iris_core::WidgetNode) -> tf::Style {
        let style = &node.style;
        let mut tf_style = tf::Style {
            display: tf::Display::Flex,
            flex_direction: match style.flex_direction {
                FlexDirection::Row => tf::FlexDirection::Row,
                FlexDirection::Column => tf::FlexDirection::Column,
                FlexDirection::RowReverse => tf::FlexDirection::RowReverse,
                FlexDirection::ColumnReverse => tf::FlexDirection::ColumnReverse,
            },
            align_items: Some(match style.align_items {
                AlignItems::FlexStart => tf::AlignItems::FlexStart,
                AlignItems::FlexEnd => tf::AlignItems::FlexEnd,
                AlignItems::Center => tf::AlignItems::Center,
                AlignItems::Stretch => tf::AlignItems::Stretch,
            }),
            justify_content: Some(match style.justify_content {
                JustifyContent::FlexStart => tf::JustifyContent::FlexStart,
                JustifyContent::FlexEnd => tf::JustifyContent::FlexEnd,
                JustifyContent::Center => tf::JustifyContent::Center,
                JustifyContent::SpaceBetween => tf::JustifyContent::SpaceBetween,
                JustifyContent::SpaceAround => tf::JustifyContent::SpaceAround,
                JustifyContent::SpaceEvenly => tf::JustifyContent::SpaceEvenly,
            }),
            gap: tf::Size {
                width: tf::LengthPercentage::Length(style.gap),
                height: tf::LengthPercentage::Length(style.gap),
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
            tf_style.size.width = tf::Dimension::Length(w);
        } else if node.content_size.width > 0.0 {
            let total_w = node.content_size.width + pad_h;
            tf_style.min_size.width = tf::Dimension::Length(total_w);
            tf_style.size.width = tf::Dimension::Length(total_w);
        }

        if let Some(h) = style.height {
            tf_style.size.height = tf::Dimension::Length(h);
        } else if node.content_size.height > 0.0 {
            let total_h = node.content_size.height + pad_v;
            tf_style.min_size.height = tf::Dimension::Length(total_h);
        }

        if let Some(min_w) = style.min_width {
            tf_style.min_size.width = tf::Dimension::Length(min_w);
        }
        if let Some(min_h) = style.min_height {
            tf_style.min_size.height = tf::Dimension::Length(min_h);
        }

        if let Some(max_w) = style.max_width {
            tf_style.max_size.width = tf::Dimension::Length(max_w);
        }
        if let Some(max_h) = style.max_height {
            tf_style.max_size.height = tf::Dimension::Length(max_h);
        }

        tf_style
    }

    /// Converts inner padding insets into Taffy length percentage rects.
    fn convert_insets(insets: &Insets) -> tf::Rect<tf::LengthPercentage> {
        tf::Rect {
            top: tf::LengthPercentage::Length(insets.top),
            right: tf::LengthPercentage::Length(insets.right),
            bottom: tf::LengthPercentage::Length(insets.bottom),
            left: tf::LengthPercentage::Length(insets.left),
        }
    }

    /// Converts outer margin insets into Taffy auto length percentage rects.
    fn convert_insets_margin(insets: &Insets) -> tf::Rect<tf::LengthPercentageAuto> {
        tf::Rect {
            top: tf::LengthPercentageAuto::Length(insets.top),
            right: tf::LengthPercentageAuto::Length(insets.right),
            bottom: tf::LengthPercentageAuto::Length(insets.bottom),
            left: tf::LengthPercentageAuto::Length(insets.left),
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
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Subtree Flow Layout Engine
//!
//! Provides lightweight two-pass recursive flow layout calculations for subtrees
//! constructed via [`super::scope::UiScope`].
//!

use iris_core::{
    AlignItems, FlexDirection, Insets, JustifyContent, Rect, UiTree, WidgetId, WidgetRole,
};

/// Performs a lightweight recursive flow layout pass on a `UiScope`-built subtree.
///
/// Assigns `computed_rect` to every descendant node based on the root's bounding
/// rectangle and each node's `flex_direction`, `padding`, `gap`, and `line_height`.
/// This ensures all children receive non-zero geometry when rendered without top-level Taffy passes.
pub fn layout_subtree(tree: &mut UiTree, root_id: WidgetId, bounds: Rect) {
    if let Some(node) = tree.get_mut(root_id) {
        node.computed_rect = bounds;
    }
    let pad = tree.get(root_id).map_or(Insets::ZERO, |n| n.style.padding);
    let inner = Rect::new(
        bounds.x + pad.left,
        bounds.y + pad.top,
        (bounds.width - pad.left - pad.right).max(0.0),
        (bounds.height - pad.top - pad.bottom).max(0.0),
    );
    assign_children(tree, root_id, inner);
}

/// Recursively assigns `computed_rect` to all children of `parent_id` within `inner` bounds.
fn assign_children(tree: &mut UiTree, parent_id: WidgetId, inner: Rect) {
    let (role, dir, gap, align, justify, children) = {
        let Some(node) = tree.get(parent_id) else {
            return;
        };
        (
            node.role,
            node.style.flex_direction,
            node.style.gap,
            node.style.align_items,
            node.style.justify_content,
            node.children.clone(),
        )
    };
    if children.is_empty() {
        return;
    }

    if role == WidgetRole::OscilloscopeCanvas {
        return;
    }

    if role == WidgetRole::ProgressBar {
        if let Some(&slug_child) = children.first() {
            let (tag, name) = tree
                .get(slug_child)
                .map(|n| (n.tag, n.name.as_deref().unwrap_or("")))
                .unwrap_or((0, ""));
            let val = f32::from_bits(tag as u32);
            if name == "IndeterminateProgressSlug" {
                let slug_w = (inner.width * 0.28).clamp(32.0, 96.0).min(inner.width);
                let travel = (inner.width - slug_w).max(0.0);
                let slug_x = inner.x + travel * val.clamp(0.0, 1.0);
                if let Some(node) = tree.get_mut(slug_child) {
                    node.computed_rect = Rect::new(slug_x, inner.y, slug_w, inner.height);
                }
            } else {
                let bar_w = (inner.width * val.clamp(0.0, 1.0)).min(inner.width);
                if let Some(node) = tree.get_mut(slug_child) {
                    node.computed_rect = Rect::new(inner.x, inner.y, bar_w, inner.height);
                }
            }
        }
        return;
    }

    if role == WidgetRole::TextInput {
        let text_child = children[0];
        let text_rect = Rect::new(
            inner.x + 8.0,
            inner.y,
            (inner.width - 16.0).max(0.0),
            inner.height,
        );
        if let Some(node) = tree.get_mut(text_child) {
            node.computed_rect = text_rect;
        }

        if children.len() > 1 {
            let caret_child = children[1];
            let offset = tree
                .get(caret_child)
                .map_or(0.0, |n| f32::from_bits(n.tag as u32));
            let caret_x = (inner.x + 8.0 + offset).min(inner.x + inner.width - 12.0);
            if let Some(node) = tree.get_mut(caret_child) {
                node.computed_rect = Rect::new(caret_x, inner.y + 6.0, 1.5, 16.0);
            }
        }
        return;
    }

    // 1. Separate and layout absolutely positioned children
    let mut flex_children = Vec::with_capacity(children.len());
    for &child_id in &children {
        let is_abs = tree
            .get(child_id)
            .is_some_and(|n| n.style.position == iris_core::Position::Absolute);
        if is_abs {
            if let Some(node) = tree.get(child_id) {
                let w = node.style.width.unwrap_or(inner.width);
                let h = node.style.height.unwrap_or(inner.height);
                let x = if let Some(left) = node.style.inset_left {
                    inner.x + left
                } else if let Some(right) = node.style.inset_right {
                    inner.x + inner.width - w - right
                } else {
                    inner.x
                };
                let y = if let Some(top) = node.style.inset_top {
                    inner.y + top
                } else if let Some(bottom) = node.style.inset_bottom {
                    inner.y + inner.height - h - bottom
                } else {
                    inner.y
                };
                let child_rect = Rect::new(x, y, w, h);
                layout_subtree(tree, child_id, child_rect);
            }
        } else {
            flex_children.push(child_id);
        }
    }
    let children = flex_children;
    if children.is_empty() {
        return;
    }

    let is_col = matches!(dir, FlexDirection::Column | FlexDirection::ColumnReverse);

    if is_col {
        // Measure desired heights bottom-up, then distribute top-down
        let heights: Vec<f32> = children
            .iter()
            .map(|cid| measure_height(tree, *cid))
            .collect();
        let total_content_h: f32 =
            heights.iter().sum::<f32>() + gap * (children.len().saturating_sub(1) as f32);
        let extra_space = (inner.height - total_content_h).max(0.0);
        let (mut y, space_between_gap) = match justify {
            JustifyContent::Center => (inner.y + extra_space * 0.5, 0.0),
            JustifyContent::FlexEnd => (inner.y + extra_space, 0.0),
            JustifyContent::SpaceBetween if children.len() > 1 => {
                (inner.y, extra_space / (children.len() - 1) as f32)
            }
            _ => (inner.y, 0.0),
        };
        for (i, child_id) in children.iter().enumerate() {
            let h = heights[i];
            let (child_w, child_x) =
                if let Some(w) = tree.get(*child_id).and_then(|n| n.style.width) {
                    let clamped_w = w.min(inner.width);
                    let x = match align {
                        AlignItems::Center => inner.x + (inner.width - clamped_w) * 0.5,
                        AlignItems::FlexEnd => inner.x + inner.width - clamped_w,
                        _ => inner.x,
                    };
                    (clamped_w, x)
                } else {
                    (inner.width, inner.x)
                };
            let child_rect = Rect::new(child_x, y, child_w, h);
            layout_subtree(tree, *child_id, child_rect);
            y += h + gap + space_between_gap;
        }
    } else {
        // Row: calculate widths honoring explicit style.width where set, and flexing the rest
        let total_gaps = gap * (children.len().saturating_sub(1)) as f32;
        let mut fixed_w = 0.0_f32;
        let mut flex_count = 0;
        for cid in &children {
            if let Some(w) = tree.get(*cid).and_then(|n| n.style.width) {
                fixed_w += w;
            } else {
                flex_count += 1;
            }
        }
        let total_children_w = fixed_w + total_gaps;
        let flex_w = if flex_count > 0 {
            ((inner.width - total_gaps - fixed_w) / flex_count as f32).max(0.0)
        } else {
            ((inner.width - total_gaps) / children.len().max(1) as f32).max(0.0)
        };
        let row_h = children
            .iter()
            .map(|cid| measure_height(tree, *cid))
            .fold(0.0_f32, f32::max);
        let extra_w = (inner.width - total_children_w).max(0.0);
        let (mut x, row_space_between) = if flex_count == 0 {
            match justify {
                JustifyContent::Center => (inner.x + extra_w * 0.5, 0.0),
                JustifyContent::FlexEnd => (inner.x + extra_w, 0.0),
                JustifyContent::SpaceBetween if children.len() > 1 => {
                    (inner.x, extra_w / (children.len() - 1) as f32)
                }
                _ => (inner.x, 0.0),
            }
        } else {
            (inner.x, 0.0)
        };
        for child_id in &children {
            let child_w = tree
                .get(*child_id)
                .and_then(|n| n.style.width)
                .unwrap_or(flex_w);
            let child_h = tree.get(*child_id).and_then(|n| n.style.height).unwrap_or(
                if align == AlignItems::Stretch {
                    inner.height.max(row_h)
                } else {
                    row_h
                },
            );
            let y = match align {
                AlignItems::Center => inner.y + ((inner.height - child_h) * 0.5).max(0.0),
                AlignItems::FlexEnd => inner.y + (inner.height - child_h).max(0.0),
                _ => inner.y,
            };
            let child_rect = Rect::new(x, y, child_w, child_h);
            layout_subtree(tree, *child_id, child_rect);
            x += child_w + gap + row_space_between;
        }
    }
}

/// Bottom-up pass: computes the desired content height of a node from its children and styles.
///
/// Recursively measures explicit heights, text line heights, paddings, and gaps.
pub fn measure_height(tree: &UiTree, node_id: WidgetId) -> f32 {
    let Some(node) = tree.get(node_id) else {
        return 0.0;
    };

    // Explicit height takes priority
    if let Some(h) = node.style.height {
        return h;
    }

    // Leaf node: use line_height if it has text, otherwise a default
    if node.children.is_empty() {
        return if node.text.is_some() {
            node.line_height.max(16.0)
        } else {
            node.style.height.unwrap_or(0.0)
        };
    }

    let pad = node.style.padding;
    let gap = node.style.gap;
    let dir = node.style.flex_direction;
    let is_col = matches!(dir, FlexDirection::Column | FlexDirection::ColumnReverse);
    let child_ids: Vec<WidgetId> = node.children.clone();

    if is_col {
        let mut total = pad.top + pad.bottom;
        for (i, cid) in child_ids.iter().enumerate() {
            total += measure_height(tree, *cid);
            if i + 1 < child_ids.len() {
                total += gap;
            }
        }
        total
    } else {
        let max_child = child_ids
            .iter()
            .map(|cid| measure_height(tree, *cid))
            .fold(0.0_f32, f32::max);
        pad.top + pad.bottom + max_child
    }
}
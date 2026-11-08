// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Mouse Scroll & Viewport Bubbling Handlers
//!
//! Traverses the UI hierarchy upward from the hit widget to resolve enclosing scroll containers
//! and viewport nodes, updating scroll offsets and synchronizing attached scrollbars.

use super::scrollbar::sync_scrollbar_thumb_geometry;
use crate::dispatch::types::DispatchResult;
use crate::geometry::Point;
use crate::node::WidgetRole;
use crate::response::{ResponseFlags, WidgetResponse};
use crate::tree::UiTree;

/// Traverses upward through ancestors looking for a scroll container (`clip_children`).
///
/// Note: Interactive child widgets (e.g. buttons, inputs) deliberately DO NOT intercept
/// or block mouse scroll events, allowing bubbling to reach the enclosing scroll container.
pub(crate) fn handle_mouse_scroll(
    tree: &mut UiTree,
    delta_x: f32,
    delta_y: f32,
    point: Point,
) -> DispatchResult {
    let Some(hit_id) = tree.hit_test_layered(point) else {
        return DispatchResult::ignored();
    };

    // Bubble upward to find the nearest scroll-handling ancestor.
    // Interactive children (buttons, sliders, inputs) must NOT stop this bubbling.
    let mut current = Some(hit_id);
    while let Some(curr_id) = current {
        if let Some(node) = tree.get(curr_id) {
            let is_explicit_viewport = node.role == WidgetRole::ScrollAreaViewport;
            let is_generic_scrollable = node.role != WidgetRole::Button
                && (node.role != WidgetRole::TextInput
                    || node.content_size.height > node.computed_rect.height + 0.5)
                && (node.style.clip_children && node.tag != 0);

            let is_scroll_container = is_explicit_viewport || is_generic_scrollable;

            let has_overflow = if is_scroll_container {
                if node.content_size.height > 0.0 {
                    node.content_size.height > node.computed_rect.height + 0.5
                        || node.style.scroll_offset_y > 0.0
                } else {
                    true
                }
            } else {
                false
            };

            if is_scroll_container {
                if has_overflow {
                    let tag = node.tag;
                    let rect = node.computed_rect;
                    let max_scroll = if node.content_size.height > node.computed_rect.height {
                        node.content_size.height - node.computed_rect.height
                    } else if node.content_size.height == 0.0 {
                        f32::MAX
                    } else {
                        node.style.scroll_offset_y
                    };

                    // Directly update the container's vertical scroll offset in the tree
                    let new_scroll = if let Some(mut_node) = tree.get_mut(curr_id) {
                        let ns = (mut_node.style.scroll_offset_y - delta_y).clamp(0.0, max_scroll);
                        mut_node.style.scroll_offset_y = ns;
                        ns
                    } else {
                        0.0
                    };
                    if tag != 0 {
                        tree.set_scroll_offset(tag, new_scroll);
                    }

                    // Autonomous synchronization of attached scrollbar thumb geometry
                    sync_scrollbar_thumb_geometry(tree, curr_id);

                    return DispatchResult {
                        consumed: true,
                        needs_redraw: true,
                        target: Some(curr_id),
                        target_tag: tag,
                        cursor: None,
                        response: Some(WidgetResponse::from_flags(
                            curr_id,
                            rect,
                            ResponseFlags::CHANGED,
                        )),
                        scroll_delta: (delta_x, delta_y),
                        key: None,
                        is_key_pressed: false,
                        text_input: None,
                        next_timeout: None,
                    };
                } else {
                    return DispatchResult {
                        consumed: true,
                        needs_redraw: false,
                        target: Some(curr_id),
                        target_tag: node.tag,
                        cursor: None,
                        response: Some(WidgetResponse::from_flags(
                            curr_id,
                            node.computed_rect,
                            ResponseFlags::HOVERED,
                        )),
                        scroll_delta: (0.0, 0.0),
                        key: None,
                        is_key_pressed: false,
                        text_input: None,
                        next_timeout: None,
                    };
                }
            }
            current = node.parent;
        } else {
            break;
        }
    }

    DispatchResult::ignored()
}
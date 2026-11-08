// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scrollbar Geometry & Viewport Association Helpers
//!
//! Provides autonomous scrollbar thumb dragging math, track click-to-jump gestures,
//! thumb geometry synchronization with viewport offset, and tree hierarchy lookup routines.

use crate::geometry::Point;
use crate::id::WidgetId;
use crate::node::{WidgetCursor, WidgetRole};
use crate::tree::UiTree;

/// Translates a thumb vertical displacement delta into a proportional viewport scroll offset.
pub(crate) fn update_scrollbar_drag(
    tree: &mut UiTree,
    thumb_id: WidgetId,
    delta_y: f32,
    initial_scroll: f32,
) {
    let Some((vp_id, track_id)) = find_associated_viewport_and_track(tree, thumb_id) else {
        return;
    };

    let (track_rect, thumb_h) = {
        let Some(track) = tree.get(track_id) else {
            return;
        };
        let Some(thumb) = tree.get(thumb_id) else {
            return;
        };
        (track.computed_rect, thumb.computed_rect.height)
    };

    let max_scroll_y = {
        let Some(vp) = tree.get(vp_id) else {
            return;
        };
        (vp.content_size.height - vp.computed_rect.height).max(0.0)
    };

    if max_scroll_y <= 0.0 {
        return;
    }

    let travel = (track_rect.height - thumb_h).max(1.0);
    let delta_scroll = (delta_y / travel) * max_scroll_y;
    let new_scroll = (initial_scroll + delta_scroll).clamp(0.0, max_scroll_y);

    let vp_tag = tree.get(vp_id).map_or(0, |n| n.tag);
    if let Some(vp) = tree.get_mut(vp_id) {
        vp.style.scroll_offset_y = new_scroll;
    }
    if vp_tag != 0 {
        tree.set_scroll_offset(vp_tag, new_scroll);
    }

    let ratio = (new_scroll / max_scroll_y).clamp(0.0, 1.0);
    let new_thumb_y = track_rect.y + ratio * travel;
    if let Some(thumb) = tree.get_mut(thumb_id) {
        thumb.computed_rect.y = new_thumb_y;
    }
}

/// Calculates and performs a direct jump scroll when the scrollbar track is clicked.
pub(crate) fn perform_track_jump_scroll(
    tree: &mut UiTree,
    vp_id: WidgetId,
    track_id: WidgetId,
    thumb_id: WidgetId,
    point: Point,
) {
    let (track_rect, thumb_h) = {
        let Some(track) = tree.get(track_id) else {
            return;
        };
        let Some(thumb) = tree.get(thumb_id) else {
            return;
        };
        (track.computed_rect, thumb.computed_rect.height)
    };

    let max_scroll_y = {
        let Some(vp) = tree.get(vp_id) else {
            return;
        };
        (vp.content_size.height - vp.computed_rect.height).max(0.0)
    };

    if max_scroll_y <= 0.0 {
        return;
    }

    let travel = (track_rect.height - thumb_h).max(1.0);
    let click_offset_y = point.y - track_rect.y - (thumb_h * 0.5);
    let ratio = (click_offset_y / travel).clamp(0.0, 1.0);
    let new_scroll = ratio * max_scroll_y;

    let vp_tag = tree.get(vp_id).map_or(0, |n| n.tag);
    if let Some(vp) = tree.get_mut(vp_id) {
        vp.style.scroll_offset_y = new_scroll;
    }
    if vp_tag != 0 {
        tree.set_scroll_offset(vp_tag, new_scroll);
    }

    let new_thumb_y = track_rect.y + ratio * travel;
    if let Some(thumb) = tree.get_mut(thumb_id) {
        thumb.computed_rect.y = new_thumb_y;
    }
}

/// Synchronizes a scrollbar thumb node's vertical coordinate with its associated viewport's offset.
pub(crate) fn sync_scrollbar_thumb_geometry(tree: &mut UiTree, viewport_id: WidgetId) {
    let (scroll_y, max_scroll_y) = {
        let Some(vp) = tree.get(viewport_id) else {
            return;
        };
        let max_scroll = (vp.content_size.height - vp.computed_rect.height).max(0.0);
        (vp.style.scroll_offset_y, max_scroll)
    };

    let Some((track_id, thumb_id)) = find_track_and_thumb_for_viewport(tree, viewport_id) else {
        return;
    };

    let (track_rect, thumb_h) = {
        let Some(track) = tree.get(track_id) else {
            return;
        };
        let Some(thumb) = tree.get(thumb_id) else {
            return;
        };
        (track.computed_rect, thumb.computed_rect.height)
    };

    let travel = (track_rect.height - thumb_h).max(0.0);
    let ratio = if max_scroll_y > 0.0 {
        (scroll_y / max_scroll_y).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let new_thumb_y = track_rect.y + ratio * travel;
    if let Some(thumb) = tree.get_mut(thumb_id) {
        thumb.computed_rect.y = new_thumb_y;
    }
}

/// Locates the associated viewport and scrollbar track for a given scrollbar thumb node.
pub(crate) fn find_viewport_in_subtree(
    tree: &UiTree,
    root_id: WidgetId,
    exclude: WidgetId,
) -> Option<WidgetId> {
    let mut stack = [WidgetId::default(); 32];
    let mut head = 0;
    if let Some(node) = tree.get(root_id) {
        for &child in &node.children {
            if child != exclude && head < stack.len() {
                stack[head] = child;
                head += 1;
            }
        }
    }
    while head > 0 {
        head -= 1;
        let curr_id = stack[head];
        if let Some(node) = tree.get(curr_id) {
            if node.role == WidgetRole::ScrollAreaViewport {
                return Some(curr_id);
            }
            for &child in &node.children {
                if child != exclude && head < stack.len() {
                    stack[head] = child;
                    head += 1;
                }
            }
        }
    }
    None
}

/// Locates the associated viewport and scrollbar track for a given scrollbar thumb node.
pub(crate) fn find_associated_viewport_and_track(
    tree: &UiTree,
    thumb_id: WidgetId,
) -> Option<(WidgetId, WidgetId)> {
    let parent_id = tree.get(thumb_id)?.parent?;
    let parent = tree.get(parent_id)?;

    // If parent is directly the viewport
    if parent.role == WidgetRole::ScrollAreaViewport
        || (parent.style.clip_children && parent.content_size.height > 0.0)
    {
        let track_id = parent.children.iter().copied().find(|&c| {
            tree.get(c)
                .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
        })?;
        return Some((parent_id, track_id));
    }

    // If thumb and track are siblings of the viewport under a shared parent container
    let vp_id = parent
        .children
        .iter()
        .copied()
        .find(|&c| {
            tree.get(c).is_some_and(|n| {
                n.role == WidgetRole::ScrollAreaViewport
                    || (n.style.clip_children && n.content_size.height > 0.0 && c != thumb_id)
            })
        })
        .or_else(|| find_viewport_in_subtree(tree, parent_id, thumb_id))?;

    let track_id = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
    })?;

    Some((vp_id, track_id))
}

/// Locates the associated viewport and scrollbar thumb for a given scrollbar track node.
pub(crate) fn find_associated_viewport_and_thumb(
    tree: &UiTree,
    track_id: WidgetId,
) -> Option<(WidgetId, WidgetId)> {
    let parent_id = tree.get(track_id)?.parent?;
    let parent = tree.get(parent_id)?;

    if parent.role == WidgetRole::ScrollAreaViewport
        || (parent.style.clip_children && parent.content_size.height > 0.0)
    {
        let thumb_id = parent.children.iter().copied().find(|&c| {
            tree.get(c)
                .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
        })?;
        return Some((parent_id, thumb_id));
    }

    let vp_id = parent
        .children
        .iter()
        .copied()
        .find(|&c| {
            tree.get(c).is_some_and(|n| {
                n.role == WidgetRole::ScrollAreaViewport
                    || (n.style.clip_children && n.content_size.height > 0.0 && c != track_id)
            })
        })
        .or_else(|| find_viewport_in_subtree(tree, parent_id, track_id))?;

    let thumb_id = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
    })?;

    Some((vp_id, thumb_id))
}

/// Locates the scrollbar track and thumb nodes belonging to a viewport node.
pub(crate) fn find_track_and_thumb_for_viewport(
    tree: &UiTree,
    viewport_id: WidgetId,
) -> Option<(WidgetId, WidgetId)> {
    let vp = tree.get(viewport_id)?;

    // Check children first
    let track_child = vp.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
    });
    let thumb_child = vp.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
    });

    if let (Some(t), Some(th)) = (track_child, thumb_child) {
        return Some((t, th));
    }

    // Check sibling elements under parent
    let parent_id = vp.parent?;
    let parent = tree.get(parent_id)?;

    let track_sib = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarTrack)
    })?;
    let thumb_sib = parent.children.iter().copied().find(|&c| {
        tree.get(c)
            .is_some_and(|n| n.role == WidgetRole::ScrollBarThumb)
    })?;

    Some((track_sib, thumb_sib))
}

/// Resolves the effective mouse cursor icon by checking the node and bubbling to ancestors.
pub(crate) fn resolve_effective_cursor(tree: &UiTree, start_id: WidgetId) -> Option<WidgetCursor> {
    let mut current = Some(start_id);
    while let Some(curr_id) = current {
        if let Some(node) = tree.get(curr_id) {
            if let Some(cursor) = node.cursor.or_else(|| node.role.default_cursor()) {
                return Some(cursor);
            }
            current = node.parent;
        } else {
            break;
        }
    }
    None
}
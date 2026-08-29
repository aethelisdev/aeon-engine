// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 5-Way Drop zone hit-testing, drag-drop state transitions, and highlight geometry.

use crate::tree::DockNodeId;
use iris_core::{Point, Rect};
use serde::{Deserialize, Serialize};

/// Identifies the specific target drop zone computed when dragging a panel over a dock leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DropZone {
    /// Inserts the dragged tab into the target leaf as an additional tab.
    Center,
    /// Partitions the target leaf horizontally, placing the tab in a new left sub-pane.
    Left,
    /// Partitions the target leaf horizontally, placing the tab in a new right sub-pane.
    Right,
    /// Partitions the target leaf vertically, placing the tab in a new top sub-pane.
    Top,
    /// Partitions the target leaf vertically, placing the tab in a new bottom sub-pane.
    Bottom,
}

/// Active drag state when a user is tearing off or moving a tab across the layout.
#[derive(Debug, Clone)]
pub struct DockDragState<T> {
    /// Identifier of the leaf from which the tab was detached.
    pub source_leaf: DockNodeId,
    /// Original index of the detached tab within the source leaf.
    pub source_tab_index: usize,
    /// Detached tab payload data.
    pub tab_data: T,
    /// Current screen-space cursor position.
    pub cursor_pos: Point,
    /// Active drop target leaf and zone, if hovering over a valid target.
    pub hover_target: Option<(DockNodeId, DropZone, Rect)>,
}

/// Calculates the active 5-way drop zone within a target leaf content rectangle.
pub fn calculate_drop_zone(content_rect: Rect, cursor_pos: Point) -> Option<DropZone> {
    if !content_rect.contains_point(cursor_pos)
        || content_rect.width <= 0.0
        || content_rect.height <= 0.0
    {
        return None;
    }

    let u = (cursor_pos.x - content_rect.x) / content_rect.width;
    let v = (cursor_pos.y - content_rect.y) / content_rect.height;

    // Check 25% edge margins first
    if u < 0.25 {
        Some(DropZone::Left)
    } else if u > 0.75 {
        Some(DropZone::Right)
    } else if v < 0.25 {
        Some(DropZone::Top)
    } else if v > 0.75 {
        Some(DropZone::Bottom)
    } else {
        Some(DropZone::Center)
    }
}

/// Calculates the semi-transparent highlight preview rectangle for a drop zone.
pub fn calculate_drop_preview_rect(content_rect: Rect, zone: DropZone) -> Rect {
    match zone {
        DropZone::Center => content_rect,
        DropZone::Left => Rect::new(
            content_rect.x,
            content_rect.y,
            content_rect.width * 0.5,
            content_rect.height,
        ),
        DropZone::Right => Rect::new(
            content_rect.x + content_rect.width * 0.5,
            content_rect.y,
            content_rect.width * 0.5,
            content_rect.height,
        ),
        DropZone::Top => Rect::new(
            content_rect.x,
            content_rect.y,
            content_rect.width,
            content_rect.height * 0.5,
        ),
        DropZone::Bottom => Rect::new(
            content_rect.x,
            content_rect.y + content_rect.height * 0.5,
            content_rect.width,
            content_rect.height * 0.5,
        ),
    }
}
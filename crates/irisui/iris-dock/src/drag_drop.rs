// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! 5-Way Drop zone hit-testing, drag-drop state transitions, and highlight geometry.

use crate::tree::DockNodeId;
use iris_core::{Point, Rect};
use serde::{Deserialize, Serialize};

/// Identifies the specific target drop zone computed when dragging a panel over a dock leaf or screen edge.
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
    /// Partitions the entire root tree horizontally, placing the tab in a window-wide left sub-pane.
    ScreenLeft,
    /// Partitions the entire root tree horizontally, placing the tab in a window-wide right sub-pane.
    ScreenRight,
    /// Partitions the entire root tree vertically, placing the tab in a window-wide top sub-pane.
    ScreenTop,
    /// Partitions the entire root tree vertically, placing the tab in a window-wide bottom sub-pane.
    ScreenBottom,
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
    /// Original bounding rectangle of the source pane when dragging initiated.
    pub source_rect: Rect,
    /// Active drop target leaf and zone, if hovering over a valid target.
    pub hover_target: Option<(DockNodeId, DropZone, Rect)>,
    /// Active reorder target within a tab bar `(target_leaf, insert_index)`, if hovering over a tab strip.
    pub tab_reorder_target: Option<(DockNodeId, usize)>,
}

/// Calculates whether the cursor is hovering over the outer boundary of the entire dock window.
/// Threshold margin is typically 32.0 px from the window boundary.
pub fn calculate_screen_drop_zone(
    window_rect: Rect,
    cursor_pos: Point,
    margin: f32,
) -> Option<DropZone> {
    if !window_rect.contains_point(cursor_pos)
        || window_rect.width <= 0.0
        || window_rect.height <= 0.0
    {
        return None;
    }

    let m = margin
        .min(window_rect.width * 0.15)
        .min(window_rect.height * 0.15);

    if cursor_pos.x < window_rect.x + m {
        Some(DropZone::ScreenLeft)
    } else if cursor_pos.x > window_rect.x + window_rect.width - m {
        Some(DropZone::ScreenRight)
    } else if cursor_pos.y < window_rect.y + m {
        Some(DropZone::ScreenTop)
    } else if cursor_pos.y > window_rect.y + window_rect.height - m {
        Some(DropZone::ScreenBottom)
    } else {
        None
    }
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

/// Calculates the 4-way edge drop zone within a target leaf content rectangle.
/// Only the outer 25% strip on each side triggers a directional split.
/// The inner 50% (center region) returns `None`, which causes the dragged tab
/// to detach as a floating window when released there.
/// Zone assignment:
/// - Left 25% strip  → `Left`  (50/50 horizontal split)
/// - Right 25% strip → `Right` (50/50 horizontal split)
/// - Top 25% strip   → `Top`   (50/50 vertical split)
/// - Bottom 25% strip → `Bottom` (50/50 vertical split)
/// - Inner 50% center → `None` (floating window)
pub fn calculate_leaf_half_drop_zone(content_rect: Rect, cursor_pos: Point) -> Option<DropZone> {
    if !content_rect.contains_point(cursor_pos)
        || content_rect.width <= 0.0
        || content_rect.height <= 0.0
    {
        return None;
    }

    // Normalized coordinates in [0.0, 1.0]
    let u = (cursor_pos.x - content_rect.x) / content_rect.width;
    let v = (cursor_pos.y - content_rect.y) / content_rect.height;

    // Only the outermost 25% strip on each axis triggers a split;
    // anything inside the 25%-75% band on both axes is the float zone.
    const EDGE: f32 = 0.25;

    if u < EDGE {
        Some(DropZone::Left)
    } else if u > 1.0 - EDGE {
        Some(DropZone::Right)
    } else if v < EDGE {
        Some(DropZone::Top)
    } else if v > 1.0 - EDGE {
        Some(DropZone::Bottom)
    } else {
        // Center 50% × 50% — caller treats None as "create floating window".
        None
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
        DropZone::ScreenLeft => Rect::new(
            content_rect.x,
            content_rect.y,
            content_rect.width * 0.5,
            content_rect.height,
        ),
        DropZone::ScreenRight => Rect::new(
            content_rect.x + content_rect.width * 0.5,
            content_rect.y,
            content_rect.width * 0.5,
            content_rect.height,
        ),
        DropZone::ScreenTop => Rect::new(
            content_rect.x,
            content_rect.y,
            content_rect.width,
            content_rect.height * 0.5,
        ),
        DropZone::ScreenBottom => Rect::new(
            content_rect.x,
            content_rect.y + content_rect.height * 0.5,
            content_rect.width,
            content_rect.height * 0.5,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_screen_drop_zone() {
        let win = Rect::new(0.0, 0.0, 1000.0, 600.0);

        // Near left edge (<32px)
        assert_eq!(
            calculate_screen_drop_zone(win, Point::new(10.0, 300.0), 32.0),
            Some(DropZone::ScreenLeft)
        );

        // Near right edge (>968px)
        assert_eq!(
            calculate_screen_drop_zone(win, Point::new(980.0, 300.0), 32.0),
            Some(DropZone::ScreenRight)
        );

        // Near top edge (<32px)
        assert_eq!(
            calculate_screen_drop_zone(win, Point::new(500.0, 15.0), 32.0),
            Some(DropZone::ScreenTop)
        );

        // Near bottom edge (>568px)
        assert_eq!(
            calculate_screen_drop_zone(win, Point::new(500.0, 580.0), 32.0),
            Some(DropZone::ScreenBottom)
        );

        // Center of screen
        assert_eq!(
            calculate_screen_drop_zone(win, Point::new(500.0, 300.0), 32.0),
            None
        );
    }

    #[test]
    fn test_calculate_leaf_half_drop_zone() {
        let leaf_rect = Rect::new(100.0, 100.0, 400.0, 300.0);

        // Near left quadrant
        assert_eq!(
            calculate_leaf_half_drop_zone(leaf_rect, Point::new(120.0, 250.0)),
            Some(DropZone::Left)
        );

        // Near right quadrant
        assert_eq!(
            calculate_leaf_half_drop_zone(leaf_rect, Point::new(480.0, 250.0)),
            Some(DropZone::Right)
        );

        // Near top quadrant
        assert_eq!(
            calculate_leaf_half_drop_zone(leaf_rect, Point::new(300.0, 120.0)),
            Some(DropZone::Top)
        );

        // Near bottom quadrant
        assert_eq!(
            calculate_leaf_half_drop_zone(leaf_rect, Point::new(300.0, 380.0)),
            Some(DropZone::Bottom)
        );

        // Outside leaf
        assert_eq!(
            calculate_leaf_half_drop_zone(leaf_rect, Point::new(50.0, 250.0)),
            None
        );

        // Center region (50%×50% inner band) — must return None so the tab floats
        assert_eq!(
            calculate_leaf_half_drop_zone(leaf_rect, Point::new(300.0, 250.0)),
            None
        );
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Edge resize hit-testing, resize direction enumerations, and cursor icon evaluation.
//!
//! Provides 8-directional edge detection along floating window borders,
//! semantic cursor icon hints, and multi-window cursor resolvers.

use iris_core::geometry::{Point, Rect};

/// Default resize border grab margin in logical pixels.
pub const DEFAULT_RESIZE_MARGIN: f32 = 6.0;

/// Edge or corner of a floating window used for 8-directional resizing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingResizeEdge {
    /// Left border edge.
    Left,
    /// Right border edge.
    Right,
    /// Top border edge.
    Top,
    /// Bottom border edge.
    Bottom,
    /// Top-left corner handle.
    TopLeft,
    /// Top-right corner handle.
    TopRight,
    /// Bottom-left corner handle.
    BottomLeft,
    /// Bottom-right corner handle.
    BottomRight,
}

/// Semantic cursor icon hint resolved from floating window interactions or resize borders.
///
/// Decouples platform-specific windowing cursor types from UI docking calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingWindowCursor {
    /// Resize along north-west to south-east diagonal.
    NwseResize,
    /// Resize along north-east to south-west diagonal.
    NeswResize,
    /// Resize horizontally (east-west / column).
    ColResize,
    /// Resize vertically (north-south / row).
    RowResize,
}

impl FloatingResizeEdge {
    /// Resolves the semantic cursor icon hint corresponding to this resize edge or corner.
    #[inline]
    pub fn cursor(&self) -> FloatingWindowCursor {
        match self {
            Self::TopLeft | Self::BottomRight => FloatingWindowCursor::NwseResize,
            Self::TopRight | Self::BottomLeft => FloatingWindowCursor::NeswResize,
            Self::Left | Self::Right => FloatingWindowCursor::ColResize,
            Self::Top | Self::Bottom => FloatingWindowCursor::RowResize,
        }
    }
}

/// Detects whether the specified cursor point falls within the edge or corner resize margin of a rectangle.
///
/// Returns the corresponding [`FloatingResizeEdge`] if the cursor is within `margin` logical pixels
/// of the border, or `None` if the cursor is inside the window body or entirely outside.
#[inline]
pub fn detect_resize_edge(rect: Rect, point: Point, margin: f32) -> Option<FloatingResizeEdge> {
    if !rect.contains_point(point) {
        return None;
    }
    let on_left = point.x <= rect.x + margin;
    let on_right = point.x >= rect.right() - margin;
    let on_top = point.y <= rect.y + margin;
    let on_bottom = point.y >= rect.bottom() - margin;

    if on_top && on_left {
        Some(FloatingResizeEdge::TopLeft)
    } else if on_top && on_right {
        Some(FloatingResizeEdge::TopRight)
    } else if on_bottom && on_left {
        Some(FloatingResizeEdge::BottomLeft)
    } else if on_bottom && on_right {
        Some(FloatingResizeEdge::BottomRight)
    } else if on_left {
        Some(FloatingResizeEdge::Left)
    } else if on_right {
        Some(FloatingResizeEdge::Right)
    } else if on_top {
        Some(FloatingResizeEdge::Top)
    } else if on_bottom {
        Some(FloatingResizeEdge::Bottom)
    } else {
        None
    }
}

/// Evaluates mouse cursor coordinates against an array of floating window bounding rectangles
/// to resolve whether an edge or corner resize cursor icon should be presented.
///
/// Windows are checked in reverse order (topmost / last rendered first).
#[inline]
pub fn evaluate_floating_resize_cursor(
    rects: &[Rect],
    cursor_pos: Point,
    margin: f32,
) -> Option<FloatingWindowCursor> {
    for rect in rects.iter().rev() {
        if let Some(edge) = detect_resize_edge(*rect, cursor_pos, margin) {
            return Some(edge.cursor());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_resize_edges_and_cursors() {
        let rect = Rect::new(100.0, 100.0, 200.0, 200.0);
        const MARGIN: f32 = 6.0;

        // Corners
        assert_eq!(
            detect_resize_edge(rect, Point::new(102.0, 102.0), MARGIN),
            Some(FloatingResizeEdge::TopLeft)
        );
        assert_eq!(
            FloatingResizeEdge::TopLeft.cursor(),
            FloatingWindowCursor::NwseResize
        );

        assert_eq!(
            detect_resize_edge(rect, Point::new(298.0, 102.0), MARGIN),
            Some(FloatingResizeEdge::TopRight)
        );
        assert_eq!(
            FloatingResizeEdge::TopRight.cursor(),
            FloatingWindowCursor::NeswResize
        );

        assert_eq!(
            detect_resize_edge(rect, Point::new(102.0, 298.0), MARGIN),
            Some(FloatingResizeEdge::BottomLeft)
        );
        assert_eq!(
            FloatingResizeEdge::BottomLeft.cursor(),
            FloatingWindowCursor::NeswResize
        );

        assert_eq!(
            detect_resize_edge(rect, Point::new(298.0, 298.0), MARGIN),
            Some(FloatingResizeEdge::BottomRight)
        );
        assert_eq!(
            FloatingResizeEdge::BottomRight.cursor(),
            FloatingWindowCursor::NwseResize
        );

        // Edges
        assert_eq!(
            detect_resize_edge(rect, Point::new(102.0, 150.0), MARGIN),
            Some(FloatingResizeEdge::Left)
        );
        assert_eq!(
            FloatingResizeEdge::Left.cursor(),
            FloatingWindowCursor::ColResize
        );

        assert_eq!(
            detect_resize_edge(rect, Point::new(298.0, 150.0), MARGIN),
            Some(FloatingResizeEdge::Right)
        );
        assert_eq!(
            FloatingResizeEdge::Right.cursor(),
            FloatingWindowCursor::ColResize
        );

        assert_eq!(
            detect_resize_edge(rect, Point::new(200.0, 102.0), MARGIN),
            Some(FloatingResizeEdge::Top)
        );
        assert_eq!(
            FloatingResizeEdge::Top.cursor(),
            FloatingWindowCursor::RowResize
        );

        assert_eq!(
            detect_resize_edge(rect, Point::new(200.0, 298.0), MARGIN),
            Some(FloatingResizeEdge::Bottom)
        );
        assert_eq!(
            FloatingResizeEdge::Bottom.cursor(),
            FloatingWindowCursor::RowResize
        );

        // Inside body
        assert_eq!(
            detect_resize_edge(rect, Point::new(200.0, 200.0), MARGIN),
            None
        );

        // Outside
        assert_eq!(
            detect_resize_edge(rect, Point::new(50.0, 50.0), MARGIN),
            None
        );
    }

    #[test]
    fn test_evaluate_floating_resize_cursor() {
        let rects = vec![
            Rect::new(100.0, 100.0, 200.0, 200.0),
            Rect::new(250.0, 100.0, 200.0, 200.0),
        ];

        // Cursor at x=102, y=102 -> TopLeft of rect 0
        let cursor = evaluate_floating_resize_cursor(&rects, Point::new(102.0, 102.0), 6.0);
        assert_eq!(cursor, Some(FloatingWindowCursor::NwseResize));

        // Cursor at x=448, y=200 -> Right of rect 1
        let cursor = evaluate_floating_resize_cursor(&rects, Point::new(448.0, 200.0), 6.0);
        assert_eq!(cursor, Some(FloatingWindowCursor::ColResize));

        // Cursor inside center of rect 0
        let cursor = evaluate_floating_resize_cursor(&rects, Point::new(150.0, 150.0), 6.0);
        assert_eq!(cursor, None);
    }
}
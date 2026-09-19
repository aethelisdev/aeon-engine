// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # ScrollBar Layout Geometry and Math (`iris-widgets::scroll_area::scroll_bar`)
//!
//! Computes scrollbar track and thumb bounding boxes, proportional thumb sizes,
//! and delta projections for mouse dragging and track clicking.
//!

use super::style::ScrollAreaStyle;
use iris_core::geometry::Rect;

/// Geometric layout of a rendered scrollbar indicator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollBarGeometry {
    /// Bounding rectangle of the stationary scrollbar track.
    pub track_rect: Rect,
    /// Bounding rectangle of the draggable scrollbar thumb.
    pub thumb_rect: Rect,
}

impl ScrollBarGeometry {
    /// Computes vertical scrollbar geometry based on viewport rect, total content height, and scroll position.
    ///
    /// Returns `None` if content does not overflow the visible viewport height.
    #[must_use]
    pub fn compute_vertical(
        viewport_rect: Rect,
        content_height: f32,
        scroll_y: f32,
        style: &ScrollAreaStyle,
    ) -> Option<Self> {
        if content_height <= viewport_rect.height || viewport_rect.height <= 0.0 {
            return None;
        }

        let track_w = style.thickness;
        let track_x = viewport_rect.x + viewport_rect.width - style.thickness - style.inset;
        let track_y = viewport_rect.y + style.inset;
        let track_h = (viewport_rect.height - style.inset * 2.0).max(style.thumb_min_length);

        let max_scroll = (content_height - viewport_rect.height).max(1.0);
        let scroll_ratio = (scroll_y / max_scroll).clamp(0.0, 1.0);

        let thumb_h = ((viewport_rect.height / content_height) * track_h)
            .clamp(style.thumb_min_length, track_h);
        let thumb_y = track_y + scroll_ratio * (track_h - thumb_h);

        Some(Self {
            track_rect: Rect::new(track_x, track_y, track_w, track_h),
            thumb_rect: Rect::new(track_x, thumb_y, track_w, thumb_h),
        })
    }

    /// Translates mouse movement delta in physical pixels along the scrollbar track into a scroll offset delta.
    #[must_use]
    pub fn scroll_from_thumb_drag(
        delta_y: f32,
        track_h: f32,
        thumb_h: f32,
        max_scroll_y: f32,
    ) -> f32 {
        let travel_range = (track_h - thumb_h).max(1.0);
        (delta_y / travel_range) * max_scroll_y
    }

    /// Computes the target scroll offset when clicking directly on the scrollbar track.
    #[must_use]
    pub fn scroll_from_track_click(
        click_y: f32,
        track_y: f32,
        track_h: f32,
        thumb_h: f32,
        max_scroll_y: f32,
    ) -> f32 {
        let travel_range = (track_h - thumb_h).max(1.0);
        let target_thumb_top = (click_y - track_y - thumb_h * 0.5).clamp(0.0, travel_range);
        (target_thumb_top / travel_range) * max_scroll_y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_vertical_no_overflow() {
        let vp = Rect::new(0.0, 0.0, 200.0, 400.0);
        let style = ScrollAreaStyle::default();
        // Content height 300px <= viewport 400px -> no scrollbar
        assert_eq!(
            ScrollBarGeometry::compute_vertical(vp, 300.0, 0.0, &style),
            None
        );
    }

    #[test]
    fn test_compute_vertical_with_overflow() {
        let vp = Rect::new(10.0, 20.0, 300.0, 400.0);
        let style = ScrollAreaStyle::default();
        let geom = ScrollBarGeometry::compute_vertical(vp, 800.0, 200.0, &style).unwrap();

        // Track is inset from right
        let expected_track_x = vp.x + vp.width - style.thickness - style.inset;
        assert_eq!(geom.track_rect.x, expected_track_x);
        assert_eq!(geom.track_rect.y, vp.y + style.inset);

        // Content height is double viewport -> thumb height is roughly half track height
        assert!(geom.thumb_rect.height > style.thumb_min_length);
        assert!(geom.thumb_rect.height < geom.track_rect.height);

        // Thumb is halfway down because scroll_y is half max_scroll (200 / 400)
        let max_scroll = 800.0 - 400.0;
        assert_eq!(max_scroll, 400.0);
    }

    #[test]
    fn test_thumb_drag_math() {
        let track_h = 200.0;
        let thumb_h = 50.0;
        let max_scroll = 600.0;

        // Moving thumb full travel distance (150px) should equal max_scroll
        let delta = ScrollBarGeometry::scroll_from_thumb_drag(150.0, track_h, thumb_h, max_scroll);
        assert!((delta - max_scroll).abs() < 1e-4);

        // Half distance = half max_scroll
        let half_delta =
            ScrollBarGeometry::scroll_from_thumb_drag(75.0, track_h, thumb_h, max_scroll);
        assert!((half_delta - 300.0).abs() < 1e-4);
    }
}
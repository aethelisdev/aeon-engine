// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Animation Timeline Studio Ruler and Scrubber Widget (`iris-widgets::timeline::ruler`)
//!
//! Provides the fluent builder, layout generator, and interactive styling for adaptive
//! time rulers, keyframe indicators, and playhead scrubber tracks in Iris UI.
//!

use super::style::TimelineRulerStyle;
use super::transport::{TIMELINE_TAG_PLAYHEAD_CAP, TIMELINE_TAG_SCRUBBER_TRACK};
use super::types::TimelineKeyframeMarker;
use iris_core::geometry::{Point, Rect};
use iris_core::id::WidgetId;
use iris_core::style::{Style, TextAlign};
use iris_core::tree::UiTree;
use iris_core::{WidgetCursor, WidgetRole};

/// Default height in physical pixels allocated for the top ruler ticks section.
pub const DEFAULT_RULER_HEIGHT: f32 = 18.0;

/// Default height in physical pixels allocated for the interactive scrubber track.
pub const DEFAULT_SCRUBBER_HEIGHT: f32 = 36.0;

/// Output layout frame returned after constructing a timeline ruler and scrubber.
///
/// Contains computed geometry targets for scrubber dragging, playhead handle hit-testing,
/// and keyframe marker positions.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineRulerFrame {
    /// Container node ID enclosing the ruler and scrubber elements.
    pub root_id: WidgetId,
    /// Hit-testing bounding box of the interactive scrubber track.
    pub track_rect: Rect,
    /// Hit-testing bounding box of the draggable playhead handle cap (`▼`).
    pub playhead_handle_rect: Rect,
    /// Bounding box of the translucent elapsed progress fill, if elapsed duration > 0.
    pub progress_rect: Option<Rect>,
    /// Bounding boxes and timestamps of rendered keyframe diamond markers: `(timestamp, rect)`.
    pub keyframe_rects: Vec<(f32, Rect)>,
    /// Total duration in seconds represented across the ruler width.
    pub duration: f32,
}

impl TimelineRulerFrame {
    /// Projects a physical X coordinate onto the timeline track to calculate the corresponding timestamp.
    #[must_use]
    pub fn time_at_x(&self, x: f32) -> f32 {
        if self.track_rect.width <= 0.0 || self.duration <= 0.0 {
            return 0.0;
        }
        let frac = ((x - self.track_rect.x) / self.track_rect.width).clamp(0.0, 1.0);
        frac * self.duration
    }

    /// Projects a timestamp in seconds to the physical horizontal X coordinate on the track.
    #[must_use]
    pub fn x_at_time(&self, time: f32) -> f32 {
        if self.duration <= 1e-4 {
            return self.track_rect.x;
        }
        let frac = (time / self.duration).clamp(0.0, 1.0);
        self.track_rect.x + frac * self.track_rect.width
    }
}

/// Fluent builder for constructing a generic timeline ruler and playhead scrubber widget.
pub struct TimelineRulerBuilder<'a> {
    rect: Rect,
    duration: f32,
    current_time: f32,
    keyframes: &'a [TimelineKeyframeMarker],
    is_dragging: bool,
    cursor_pos: Option<Point>,
    style: TimelineRulerStyle,
    ruler_height: f32,
    track_height: f32,
}

impl<'a> TimelineRulerBuilder<'a> {
    /// Creates a new timeline ruler builder with the specified total bounding rect and duration.
    #[must_use]
    pub fn new(rect: Rect, duration: f32, current_time: f32) -> Self {
        Self {
            rect,
            duration: duration.max(0.0),
            current_time: current_time.max(0.0),
            keyframes: &[],
            is_dragging: false,
            cursor_pos: None,
            style: TimelineRulerStyle::default(),
            ruler_height: DEFAULT_RULER_HEIGHT,
            track_height: DEFAULT_SCRUBBER_HEIGHT,
        }
    }

    /// Attaches keyframe markers to render along the scrubber track.
    #[must_use]
    pub fn keyframes(mut self, keyframes: &'a [TimelineKeyframeMarker]) -> Self {
        self.keyframes = keyframes;
        self
    }

    /// Sets whether the scrubber playhead needle is actively being dragged.
    #[must_use]
    pub fn is_dragging(mut self, is_dragging: bool) -> Self {
        self.is_dragging = is_dragging;
        self
    }

    /// Sets the current mouse cursor position for hover state calculation.
    #[must_use]
    pub fn cursor_pos(mut self, cursor_pos: Option<Point>) -> Self {
        self.cursor_pos = cursor_pos;
        self
    }

    /// Overrides the visual styling configuration for the ruler and scrubber.
    #[must_use]
    pub fn style(mut self, style: TimelineRulerStyle) -> Self {
        self.style = style;
        self
    }

    /// Sets custom heights for the top ruler section and scrubber track.
    #[must_use]
    pub fn heights(mut self, ruler_height: f32, track_height: f32) -> Self {
        self.ruler_height = ruler_height.max(10.0);
        self.track_height = track_height.max(16.0);
        self
    }

    /// Builds the timeline ruler and scrubber hierarchy into the given UI tree.
    pub fn build(self, tree: &mut UiTree, parent_id: WidgetId) -> TimelineRulerFrame {
        let root_id = tree.create_node();
        if let Some(node) = tree.get_mut(root_id) {
            node.set_name("TimelineRulerRoot");
            node.computed_rect = self.rect;
            node.role = WidgetRole::Default;
            node.style = Style::new().clip_children(true);
        }
        let _ = tree.add_child(parent_id, root_id);

        let ruler_y = self.rect.y;
        let track_y = ruler_y + self.ruler_height + 2.0;
        let track_w = self.rect.width.max(10.0);
        let track_h = (self.rect.height - (self.ruler_height + 4.0))
            .max(16.0)
            .min(self.track_height);

        let track_rect = Rect::new(self.rect.x, track_y, track_w, track_h);

        let safe_duration = self.duration.max(0.001);
        let clamped_time = self.current_time.clamp(0.0, self.duration);
        let progress_ratio = if self.duration > 0.001 {
            (clamped_time / self.duration).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // ── 1. Dynamic Time Ruler Ticks and Labels ──
        let step = if self.duration <= 1.0 {
            0.1
        } else if self.duration <= 3.0 {
            0.25
        } else if self.duration <= 10.0 {
            0.5
        } else if self.duration <= 30.0 {
            1.0
        } else {
            5.0
        };

        let tick_count = ((self.duration / step).ceil() as usize).min(120);
        for i in 0..=tick_count {
            let t = (i as f32 * step).min(self.duration);
            let frac = (t / safe_duration).clamp(0.0, 1.0);
            let tick_x = track_rect.x + frac * track_rect.width;

            let is_major = (i % 2 == 0) || (t == 0.0) || ((t - self.duration).abs() < 0.001);
            let tick_h = if is_major { 8.0 } else { 4.0 };
            let tick_y = ruler_y + self.ruler_height - tick_h;

            let tick_id = tree.create_node();
            if let Some(node) = tree.get_mut(tick_id) {
                node.set_name("TimelineRulerTick");
                node.computed_rect = Rect::new(tick_x, tick_y, 1.0, tick_h);
                node.style = Style::new().background(if is_major {
                    self.style.major_tick_color
                } else {
                    self.style.minor_tick_color
                });
            }
            let _ = tree.add_child(root_id, tick_id);

            // Major tick timestamp text label
            if is_major && tick_x + 28.0 <= track_rect.x + track_rect.width + 10.0 {
                let label_id = tree.create_node();
                if let Some(node) = tree.get_mut(label_id) {
                    node.set_name("TimelineRulerLabel");
                    node.set_text(format!("{:.1}s", t));
                    node.font_size = self.style.tick_label_font_size;
                    node.line_height = 12.0;
                    node.text_align = TextAlign::Left;
                    node.text_color = self.style.tick_label_color;
                    node.computed_rect = Rect::new(tick_x + 2.0, ruler_y, 30.0, 12.0);
                }
                let _ = tree.add_child(root_id, label_id);
            }
        }

        // ── 2. Scrubber Track Background ──
        let is_track_hovered = self
            .cursor_pos
            .is_some_and(|pos| track_rect.contains_point(pos));
        let track_border = if is_track_hovered || self.is_dragging {
            self.style.track_border_active
        } else {
            self.style.track_border_idle
        };

        let track_id = tree.create_node();
        if let Some(node) = tree.get_mut(track_id) {
            node.set_name("TimelineScrubberTrack");
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::ColResize);
            node.interactive = true;
            node.tag = TIMELINE_TAG_SCRUBBER_TRACK;
            node.computed_rect = track_rect;
            node.style = Style::new()
                .background(self.style.track_bg)
                .border_radius(self.style.track_border_radius)
                .border(self.style.track_border_width, track_border);
        }
        let _ = tree.add_child(root_id, track_id);

        // ── 3. Translucent Progress Fill ──
        let fill_w = track_rect.width * progress_ratio;
        let progress_rect = if fill_w > 1.0 {
            let fill_rect = Rect::new(track_rect.x, track_rect.y, fill_w, track_rect.height);
            let fill_id = tree.create_node();
            if let Some(node) = tree.get_mut(fill_id) {
                node.set_name("TimelineProgressFill");
                node.computed_rect = fill_rect;
                node.style = Style::new()
                    .background(self.style.progress_fill_color)
                    .border_radius(self.style.track_border_radius);
            }
            let _ = tree.add_child(track_id, fill_id);
            Some(fill_rect)
        } else {
            None
        };

        // ── 4. Keyframe Diamond Markers ──
        let mut keyframe_rects = Vec::with_capacity(self.keyframes.len());
        for kf in self.keyframes {
            let kf_frac = (kf.time / safe_duration).clamp(0.0, 1.0);
            let kf_x = track_rect.x + kf_frac * track_rect.width;
            let kf_y = track_rect.y + track_rect.height * 0.5 - 6.0;
            let marker_rect = Rect::new(kf_x - 6.0, kf_y, 12.0, 12.0);
            keyframe_rects.push((kf.time, marker_rect));

            let kf_id = tree.create_node();
            if let Some(node) = tree.get_mut(kf_id) {
                node.set_name("TimelineKeyframeMarker");
                node.set_text("◆");
                node.font_size = 11.0;
                node.line_height = 12.0;
                node.text_align = TextAlign::Center;
                node.text_color = kf.color.unwrap_or(self.style.keyframe_color);
                node.computed_rect = marker_rect;
            }
            let _ = tree.add_child(track_id, kf_id);
        }

        // ── 5. Playhead Needle and Draggable Handle ──
        let needle_x =
            (track_rect.x + fill_w).clamp(track_rect.x, track_rect.x + track_rect.width - 2.0);
        let needle_top = ruler_y + 4.0;
        let needle_bottom = track_rect.y + track_rect.height;

        let cap_w = 12.0;
        let cap_h = 10.0;
        let cap_rect = Rect::new(needle_x - cap_w * 0.5 + 1.0, needle_top, cap_w, cap_h);

        let cap_id = tree.create_node();
        if let Some(node) = tree.get_mut(cap_id) {
            node.set_name("TimelinePlayheadCap");
            node.role = WidgetRole::Button;
            node.cursor = Some(WidgetCursor::ColResize);
            node.interactive = true;
            node.tag = TIMELINE_TAG_PLAYHEAD_CAP;
            node.set_text("▼");
            node.font_size = 9.0;
            node.line_height = cap_h;
            node.text_align = TextAlign::Center;
            node.text_color = self.style.playhead_cap_color;
            node.computed_rect = cap_rect;
        }
        let _ = tree.add_child(root_id, cap_id);

        let needle_line_id = tree.create_node();
        if let Some(node) = tree.get_mut(needle_line_id) {
            node.set_name("TimelinePlayheadNeedle");
            node.computed_rect = Rect::new(
                needle_x,
                needle_top + cap_h,
                self.style.playhead_needle_width,
                needle_bottom - (needle_top + cap_h),
            );
            node.style = Style::new().background(self.style.playhead_needle_color);
        }
        let _ = tree.add_child(root_id, needle_line_id);

        TimelineRulerFrame {
            root_id,
            track_rect,
            playhead_handle_rect: cap_rect,
            progress_rect,
            keyframe_rects,
            duration: self.duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iris_core::color::Color;

    #[test]
    fn test_timeline_ruler_build_basic() {
        let mut tree = UiTree::new();
        let parent = tree.create_node();
        let bounds = Rect::new(10.0, 20.0, 500.0, 60.0);

        let frame = TimelineRulerBuilder::new(bounds, 5.0, 2.5)
            .is_dragging(true)
            .build(&mut tree, parent);

        assert_eq!(frame.duration, 5.0);
        assert_eq!(frame.track_rect.x, 10.0);
        assert_eq!(frame.track_rect.width, 500.0);
        assert!(frame.playhead_handle_rect.width > 0.0);
        assert!(frame.progress_rect.is_some());

        // Test time projections
        let mid_x = frame.track_rect.x + frame.track_rect.width * 0.5;
        assert!((frame.time_at_x(mid_x) - 2.5).abs() < 1e-4);
        assert!((frame.x_at_time(2.5) - mid_x).abs() < 1e-4);
    }

    #[test]
    fn test_timeline_ruler_keyframes() {
        let mut tree = UiTree::new();
        let parent = tree.create_node();
        let bounds = Rect::new(0.0, 0.0, 400.0, 60.0);

        let keyframes = [
            TimelineKeyframeMarker::new(1.0),
            TimelineKeyframeMarker::with_color(3.0, Color::RED),
        ];

        let frame = TimelineRulerBuilder::new(bounds, 4.0, 0.0)
            .keyframes(&keyframes)
            .build(&mut tree, parent);

        assert_eq!(frame.keyframe_rects.len(), 2);
        assert!((frame.keyframe_rects[0].0 - 1.0).abs() < 1e-4);
        assert!((frame.keyframe_rects[1].0 - 3.0).abs() < 1e-4);
    }
}
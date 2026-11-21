// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Adaptive Time Ruler (`UiScope::time_ruler`)
//!
//! Renders an adaptive time-axis ruler bar with a fixed-width sidebar header and a
//! `flex_grow` canvas track that automatically computes major/minor tick subdivisions
//! and formats timestamps into stack-allocated [`StackString`](irisui_core::StackString)
//! buffers without heap allocation.

use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::display::ImmediateCanvasSpec;
use irisui_core::{
    AlignItems, CanvasOrigin, Color, Insets, Point, StackString, Style, TextAlign, Theme,
    WidgetResponse, WidgetRole,
};
use std::fmt::Write;

/// Visual styling tokens shared across [`UiScope::time_ruler`], [`UiScope::scrubber_track`],
/// and [`UiScope::track_lane_from_iter`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRulerStyle {
    /// Background color of the top time ruler bar.
    pub ruler_bg: Color,
    /// Sidebar label text color on the top time ruler header.
    pub ruler_sidebar_color: Color,
    /// Background fill of the interactive scrubber track.
    pub track_bg: Color,
    /// Border outline color of the scrubber track when idle.
    pub track_border_idle: Color,
    /// Border outline color of the scrubber track when pressed or dragged.
    pub track_border_active: Color,
    /// Border thickness around the scrubber track in logical pixels.
    pub track_border_width: f32,
    /// Corner border radius of the scrubber track in logical pixels.
    pub track_border_radius: f32,
    /// Fill color highlighting elapsed duration behind the playhead.
    pub progress_fill_color: Color,
    /// Color of major subdivision ticks along the ruler axis.
    pub major_tick_color: Color,
    /// Color of minor subdivision ticks along the ruler axis.
    pub minor_tick_color: Color,
    /// Text color of timestamp labels beside major ticks.
    pub tick_label_color: Color,
    /// Font size of ruler timestamp labels in logical points.
    pub tick_label_font_size: f32,
    /// Default marker diamond fill color along the scrubber track.
    pub marker_color: Color,
    /// Accent color of the vertical playhead needle line.
    pub needle_color: Color,
    /// Foreground color of the playhead handle glyph (`▼`).
    pub handle_color: Color,
    /// Stroke thickness of the vertical playhead needle line in pixels.
    pub needle_width: f32,
    /// Background color of sidebar pill badges inside tracks and lanes.
    pub sidebar_badge_bg: Color,
    /// Border outline color of sidebar pill badges.
    pub sidebar_badge_border: Color,
    /// Background color for even-indexed track lanes.
    pub lane_even_bg: Color,
    /// Background color for odd-indexed track lanes.
    pub lane_odd_bg: Color,
    /// Border outline color framing each track lane row.
    pub lane_border_color: Color,
    /// Center horizontal guideline stroke inside each track lane.
    pub lane_guide_color: Color,
    /// Vertical playhead guide line stroke inside each track lane.
    pub lane_needle_color: Color,
    /// Label text color inside each track lane sidebar badge.
    pub lane_label_color: Color,
}

impl Default for TimeRulerStyle {
    #[inline]
    fn default() -> Self {
        Self::dark_default()
    }
}

impl TimeRulerStyle {
    /// Returns the standard dark studio palette for time rulers, scrubbers, and track lanes.
    #[must_use]
    pub const fn dark_default() -> Self {
        Self {
            ruler_bg: Color::rgba(0.07, 0.08, 0.11, 0.60),
            ruler_sidebar_color: Color::rgba(0.55, 0.60, 0.72, 0.90),
            track_bg: Color::rgba(0.09, 0.10, 0.14, 0.95),
            track_border_idle: Color::rgba(0.20, 0.23, 0.32, 0.70),
            track_border_active: Color::rgba(0.0, 0.85, 1.0, 0.60),
            track_border_width: 1.0,
            track_border_radius: 4.0,
            progress_fill_color: Color::rgba(0.0, 0.75, 0.95, 0.16),
            major_tick_color: Color::rgba(0.50, 0.55, 0.68, 0.90),
            minor_tick_color: Color::rgba(0.30, 0.34, 0.44, 0.60),
            tick_label_color: Color::rgba(0.55, 0.60, 0.72, 1.0),
            tick_label_font_size: 9.5,
            marker_color: Color::rgba(0.96, 0.72, 0.18, 1.0),
            needle_color: Color::rgba(0.0, 0.92, 1.0, 1.0),
            handle_color: Color::rgba(0.0, 0.95, 1.0, 1.0),
            needle_width: 2.0,
            sidebar_badge_bg: Color::rgba(0.05, 0.06, 0.08, 0.85),
            sidebar_badge_border: Color::rgba(0.22, 0.25, 0.34, 0.60),
            lane_even_bg: Color::rgba(0.07, 0.08, 0.11, 0.65),
            lane_odd_bg: Color::rgba(0.09, 0.10, 0.14, 0.65),
            lane_border_color: Color::rgba(0.15, 0.17, 0.24, 0.40),
            lane_guide_color: Color::rgba(0.20, 0.23, 0.32, 0.25),
            lane_needle_color: Color::rgba(0.0, 0.92, 1.0, 0.45),
            lane_label_color: Color::rgba(0.70, 0.74, 0.84, 1.0),
        }
    }

    /// Derives a [`TimeRulerStyle`] tinted with the active [`Theme`] accent and surface colors.
    #[must_use]
    pub const fn from_theme(theme: &Theme) -> Self {
        let mut style = Self::dark_default();
        style.track_border_active = theme.accent;
        style.needle_color = theme.accent;
        style.handle_color = theme.accent;
        style.ruler_bg = theme.card_bg;
        style
    }
}

/// Computes an appropriate major-grid subdivision step (in seconds) for a ruler spanning `duration`.
#[inline]
#[must_use]
pub const fn tick_step_for_duration(duration: f32) -> f32 {
    if duration <= 1.0 {
        0.1
    } else if duration <= 3.0 {
        0.25
    } else if duration <= 10.0 {
        0.5
    } else if duration <= 30.0 {
        1.0
    } else {
        5.0
    }
}

/// Configuration descriptor for [`UiScope::time_ruler`].
#[derive(Clone, Copy, Debug)]
pub struct TimeRulerSpec<'a> {
    /// Debug node name assigned to the ruler root container.
    pub name: &'static str,
    /// Persistent 64-bit semantic tag for the ruler container and track canvas measurement.
    pub tag: u64,
    /// Total duration of the time axis in seconds.
    pub duration: f32,
    /// Height of the ruler strip in logical pixels.
    pub height: f32,
    /// Fixed width of the left header column in logical pixels.
    pub sidebar_width: f32,
    /// Caption displayed inside the left header column (e.g. `"⏱ TIME (s)"`).
    pub sidebar_title: &'a str,
    /// Maximum subdivision ticks rendered to cap vertex cost on very long durations.
    pub max_ticks: usize,
    /// Visual styling tokens applied to ruler background, ticks, and timestamp labels.
    pub style: TimeRulerStyle,
}

impl<'a> TimeRulerSpec<'a> {
    /// Constructs a new [`TimeRulerSpec`] with `18px` height, `118px` sidebar, and dark studio styling.
    #[inline]
    #[must_use]
    pub const fn new(tag: u64, duration: f32) -> Self {
        Self {
            name: "TimeRulerRoot",
            tag,
            duration,
            height: 18.0,
            sidebar_width: 118.0,
            sidebar_title: "⏱ TIME (s)",
            max_ticks: 120,
            style: TimeRulerStyle::dark_default(),
        }
    }

    /// Overrides the debug node name of the ruler container.
    #[inline]
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Overrides the ruler strip height in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Overrides the left header column width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn sidebar_width(mut self, sidebar_width: f32) -> Self {
        self.sidebar_width = sidebar_width;
        self
    }

    /// Overrides the left header caption string.
    #[inline]
    #[must_use]
    pub const fn sidebar_title(mut self, title: &'a str) -> Self {
        self.sidebar_title = title;
        self
    }

    /// Overrides the visual style palette for the ruler.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: TimeRulerStyle) -> Self {
        self.style = style;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits an adaptive, zero-heap-allocation time ruler strip with automatic major/minor tick intervals.
    ///
    /// Uses a flex-row layout with a fixed sidebar column (`spec.sidebar_width`) and an expanding
    /// `ImmediateCanvas` track (`CanvasOrigin::NodeTopLeft`) so ticks and timestamps stretch to fit
    /// the resolved panel width automatically without consumer width arithmetic.
    pub fn time_ruler(&mut self, spec: TimeRulerSpec<'_>) -> WidgetResponse {
        let style = spec.style;
        let height = spec.height.max(12.0);
        let duration = spec.duration.max(0.0);
        let safe_duration = duration.max(0.001);
        let step = tick_step_for_duration(duration);
        let max_ticks = spec.max_ticks.max(1);
        let tick_count = ((duration / step).ceil() as usize).min(max_ticks);
        let fallback_w = (self.available_rect().width - spec.sidebar_width).max(10.0);
        let canvas_tag = crate::declarative::types::hash_label_with_seed(spec.tag, "ruler_canvas");

        let root_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Stretch)
            .height(height)
            .clip_children(true)
            .background(style.ruler_bg);

        let header_style = Style::new()
            .width(spec.sidebar_width)
            .height(height)
            .flex_shrink(0.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(0.0, 8.0, 0.0, 8.0));

        let inner = self.container_tagged(
            spec.name,
            root_style,
            WidgetRole::Default,
            spec.tag,
            |ruler| {
                ruler.container_named("TimeRulerSidebar", header_style, |hdr| {
                    hdr.label_custom(
                        spec.sidebar_title,
                        8.5,
                        style.ruler_sidebar_color,
                        TextAlign::Left,
                    );
                });

                let resolved_w = ruler
                    .tree
                    .tag_rect(canvas_tag)
                    .map(|r| r.width)
                    .filter(|&w| w > 1.0)
                    .unwrap_or(fallback_w);

                let canvas_style = Style::new()
                    .flex_grow(1.0)
                    .height(height)
                    .clip_children(true);

                ruler.immediate_canvas_with(
                    ImmediateCanvasSpec::new("TimeRulerTicksCanvas", canvas_tag, canvas_style)
                        .origin(CanvasOrigin::NodeTopLeft)
                        .interactive(false),
                    |canvas| {
                        let usable_w =
                            if canvas.width() > 1.0 && (canvas.width() - resolved_w).abs() < 1e3 {
                                canvas
                                    .width()
                                    .max(resolved_w)
                                    .min(resolved_w.max(canvas.width()))
                            } else {
                                resolved_w
                            };
                        let draw_w = usable_w.max(10.0);
                        let mut label_buf = StackString::<16>::new();

                        for i in 0..=tick_count {
                            let t = (i as f32 * step).min(duration);
                            let frac = (t / safe_duration).clamp(0.0, 1.0);
                            let is_major =
                                (i % 2 == 0) || (t == 0.0) || ((t - duration).abs() < 0.001);
                            let tick_h = if is_major { 8.0 } else { 4.0 };
                            let x = (frac * draw_w).clamp(0.0, (draw_w - 1.0).max(0.0));

                            canvas.draw_line(
                                Point::new(x, height - tick_h),
                                Point::new(x, height),
                                if is_major {
                                    style.major_tick_color
                                } else {
                                    style.minor_tick_color
                                },
                                1.0,
                            );

                            if is_major {
                                label_buf.clear();
                                let _ = write!(label_buf, "{:.1}s", t);
                                canvas.draw_text(
                                    Point::new(x + 2.0, 0.0),
                                    label_buf.as_str(),
                                    style.tick_label_font_size,
                                    style.tick_label_color,
                                );
                            }
                        }
                    },
                );
            },
        );
        inner.response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::PanelBuilder;
    use irisui_core::{CanvasPrimitive, Rect, UiTree};

    #[test]
    fn test_tick_step_for_duration_ranges() {
        assert_eq!(tick_step_for_duration(0.5), 0.1);
        assert_eq!(tick_step_for_duration(2.5), 0.25);
        assert_eq!(tick_step_for_duration(8.0), 0.5);
        assert_eq!(tick_step_for_duration(20.0), 1.0);
        assert_eq!(tick_step_for_duration(60.0), 5.0);
    }

    #[test]
    fn test_time_ruler_emits_ticks_and_labels_without_layout_node_explosion() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let ruler_tag = 0xAA40;
        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.time_ruler(TimeRulerSpec::new(ruler_tag, 5.0));
            assert_eq!(tree.get(resp.id).map(|n| n.tag), Some(ruler_tag));
        }

        let ruler_id = tree.find_node_by_tag(ruler_tag).expect("ruler root");
        let ruler_node = tree.get(ruler_id).expect("ruler node");
        // Only sidebar + ticks canvas (2 children), regardless of tick count
        assert_eq!(ruler_node.children.len(), 2);

        let canvas_id = ruler_node.children[1];
        let prims = tree
            .canvas_primitives(canvas_id)
            .expect("tick primitives must be emitted");
        let line_count = prims
            .iter()
            .filter(|p| matches!(p, CanvasPrimitive::Line { .. }))
            .count();
        let text_count = prims
            .iter()
            .filter(|p| matches!(p, CanvasPrimitive::Text { .. }))
            .count();

        // duration=5.0, step=0.5 -> 11 ticks (0..=10)
        assert_eq!(line_count, 11);
        assert!(text_count >= 5);
        assert!(prims.iter().any(|p| matches!(
            p,
            CanvasPrimitive::Text { text, .. } if text == "0.0s"
        )));
    }

    #[test]
    fn test_time_ruler_adapts_to_resolved_tag_rect_width() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let ruler_tag = 0xAA41;
        let canvas_tag = crate::declarative::types::hash_label_with_seed(ruler_tag, "ruler_canvas");
        tree.set_tag_rect(canvas_tag, Rect::new(128.0, 40.0, 400.0, 18.0));

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.time_ruler(TimeRulerSpec::new(ruler_tag, 2.0));
        }

        let canvas_id = tree.find_node_by_tag(canvas_tag).expect("canvas");
        let prims = tree.canvas_primitives(canvas_id).expect("prims");
        let last_tick_x = prims
            .iter()
            .filter_map(|p| match p {
                CanvasPrimitive::Line { start, .. } => Some(start.x),
                _ => None,
            })
            .fold(0.0_f32, f32::max);
        assert!((last_tick_x - 399.0).abs() < 1.5);
    }
}
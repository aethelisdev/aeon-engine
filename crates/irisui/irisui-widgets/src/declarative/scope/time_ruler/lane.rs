// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Multi-Channel Track Lane Row (`UiScope::track_lane_from_iter`)
//!
//! Renders individual channel lane rows with zebra striping, a sidebar property badge,
//! a right-hand `ImmediateCanvas` baseline, playhead needle guide, and zero-allocation
//! keyframe diamond markers sourced directly from an `IntoIterator<Item = f32>`.

use super::ruler::TimeRulerStyle;
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::display::ImmediateCanvasSpec;
use irisui_core::{
    AlignItems, CanvasOrigin, Color, Insets, Point, StackString, Style, WidgetResponse,
};

/// Configuration descriptor for [`UiScope::track_lane_group`], grouping static track lanes
/// beneath a shared non-interactive live vertical playhead overlay.
#[derive(Clone, Copy, Debug)]
pub struct TrackLaneGroupSpec {
    /// Debug node name assigned to the track group container.
    pub name: &'static str,
    /// Persistent 64-bit semantic tag bound to the volatile vertical playhead overlay canvas.
    pub tag: u64,
    /// Total sequence duration in seconds.
    pub duration: f32,
    /// Active playhead timestamp in seconds.
    pub current_time: f32,
    /// Left sidebar column width skipped by the right-hand playhead overlay canvas.
    pub sidebar_width: f32,
    /// Vertical spacing gap between stacked channel lane rows in logical pixels.
    pub gap: f32,
    /// Visual style palette supplying `lane_needle_color`.
    pub style: TimeRulerStyle,
}

impl TrackLaneGroupSpec {
    /// Constructs a new [`TrackLaneGroupSpec`] with default `2.0px` row gap and `118px` sidebar clearance.
    #[inline]
    #[must_use]
    pub const fn new(tag: u64, duration: f32, current_time: f32) -> Self {
        Self {
            name: "TrackLaneGroup",
            tag,
            duration,
            current_time,
            sidebar_width: 118.0,
            gap: 2.0,
            style: TimeRulerStyle::dark_default(),
        }
    }

    /// Overrides the debug node name assigned to the track group container.
    #[inline]
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Overrides the left sidebar width skipped by the playhead overlay canvas.
    #[inline]
    #[must_use]
    pub const fn sidebar_width(mut self, sidebar_width: f32) -> Self {
        self.sidebar_width = sidebar_width;
        self
    }

    /// Overrides the vertical gap between lane rows in logical pixels.
    #[inline]
    #[must_use]
    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Overrides the visual style palette for the group playhead overlay.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: TimeRulerStyle) -> Self {
        self.style = style;
        self
    }
}

/// Configuration descriptor for emitting a channel lane via [`UiScope::track_lane_from_iter`].
#[derive(Clone, Copy, Debug)]
pub struct TrackLaneSpec<'a> {
    /// Debug node name assigned to the lane row container.
    pub name: &'static str,
    /// Short property type code shown inside the sidebar pill (e.g. `"POS"`, `"ROT"`, `"SCL"`).
    pub badge_code: &'a str,
    /// Accent color applied to `badge_code` and lane marker diamonds.
    pub badge_color: Color,
    /// Human-readable lane label shown beside `badge_code` (e.g. `"Joint #0"`).
    pub label: &'a str,
    /// Total sequence duration in seconds.
    pub duration: f32,
    /// Active playhead timestamp in seconds.
    pub current_time: f32,
    /// Zero-based row index used for alternating zebra striping.
    pub row_index: usize,
    /// Row height in logical pixels.
    pub height: f32,
    /// Fixed sidebar column width in logical pixels.
    pub sidebar_width: f32,
    /// Whether this lane draws its own inline vertical playhead needle (disable when using a shared live overlay).
    pub show_playhead: bool,
    /// Visual style palette shared with the enclosing time ruler and scrubber.
    pub style: TimeRulerStyle,
}

impl<'a> TrackLaneSpec<'a> {
    /// Constructs a new [`TrackLaneSpec`] with default `24px` height and `118px` sidebar width.
    #[inline]
    #[must_use]
    pub const fn new(
        badge_code: &'a str,
        badge_color: Color,
        label: &'a str,
        duration: f32,
        current_time: f32,
    ) -> Self {
        Self {
            name: "TrackLaneRow",
            badge_code,
            badge_color,
            label,
            duration,
            current_time,
            row_index: 0,
            height: 24.0,
            sidebar_width: 118.0,
            show_playhead: true,
            style: TimeRulerStyle::dark_default(),
        }
    }

    /// Sets the zero-based row index used for alternating zebra backgrounds.
    #[inline]
    #[must_use]
    pub const fn row_index(mut self, index: usize) -> Self {
        self.row_index = index;
        self
    }

    /// Overrides the lane row height in logical pixels.
    #[inline]
    #[must_use]
    pub const fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Overrides the left sidebar column width in logical pixels.
    #[inline]
    #[must_use]
    pub const fn sidebar_width(mut self, sidebar_width: f32) -> Self {
        self.sidebar_width = sidebar_width;
        self
    }

    /// Controls whether the per-lane vertical playhead guide line is drawn inside this row.
    #[inline]
    #[must_use]
    pub const fn show_playhead(mut self, show: bool) -> Self {
        self.show_playhead = show;
        self
    }

    /// Overrides the visual style palette for the track lane.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: TimeRulerStyle) -> Self {
        self.style = style;
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits a channel track lane with stack-formatted sidebar label and zero-allocation
    /// marker diamond iterator rendering.
    pub fn track_lane_fmt<const N: usize, I, F>(
        &mut self,
        spec: TrackLaneSpec<'_>,
        marker_times: I,
        fmt_label: F,
    ) -> WidgetResponse
    where
        I: IntoIterator<Item = f32>,
        F: FnOnce(&mut StackString<N>),
    {
        let mut buf = StackString::<N>::new();
        fmt_label(&mut buf);
        let resolved_spec = TrackLaneSpec {
            label: buf.as_str(),
            ..spec
        };
        self.track_lane_from_iter(resolved_spec, marker_times)
    }

    /// Emits a channel track lane row populated from a borrowed slice of timestamps.
    #[inline]
    pub fn track_lane(&mut self, spec: TrackLaneSpec<'_>, marker_times: &[f32]) -> WidgetResponse {
        self.track_lane_from_iter(spec, marker_times.iter().copied())
    }

    /// Emits a channel track lane row using an iterator of timestamp values, avoiding intermediate
    /// `Vec` allocations when collecting markers from heterogeneous tracks.
    ///
    /// Renders a fixed sidebar pill (`badge_code` + `label`) alongside a `flex_grow` canvas
    /// containing the horizontal baseline guide, vertical playhead guide, and diamond markers.
    pub fn track_lane_from_iter<I>(
        &mut self,
        spec: TrackLaneSpec<'_>,
        marker_times: I,
    ) -> WidgetResponse
    where
        I: IntoIterator<Item = f32>,
    {
        let style = spec.style;
        let height = spec.height.max(14.0);
        let duration = spec.duration.max(0.0);
        let safe_duration = duration.max(0.001);
        let progress_ratio =
            (spec.current_time.clamp(0.0, duration) / safe_duration).clamp(0.0, 1.0);
        let bg_color = if spec.row_index.is_multiple_of(2) {
            style.lane_even_bg
        } else {
            style.lane_odd_bg
        };

        let lane_tag = crate::declarative::types::hash_label_with_seed(
            spec.row_index as u64 + 1,
            "track_lane_canvas",
        );
        let fallback_w = (self.available_rect().width - spec.sidebar_width).max(10.0);
        let resolved_w = self
            .tree
            .tag_rect(lane_tag)
            .map(|r| r.width)
            .filter(|&w| w > 1.0)
            .unwrap_or(fallback_w);

        let lane_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Stretch)
            .height(height)
            .background(bg_color)
            .border(1.0, style.lane_border_color)
            .border_radius(2.0)
            .clip_children(true);

        let sidebar_col_style = Style::new()
            .width(spec.sidebar_width)
            .height(height)
            .flex_shrink(0.0)
            .padding_insets(Insets::new(2.0, 4.0, 2.0, 4.0))
            .align_items(AlignItems::FlexStart);

        let badge_style = Style::new()
            .width((spec.sidebar_width - 8.0).max(20.0))
            .height((height - 6.0).max(12.0))
            .flex_row()
            .align_items(AlignItems::Center)
            .gap(4.0)
            .padding_insets(Insets::new(0.0, 4.0, 0.0, 4.0))
            .background(style.sidebar_badge_bg)
            .border(1.0, style.sidebar_badge_border)
            .border_radius(3.0);

        let inner = self.container_named(spec.name, lane_style, |lane| {
            lane.container_named("TrackLaneSidebar", sidebar_col_style, |col| {
                col.container_named("TrackLaneBadge", badge_style, |badge| {
                    badge.colored_label(spec.badge_color, spec.badge_code);
                    badge.colored_label(style.lane_label_color, spec.label);
                });
            });

            let canvas_style = Style::new().flex_grow(1.0).height(height);
            lane.immediate_canvas_with(
                ImmediateCanvasSpec::new("TrackLaneCanvas", lane_tag, canvas_style)
                    .origin(CanvasOrigin::NodeTopLeft)
                    .interactive(false),
                |canvas| {
                    let draw_w = if canvas.width() > 1.0 && canvas.bounds().x > 0.0 {
                        canvas.width()
                    } else {
                        resolved_w
                    }
                    .max(10.0);

                    let mid_y = height * 0.5;
                    canvas.draw_line(
                        Point::new(0.0, mid_y),
                        Point::new(draw_w, mid_y),
                        style.lane_guide_color,
                        1.0,
                    );

                    if spec.show_playhead {
                        let needle_x =
                            (progress_ratio * draw_w).clamp(1.0, (draw_w - 1.0).max(1.0));
                        canvas.draw_line(
                            Point::new(needle_x, 0.0),
                            Point::new(needle_x, height),
                            style.lane_needle_color,
                            1.0,
                        );
                    }

                    let kf_y = (height - 10.0) * 0.5 - 1.0;
                    for time in marker_times {
                        let frac = (time / safe_duration).clamp(0.0, 1.0);
                        let kf_x = (frac * draw_w - 5.0).clamp(0.0, (draw_w - 10.0).max(0.0));
                        canvas.draw_text(Point::new(kf_x, kf_y), "◆", 10.0, spec.badge_color);
                    }
                },
            );
        });
        inner.response
    }

    /// Emits a vertical group of track lanes covered by one shared non-interactive live
    /// playhead needle overlay (`spec.tag`) spanning past `spec.sidebar_width`.
    ///
    /// Pairs with [`Self::update_track_lane_playhead`] to animate the needle at 60 FPS
    /// without touching static keyframe marker canvases or lane rows.
    pub fn track_lane_group<F>(
        &mut self,
        spec: TrackLaneGroupSpec,
        build_lanes: F,
    ) -> WidgetResponse
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        let fallback_w = (self.available_rect().width - spec.sidebar_width).max(10.0);
        let group_style = Style::new().flex_col().gap(spec.gap);
        let overlay_style = Style::new()
            .position_absolute()
            .left(spec.sidebar_width)
            .top(0.0)
            .right(0.0)
            .bottom(0.0);

        self.container_named(spec.name, group_style, |area| {
            build_lanes(area);
            area.live_canvas_overlay(
                "TrackLaneGroupPlayhead",
                spec.tag,
                CanvasOrigin::NodeTopLeft,
                overlay_style,
                |canvas| {
                    draw_group_playhead_line(
                        canvas,
                        spec.duration,
                        spec.current_time,
                        fallback_w,
                        spec.style,
                    );
                },
            );
        })
        .response
    }

    /// Updates a shared track-lane group playhead overlay (`tag`) in place during `InPlaceRefresh`.
    pub fn update_track_lane_playhead(
        &mut self,
        tag: u64,
        duration: f32,
        current_time: f32,
        style: TimeRulerStyle,
    ) -> bool {
        self.update_live_canvas(tag, |canvas| {
            let fallback_w = canvas.width().max(10.0);
            draw_group_playhead_line(canvas, duration, current_time, fallback_w, style);
        })
    }
}

fn draw_group_playhead_line(
    canvas: &mut crate::canvas::ImmediateCanvas<'_>,
    duration: f32,
    current_time: f32,
    fallback_w: f32,
    style: TimeRulerStyle,
) {
    let draw_w = if canvas.width() > 1.0 && canvas.bounds().x > 0.0 {
        canvas.width()
    } else {
        fallback_w
    }
    .max(10.0);
    let safe_duration = duration.max(0.001);
    let progress_ratio =
        (current_time.clamp(0.0, duration.max(0.0)) / safe_duration).clamp(0.0, 1.0);
    let needle_x = (progress_ratio * draw_w).clamp(1.0, (draw_w - 1.0).max(1.0));
    canvas.draw_line(
        Point::new(needle_x, 0.0),
        Point::new(needle_x, canvas.height()),
        style.lane_needle_color,
        1.0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::PanelBuilder;
    use irisui_core::{CanvasPrimitive, Rect, UiTree};

    #[test]
    fn test_track_lane_positions_markers_in_right_canvas_past_sidebar() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.track_lane(
                TrackLaneSpec::new("ROT", Color::CYAN, "Joint #2", 4.0, 1.0).row_index(1),
                &[0.0, 2.0, 4.0],
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 618.0, 40.0));
        }

        let canvas_node = tree
            .node_named("TrackLaneCanvas")
            .expect("TrackLaneCanvas node exists");
        assert!(
            canvas_node.computed_rect.x >= 118.0,
            "TrackLaneCanvas must start after the 118px sidebar column"
        );

        let canvas_id = tree
            .iter()
            .find(|(_, n)| n.name.as_deref() == Some("TrackLaneCanvas"))
            .map(|(id, _)| id)
            .expect("canvas id");
        let prims = tree.canvas_primitives(canvas_id).expect("canvas prims");
        let diamond_count = prims
            .iter()
            .filter(|p| matches!(p, CanvasPrimitive::Text { text, .. } if text == "◆"))
            .count();
        assert_eq!(diamond_count, 3);
    }

    #[test]
    fn test_track_lane_fmt_formats_label_without_heap_strings() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.track_lane_fmt::<24, _, _>(
                TrackLaneSpec::new("POS", Color::GREEN, "", 2.0, 0.5),
                [0.25_f32, 1.25_f32],
                |b| {
                    let _ = std::fmt::Write::write_fmt(b, format_args!("Joint #{}", 7));
                },
            );
        }

        assert!(
            tree.iter()
                .any(|(_, n)| n.text.as_deref() == Some("Joint #7")),
            "Stack-formatted joint label must be emitted inside sidebar badge"
        );
    }

    #[test]
    fn test_track_lane_group_overlay_is_passive_and_updates_in_place() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let overlay_tag = 0xAA70;
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.track_lane_group(
                TrackLaneGroupSpec::new(overlay_tag, 4.0, 1.0).sidebar_width(118.0),
                |group| {
                    group.track_lane(
                        TrackLaneSpec::new("ROT", Color::CYAN, "Joint #1", 4.0, 1.0)
                            .show_playhead(false),
                        &[0.5, 2.5],
                    );
                },
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 518.0, 40.0));
        }

        let overlay_node = tree
            .node_tagged(overlay_tag)
            .expect("group playhead overlay exists");
        assert!(!overlay_node.interactive);
        assert!(overlay_node.computed_rect.x >= 118.0);
        assert!(tree.is_live_tag(overlay_tag));

        let updated = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.update_track_lane_playhead(overlay_tag, 4.0, 3.0, TimeRulerStyle::dark_default())
        };
        assert!(updated);
    }
}
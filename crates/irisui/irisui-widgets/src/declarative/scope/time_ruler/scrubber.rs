// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Interactive Timeline Scrubber Track (`UiScope::scrubber_track`)
//!
//! Emits a responsive scrubber strip combining a left sidebar badge with an interactive
//! playhead track supporting click-to-seek, continuous pointer drag scrubbing, marker diamonds,
//! elapsed progress shading, and a playhead handle glyph (`▼`).

use super::ruler::TimeRulerStyle;
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::display::ImmediateCanvasSpec;
use irisui_core::{
    AlignItems, CanvasOrigin, Color, Insets, JustifyContent, Point, Rect, Style, TextAlign,
    WidgetCursor, WidgetResponse, WidgetRole,
};

/// A marker diamond rendered at a timestamp offset along a [`UiScope::scrubber_track`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrubberMarker {
    /// Timestamp position in seconds along the sequence duration.
    pub time: f32,
    /// Optional accent color override; falls back to [`TimeRulerStyle::marker_color`].
    pub color: Option<Color>,
}

impl ScrubberMarker {
    /// Creates a marker using the default [`TimeRulerStyle::marker_color`].
    #[inline]
    #[must_use]
    pub const fn new(time: f32) -> Self {
        Self { time, color: None }
    }

    /// Creates a marker with an explicit custom fill color.
    #[inline]
    #[must_use]
    pub const fn with_color(time: f32, color: Color) -> Self {
        Self {
            time,
            color: Some(color),
        }
    }
}

/// Configuration descriptor for [`UiScope::scrubber_track`] and [`UiScope::scrubber_track_value`].
#[derive(Clone, Copy, Debug)]
pub struct ScrubberTrackSpec<'a> {
    /// Debug node name assigned to the scrubber root container.
    pub name: &'static str,
    /// Persistent 64-bit semantic tag bound to the interactive track canvas.
    pub tag: u64,
    /// Optional semantic tag for a split non-interactive live playhead overlay canvas (`live_canvas_anchored`).
    ///
    /// When non-zero, keyframe markers stay in the static `tag` canvas while elapsed fill, needle,
    /// and handle cap render on a dedicated volatile overlay updatable at 60 FPS via [`UiScope::update_scrubber_playhead`].
    pub playhead_tag: u64,
    /// Optional semantic tag reserved for the top playhead cap indicator (`▼`).
    pub cap_tag: u64,
    /// Total duration represented by the full track width in seconds.
    pub duration: f32,
    /// Fixed height of the scrubber strip in logical pixels.
    pub height: f32,
    /// Width of the left sidebar column in logical pixels.
    pub sidebar_width: f32,
    /// Label rendered inside the left sidebar badge pill.
    pub sidebar_badge: &'a str,
    /// Read-only slice of key/event markers rendered along the scrubber track.
    pub markers: &'a [ScrubberMarker],
    /// Visual style palette for track background, fill, markers, and playhead needle.
    pub style: TimeRulerStyle,
}

impl<'a> ScrubberTrackSpec<'a> {
    /// Constructs a new [`ScrubberTrackSpec`] with default `36px` height and `118px` sidebar.
    #[inline]
    #[must_use]
    pub const fn new(tag: u64, duration: f32) -> Self {
        Self {
            name: "ScrubberTrackRoot",
            tag,
            playhead_tag: 0,
            cap_tag: 0,
            duration,
            height: 36.0,
            sidebar_width: 118.0,
            sidebar_badge: "✦ MASTER",
            markers: &[],
            style: TimeRulerStyle::dark_default(),
        }
    }

    /// Assigns a semantic tag for splitting elapsed progress, playhead needle, and cap onto a volatile live overlay canvas.
    #[inline]
    #[must_use]
    pub const fn playhead_tag(mut self, playhead_tag: u64) -> Self {
        self.playhead_tag = playhead_tag;
        self
    }

    /// Assigns an explicit semantic tag to the playhead cap glyph handle (`▼`).
    #[inline]
    #[must_use]
    pub const fn cap_tag(mut self, cap_tag: u64) -> Self {
        self.cap_tag = cap_tag;
        self
    }

    /// Overrides the debug node name for the scrubber root container.
    #[inline]
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Overrides the track height in logical pixels.
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

    /// Overrides the badge caption displayed in the left sidebar column.
    #[inline]
    #[must_use]
    pub const fn sidebar_badge(mut self, badge: &'a str) -> Self {
        self.sidebar_badge = badge;
        self
    }

    /// Supplies marker diamonds to render along the track axis.
    #[inline]
    #[must_use]
    pub const fn markers(mut self, markers: &'a [ScrubberMarker]) -> Self {
        self.markers = markers;
        self
    }

    /// Overrides the visual style palette for the scrubber track.
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: TimeRulerStyle) -> Self {
        self.style = style;
        self
    }
}

/// Interaction response returned by [`UiScope::scrubber_track`] and [`UiScope::scrubber_track_value`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrubberResponse {
    /// Underlying interaction response of the interactive track surface.
    pub response: WidgetResponse,
    /// Newly scrubbed timestamp in `[0.0, duration]` when the pointer interacted with the track.
    pub scrubbed_time: Option<f32>,
    /// `true` while the track is actively pressed, clicked, or captured by a drag gesture.
    pub active: bool,
}

impl ScrubberResponse {
    /// Returns `true` when pointer click or drag scrubbing updated the timestamp this frame.
    #[inline]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.scrubbed_time.is_some()
    }

    /// Returns `true` while the scrubber playhead is actively pressed or captured.
    #[inline]
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Invokes `f(new_time)` if scrubbing occurred during the current frame, preserving fluent chaining.
    #[inline]
    pub fn on_scrub(self, f: impl FnOnce(f32)) -> Self {
        if let Some(t) = self.scrubbed_time {
            f(t);
        }
        self
    }
}

/// Evaluates normalized time in `[0.0, duration]` for a screen-space pointer `cursor_x`
/// inside a scrubber area `[track_x, track_x + track_width]`.
#[inline]
#[must_use]
pub fn scrub_time_from_x(cursor_x: f32, track_x: f32, track_width: f32, duration: f32) -> f32 {
    if !cursor_x.is_finite() || !track_x.is_finite() || track_width <= 1e-4 {
        return 0.0;
    }
    let safe_duration = duration.max(0.0);
    let frac = ((cursor_x - track_x) / track_width).clamp(0.0, 1.0);
    frac * safe_duration
}

impl<'a> UiScope<'a> {
    /// Emits a two-way interactive scrubber track that updates `*current_time` in place on pointer
    /// click or drag and returns a [`ScrubberResponse`] for chaining `.on_scrub(...)`.
    pub fn scrubber_track(
        &mut self,
        spec: ScrubberTrackSpec<'_>,
        current_time: &mut f32,
    ) -> ScrubberResponse {
        let resp = self.scrubber_track_value(spec, *current_time);
        if let Some(new_t) = resp.scrubbed_time {
            *current_time = new_t;
        }
        resp
    }

    /// Emits a declarative interactive scrubber track driven by a read-only `current_time` value,
    /// computing any user click or drag seek into [`ScrubberResponse::scrubbed_time`] for
    /// `ActionBus` emission without requiring intermediate mutable `let` bindings in views.
    pub fn scrubber_track_value(
        &mut self,
        spec: ScrubberTrackSpec<'_>,
        current_time: f32,
    ) -> ScrubberResponse {
        let style = spec.style;
        let height = spec.height.max(16.0);
        let duration = spec.duration.max(0.0);
        let safe_duration = duration.max(0.001);
        let fallback_w = (self.available_rect().width - spec.sidebar_width).max(10.0);

        let is_active = self.is_tag_clicked(spec.tag)
            || self.is_tag_pressed(spec.tag)
            || self.is_tag_captured(spec.tag)
            || (spec.cap_tag != 0
                && (self.is_tag_pressed(spec.cap_tag) || self.is_tag_captured(spec.cap_tag)));

        let track_bounds = self
            .tree
            .resolve_anchor_rect(spec.tag)
            .filter(|r| r.width > 1.0)
            .unwrap_or_else(|| {
                let origin = self.parent_screen_origin();
                Rect::new(origin.x + spec.sidebar_width, origin.y, fallback_w, height)
            });

        let scrubbed_time = if is_active
            && let Some(pointer) = self
                .tree
                .event_state
                .frame_clicked_pos
                .or(self.tree.event_state.last_pointer_pos)
        {
            Some(scrub_time_from_x(
                pointer.x,
                track_bounds.x,
                track_bounds.width,
                duration,
            ))
        } else {
            None
        };

        let display_time = scrubbed_time.unwrap_or(current_time).clamp(0.0, duration);
        let progress_ratio = (display_time / safe_duration).clamp(0.0, 1.0);

        let outer_style = Style::new()
            .flex_row()
            .align_items(AlignItems::Stretch)
            .height(height)
            .background(style.track_bg)
            .border_radius(style.track_border_radius)
            .border(
                style.track_border_width,
                if is_active {
                    style.track_border_active
                } else {
                    style.track_border_idle
                },
            )
            .clip_children(false);

        let sidebar_col_style = Style::new()
            .width(spec.sidebar_width)
            .height(height)
            .flex_shrink(0.0)
            .padding_insets(Insets::new(3.0, 4.0, 3.0, 4.0))
            .align_items(AlignItems::FlexStart);

        let badge_style = Style::new()
            .width((spec.sidebar_width - 8.0).max(20.0))
            .height(18.0)
            .flex_row()
            .align_items(AlignItems::Center)
            .padding_insets(Insets::new(0.0, 6.0, 0.0, 6.0))
            .background(style.sidebar_badge_bg)
            .border(1.0, style.sidebar_badge_border)
            .border_radius(3.0);

        let playhead_tag = spec.playhead_tag;
        let cap_tag = spec.cap_tag;
        let markers = spec.markers;
        let sidebar_badge = spec.sidebar_badge;
        let resolved_track_w = track_bounds.width.max(10.0);

        let inner = self.container_named(spec.name, outer_style, |row| {
            row.container_named("ScrubberSidebarCol", sidebar_col_style, |col| {
                col.container_named("ScrubberMasterBadge", badge_style, |badge| {
                    badge.label_custom(sidebar_badge, 8.5, style.needle_color, TextAlign::Left);
                });
            });

            if playhead_tag != 0 {
                let stage_style = Style::new().flex_grow(1.0).height(height);
                let layer_style = Style::new()
                    .position_absolute()
                    .left(0.0)
                    .top(0.0)
                    .right(0.0)
                    .bottom(0.0);
                row.container_named("ScrubberTrackStage", stage_style, |stage| {
                    stage.immediate_canvas_with(
                        ImmediateCanvasSpec::new("ScrubberTrackCanvas", spec.tag, layer_style)
                            .origin(CanvasOrigin::NodeTopLeft)
                            .interactive(true)
                            .cursor(WidgetCursor::EwResize),
                        |canvas| {
                            let draw_w = if canvas.width() > 1.0 && canvas.bounds().x > 0.0 {
                                canvas.width()
                            } else {
                                resolved_track_w
                            }
                            .max(10.0);

                            for marker in markers {
                                let m_frac = (marker.time / safe_duration).clamp(0.0, 1.0);
                                let mx =
                                    (m_frac * draw_w - 4.0).clamp(0.0, (draw_w - 10.0).max(0.0));
                                let my = (height - 12.0) * 0.5;
                                canvas.draw_text(
                                    Point::new(mx, my),
                                    "◆",
                                    11.0,
                                    marker.color.unwrap_or(style.marker_color),
                                );
                            }
                        },
                    );
                    stage.live_canvas_overlay(
                        "ScrubberPlayheadOverlay",
                        playhead_tag,
                        CanvasOrigin::NodeTopLeft,
                        layer_style,
                        |canvas| {
                            draw_scrubber_playhead(
                                canvas,
                                height,
                                progress_ratio,
                                resolved_track_w,
                                style,
                            );
                        },
                    );
                });
            } else {
                let canvas_style = Style::new().flex_grow(1.0).height(height);
                row.immediate_canvas_with(
                    ImmediateCanvasSpec::new("ScrubberTrackCanvas", spec.tag, canvas_style)
                        .origin(CanvasOrigin::NodeTopLeft)
                        .interactive(true)
                        .cursor(WidgetCursor::EwResize),
                    |canvas| {
                        let draw_w = if canvas.width() > 1.0 && canvas.bounds().x > 0.0 {
                            canvas.width()
                        } else {
                            resolved_track_w
                        }
                        .max(10.0);

                        let fill_w = progress_ratio * draw_w;
                        if fill_w > 0.0 {
                            canvas.fill_rect(
                                Rect::new(0.0, 0.0, fill_w, height),
                                style.progress_fill_color,
                                style.track_border_radius,
                            );
                        }

                        for marker in markers {
                            let m_frac = (marker.time / safe_duration).clamp(0.0, 1.0);
                            let mx = (m_frac * draw_w - 4.0).clamp(0.0, (draw_w - 10.0).max(0.0));
                            let my = (height - 12.0) * 0.5;
                            canvas.draw_text(
                                Point::new(mx, my),
                                "◆",
                                11.0,
                                marker.color.unwrap_or(style.marker_color),
                            );
                        }

                        let needle_x =
                            (progress_ratio * draw_w).clamp(1.0, (draw_w - 1.0).max(1.0));
                        canvas.draw_line(
                            Point::new(needle_x, 0.0),
                            Point::new(needle_x, height),
                            style.needle_color,
                            style.needle_width,
                        );

                        if cap_tag == 0 {
                            let cap_x = (needle_x - 5.0).clamp(0.0, (draw_w - 10.0).max(0.0));
                            canvas.draw_text(Point::new(cap_x, -1.0), "▼", 9.0, style.handle_color);
                        }
                    },
                );

                if cap_tag != 0 {
                    let cap_w = 12.0;
                    let cap_x = spec.sidebar_width
                        + (progress_ratio * resolved_track_w - cap_w * 0.5)
                            .clamp(0.0, (resolved_track_w - cap_w).max(0.0));
                    let cap_style = Style::new()
                        .position_absolute()
                        .left(cap_x)
                        .top(0.0)
                        .width(cap_w)
                        .height(10.0)
                        .align_items(AlignItems::Center)
                        .justify_content(JustifyContent::Center);
                    row.container_tagged(
                        "ScrubberPlayheadCap",
                        cap_style,
                        WidgetRole::Button,
                        cap_tag,
                        |cap| {
                            cap.label_custom("▼", 9.0, style.handle_color, TextAlign::Center);
                        },
                    );
                }
            }
        });

        let track_resp = inner
            .response
            .with_pointer_pos(self.tree.event_state.last_pointer_pos);
        let merged_scrub = scrubbed_time.or_else(|| {
            if track_resp.clicked()
                && let Some(pt) = self.tree.event_state.frame_clicked_pos
            {
                Some(scrub_time_from_x(
                    pt.x,
                    track_bounds.x,
                    track_bounds.width,
                    duration,
                ))
            } else {
                None
            }
        });

        ScrubberResponse {
            response: track_resp,
            scrubbed_time: merged_scrub,
            active: is_active,
        }
    }

    /// Updates a split scrubber playhead overlay (`playhead_tag`) in-place during `InPlaceRefresh`
    /// without re-emitting marker glyphs or triggering Taffy layout recomputation.
    ///
    /// Returns `true` when the live playhead geometry actually changed.
    pub fn update_scrubber_playhead(
        &mut self,
        playhead_tag: u64,
        duration: f32,
        current_time: f32,
        height: f32,
        style: TimeRulerStyle,
    ) -> bool {
        let clamped_h = height.max(16.0);
        let safe_duration = duration.max(0.001);
        let progress_ratio =
            (current_time.clamp(0.0, duration.max(0.0)) / safe_duration).clamp(0.0, 1.0);
        self.update_live_canvas(playhead_tag, |canvas| {
            let fallback_w = canvas.width().max(10.0);
            draw_scrubber_playhead(canvas, clamped_h, progress_ratio, fallback_w, style);
        })
    }
}

fn draw_scrubber_playhead(
    canvas: &mut crate::canvas::ImmediateCanvas<'_>,
    height: f32,
    progress_ratio: f32,
    fallback_w: f32,
    style: TimeRulerStyle,
) {
    let draw_w = if canvas.width() > 1.0 && canvas.bounds().x > 0.0 {
        canvas.width()
    } else {
        fallback_w
    }
    .max(10.0);

    let fill_w = progress_ratio * draw_w;
    if fill_w > 0.0 {
        canvas.fill_rect(
            Rect::new(0.0, 0.0, fill_w, height),
            style.progress_fill_color,
            style.track_border_radius,
        );
    }

    let needle_x = (progress_ratio * draw_w).clamp(1.0, (draw_w - 1.0).max(1.0));
    canvas.draw_line(
        Point::new(needle_x, 0.0),
        Point::new(needle_x, height),
        style.needle_color,
        style.needle_width,
    );

    let cap_x = (needle_x - 5.0).clamp(0.0, (draw_w - 10.0).max(0.0));
    canvas.fill_rect(Rect::new(cap_x, 0.0, 10.0, 5.0), style.handle_color, 2.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::PanelBuilder;
    use irisui_core::{CanvasPrimitive, UiEvent, UiTree};

    #[test]
    fn test_scrub_time_from_x_clamps_bounds() {
        assert_eq!(scrub_time_from_x(50.0, 100.0, 200.0, 4.0), 0.0);
        assert_eq!(scrub_time_from_x(200.0, 100.0, 200.0, 4.0), 2.0);
        assert_eq!(scrub_time_from_x(400.0, 100.0, 200.0, 4.0), 4.0);
        assert_eq!(scrub_time_from_x(f32::NAN, 100.0, 200.0, 4.0), 0.0);
    }

    #[test]
    fn test_scrubber_track_renders_markers_and_handles_click_seek() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let track_tag = 0xAA50;
        let cap_tag = 0xAA51;
        let markers = [ScrubberMarker::new(0.5), ScrubberMarker::new(2.0)];
        let mut cur_time = 1.0_f32;

        // First layout pass to compute concrete screen rect of track canvas
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.scrubber_track(
                ScrubberTrackSpec::new(track_tag, 4.0)
                    .cap_tag(cap_tag)
                    .markers(&markers),
                &mut cur_time,
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 518.0, 40.0));
        }

        let canvas_id = tree.find_node_by_tag(track_tag).expect("track canvas");
        let canvas_rect = tree.get(canvas_id).expect("node").computed_rect;
        assert!(canvas_rect.width >= 390.0);

        let prims = tree.canvas_primitives(canvas_id).expect("primitives");
        assert!(
            prims
                .iter()
                .any(|p| matches!(p, CanvasPrimitive::Rect { .. }))
        );
        assert!(
            prims
                .iter()
                .any(|p| matches!(p, CanvasPrimitive::Line { .. }))
        );
        let diamond_count = prims
            .iter()
            .filter(|p| matches!(p, CanvasPrimitive::Text { text, .. } if text == "◆"))
            .count();
        assert_eq!(diamond_count, 2);

        // Simulate pointer click at 75% across the canvas width -> should seek to 3.0s
        let click_x = canvas_rect.x + canvas_rect.width * 0.75;
        let click_y = canvas_rect.y + canvas_rect.height * 0.5;
        let _ = tree.dispatch_event(&UiEvent::MouseDown {
            button: irisui_core::MouseButton::Left,
            point: Point::new(click_x, click_y),
        });
        let _ = tree.dispatch_event(&UiEvent::MouseUp {
            button: irisui_core::MouseButton::Left,
            point: Point::new(click_x, click_y),
        });

        let mut scrub_fired = None;
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope
                .scrubber_track(
                    ScrubberTrackSpec::new(track_tag, 4.0).markers(&markers),
                    &mut cur_time,
                )
                .on_scrub(|t| scrub_fired = Some(t));
        }

        assert!(
            scrub_fired.is_some(),
            "on_scrub callback must fire on click"
        );
        assert!((cur_time - 3.0).abs() < 0.05);
    }

    #[test]
    fn test_scrubber_split_playhead_updates_in_place_without_touching_markers() {
        let mut tree = UiTree::new();
        let root = PanelBuilder::new(&mut tree).build();
        let _ = tree.set_root(root);

        let track_tag = 0xAA60;
        let playhead_tag = 0xAA61;
        let markers = [ScrubberMarker::new(1.0), ScrubberMarker::new(3.0)];
        {
            let mut scope = UiScope::new(&mut tree, root);
            scope.scrubber_track_value(
                ScrubberTrackSpec::new(track_tag, 4.0)
                    .playhead_tag(playhead_tag)
                    .markers(&markers),
                1.0,
            );
            scope.finish_layout(Rect::new(0.0, 0.0, 518.0, 40.0));
        }

        let track_id = tree.find_node_by_tag(track_tag).expect("static track");
        let playhead_id = tree.find_node_by_tag(playhead_tag).expect("live playhead");
        assert!(tree.is_live_tag(playhead_tag));
        assert_eq!(tree.canvas_primitives(track_id).expect("markers").len(), 2);
        let initial_line_x = match tree
            .canvas_primitives(playhead_id)
            .expect("playhead")
            .iter()
            .find(|p| matches!(p, CanvasPrimitive::Line { .. }))
        {
            Some(CanvasPrimitive::Line { start, .. }) => start.x,
            _ => panic!("expected playhead line"),
        };

        let updated = {
            let mut scope = UiScope::new(&mut tree, root);
            scope.update_scrubber_playhead(
                playhead_tag,
                4.0,
                3.0,
                36.0,
                TimeRulerStyle::dark_default(),
            )
        };
        assert!(updated);
        let updated_line_x = match tree
            .canvas_primitives(playhead_id)
            .expect("playhead")
            .iter()
            .find(|p| matches!(p, CanvasPrimitive::Line { .. }))
        {
            Some(CanvasPrimitive::Line { start, .. }) => start.x,
            _ => panic!("expected updated playhead line"),
        };
        assert!(
            (updated_line_x - 300.0).abs() < 2.0,
            "initial_line_x={initial_line_x}, updated_line_x={updated_line_x}, playhead_rect={:?}, track_rect={:?}",
            tree.get(playhead_id).map(|n| n.computed_rect),
            tree.get(track_id).map(|n| n.computed_rect),
        );
        assert!(
            !tree.get(playhead_id).expect("playhead node").interactive,
            "playhead overlay must be non-interactive so clicks pass through to track"
        );
        assert_eq!(tree.canvas_primitives(track_id).expect("markers").len(), 2);
    }
}
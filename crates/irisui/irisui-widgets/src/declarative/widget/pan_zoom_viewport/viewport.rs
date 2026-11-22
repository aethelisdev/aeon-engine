// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # `UiScope::pan_zoom_viewport` Implementation
//!
//! Renders the outer letterbox container (`WidgetRole::PanZoomViewport`), handles wheel zoom
//! and middle-mouse pan state updates, draws the background grid on a single passive
//! [`ImmediateCanvas`](crate::canvas::ImmediateCanvas), and invokes user content inside [`PanZoomScope`].

use super::math::PanZoomTransform;
use super::scope::PanZoomScope;
use super::spec::PanZoomSpec;
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::display::ImmediateCanvasSpec;
use crate::declarative::types::WidgetResponse;
use irisui_core::{
    CanvasOrigin, Color, Point, Position, Rect, Size, StackString, Style, TextAlign, UiTag,
    WidgetRole,
};

/// Result returned by [`UiScope::pan_zoom_viewport`].
///
/// Carries the computed [`PanZoomTransform`], whether wheel zoom or middle-drag panning
/// mutated state during this frame, the viewport [`WidgetResponse`], and the closure result `R`.
pub struct PanZoomViewportResponse<R> {
    /// Interaction response of the outer letterbox viewport container.
    pub response: WidgetResponse,
    /// Computed geometric transformation used for this frame.
    pub transform: PanZoomTransform,
    /// `true` if scroll-wheel zoom or middle-mouse drag modified `zoom` or `pan_offset`.
    pub view_changed: bool,
    /// User-returned value from the viewport content closure.
    pub inner: R,
}

impl<R> PanZoomViewportResponse<R> {
    /// Invokes `f` when wheel zooming or panning changed the viewport transform this frame.
    #[inline]
    pub fn on_view_changed(self, f: impl FnOnce()) -> Self {
        if self.view_changed {
            f();
        }
        self
    }

    /// Invokes `f` when the user clicked directly on empty letterbox/canvas background.
    #[inline]
    pub fn on_background_click(self, f: impl FnOnce()) -> Self {
        if self.response.clicked() {
            f();
        }
        self
    }
}

impl<'a> UiScope<'a> {
    /// Emits an interactive 2D pan-and-zoom viewport with aspect-fitted canvas, single-canvas
    /// SDF grid overlay, scroll-wheel zoom, and middle-mouse drag panning.
    ///
    /// Automatically resolves container bounds from layout memory (`node_rect_tagged`) with
    /// preceding sibling height accounting for cold frame #0 layout accuracy.
    pub fn pan_zoom_viewport<R, F>(
        &mut self,
        spec: PanZoomSpec<'_>,
        f: F,
    ) -> PanZoomViewportResponse<R>
    where
        F: FnOnce(&mut PanZoomScope<'_, '_>) -> R,
    {
        let mut view_changed = false;

        if spec.tag != 0
            && let Some((_, dy)) = self.tag_scroll_delta(spec.tag)
            && dy.abs() > f32::EPSILON
        {
            let step = if dy.abs() <= 1.0 {
                dy * 0.1
            } else {
                dy.signum() * 0.1
            };
            let next_zoom = (*spec.zoom + step).clamp(spec.min_zoom, spec.max_zoom);
            if (next_zoom - *spec.zoom).abs() > f32::EPSILON {
                *spec.zoom = next_zoom;
                view_changed = true;
            }
        }

        if spec.tag != 0
            && let Some(pan_delta) = self.tag_drag_delta(spec.tag)
            && (pan_delta.x.abs() > f32::EPSILON || pan_delta.y.abs() > f32::EPSILON)
        {
            spec.pan_offset[0] += pan_delta.x;
            spec.pan_offset[1] += pan_delta.y;
            view_changed = true;
        }

        let viewport_rect = self
            .node_rect_tagged(spec.tag, Point::ZERO)
            .filter(|r| r.width > 1.0 && r.height > 1.0)
            .unwrap_or_else(|| self.fallback_remaining_rect());

        let transform = PanZoomTransform::compute(
            viewport_rect,
            spec.virtual_size,
            spec.margin,
            *spec.zoom,
            *spec.pan_offset,
        );

        let letterbox_style = Style::new()
            .flex_col()
            .flex_grow(1.0)
            .min_height(0.0)
            .flex_shrink(1.0)
            .background(spec.letterbox_color)
            .clip_children(true);

        let canvas_local = transform.canvas_local_rect;
        let grid_tag = UiTag(spec.canvas_tag).child("grid").raw();
        let show_grid = spec.show_grid;
        let grid_step_virtual = spec.grid_step_virtual;
        let grid_major_interval = spec.grid_major_interval.max(1);
        let grid_minor_color = spec.grid_minor_color;
        let grid_major_color = spec.grid_major_color;
        let show_badge = spec.show_dimensions_badge;
        let canvas_name = spec.canvas_name;
        let canvas_tag = spec.canvas_tag;

        let canvas_style = Style::new()
            .position_absolute()
            .left(canvas_local.x)
            .top(canvas_local.y)
            .width(canvas_local.width)
            .height(canvas_local.height)
            .background(spec.canvas_background)
            .border(spec.canvas_border_width, spec.canvas_border_color)
            .clip_children(true);

        let inner_resp = self.container_tagged(
            spec.name,
            letterbox_style,
            WidgetRole::PanZoomViewport,
            spec.tag,
            |lb_scope| {
                lb_scope
                    .container_tagged(
                        canvas_name,
                        canvas_style,
                        WidgetRole::Default,
                        canvas_tag,
                        |canvas_scope| {
                            if show_grid {
                                emit_grid_canvas(
                                    canvas_scope,
                                    PanZoomGridParams {
                                        grid_tag,
                                        canvas_size: Size::new(
                                            canvas_local.width,
                                            canvas_local.height,
                                        ),
                                        grid_step: grid_step_virtual * transform.base_scale,
                                        major_interval: grid_major_interval,
                                        minor_color: grid_minor_color,
                                        major_color: grid_major_color,
                                    },
                                );
                            }

                            let mut pz_scope = PanZoomScope::new(canvas_scope, transform);
                            let out = f(&mut pz_scope);

                            if show_badge {
                                emit_dimensions_badge(
                                    canvas_scope,
                                    transform.virtual_size.width,
                                    transform.virtual_size.height,
                                    canvas_local.height,
                                );
                            }

                            out
                        },
                    )
                    .inner
            },
        );

        PanZoomViewportResponse {
            response: inner_resp.response,
            transform,
            view_changed,
            inner: inner_resp.inner,
        }
    }

    /// Computes remaining parent rectangle on cold frame #0 by subtracting fixed-height preceding siblings.
    fn fallback_remaining_rect(&self) -> Rect {
        let p = self.parent_rect();
        let used_top: f32 = self
            .node_ref(self.parent)
            .map(|parent_node| {
                parent_node
                    .children
                    .iter()
                    .filter_map(|&cid| self.node_ref(cid))
                    .filter(|child| child.style.position != Position::Absolute)
                    .filter_map(|child| child.style.height)
                    .sum()
            })
            .unwrap_or(0.0);

        Rect::new(
            p.x,
            p.y + used_top,
            p.width,
            (p.height - used_top).max(120.0),
        )
    }
}

struct PanZoomGridParams {
    grid_tag: u64,
    canvas_size: Size,
    grid_step: f32,
    major_interval: usize,
    minor_color: Color,
    major_color: Color,
}

/// Emits a single passive [`ImmediateCanvas`](crate::canvas::ImmediateCanvas) drawing minor/major grid lines.
fn emit_grid_canvas(scope: &mut UiScope<'_>, grid: PanZoomGridParams) {
    let canvas_w = grid.canvas_size.width;
    let canvas_h = grid.canvas_size.height;
    if grid.grid_step < 8.0 || canvas_w <= 1.0 || canvas_h <= 1.0 {
        return;
    }

    let grid_spec = ImmediateCanvasSpec::new(
        "PanZoomGridCanvas",
        grid.grid_tag,
        Style::new()
            .position_absolute()
            .left(0.0)
            .top(0.0)
            .width(canvas_w)
            .height(canvas_h),
    )
    .origin(CanvasOrigin::NodeTopLeft)
    .interactive(false);

    scope.immediate_canvas_with(grid_spec, |canvas| {
        let cols = (canvas_w / grid.grid_step).floor() as usize + 1;
        for col in 0..cols {
            let x = (col as f32) * grid.grid_step;
            let color = if col.is_multiple_of(grid.major_interval) {
                grid.major_color
            } else {
                grid.minor_color
            };
            canvas.draw_line(Point::new(x, 0.0), Point::new(x, canvas_h), color, 1.0);
        }

        let rows = (canvas_h / grid.grid_step).floor() as usize + 1;
        for row in 0..rows {
            let y = (row as f32) * grid.grid_step;
            let color = if row.is_multiple_of(grid.major_interval) {
                grid.major_color
            } else {
                grid.minor_color
            };
            canvas.draw_line(Point::new(0.0, y), Point::new(canvas_w, y), color, 1.0);
        }
    });
}

/// Emits the bottom-left virtual resolution indicator without heap string allocations.
fn emit_dimensions_badge(scope: &mut UiScope<'_>, width: f32, height: f32, canvas_h: f32) {
    let dim_style = Style::new()
        .position_absolute()
        .left(8.0)
        .top((canvas_h - 22.0).max(4.0))
        .width(120.0)
        .height(16.0);
    let badge = StackString::<32>::from_args(format_args!("{width:.0} × {height:.0}"));
    scope.label_styled_passive(
        "PanZoomDimensionsBadge",
        badge.as_str(),
        10.0,
        Color::rgba(0.0, 0.706, 0.941, 0.75),
        TextAlign::Left,
        dim_style,
    );
}
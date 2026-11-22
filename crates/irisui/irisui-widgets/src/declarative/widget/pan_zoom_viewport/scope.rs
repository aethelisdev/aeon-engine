// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Scoped Helpers for `PanZoomViewport`
//!
//! Provides [`PanZoomScope`] with virtual coordinate drawing, draggable element boxes
//! (including passive selection outlines & corner handles), and anchor/pin guide overlays.

use super::math::PanZoomTransform;
use crate::declarative::scope::core::UiScope;
use crate::declarative::types::WidgetResponse;
use crate::declarative::widget::empty_box::EmptyBox;
use irisui_core::{Color, Point, Rect, StackString, Style, TextAlign, WidgetRole};
use std::ops::{Deref, DerefMut};

/// Configuration descriptor for an interactive draggable/selectable box inside a [`PanZoomScope`].
#[derive(Debug, Clone, Copy)]
pub struct PanZoomBoxSpec {
    /// Diagnostic widget name attached to the interactive hitbox node.
    pub name: &'static str,
    /// Persistent 64-bit semantic tag identifying the element across frames.
    pub tag: u64,
    /// Element bounding box `[min_x, min_y, max_x, max_y]` in virtual canvas space.
    pub virtual_rect: [f32; 4],
    /// Whether this box should display active selection outline and corner handles.
    pub selected: bool,
    /// Border color used for the active selection outline.
    pub outline_color: Color,
    /// Border color used when hovered while unselected.
    pub hover_outline_color: Color,
    /// Size of the 4 corner handle squares in logical pixels when selected.
    pub handle_size: f32,
    /// Optional anchor pin guide `(anchor_virtual_pos, label)` rendered when selected.
    pub anchor_guide: Option<(Point, &'static str)>,
}

impl PanZoomBoxSpec {
    /// Creates a new [`PanZoomBoxSpec`] with standard cyan selection styling.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, tag: u64, virtual_rect: [f32; 4], selected: bool) -> Self {
        Self {
            name,
            tag,
            virtual_rect,
            selected,
            outline_color: Color::rgba(0.0, 0.85, 1.0, 0.95),
            hover_outline_color: Color::rgba(0.0, 0.706, 0.941, 0.50),
            handle_size: 6.0,
            anchor_guide: None,
        }
    }

    /// Attaches an anchor pin guide rendered automatically while `self.selected` is true.
    #[inline]
    #[must_use]
    pub const fn with_anchor_guide(
        mut self,
        enabled: bool,
        anchor_virtual: Point,
        label: &'static str,
    ) -> Self {
        if enabled {
            self.anchor_guide = Some((anchor_virtual, label));
        }
        self
    }
}

/// Fluent interaction response returned by [`PanZoomScope::draggable_box`].
///
/// Enables chainable `.on_click`, `.on_drag_virtual`, `.on_drag_total_virtual`,
/// and `.on_drag_end` callbacks without imperative intermediate bindings.
#[derive(Debug, Clone)]
pub struct PanZoomBoxResponse {
    /// Underlying [`WidgetResponse`] of the interactive hitbox node.
    pub widget: WidgetResponse,
    /// Resolved local pixel rectangle inside the scaled canvas surface.
    pub local_rect: Rect,
    /// Screen-to-virtual displacement for the current frame while actively dragged.
    pub frame_virtual_delta: Option<Point>,
    /// Cumulative screen-to-virtual displacement since pointer press origin while actively dragged.
    pub total_virtual_delta: Option<Point>,
    /// Whether an active drag gesture on this box ended during the current frame.
    pub drag_released: bool,
}

impl PanZoomBoxResponse {
    /// Returns `true` if this element was clicked during the current frame.
    #[inline]
    #[must_use]
    pub const fn clicked(&self) -> bool {
        self.widget.clicked()
    }

    /// Returns `true` if this element is currently hovered by the pointer.
    #[inline]
    #[must_use]
    pub const fn hovered(&self) -> bool {
        self.widget.hovered()
    }

    /// Invokes `f` when this element receives a primary click without dragging.
    #[inline]
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        if self.widget.clicked() {
            f();
        }
        self
    }

    /// Invokes `f` with the per-frame virtual displacement delta during active dragging.
    #[inline]
    pub fn on_drag_virtual(self, f: impl FnOnce(Point)) -> Self {
        if let Some(delta) = self.frame_virtual_delta {
            f(delta);
        }
        self
    }

    /// Invokes `f` with the cumulative virtual displacement since press origin during active dragging.
    #[inline]
    pub fn on_drag_total_virtual(self, f: impl FnOnce(Point)) -> Self {
        if let Some(total) = self.total_virtual_delta {
            f(total);
        }
        self
    }

    /// Invokes `f` when a drag gesture on this element is released during the current frame.
    #[inline]
    pub fn on_drag_end(self, f: impl FnOnce()) -> Self {
        if self.drag_released {
            f();
        }
        self
    }
}

/// Configuration descriptor for [`PanZoomScope::pin_guide`].
///
/// Renders an anchor pin marker, horizontal/vertical distance guide lines from an anchor
/// coordinate to an element center, and a compact stack-formatted badge label.
#[derive(Debug, Clone, Copy)]
pub struct PinGuideSpec<'a> {
    /// Anchor origin coordinate in virtual canvas units.
    pub anchor_virtual: Point,
    /// Target element rectangle in local canvas pixel coordinates.
    pub element_local_rect: Rect,
    /// Human-readable anchor caption rendered next to the pin (`"⚓ {anchor_label}"`).
    pub anchor_label: &'a str,
    /// Color tint for guide lines, pin marker, and badge caption.
    pub color: Color,
}

impl<'a> PinGuideSpec<'a> {
    /// Creates a new [`PinGuideSpec`] with standard amber guide styling.
    #[inline]
    #[must_use]
    pub const fn new(
        anchor_virtual: Point,
        element_local_rect: Rect,
        anchor_label: &'a str,
    ) -> Self {
        Self {
            anchor_virtual,
            element_local_rect,
            anchor_label,
            color: Color::rgba(1.0, 0.65, 0.15, 0.85),
        }
    }
}

/// Scoped declarative builder context exposed inside [`UiScope::pan_zoom_viewport`](crate::declarative::scope::UiScope::pan_zoom_viewport).
///
/// Dereferences transparently to the inner scaled canvas [`UiScope`] while providing
/// virtual-space primitive drawing, draggable element hitboxes, and anchor guide overlays.
pub struct PanZoomScope<'a, 's> {
    scope: &'a mut UiScope<'s>,
    transform: PanZoomTransform,
}

impl<'a, 's> Deref for PanZoomScope<'a, 's> {
    type Target = UiScope<'s>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.scope
    }
}

impl<'a, 's> DerefMut for PanZoomScope<'a, 's> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.scope
    }
}

impl<'a, 's> PanZoomScope<'a, 's> {
    /// Constructs a new [`PanZoomScope`] wrapping the inner canvas [`UiScope`] and [`PanZoomTransform`].
    #[inline]
    pub fn new(scope: &'a mut UiScope<'s>, transform: PanZoomTransform) -> Self {
        Self { scope, transform }
    }

    /// Returns the resolved geometric transformation for this frame.
    #[inline]
    #[must_use]
    pub const fn transform(&self) -> &PanZoomTransform {
        &self.transform
    }

    /// Returns the pointer cursor position projected into virtual canvas coordinates,
    /// or `None` when the pointer is absent or outside the scaled canvas surface.
    #[must_use]
    pub fn pointer_virtual_pos(&self) -> Option<Point> {
        let cursor = self.scope.cursor_pos()?;
        if self.transform.contains_screen_point(cursor) {
            Some(self.transform.screen_to_virtual(cursor))
        } else {
            None
        }
    }

    /// Emits a passive filled/bordered rectangle transformed from virtual bounds `[min_x, min_y, max_x, max_y]`.
    pub fn draw_filled_rect_virtual(
        &mut self,
        virtual_bounds: [f32; 4],
        fill: Color,
        border_color: Option<Color>,
        border_width: f32,
        border_radius: f32,
    ) {
        let local = self.transform.virtual_bounds_to_local(virtual_bounds);
        let scale = self.transform.base_scale;
        let mut style = Style::new()
            .position_absolute()
            .left(local.x)
            .top(local.y)
            .width(local.width)
            .height(local.height)
            .background(fill);

        if let Some(bc) = border_color
            && bc.a > 0.01
            && border_width > 0.0
        {
            style = style.border((border_width * scale).max(1.0), bc);
        }
        if border_radius > 0.0 {
            style = style.border_radius(border_radius * scale);
        }
        self.scope.empty_box_passive(style);
    }

    /// Emits a passive tinted quad transformed from virtual bounds `[min_x, min_y, max_x, max_y]`.
    pub fn draw_image_rect_virtual(&mut self, virtual_bounds: [f32; 4], tint: Color) {
        let local = self.transform.virtual_bounds_to_local(virtual_bounds);
        let style = Style::new()
            .position_absolute()
            .left(local.x)
            .top(local.y)
            .width(local.width)
            .height(local.height)
            .background(tint);
        self.scope.empty_box_passive(style);
    }

    /// Emits a passive aligned text label positioned in virtual canvas coordinates.
    pub fn draw_text_virtual(
        &mut self,
        virtual_pos: Point,
        text: &str,
        font_size: f32,
        color: Color,
        align: TextAlign,
    ) {
        let local = self.transform.virtual_to_local(virtual_pos);
        let scaled_font = (font_size * self.transform.base_scale).max(8.0);
        let approx_w = (text.len() as f32) * (scaled_font * 0.65);
        let approx_h = scaled_font * 1.3;
        let text_x = match align {
            TextAlign::Left => local.x,
            TextAlign::Center => local.x - approx_w * 0.5,
            TextAlign::Right => local.x - approx_w,
        };
        let text_y = local.y - approx_h * 0.5;
        self.scope.label_styled_passive(
            "PanZoomVirtualText",
            text,
            scaled_font,
            color,
            align,
            Style::new()
                .position_absolute()
                .left(text_x)
                .top(text_y)
                .width(approx_w)
                .height(approx_h),
        );
    }

    /// Emits an interactive selectable/draggable element box with passive hover, selection outline,
    /// and corner sizing handles.
    ///
    /// Passive outlines and corner handles never intercept pointer hit-testing against the element.
    pub fn draggable_box(&mut self, spec: PanZoomBoxSpec) -> PanZoomBoxResponse {
        let raw_local = self.transform.virtual_bounds_to_local(spec.virtual_rect);
        let local_rect = Rect::new(
            raw_local.x,
            raw_local.y,
            raw_local.width.max(1.0),
            raw_local.height.max(1.0),
        );

        let elem_style = Style::new()
            .position_absolute()
            .left(local_rect.x)
            .top(local_rect.y)
            .width(local_rect.width)
            .height(local_rect.height)
            .background(Color::TRANSPARENT);

        let widget = self.scope.add(
            EmptyBox::new(spec.name, elem_style)
                .role(WidgetRole::Button)
                .tag(spec.tag),
        );

        let frame_virtual_delta = self
            .scope
            .tag_drag_delta(spec.tag)
            .map(|d| self.transform.screen_delta_to_virtual(d));
        let total_virtual_delta = self
            .scope
            .tag_drag_total_delta(spec.tag)
            .map(|d| self.transform.screen_delta_to_virtual(d));
        let drag_released = self.scope.is_tag_drag_released(spec.tag);

        if spec.selected {
            let outline_style = Style::new()
                .position_absolute()
                .left(local_rect.x - 2.0)
                .top(local_rect.y - 2.0)
                .width(local_rect.width + 4.0)
                .height(local_rect.height + 4.0)
                .border(1.8, spec.outline_color)
                .border_radius(2.0);
            self.scope.empty_box_passive(outline_style);

            let half = spec.handle_size * 0.5;
            let corners = [
                Point::new(local_rect.x, local_rect.y),
                Point::new(local_rect.right(), local_rect.y),
                Point::new(local_rect.x, local_rect.bottom()),
                Point::new(local_rect.right(), local_rect.bottom()),
            ];
            for corner in corners {
                let handle_style = Style::new()
                    .position_absolute()
                    .left(corner.x - half)
                    .top(corner.y - half)
                    .width(spec.handle_size)
                    .height(spec.handle_size)
                    .background(Color::rgba(1.0, 1.0, 1.0, 0.98))
                    .border(1.0, Color::rgba(0.0, 0.5, 0.8, 0.9))
                    .border_radius(1.0);
                self.scope.empty_box_passive(handle_style);
            }

            if let Some((anchor_virtual, anchor_label)) = spec.anchor_guide {
                self.pin_guide(PinGuideSpec::new(anchor_virtual, local_rect, anchor_label));
            }
        } else if widget.hovered() || self.scope.is_tag_hovered(spec.tag) {
            let hover_style = Style::new()
                .position_absolute()
                .left(local_rect.x - 1.0)
                .top(local_rect.y - 1.0)
                .width(local_rect.width + 2.0)
                .height(local_rect.height + 2.0)
                .border(1.0, spec.hover_outline_color)
                .border_radius(2.0);
            self.scope.empty_box_passive(hover_style);
        }

        PanZoomBoxResponse {
            widget,
            local_rect,
            frame_virtual_delta,
            total_virtual_delta,
            drag_released,
        }
    }

    /// Renders a passive anchor pin marker, horizontal/vertical distance guide lines, and stack label.
    ///
    /// Uses `StackString::<48>` for zero-heap caption formatting.
    pub fn pin_guide(&mut self, spec: PinGuideSpec<'_>) {
        let anchor_local = self.transform.virtual_to_local(spec.anchor_virtual);
        let elem_center = Point::new(
            spec.element_local_rect.x + spec.element_local_rect.width * 0.5,
            spec.element_local_rect.y + spec.element_local_rect.height * 0.5,
        );

        let guide_color = Color::rgba(spec.color.r, spec.color.g, spec.color.b, 0.70);

        if (elem_center.x - anchor_local.x).abs() > 2.0 {
            let min_x = anchor_local.x.min(elem_center.x);
            let width = (elem_center.x - anchor_local.x).abs();
            let h_style = Style::new()
                .position_absolute()
                .left(min_x)
                .top(anchor_local.y - 0.5)
                .width(width)
                .height(1.0)
                .background(guide_color);
            self.scope
                .empty_box_passive_named("PanZoomAnchorGuideH", h_style);
        }

        if (elem_center.y - anchor_local.y).abs() > 2.0 {
            let min_y = anchor_local.y.min(elem_center.y);
            let height = (elem_center.y - anchor_local.y).abs();
            let v_style = Style::new()
                .position_absolute()
                .left(elem_center.x - 0.5)
                .top(min_y)
                .width(1.0)
                .height(height)
                .background(guide_color);
            self.scope
                .empty_box_passive_named("PanZoomAnchorGuideV", v_style);
        }

        let pin_style = Style::new()
            .position_absolute()
            .left(anchor_local.x - 4.0)
            .top(anchor_local.y - 4.0)
            .width(8.0)
            .height(8.0)
            .background(Color::rgba(spec.color.r, spec.color.g, spec.color.b, 0.95))
            .border(1.0, Color::rgba(1.0, 1.0, 1.0, 0.95))
            .border_radius(4.0);
        self.scope
            .empty_box_passive_named("PanZoomAnchorPin", pin_style);

        let label_y = if anchor_local.y > 24.0 {
            anchor_local.y - 16.0
        } else {
            anchor_local.y + 8.0
        };
        let label_x = (anchor_local.x + 8.0).min(self.transform.canvas_rect.width - 96.0);
        let label_style = Style::new()
            .position_absolute()
            .left(label_x)
            .top(label_y)
            .width(100.0)
            .height(14.0);

        let caption = StackString::<48>::from_args(format_args!("⚓ {}", spec.anchor_label));
        self.scope.label_styled_passive(
            "PanZoomAnchorLabel",
            caption.as_str(),
            9.5,
            Color::rgba(spec.color.r, spec.color.g, spec.color.b, 0.95),
            TextAlign::Left,
            label_style,
        );
    }
}
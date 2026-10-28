// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Frameless Window Host and Event Dispatcher
//!
//! Handles native window events for frameless desktop windows, orchestrating 8-way border
//! hit-testing, dynamic cursor icon updates, clean cursor restoration, and interactive hardware drag-resizing.

use super::geometry::{DEFAULT_BORDER_THICKNESS, DEFAULT_CORNER_SIZE, evaluate_resize_direction};
use super::window_trait::WindowInterface;
use iris_core::event::UiEvent;
use iris_core::geometry::{Point, Size};
use std::cell::Cell;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::window::CursorIcon;

/// Coordinates frameless window interactions and handles native window events.
///
/// Evaluates window borders and corners against incoming pointer events. When the pointer
/// hovers over a resize border, the appropriate hardware cursor icon is requested. When the
/// primary mouse button is pressed on a resize border, an interactive OS drag-resize operation
/// is triggered.
///
/// Tracks cursor state across window focus and HighDPI scale changes, guaranteeing that
/// resize cursors are cleanly restored to default when returning to the safe interior or leaving
/// window bounds, and preventing stale hover state during window maximization.
#[derive(Debug, Clone, PartialEq)]
pub struct FramelessWindowHost {
    /// Border grab margin thickness in logical points.
    pub border_thickness: f32,
    /// Corner diagonal grab radius/dimension in logical points.
    pub corner_size: f32,
    /// Master toggle enabling or disabling frameless edge detection and interactive resizing.
    pub enabled: bool,
    /// Active HighDPI scale factor for converting physical cursor coordinates to logical points.
    scale_factor: Cell<f64>,
    /// Last recorded cursor coordinate in logical points, or `None` if outside the window.
    cursor_pos: Cell<Option<Point>>,
    /// Tracks whether a resize cursor is actively applied on the window by this host.
    hovering_border: Cell<bool>,
}

impl Default for FramelessWindowHost {
    /// Constructs a `FramelessWindowHost` with default 6.0 pt border thickness and 12.0 pt corner size.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl FramelessWindowHost {
    /// Creates a new `FramelessWindowHost` with default settings and 1.0 HighDPI scale factor.
    ///
    /// Default border thickness is [`DEFAULT_BORDER_THICKNESS`] (6.0 pt) and corner size
    /// is [`DEFAULT_CORNER_SIZE`] (12.0 pt).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            border_thickness: DEFAULT_BORDER_THICKNESS,
            corner_size: DEFAULT_CORNER_SIZE,
            enabled: true,
            scale_factor: Cell::new(1.0),
            cursor_pos: Cell::new(None),
            hovering_border: Cell::new(false),
        }
    }

    /// Sets the border grab thickness in logical points.
    #[must_use]
    pub const fn with_border_thickness(mut self, border: f32) -> Self {
        self.border_thickness = border;
        self
    }

    /// Sets the corner grab size in logical points.
    #[must_use]
    pub const fn with_corner_size(mut self, corner: f32) -> Self {
        self.corner_size = corner;
        self
    }

    /// Sets whether frameless handling is enabled.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the initial HighDPI scale factor.
    ///
    /// If `scale_factor <= 0.0` or is non-finite (`f64::NAN`, `f64::INFINITY`), defaults to `1.0`
    /// to guard against zero, negative, or invalid scaling values.
    #[must_use]
    pub fn with_scale_factor(self, scale_factor: f64) -> Self {
        let factor = if scale_factor > 0.0 && scale_factor.is_finite() {
            scale_factor
        } else {
            1.0
        };
        self.scale_factor.set(factor);
        self
    }

    /// Returns the active HighDPI scale factor.
    #[must_use]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor.get()
    }

    /// Sets the active HighDPI scale factor.
    ///
    /// If `scale_factor <= 0.0` or is non-finite (`f64::NAN`, `f64::INFINITY`), defaults to `1.0`.
    pub fn set_scale_factor(&self, scale_factor: f64) {
        let factor = if scale_factor > 0.0 && scale_factor.is_finite() {
            scale_factor
        } else {
            1.0
        };
        self.scale_factor.set(factor);
    }

    /// Returns the last recorded cursor coordinate in logical points, if available.
    #[must_use]
    pub fn cursor_pos(&self) -> Option<Point> {
        self.cursor_pos.get()
    }

    /// Explicitly updates or clears the cached cursor coordinate in logical points.
    pub fn set_cursor_pos(&self, pos: Option<Point>) {
        if let Some(p) = pos {
            if p.x.is_finite() && p.y.is_finite() {
                self.cursor_pos.set(Some(p));
            } else {
                self.cursor_pos.set(None);
            }
        } else {
            self.cursor_pos.set(None);
        }
    }

    /// Returns whether the cursor is currently hovering over an interactive resize boundary.
    #[must_use]
    pub fn is_hovering_border(&self) -> bool {
        self.hovering_border.get()
    }

    /// Resets any active resize hover cursor back to default if this host previously modified it.
    pub fn reset_hover_cursor<W: WindowInterface>(&self, window: &mut W) {
        if self.hovering_border.replace(false) {
            window.set_cursor(CursorIcon::Default);
        }
    }

    /// Handles HighDPI scale factor changes, updating internal scale and resetting cursor state.
    ///
    /// Clears any cached cursor coordinates and cleanly restores the platform cursor to default
    /// if a resize border hover was active, preventing stale hover state across display boundaries.
    /// Clamps invalid or non-finite (`NaN`, `INFINITY`) scale factors to `1.0`.
    pub fn handle_scale_factor_changed<W: WindowInterface>(
        &self,
        window: &mut W,
        scale_factor: f64,
    ) {
        self.set_scale_factor(scale_factor);
        self.cursor_pos.set(None);
        self.reset_hover_cursor(window);
    }

    /// Handles a native Winit window event, updating cursor hover and initiating drag-resizing.
    ///
    /// Returns `true` if the event was consumed by the frameless resize system (e.g., hover
    /// cursor applied on a border or drag-resize initiated on primary click), or `false`
    /// if the event should continue downstream to regular UI widgets.
    ///
    /// # Behavior
    /// - Always updates internal HighDPI scale factor on [`WindowEvent::ScaleFactorChanged`],
    ///   clearing cached cursor position and resetting hover cursor to prevent stale coordinates.
    /// - Clears cached cursor coordinate and restores default cursor on [`WindowEvent::CursorLeft`]
    ///   and [`WindowEvent::Focused(false)`].
    /// - Converts physical cursor positions to logical space, rejecting non-finite coordinates.
    /// - When hovering over straight edges or diagonal corners, applies the directional cursor icon
    ///   and consumes the event (`true`).
    /// - Restores [`CursorIcon::Default`] cleanly when transitioning from an edge to the safe interior.
    /// - When the primary mouse button is pressed over a border, invokes [`WindowInterface::drag_resize_window`]
    ///   and consumes the event (`true`).
    /// - Suppresses resize hover and dragging when `!self.enabled` or `window.is_maximized()`.
    pub fn handle_window_event<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
        logical_size: Size,
    ) -> bool {
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.handle_scale_factor_changed(window, *scale_factor);
                false
            }
            WindowEvent::Focused(false) => {
                self.cursor_pos.set(None);
                self.reset_hover_cursor(window);
                false
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_pos.set(None);
                self.reset_hover_cursor(window);
                false
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !position.x.is_finite() || !position.y.is_finite() {
                    self.cursor_pos.set(None);
                    self.reset_hover_cursor(window);
                    return false;
                }

                let factor = self.scale_factor.get();
                let factor = if factor > 0.0 && factor.is_finite() {
                    factor
                } else {
                    1.0
                };
                let lx = (position.x / factor) as f32;
                let ly = (position.y / factor) as f32;
                if !lx.is_finite() || !ly.is_finite() {
                    self.cursor_pos.set(None);
                    self.reset_hover_cursor(window);
                    return false;
                }
                let logical_point = Point::new(lx, ly);
                self.cursor_pos.set(Some(logical_point));

                if !self.enabled || window.is_maximized() {
                    self.reset_hover_cursor(window);
                    return false;
                }

                if let Some(direction) = evaluate_resize_direction(
                    logical_point,
                    logical_size,
                    self.border_thickness,
                    self.corner_size,
                ) {
                    self.hovering_border.set(true);
                    window.set_cursor(direction.to_winit_cursor());
                    true
                } else {
                    self.reset_hover_cursor(window);
                    false
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if !self.enabled || window.is_maximized() {
                    return false;
                }

                if let Some(pos) = self.cursor_pos.get() {
                    if let Some(direction) = evaluate_resize_direction(
                        pos,
                        logical_size,
                        self.border_thickness,
                        self.corner_size,
                    ) {
                        self.hovering_border.set(true);
                        let _ = window.drag_resize_window(direction);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Handles an Iris UI [`UiEvent`], updating cursor hover and initiating drag-resizing.
    ///
    /// Accepts pre-normalized logical coordinates directly from the Iris event system,
    /// providing ergonomic compatibility when events have already been translated from raw Winit events.
    pub fn handle_ui_event<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
        logical_size: Size,
    ) -> bool {
        match event {
            UiEvent::MouseMove { point } => {
                if !point.x.is_finite() || !point.y.is_finite() {
                    self.cursor_pos.set(None);
                    self.reset_hover_cursor(window);
                    return false;
                }

                self.cursor_pos.set(Some(*point));

                if !self.enabled || window.is_maximized() {
                    self.reset_hover_cursor(window);
                    return false;
                }

                if let Some(direction) = evaluate_resize_direction(
                    *point,
                    logical_size,
                    self.border_thickness,
                    self.corner_size,
                ) {
                    self.hovering_border.set(true);
                    window.set_cursor(direction.to_winit_cursor());
                    true
                } else {
                    self.reset_hover_cursor(window);
                    false
                }
            }
            UiEvent::MouseDown {
                point,
                button: iris_core::event::MouseButton::Left,
            } => {
                if !point.x.is_finite() || !point.y.is_finite() {
                    self.cursor_pos.set(None);
                    self.reset_hover_cursor(window);
                    return false;
                }

                self.cursor_pos.set(Some(*point));

                if !self.enabled || window.is_maximized() {
                    return false;
                }

                if let Some(direction) = evaluate_resize_direction(
                    *point,
                    logical_size,
                    self.border_thickness,
                    self.corner_size,
                ) {
                    self.hovering_border.set(true);
                    let _ = window.drag_resize_window(direction);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Handles an Iris UI [`UiEvent`] using the window's own [`WindowInterface::logical_size`].
    ///
    /// Automatically queries the logical dimensions from `window` and forwards
    /// to [`Self::handle_ui_event`].
    #[inline]
    pub fn handle_ui_event_for_window<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
    ) -> bool {
        let size = window.logical_size();
        self.handle_ui_event(window, event, size)
    }

    /// Handles a native Winit [`WindowEvent`] using the window's own [`WindowInterface::logical_size`].
    ///
    /// Automatically queries the logical dimensions from `window` and forwards
    /// to [`Self::handle_window_event`].
    #[inline]
    pub fn handle_window_event_for_window<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
    ) -> bool {
        let size = window.logical_size();
        self.handle_window_event(window, event, size)
    }
}
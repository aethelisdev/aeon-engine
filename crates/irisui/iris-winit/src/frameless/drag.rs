// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Window Title Bar Dragging and Double-Click Maximize Host
//!
//! Provides the [`WindowDragHost`] event coordinator for frameless and custom-decorated desktop
//! application windows.
//!
//! Manages native window dragging gestures initiated from title bar surfaces and detects
//! rapid double-click events within strict spatial ($\Delta r < 4.0\text{px}$) and temporal
//! ($t \le 300\text{ms}$) thresholds to toggle window maximization state.
//!
//! Integrates directly with [`WindowInterface`], native Winit [`WindowEvent`]s, Iris UI [`UiEvent`]s,
//! and declarative [`TitleBarResponse`]s.

use super::window_trait::WindowInterface;
use iris_core::event::{MouseButton as UiMouseButton, UiEvent};
use iris_core::geometry::{Point, Rect, Size};
use iris_widgets::declarative::widget::{TitleBarAction, TitleBarResponse};
use std::cell::Cell;
use std::time::{Duration, Instant};
use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent};

/// Default time threshold (300 ms) within which two consecutive primary clicks register as a double-click.
pub const DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD: Duration = Duration::from_millis(300);

/// Default spatial tolerance (4.0 px) within which two consecutive primary clicks must land to register as a double-click ($\Delta r < 4.0\text{px}$).
pub const DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD: f32 = 4.0;

/// Calculates the Euclidean distance between two 2D points in logical points.
///
/// Returns [`f32::INFINITY`] if any coordinate component is non-finite (`NaN` or `INFINITY`).
///
/// # Arguments
/// * `p1` - First 2D point coordinate.
/// * `p2` - Second 2D point coordinate.
#[inline]
#[must_use]
pub fn point_distance(p1: Point, p2: Point) -> f32 {
    if !p1.x.is_finite() || !p1.y.is_finite() || !p2.x.is_finite() || !p2.y.is_finite() {
        return f32::INFINITY;
    }
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    (dx * dx + dy * dy).sqrt()
}

/// Evaluates whether two consecutive click events satisfy the spatial and temporal
/// constraints of a double-click gesture.
///
/// Returns `true` if `t2 >= t1`, `t2.duration_since(t1) <= time_threshold`, and
/// `point_distance(p1, p2) < distance_threshold`.
///
/// # Arguments
/// * `p1` - Coordinates of the first click.
/// * `t1` - Timestamp of the first click.
/// * `p2` - Coordinates of the second click.
/// * `t2` - Timestamp of the second click.
/// * `time_threshold` - Maximum allowed duration between clicks.
/// * `distance_threshold` - Maximum allowed Euclidean distance in logical points.
#[inline]
#[must_use]
pub fn is_double_click(
    p1: Point,
    t1: Instant,
    p2: Point,
    t2: Instant,
    time_threshold: Duration,
    distance_threshold: f32,
) -> bool {
    if t2 < t1 {
        return false;
    }
    let elapsed = t2.duration_since(t1);
    if elapsed > time_threshold {
        return false;
    }
    let dist = point_distance(p1, p2);
    dist < distance_threshold
}

/// Outcome of evaluating an interactive window title bar interaction.
///
/// Returned by [`WindowDragHost`] inspection and dispatch routines to inform callers
/// of the exact platform window command that was triggered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowDragOutcome {
    /// No window management or dragging action was taken (e.g., event was outside the drag area or host was disabled).
    None,
    /// Single click initiated native window dragging via [`WindowInterface::drag_window`].
    DragStarted,
    /// Double-click or maximize button toggled window maximize/restore state via [`WindowInterface::toggle_maximize`].
    MaximizeToggled,
    /// Minimize button triggered window minimization via [`WindowInterface::minimize`].
    Minimized,
    /// Close button requested window closure via [`WindowInterface::close`].
    Closed,
}

impl WindowDragOutcome {
    /// Returns `true` if an interactive window operation was initiated.
    #[inline]
    #[must_use]
    pub const fn is_consumed(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns `true` if native window dragging was initiated.
    #[inline]
    #[must_use]
    pub const fn is_drag_started(self) -> bool {
        matches!(self, Self::DragStarted)
    }

    /// Returns `true` if window maximize/restore state was toggled.
    #[inline]
    #[must_use]
    pub const fn is_maximize_toggled(self) -> bool {
        matches!(self, Self::MaximizeToggled)
    }

    /// Returns `true` if window minimization was requested.
    #[inline]
    #[must_use]
    pub const fn is_minimized(self) -> bool {
        matches!(self, Self::Minimized)
    }

    /// Returns `true` if window closure was requested.
    #[inline]
    #[must_use]
    pub const fn is_closed(self) -> bool {
        matches!(self, Self::Closed)
    }
}

/// Coordinates title bar window dragging and double-click maximize detection.
///
/// Intercepts mouse clicks on draggable title bar surfaces, executing [`WindowInterface::drag_window`]
/// on single primary mouse down. If a subsequent primary click occurs within [`DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD`]
/// (300 ms) and within [`DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD`] ($\Delta r < 4.0\text{px}$),
/// [`WindowInterface::toggle_maximize`] is triggered autonomously.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowDragHost {
    /// Master toggle enabling or disabling title bar window dragging and maximize toggling.
    pub enabled: bool,
    /// Maximum time elapsed between two consecutive clicks to register as a double-click.
    pub double_click_time_threshold: Duration,
    /// Maximum spatial distance between two consecutive clicks in logical points to register as a double-click.
    pub double_click_distance_threshold: f32,
    /// Active HighDPI scale factor for converting physical cursor coordinates to logical points.
    scale_factor: Cell<f64>,
    /// Last recorded cursor coordinate in logical points, or `None` if outside the window.
    cursor_pos: Cell<Option<Point>>,
    /// Timestamp and coordinate of previous primary click for double-click maximize evaluation.
    last_click: Cell<Option<(Point, Instant)>>,
    /// Draggable bounding rectangle in logical points, if statically specified.
    drag_rect: Cell<Option<Rect>>,
    /// Custom title bar height in logical points (from top of window: `0.0..title_bar_height`), if configured.
    title_bar_height: Cell<Option<f32>>,
}

impl Default for WindowDragHost {
    /// Constructs a `WindowDragHost` with default settings (300 ms time threshold, 4.0 px distance threshold).
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl WindowDragHost {
    /// Creates a new `WindowDragHost` with default 300 ms time threshold, 4.0 px distance threshold, and 1.0 HighDPI scale factor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            enabled: true,
            double_click_time_threshold: DEFAULT_DOUBLE_CLICK_TIME_THRESHOLD,
            double_click_distance_threshold: DEFAULT_DOUBLE_CLICK_DISTANCE_THRESHOLD,
            scale_factor: Cell::new(1.0),
            cursor_pos: Cell::new(None),
            last_click: Cell::new(None),
            drag_rect: Cell::new(None),
            title_bar_height: Cell::new(None),
        }
    }

    /// Sets whether title bar dragging and maximize detection is enabled.
    #[must_use]
    pub const fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets the double-click time threshold duration.
    #[must_use]
    pub const fn with_double_click_time(mut self, duration: Duration) -> Self {
        self.double_click_time_threshold = duration;
        self
    }

    /// Sets the double-click spatial distance threshold in logical points.
    #[must_use]
    pub const fn with_double_click_distance(mut self, distance: f32) -> Self {
        self.double_click_distance_threshold = distance;
        self
    }

    /// Sets the static draggable bounding rectangle in logical points.
    #[must_use]
    pub fn with_drag_rect(self, rect: Rect) -> Self {
        self.drag_rect.set(Some(rect));
        self
    }

    /// Sets the title bar height in logical points extending from the top edge (`0.0..height`).
    #[must_use]
    pub fn with_title_bar_height(self, height: f32) -> Self {
        if height > 0.0 && height.is_finite() {
            self.title_bar_height.set(Some(height));
        } else {
            self.title_bar_height.set(None);
        }
        self
    }

    /// Sets the initial HighDPI scale factor.
    ///
    /// Falls back to `1.0` if `scale_factor <= 0.0` or is non-finite (`NaN`, `INFINITY`).
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

    /// Updates the active HighDPI scale factor.
    ///
    /// Falls back to `1.0` if `scale_factor <= 0.0` or is non-finite.
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

    /// Explicitly sets or clears the cached cursor coordinate in logical points.
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

    /// Returns the timestamp and coordinate of the last recorded primary click, if any.
    #[must_use]
    pub fn last_click(&self) -> Option<(Point, Instant)> {
        self.last_click.get()
    }

    /// Returns the configured static draggable bounding rectangle, if any.
    #[must_use]
    pub fn drag_rect(&self) -> Option<Rect> {
        self.drag_rect.get()
    }

    /// Sets or clears the static draggable bounding rectangle in logical points.
    pub fn set_drag_rect(&self, rect: Option<Rect>) {
        self.drag_rect.set(rect);
    }

    /// Returns the configured title bar height, if any.
    #[must_use]
    pub fn title_bar_height(&self) -> Option<f32> {
        self.title_bar_height.get()
    }

    /// Sets or clears the title bar height extending from the top edge (`0.0..height`).
    pub fn set_title_bar_height(&self, height: Option<f32>) {
        if let Some(h) = height {
            if h > 0.0 && h.is_finite() {
                self.title_bar_height.set(Some(h));
            } else {
                self.title_bar_height.set(None);
            }
        } else {
            self.title_bar_height.set(None);
        }
    }

    /// Resets all interaction tracking state, clearing cached cursor position and pending double-click history.
    ///
    /// Recommended to call when window loses focus or cursor leaves the window bounds.
    pub fn reset(&self) {
        self.cursor_pos.set(None);
        self.last_click.set(None);
    }

    /// Checks whether a logical point falls within the configured drag area.
    ///
    /// Evaluates against [`Self::drag_rect`] first, then against [`Self::title_bar_height`].
    #[must_use]
    pub fn is_point_in_drag_area(&self, point: Point, logical_size: Option<Size>) -> bool {
        if !point.x.is_finite() || !point.y.is_finite() {
            return false;
        }

        if let Some(rect) = self.drag_rect.get()
            && rect.contains_point(point)
        {
            return true;
        }

        if let Some(height) = self.title_bar_height.get()
            && point.y >= 0.0
            && point.y <= height
        {
            if let Some(size) = logical_size {
                if point.x >= 0.0 && point.x <= size.width {
                    return true;
                }
            } else if point.x >= 0.0 {
                return true;
            }
        }

        false
    }

    /// Evaluates an interactive click at the specified point with an explicit timestamp.
    ///
    /// If the click lands within the double-click threshold ($\Delta r < \text{distance\_threshold}$
    /// and $\Delta t \le \text{time\_threshold}$), triggers [`WindowInterface::toggle_maximize`]
    /// and resets the click state. Otherwise, updates the last click record and invokes
    /// [`WindowInterface::drag_window`].
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `point` - Coordinates of the click in logical points.
    /// * `now` - Explicit timestamp of the click event (enables deterministic headless testing).
    pub fn process_click_at<W: WindowInterface>(
        &self,
        window: &mut W,
        point: Point,
        now: Instant,
    ) -> WindowDragOutcome {
        if !self.enabled || !point.x.is_finite() || !point.y.is_finite() {
            return WindowDragOutcome::None;
        }

        if let Some((last_point, last_time)) = self.last_click.get()
            && is_double_click(
                last_point,
                last_time,
                point,
                now,
                self.double_click_time_threshold,
                self.double_click_distance_threshold,
            )
        {
            self.last_click.set(None);
            window.toggle_maximize();
            return WindowDragOutcome::MaximizeToggled;
        }

        self.last_click.set(Some((point, now)));
        let _ = window.drag_window();
        WindowDragOutcome::DragStarted
    }

    /// Evaluates an interactive click at the specified point using the current system wall-clock time ([`Instant::now`]).
    #[inline]
    pub fn process_click<W: WindowInterface>(
        &self,
        window: &mut W,
        point: Point,
    ) -> WindowDragOutcome {
        self.process_click_at(window, point, Instant::now())
    }

    /// Evaluates an interactive click at the specified point, returning `true` if consumed.
    #[inline]
    pub fn handle_click_at<W: WindowInterface>(
        &self,
        window: &mut W,
        point: Point,
        now: Instant,
    ) -> bool {
        self.process_click_at(window, point, now).is_consumed()
    }

    /// Evaluates an interactive click at the specified point using [`Instant::now`], returning `true` if consumed.
    #[inline]
    pub fn handle_click<W: WindowInterface>(&self, window: &mut W, point: Point) -> bool {
        self.process_click(window, point).is_consumed()
    }

    /// Dispatches a declarative [`TitleBarAction`] directly to the platform [`WindowInterface`].
    ///
    /// Automatically resets pending double-click state and executes [`WindowInterface::minimize`],
    /// [`WindowInterface::toggle_maximize`], or [`WindowInterface::close`].
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `action` - Title bar control button action to execute.
    pub fn handle_title_bar_action<W: WindowInterface>(
        &self,
        window: &mut W,
        action: TitleBarAction,
    ) -> WindowDragOutcome {
        if !self.enabled {
            return WindowDragOutcome::None;
        }

        self.last_click.set(None);
        match action {
            TitleBarAction::Minimize => {
                window.minimize();
                WindowDragOutcome::Minimized
            }
            TitleBarAction::Maximize => {
                window.toggle_maximize();
                WindowDragOutcome::MaximizeToggled
            }
            TitleBarAction::Close => {
                window.close();
                WindowDragOutcome::Closed
            }
        }
    }

    /// Evaluates an autonomous declarative [`TitleBarResponse`] with an explicit timestamp.
    ///
    /// Automatically routes control button actions ([`TitleBarAction::Minimize`],
    /// [`TitleBarAction::Maximize`], [`TitleBarAction::Close`]) to their corresponding
    /// [`WindowInterface`] handlers.
    ///
    /// If the draggable title bar surface was clicked ([`TitleBarResponse::drag_area_clicked`]),
    /// invokes [`Self::process_click_at`] using the interaction pointer position (or bounding
    /// box origin if pointer position is unavailable).
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `response` - Evaluated declarative response from a [`TitleBar`] widget.
    /// * `now` - Explicit timestamp of the interaction pass.
    pub fn process_title_bar_response_at<W: WindowInterface>(
        &self,
        window: &mut W,
        response: &TitleBarResponse,
        now: Instant,
    ) -> WindowDragOutcome {
        if !self.enabled {
            return WindowDragOutcome::None;
        }

        if let Some(action) = response.action() {
            return self.handle_title_bar_action(window, action);
        }

        if response.drag_area_clicked() {
            let point = response.pointer_pos.unwrap_or(response.rect.origin());
            return self.process_click_at(window, point, now);
        }

        WindowDragOutcome::None
    }

    /// Evaluates an autonomous declarative [`TitleBarResponse`] using current system wall-clock time ([`Instant::now`]).
    #[inline]
    pub fn process_title_bar_response<W: WindowInterface>(
        &self,
        window: &mut W,
        response: &TitleBarResponse,
    ) -> WindowDragOutcome {
        self.process_title_bar_response_at(window, response, Instant::now())
    }

    /// Evaluates an autonomous declarative [`TitleBarResponse`], returning `true` if an operation was initiated.
    #[inline]
    pub fn handle_title_bar_response<W: WindowInterface>(
        &self,
        window: &mut W,
        response: &TitleBarResponse,
    ) -> bool {
        self.process_title_bar_response(window, response)
            .is_consumed()
    }

    /// Evaluates a native Winit [`WindowEvent`] against an explicit draggable boundary rectangle with an explicit timestamp.
    ///
    /// Tracks cursor coordinates, scale factor updates, and window focus loss. When primary
    /// mouse button is pressed inside `drag_rect`, initiates dragging or double-click maximize toggle.
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `event` - Native Winit window event.
    /// * `drag_rect` - Explicit draggable bounding rectangle in logical points.
    /// * `now` - Explicit timestamp of the event.
    pub fn process_window_event_at<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
        drag_rect: Rect,
        now: Instant,
    ) -> WindowDragOutcome {
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.set_scale_factor(*scale_factor);
                self.cursor_pos.set(None);
                WindowDragOutcome::None
            }
            WindowEvent::Focused(false) | WindowEvent::CursorLeft { .. } => {
                self.reset();
                WindowDragOutcome::None
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !position.x.is_finite() || !position.y.is_finite() {
                    self.cursor_pos.set(None);
                    return WindowDragOutcome::None;
                }
                let factor = self.scale_factor.get();
                let factor = if factor > 0.0 && factor.is_finite() {
                    factor
                } else {
                    1.0
                };
                let lx = (position.x / factor) as f32;
                let ly = (position.y / factor) as f32;
                if lx.is_finite() && ly.is_finite() {
                    self.cursor_pos.set(Some(Point::new(lx, ly)));
                } else {
                    self.cursor_pos.set(None);
                }
                WindowDragOutcome::None
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                if !self.enabled {
                    return WindowDragOutcome::None;
                }
                if let Some(pos) = self.cursor_pos.get()
                    && drag_rect.contains_point(pos)
                {
                    return self.process_click_at(window, pos, now);
                }
                WindowDragOutcome::None
            }
            _ => WindowDragOutcome::None,
        }
    }

    /// Evaluates a native Winit [`WindowEvent`] against an explicit draggable boundary rectangle using [`Instant::now`].
    #[inline]
    pub fn process_window_event<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
        drag_rect: Rect,
    ) -> WindowDragOutcome {
        self.process_window_event_at(window, event, drag_rect, Instant::now())
    }

    /// Evaluates a native Winit [`WindowEvent`] against an explicit draggable boundary rectangle, returning `true` if consumed.
    #[inline]
    pub fn handle_window_event<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
        drag_rect: Rect,
    ) -> bool {
        self.process_window_event(window, event, drag_rect)
            .is_consumed()
    }

    /// Evaluates a native Winit [`WindowEvent`] against a custom draggable hit-test predicate with an explicit timestamp.
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `event` - Native Winit window event.
    /// * `is_draggable` - Predicate function returning `true` if a given point is in the draggable region.
    /// * `now` - Explicit timestamp of the event.
    pub fn process_window_event_with_area<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
        is_draggable: impl Fn(Point) -> bool,
        now: Instant,
    ) -> WindowDragOutcome {
        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.set_scale_factor(*scale_factor);
                self.cursor_pos.set(None);
                WindowDragOutcome::None
            }
            WindowEvent::Focused(false) | WindowEvent::CursorLeft { .. } => {
                self.reset();
                WindowDragOutcome::None
            }
            WindowEvent::CursorMoved { position, .. } => {
                if !position.x.is_finite() || !position.y.is_finite() {
                    self.cursor_pos.set(None);
                    return WindowDragOutcome::None;
                }
                let factor = self.scale_factor.get();
                let factor = if factor > 0.0 && factor.is_finite() {
                    factor
                } else {
                    1.0
                };
                let lx = (position.x / factor) as f32;
                let ly = (position.y / factor) as f32;
                if lx.is_finite() && ly.is_finite() {
                    self.cursor_pos.set(Some(Point::new(lx, ly)));
                } else {
                    self.cursor_pos.set(None);
                }
                WindowDragOutcome::None
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: WinitMouseButton::Left,
                ..
            } => {
                if !self.enabled {
                    return WindowDragOutcome::None;
                }
                if let Some(pos) = self.cursor_pos.get()
                    && is_draggable(pos)
                {
                    return self.process_click_at(window, pos, now);
                }
                WindowDragOutcome::None
            }

            _ => WindowDragOutcome::None,
        }
    }

    /// Evaluates a native Winit [`WindowEvent`] for the window, using configured [`Self::drag_rect`] or [`Self::title_bar_height`].
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `event` - Native Winit window event.
    /// * `now` - Explicit timestamp of the event.
    pub fn process_window_event_for_window_at<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
        now: Instant,
    ) -> WindowDragOutcome {
        let size = window.logical_size();
        self.process_window_event_with_area(
            window,
            event,
            |p| self.is_point_in_drag_area(p, Some(size)),
            now,
        )
    }

    /// Evaluates a native Winit [`WindowEvent`] for the window using configured drag parameters and [`Instant::now`].
    #[inline]
    pub fn process_window_event_for_window<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
    ) -> WindowDragOutcome {
        self.process_window_event_for_window_at(window, event, Instant::now())
    }

    /// Evaluates a native Winit [`WindowEvent`] for the window using configured drag parameters, returning `true` if consumed.
    #[inline]
    pub fn handle_window_event_for_window<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &WindowEvent,
    ) -> bool {
        self.process_window_event_for_window(window, event)
            .is_consumed()
    }

    /// Evaluates an Iris UI [`UiEvent`] against an explicit draggable boundary rectangle with an explicit timestamp.
    ///
    /// Accepts pre-normalized logical coordinates directly from Iris UI input pipelines.
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `event` - Normalized Iris UI interaction event.
    /// * `drag_rect` - Draggable bounding rectangle in logical points.
    /// * `now` - Explicit timestamp of the event.
    pub fn process_ui_event_at<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
        drag_rect: Rect,
        now: Instant,
    ) -> WindowDragOutcome {
        match event {
            UiEvent::MouseMove { point } => {
                if point.x.is_finite() && point.y.is_finite() {
                    self.cursor_pos.set(Some(*point));
                } else {
                    self.cursor_pos.set(None);
                }
                WindowDragOutcome::None
            }
            UiEvent::MouseDown {
                point,
                button: UiMouseButton::Left,
            } => {
                if !self.enabled || !point.x.is_finite() || !point.y.is_finite() {
                    return WindowDragOutcome::None;
                }
                self.cursor_pos.set(Some(*point));
                if drag_rect.contains_point(*point) {
                    return self.process_click_at(window, *point, now);
                }
                WindowDragOutcome::None
            }
            _ => WindowDragOutcome::None,
        }
    }

    /// Evaluates an Iris UI [`UiEvent`] against an explicit draggable boundary rectangle using [`Instant::now`].
    #[inline]
    pub fn process_ui_event<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
        drag_rect: Rect,
    ) -> WindowDragOutcome {
        self.process_ui_event_at(window, event, drag_rect, Instant::now())
    }

    /// Evaluates an Iris UI [`UiEvent`] against an explicit draggable boundary rectangle, returning `true` if consumed.
    #[inline]
    pub fn handle_ui_event<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
        drag_rect: Rect,
    ) -> bool {
        self.process_ui_event(window, event, drag_rect)
            .is_consumed()
    }

    /// Evaluates an Iris UI [`UiEvent`] against a custom draggable hit-test predicate with an explicit timestamp.
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `event` - Normalized Iris UI interaction event.
    /// * `is_draggable` - Predicate function returning `true` if a given point is in the draggable region.
    /// * `now` - Explicit timestamp of the event.
    pub fn process_ui_event_with_area<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
        is_draggable: impl Fn(Point) -> bool,
        now: Instant,
    ) -> WindowDragOutcome {
        match event {
            UiEvent::MouseMove { point } => {
                if point.x.is_finite() && point.y.is_finite() {
                    self.cursor_pos.set(Some(*point));
                } else {
                    self.cursor_pos.set(None);
                }
                WindowDragOutcome::None
            }
            UiEvent::MouseDown {
                point,
                button: UiMouseButton::Left,
            } => {
                if !self.enabled || !point.x.is_finite() || !point.y.is_finite() {
                    return WindowDragOutcome::None;
                }
                self.cursor_pos.set(Some(*point));
                if is_draggable(*point) {
                    return self.process_click_at(window, *point, now);
                }
                WindowDragOutcome::None
            }
            _ => WindowDragOutcome::None,
        }
    }

    /// Evaluates an Iris UI [`UiEvent`] for the window, using configured [`Self::drag_rect`] or [`Self::title_bar_height`].
    ///
    /// # Arguments
    /// * `window` - Target platform window implementation.
    /// * `event` - Normalized Iris UI interaction event.
    /// * `now` - Explicit timestamp of the event.
    pub fn process_ui_event_for_window_at<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
        now: Instant,
    ) -> WindowDragOutcome {
        let size = window.logical_size();
        self.process_ui_event_with_area(
            window,
            event,
            |p| self.is_point_in_drag_area(p, Some(size)),
            now,
        )
    }

    /// Evaluates an Iris UI [`UiEvent`] for the window using configured drag parameters and [`Instant::now`].
    #[inline]
    pub fn process_ui_event_for_window<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
    ) -> WindowDragOutcome {
        self.process_ui_event_for_window_at(window, event, Instant::now())
    }

    /// Evaluates an Iris UI [`UiEvent`] for the window using configured drag parameters, returning `true` if consumed.
    #[inline]
    pub fn handle_ui_event_for_window<W: WindowInterface>(
        &self,
        window: &mut W,
        event: &UiEvent,
    ) -> bool {
        self.process_ui_event_for_window(window, event)
            .is_consumed()
    }
}
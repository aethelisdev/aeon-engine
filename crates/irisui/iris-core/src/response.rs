// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative Widget Interaction Response (`iris-core::response`)
//!
//! Provides the canonical [`WidgetResponse`] and [`ResponseFlags`] types representing
//! immediate-mode interaction outcomes (clicks, drags, hover, focus transitions, value changes)
//! emitted by declarative UI scopes and retained UI tree event dispatching.
//!

use bitflags::bitflags;

use crate::geometry::{Point, Rect};
use crate::id::WidgetId;

bitflags! {
    /// Fine-grained bitflags representing user interaction states on an emitted widget.
    ///
    /// Evaluated during tree hit-testing and event bubbling passes, enabling immediate-mode
    /// querying like `if response.clicked() { ... }` or `if response.changed() { ... }`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    pub struct ResponseFlags: u16 {
        /// Pointer cursor is physically within the visible bounding rectangle of the widget.
        const HOVERED = 1 << 0;
        /// Primary mouse button is actively held down over the widget.
        const PRESSED = 1 << 1;
        /// Primary mouse button was pressed and released within the same widget bounds during this frame.
        const CLICKED = 1 << 2;
        /// Rapid successive click detected within double-click duration threshold.
        const DOUBLE_CLICKED = 1 << 3;
        /// Secondary (right-click) mouse button was pressed on the widget (context menu trigger).
        const SECONDARY_CLICKED = 1 << 4;
        /// Middle mouse button click detected (e.g. close tab gesture).
        const MIDDLE_CLICKED = 1 << 5;
        /// Active pointer drag gesture is occurring over or captured by this widget.
        const DRAGGED = 1 << 6;
        /// The widget's bound value or text model was mutated during this interaction pass.
        const CHANGED = 1 << 7;
        /// Widget currently holds active keyboard navigation or text input focus.
        const FOCUSED = 1 << 8;
        /// Widget transitioned into focused state during this frame.
        const GAINED_FOCUS = 1 << 9;
        /// Widget transitioned out of focused state during this frame.
        const LOST_FOCUS = 1 << 10;
        /// Active pointer drag gesture was released during this frame (drop trigger).
        const DRAG_RELEASED = 1 << 11;
    }
}

/// Interaction response returned by declarative widget emissions in Iris UI.
///
/// Designed to emulate immediate-mode ergonomics (`if ui.button("Save").clicked() { ... }`)
/// while remaining backed by the high-performance generational arena of `iris-core`.
///
/// Stack-allocated, lightweight `Copy` struct with zero heap allocation overhead.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WidgetResponse {
    /// Unique generational arena identifier of the emitted widget node.
    pub id: WidgetId,
    /// Absolute layout bounding box of the widget in screen pixels.
    pub rect: Rect,
    /// Detailed interaction bitflags representing the single source of truth (SSOT).
    pub flags: ResponseFlags,
    /// Absolute cursor screen position when this interaction occurred, if captured.
    pub pointer_pos: Option<Point>,
    /// Continuous displacement delta vector if an active dragging gesture is in progress.
    pub drag_delta: Option<Point>,
}

impl WidgetResponse {
    /// Constructs a basic widget response with direct state flags.
    #[inline]
    pub const fn new(id: WidgetId, clicked: bool, hovered: bool, changed: bool) -> Self {
        let mut flags = ResponseFlags::empty();
        if clicked {
            flags = flags.union(ResponseFlags::CLICKED);
        }
        if hovered {
            flags = flags.union(ResponseFlags::HOVERED);
        }
        if changed {
            flags = flags.union(ResponseFlags::CHANGED);
        }

        Self {
            id,
            rect: Rect::ZERO,
            flags,
            pointer_pos: None,
            drag_delta: None,
        }
    }

    /// Constructs a rich widget response with explicit bounding box and interaction flags.
    #[inline]
    pub const fn from_flags(id: WidgetId, rect: Rect, flags: ResponseFlags) -> Self {
        Self {
            id,
            rect,
            flags,
            pointer_pos: None,
            drag_delta: None,
        }
    }

    /// Constructs an un-interacted empty response for a widget node.
    #[inline]
    pub const fn empty(id: WidgetId) -> Self {
        Self {
            id,
            rect: Rect::ZERO,
            flags: ResponseFlags::empty(),
            pointer_pos: None,
            drag_delta: None,
        }
    }

    /// Returns the unique generational identifier of the widget node.
    #[inline]
    pub const fn id(&self) -> WidgetId {
        self.id
    }

    /// Attaches the cursor screen position recorded during this interaction.
    #[inline]
    pub const fn with_pointer_pos(mut self, pos: Option<Point>) -> Self {
        self.pointer_pos = pos;
        self
    }

    /// Returns the absolute screen position of the cursor when this interaction occurred, if any.
    ///
    /// Useful for anchoring context menus, popups, and dragging gestures without manual state tracking.
    #[inline]
    pub const fn interact_pointer_pos(&self) -> Option<Point> {
        self.pointer_pos
    }

    /// Returns the active dragging displacement delta vector, if any.
    #[inline]
    pub const fn drag_delta(&self) -> Option<Point> {
        self.drag_delta
    }

    /// Sets the dragging displacement delta vector and returns updated response.
    #[inline]
    pub const fn with_drag_delta(mut self, delta: Option<Point>) -> Self {
        self.drag_delta = delta;
        self
    }

    /// Returns the absolute layout bounding box of the widget.
    #[inline]
    pub const fn rect(&self) -> Rect {
        self.rect
    }

    /// Sets the layout bounding box and returns updated response.
    #[inline]
    pub const fn with_rect(mut self, rect: Rect) -> Self {
        self.rect = rect;
        self
    }

    /// Returns `true` if primary mouse button clicked the widget during this frame.
    #[inline]
    pub const fn clicked(&self) -> bool {
        self.flags.contains(ResponseFlags::CLICKED)
    }

    /// Sets the clicked state flag and returns updated response.
    #[inline]
    pub const fn with_clicked(mut self, clicked: bool) -> Self {
        if clicked {
            self.flags = self.flags.union(ResponseFlags::CLICKED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::CLICKED);
        }
        self
    }

    /// Returns `true` if double-click was detected on the widget during this frame.
    #[inline]
    pub const fn double_clicked(&self) -> bool {
        self.flags.contains(ResponseFlags::DOUBLE_CLICKED)
    }

    /// Sets the double-clicked state flag and returns updated response.
    #[inline]
    pub const fn with_double_clicked(mut self, double_clicked: bool) -> Self {
        if double_clicked {
            self.flags = self.flags.union(ResponseFlags::DOUBLE_CLICKED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::DOUBLE_CLICKED);
        }
        self
    }

    /// Returns `true` if secondary (right-click) occurred on this widget.
    #[inline]
    pub const fn secondary_clicked(&self) -> bool {
        self.flags.contains(ResponseFlags::SECONDARY_CLICKED)
    }

    /// Sets the secondary clicked state flag and returns updated response.
    #[inline]
    pub const fn with_secondary_clicked(mut self, secondary_clicked: bool) -> Self {
        if secondary_clicked {
            self.flags = self.flags.union(ResponseFlags::SECONDARY_CLICKED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::SECONDARY_CLICKED);
        }
        self
    }

    /// Returns `true` if middle mouse button (wheel click) clicked this widget.
    ///
    /// Useful for tab-close gestures and background open actions.
    #[inline]
    pub const fn middle_clicked(&self) -> bool {
        self.flags.contains(ResponseFlags::MIDDLE_CLICKED)
    }

    /// Sets the middle clicked state flag and returns updated response.
    #[inline]
    pub const fn with_middle_clicked(mut self, middle_clicked: bool) -> Self {
        if middle_clicked {
            self.flags = self.flags.union(ResponseFlags::MIDDLE_CLICKED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::MIDDLE_CLICKED);
        }
        self
    }

    /// Returns `true` if the pointer is currently hovering over the widget.
    #[inline]
    pub const fn hovered(&self) -> bool {
        self.flags.contains(ResponseFlags::HOVERED)
    }

    /// Sets the hovered state flag and returns updated response.
    #[inline]
    pub const fn with_hovered(mut self, hovered: bool) -> Self {
        if hovered {
            self.flags = self.flags.union(ResponseFlags::HOVERED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::HOVERED);
        }
        self
    }

    /// Returns `true` if primary mouse button is actively held down over this widget.
    #[inline]
    pub const fn pressed(&self) -> bool {
        self.flags.contains(ResponseFlags::PRESSED)
    }

    /// Sets the pressed state flag and returns updated response.
    #[inline]
    pub const fn with_pressed(mut self, pressed: bool) -> Self {
        if pressed {
            self.flags = self.flags.union(ResponseFlags::PRESSED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::PRESSED);
        }
        self
    }

    /// Returns `true` if the underlying model property was mutated during this frame.
    #[inline]
    pub const fn changed(&self) -> bool {
        self.flags.contains(ResponseFlags::CHANGED)
    }

    /// Sets the changed state flag and returns updated response.
    #[inline]
    pub const fn with_changed(mut self, changed: bool) -> Self {
        if changed {
            self.flags = self.flags.union(ResponseFlags::CHANGED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::CHANGED);
        }
        self
    }

    /// Returns `true` if an active drag gesture is occurring on this widget.
    #[inline]
    pub const fn dragged(&self) -> bool {
        self.flags.contains(ResponseFlags::DRAGGED)
    }

    /// Sets the dragged state flag and returns updated response.
    #[inline]
    pub const fn with_dragged(mut self, dragged: bool) -> Self {
        if dragged {
            self.flags = self.flags.union(ResponseFlags::DRAGGED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::DRAGGED);
        }
        self
    }

    /// Returns `true` if an active drag gesture was released during this frame on this widget.
    #[inline]
    pub const fn drag_released(&self) -> bool {
        self.flags.contains(ResponseFlags::DRAG_RELEASED)
    }

    /// Sets the drag released state flag and returns updated response.
    #[inline]
    pub const fn with_drag_released(mut self, released: bool) -> Self {
        if released {
            self.flags = self.flags.union(ResponseFlags::DRAG_RELEASED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::DRAG_RELEASED);
        }
        self
    }

    /// Returns `true` if the widget currently holds keyboard/input focus.
    #[inline]
    pub const fn is_focused(&self) -> bool {
        self.flags.contains(ResponseFlags::FOCUSED)
    }

    /// Returns `true` if the widget currently holds keyboard/input focus.
    ///
    /// Alias for [`Self::is_focused`] conforming to standard fluent property query conventions.
    #[inline]
    pub const fn focused(&self) -> bool {
        self.is_focused()
    }

    /// Sets the focused state flag and returns updated response.
    #[inline]
    pub const fn with_focused(mut self, focused: bool) -> Self {
        if focused {
            self.flags = self.flags.union(ResponseFlags::FOCUSED);
        } else {
            self.flags = self.flags.difference(ResponseFlags::FOCUSED);
        }
        self
    }

    /// Returns `true` if the widget gained focus during this interaction pass.
    #[inline]
    pub const fn gained_focus(&self) -> bool {
        self.flags.contains(ResponseFlags::GAINED_FOCUS)
    }

    /// Sets the gained_focus state flag and returns updated response.
    #[inline]
    pub const fn with_gained_focus(mut self, gained: bool) -> Self {
        if gained {
            self.flags = self.flags.union(ResponseFlags::GAINED_FOCUS);
        } else {
            self.flags = self.flags.difference(ResponseFlags::GAINED_FOCUS);
        }
        self
    }

    /// Returns `true` if the widget lost focus during this interaction pass.
    #[inline]
    pub const fn lost_focus(&self) -> bool {
        self.flags.contains(ResponseFlags::LOST_FOCUS)
    }

    /// Sets the lost_focus state flag and returns updated response.
    #[inline]
    pub const fn with_lost_focus(mut self, lost: bool) -> Self {
        if lost {
            self.flags = self.flags.union(ResponseFlags::LOST_FOCUS);
        } else {
            self.flags = self.flags.difference(ResponseFlags::LOST_FOCUS);
        }
        self
    }

    /// Executes the provided closure if this widget was clicked with the primary mouse button
    /// without being a double-click or active drag gesture.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_click(self, f: impl FnOnce()) -> Self {
        if self.clicked() && !self.double_clicked() && !self.dragged() {
            f();
        }
        self
    }

    /// Executes the provided closure if this widget was clicked with the secondary (right) mouse button
    /// without an active drag gesture.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_secondary_click(self, f: impl FnOnce()) -> Self {
        if self.secondary_clicked() && !self.dragged() {
            f();
        }
        self
    }

    /// Executes the provided closure if this widget was clicked with either primary or secondary mouse button
    /// without being a double-click or active drag gesture.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_any_click(self, f: impl FnOnce()) -> Self {
        if (self.secondary_clicked() || self.clicked()) && !self.double_clicked() && !self.dragged()
        {
            f();
        }
        self
    }

    /// Executes the provided closure if this widget was double-clicked.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_double_click(self, f: impl FnOnce()) -> Self {
        if self.double_clicked() && !self.dragged() {
            f();
        }
        self
    }

    /// Executes the provided closure if an active drag gesture is occurring on this widget.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_drag(self, f: impl FnOnce()) -> Self {
        if self.dragged() {
            f();
        }
        self
    }

    /// Executes the provided closure if an active drag gesture was released during this frame on this widget.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_drag_release(self, f: impl FnOnce()) -> Self {
        if self.drag_released() {
            f();
        }
        self
    }

    /// Executes the provided closure if this widget's value or interactive state changed.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_change(self, f: impl FnOnce()) -> Self {
        if self.changed() {
            f();
        }
        self
    }

    /// Executes the provided closure if this widget was hovered during this frame.
    ///
    /// Returns `self` unmodified to support fluent declarative method chaining without intermediate `let` bindings.
    #[inline]
    pub fn on_hover(self, f: impl FnOnce()) -> Self {
        if self.hovered() {
            f();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_response_flags_and_helpers() {
        let id = WidgetId::default();
        let rect = Rect::new(10.0, 20.0, 100.0, 30.0);
        let resp = WidgetResponse::from_flags(
            id,
            rect,
            ResponseFlags::CLICKED | ResponseFlags::HOVERED | ResponseFlags::DOUBLE_CLICKED,
        );

        assert_eq!(resp.id, id);
        assert_eq!(resp.rect(), rect);
        assert!(resp.clicked());
        assert!(resp.hovered());
        assert!(resp.double_clicked());
        assert!(!resp.changed());
        assert!(!resp.dragged());
        assert!(!resp.is_focused());
    }

    #[test]
    fn test_widget_response_builder_pattern() {
        let id = WidgetId::default();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        let resp = WidgetResponse::empty(id)
            .with_rect(rect)
            .with_clicked(true)
            .with_hovered(true)
            .with_pressed(true)
            .with_changed(true)
            .with_dragged(true)
            .with_secondary_clicked(true)
            .with_middle_clicked(true);

        assert!(resp.clicked());
        assert!(resp.hovered());
        assert!(resp.pressed());
        assert!(resp.changed());
        assert!(resp.dragged());
        assert!(resp.secondary_clicked());
        assert!(resp.middle_clicked());
        assert_eq!(resp.rect(), rect);

        let cleared = resp.with_clicked(false).with_dragged(false);
        assert!(!cleared.clicked());
        assert!(!cleared.dragged());
        assert!(cleared.changed());
    }

    #[test]
    fn test_widget_response_focus_transitions() {
        let id = WidgetId::default();
        let resp = WidgetResponse::empty(id)
            .with_focused(true)
            .with_gained_focus(true);

        assert!(resp.is_focused());
        assert!(resp.gained_focus());
        assert!(!resp.lost_focus());

        let lost = resp.with_focused(false).with_lost_focus(true);
        assert!(!lost.is_focused());
        assert!(lost.lost_focus());
    }

    #[test]
    fn test_widget_response_pointer_pos() {
        let id = WidgetId::default();
        let resp = WidgetResponse::empty(id);
        assert_eq!(resp.interact_pointer_pos(), None);

        let pos = Point::new(142.5, 87.0);
        let resp_with_pos = resp.with_pointer_pos(Some(pos));
        assert_eq!(resp_with_pos.interact_pointer_pos(), Some(pos));
    }

    #[test]
    fn test_widget_response_fluent_handlers() {
        let id = WidgetId::default();
        let mut clicked = false;
        let mut double_clicked = false;
        let mut dragged = false;
        let mut changed = false;
        let mut hovered = false;
        let mut any_clicked = false;

        let resp = WidgetResponse::empty(id)
            .with_clicked(true)
            .with_hovered(true)
            .on_click(|| clicked = true)
            .on_any_click(|| any_clicked = true)
            .on_double_click(|| double_clicked = true)
            .on_drag(|| dragged = true)
            .on_change(|| changed = true)
            .on_hover(|| hovered = true);

        assert!(clicked);
        assert!(any_clicked);
        assert!(!double_clicked);
        assert!(!dragged);
        assert!(!changed);
        assert!(hovered);
        assert_eq!(resp.id, id);
    }

    #[test]
    fn test_widget_response_drag_release() {
        let id = WidgetId::default();
        let mut released = false;

        let resp = WidgetResponse::empty(id)
            .with_drag_released(true)
            .on_drag_release(|| released = true);

        assert!(resp.drag_released());
        assert!(released);

        let cleared = resp.with_drag_released(false);
        assert!(!cleared.drag_released());
    }
}
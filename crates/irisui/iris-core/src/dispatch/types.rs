// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Event Dispatch Data Models & Interaction States (`iris-core::dispatch::types`)
//!
//! Defines the core data structures tracking user interaction states across frames
//! and capturing the outcomes of dispatching UI events through the hierarchy.
//!

use std::time::Instant;

use crate::event::{KeyCode, MouseButton};
use crate::geometry::Point;
use crate::id::WidgetId;
use crate::node::WidgetCursor;
use crate::response::WidgetResponse;

/// Maximum allowable time delta in milliseconds between consecutive clicks to qualify as a double-click.
pub const DOUBLE_CLICK_MAX_DELAY_MS: u128 = 400;

/// Maximum allowable squared Euclidean distance in pixels between consecutive clicks for double-click.
pub const DOUBLE_CLICK_MAX_DISTANCE_SQ: f32 = 25.0;

/// Persistent state tracking the currently open popup or context menu.
///
/// Maintained across frames to support retained popup flyouts, contextual actions,
/// and dismissive outside-click gesture cancellation without leaking into application state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopupMenuState {
    /// Widget node identifier owning this context menu.
    pub owner_id: WidgetId,
    /// Semantic 64-bit tag of the owner widget, if assigned.
    pub owner_tag: u64,
    /// Absolute anchor screen position where the context menu was triggered.
    pub anchor_pos: Point,
    /// Allocated container WidgetId in the active UI tree during rendering, if known.
    pub container_id: Option<WidgetId>,
}

/// Persistent tracking state of active user interaction within a [`crate::tree::UiTree`].
///
/// Maintained across frames to orchestrate pointer capture during drags,
/// hover state transitions, double-click gesture detection, and keyboard focus routing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EventState {
    /// Active widget node currently capturing all pointer movements and releases.
    pub pointer_capture: Option<WidgetId>,
    /// Persistent 64-bit semantic tag of the widget capturing all pointer movements and releases, if assigned.
    pub pointer_capture_tag: Option<u64>,
    /// Deepest interactive widget node currently underneath the pointer cursor.
    pub hovered: Option<WidgetId>,
    /// Widget node where the mouse button was originally pressed down.
    pub pressed: Option<WidgetId>,
    /// Mouse button variant (Left, Right, Middle) that initiated the current press gesture.
    pub pressed_button: Option<MouseButton>,
    /// Persistent 64-bit semantic tag of the widget where the mouse button was originally pressed down, if assigned.
    pub pressed_tag: Option<u64>,
    /// Widget node currently holding active keyboard navigation or text input focus.
    pub focused: Option<WidgetId>,
    /// Persistent 64-bit semantic tag currently holding keyboard focus, if assigned.
    pub focused_tag: Option<u64>,
    /// Screen coordinate where the primary mouse press initiated.
    pub press_origin: Option<Point>,
    /// Scroll offset at the inception of an active scrollbar thumb drag.
    pub initial_scroll_y: f32,
    /// Last confirmed click target widget ID for double-click gesture detection.
    pub last_click_id: Option<WidgetId>,
    /// Screen coordinate of the last confirmed click gesture.
    pub last_click_pos: Option<Point>,
    /// Monotonic timestamp of the last confirmed click gesture.
    pub last_click_time: Option<Instant>,
    /// Widget node confirmed as clicked during the current dispatch cycle, if any.
    pub frame_clicked_id: Option<WidgetId>,
    /// Persistent 64-bit semantic tag confirmed as clicked during the current dispatch cycle, if any.
    pub frame_clicked_tag: Option<u64>,
    /// Widget node confirmed as double-clicked during the current dispatch cycle, if any.
    pub frame_double_clicked_id: Option<WidgetId>,
    /// Persistent 64-bit semantic tag confirmed as double-clicked during the current dispatch cycle, if any.
    pub frame_double_clicked_tag: Option<u64>,
    /// Widget node confirmed as secondary (right) clicked during the current dispatch cycle, if any.
    pub frame_secondary_clicked_id: Option<WidgetId>,
    /// Persistent 64-bit semantic tag confirmed as secondary (right) clicked during the current dispatch cycle, if any.
    pub frame_secondary_clicked_tag: Option<u64>,
    /// Absolute screen coordinate where the current frame interaction (click, secondary click, press) occurred.
    pub frame_clicked_pos: Option<Point>,
    /// Pointer delta movement vector accumulated during active dragging gestures in the current frame.
    pub frame_drag_delta: Option<Point>,
    /// Previous cursor screen position recorded during dragging to calculate incremental frame deltas.
    pub last_pointer_pos: Option<Point>,
    /// Last keyboard key received during the current dispatch cycle.
    pub frame_key: Option<KeyCode>,
    /// Last committed textual input string received during the current dispatch cycle.
    pub frame_text_input: Option<String>,
    /// Whether [`Self::frame_key`] was pressed (`true`) or released (`false`).
    pub frame_key_pressed: bool,
    /// Active context menu or popup anchor state, if one is currently opened.
    pub active_context_menu: Option<PopupMenuState>,
}

impl EventState {
    /// Constructs a clean, unpopulated interaction state.
    #[inline]
    pub const fn new() -> Self {
        Self {
            pointer_capture: None,
            pointer_capture_tag: None,
            hovered: None,
            pressed: None,
            pressed_button: None,
            pressed_tag: None,
            focused: None,
            focused_tag: None,
            press_origin: None,
            initial_scroll_y: 0.0,
            last_click_id: None,
            last_click_pos: None,
            last_click_time: None,
            frame_clicked_id: None,
            frame_clicked_tag: None,
            frame_double_clicked_id: None,
            frame_double_clicked_tag: None,
            frame_secondary_clicked_id: None,
            frame_secondary_clicked_tag: None,
            frame_clicked_pos: None,
            frame_drag_delta: None,
            last_pointer_pos: None,
            frame_key: None,
            frame_text_input: None,
            frame_key_pressed: false,
            active_context_menu: None,
        }
    }

    /// Resets transient interaction states while preserving persistent focus.
    #[inline]
    pub fn reset_transient_pointer(&mut self) {
        self.pointer_capture = None;
        self.hovered = None;
        self.pressed = None;
        self.pressed_button = None;
        self.pressed_tag = None;
        self.press_origin = None;
        self.initial_scroll_y = 0.0;
        self.last_click_id = None;
        self.last_click_pos = None;
        self.last_click_time = None;
        self.frame_clicked_id = None;
        self.frame_clicked_tag = None;
        self.frame_double_clicked_id = None;
        self.frame_double_clicked_tag = None;
        self.frame_secondary_clicked_id = None;
        self.frame_secondary_clicked_tag = None;
        self.frame_clicked_pos = None;
        self.frame_drag_delta = None;
        self.last_pointer_pos = None;
        self.frame_key = None;
        self.frame_text_input = None;
        self.frame_key_pressed = false;
    }

    /// Clears any confirmed click state recorded for the current frame.
    #[inline]
    pub fn clear_frame_clicks(&mut self) {
        self.frame_clicked_id = None;
        self.frame_clicked_tag = None;
        self.frame_double_clicked_id = None;
        self.frame_double_clicked_tag = None;
        self.frame_secondary_clicked_id = None;
        self.frame_secondary_clicked_tag = None;
        self.frame_clicked_pos = None;
        self.frame_drag_delta = None;
        self.frame_key = None;
        self.frame_text_input = None;
        self.frame_key_pressed = false;
    }

    /// Returns `true` if any confirmed clicks, double clicks, secondary clicks, or drag gestures
    /// are currently pending consumption within the current frame.
    #[inline]
    #[must_use]
    pub fn has_frame_interactions(&self) -> bool {
        self.frame_clicked_id.is_some()
            || self.frame_clicked_tag.is_some()
            || self.frame_double_clicked_id.is_some()
            || self.frame_double_clicked_tag.is_some()
            || self.frame_secondary_clicked_id.is_some()
            || self.frame_secondary_clicked_tag.is_some()
            || self.frame_drag_delta.is_some()
    }

    /// Clears active keyboard and semantic tag focus.
    #[inline]
    pub fn clear_focus(&mut self) {
        self.focused = None;
        self.focused_tag = None;
    }

    /// Closes the active context menu popup, if one is currently opened.
    #[inline]
    pub fn close_context_menu(&mut self) {
        self.active_context_menu = None;
    }

    /// Returns `true` if an autonomous context menu is currently open for the given owner identifier.
    #[inline]
    pub fn is_context_menu_open(&self, owner_id: WidgetId) -> bool {
        match self.active_context_menu {
            Some(menu) => menu.owner_id == owner_id,
            None => false,
        }
    }
}

/// Comprehensive outcome of dispatching a [`crate::event::UiEvent`] through the hierarchical UI graph.
///
/// Returned directly by [`crate::tree::UiTree::dispatch_event`] to inform the caller whether the event
/// was consumed by a widget, which node was targeted, what hardware cursor should display,
/// and any attached scroll/keyboard payloads.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DispatchResult {
    /// Whether the interaction was consumed by a target widget node in the hierarchy.
    pub consumed: bool,
    /// Whether this interaction mutated state or requires the containing UI tree to re-render.
    pub needs_redraw: bool,
    /// The target widget node ID that processed or originated the event.
    pub target: Option<WidgetId>,
    /// The 64-bit persistent semantic tag of the target widget node, if assigned (`0` if none).
    pub target_tag: u64,
    /// Desired mouse cursor shape override requested by the hovered or capturing widget.
    pub cursor: Option<WidgetCursor>,
    /// The immediate-mode declarative interaction response emitted for the targeted widget.
    pub response: Option<WidgetResponse>,
    /// Scroll offset delta `(delta_x, delta_y)` delivered by mouse wheel interactions.
    pub scroll_delta: (f32, f32),
    /// Virtual keyboard key associated with this interaction, if a keyboard event was processed.
    pub key: Option<KeyCode>,
    /// Whether the keyboard key is currently pressed down.
    pub is_key_pressed: bool,
    /// Printable text or committed character string delivered by typing or IME input.
    pub text_input: Option<String>,
}

impl DispatchResult {
    /// Constructs an unhandled pass-through result indicating the event was ignored.
    #[inline]
    pub const fn ignored() -> Self {
        Self {
            consumed: false,
            needs_redraw: false,
            target: None,
            target_tag: 0,
            cursor: None,
            response: None,
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        }
    }

    /// Constructs a consumed result targeted at a specific widget node.
    #[inline]
    pub const fn consumed_at(target: WidgetId, target_tag: u64) -> Self {
        Self {
            consumed: true,
            needs_redraw: true,
            target: Some(target),
            target_tag,
            cursor: None,
            response: None,
            scroll_delta: (0.0, 0.0),
            key: None,
            is_key_pressed: false,
            text_input: None,
        }
    }

    /// Returns `true` if the event was absorbed by a widget node.
    #[inline]
    pub const fn is_consumed(&self) -> bool {
        self.consumed
    }

    /// Returns `true` if this event requires an immediate visual re-render of the target UI tree.
    #[inline]
    pub const fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// Attaches a hardware cursor override to this dispatch result.
    #[inline]
    pub const fn with_cursor(mut self, cursor: Option<WidgetCursor>) -> Self {
        self.cursor = cursor;
        self
    }

    /// Attaches an immediate-mode interaction response to this dispatch result.
    #[inline]
    pub const fn with_response(mut self, response: Option<WidgetResponse>) -> Self {
        self.response = response;
        self
    }

    /// Attaches a mouse scroll delta `(delta_x, delta_y)` to this dispatch result.
    #[inline]
    pub const fn with_scroll_delta(mut self, delta_x: f32, delta_y: f32) -> Self {
        self.scroll_delta = (delta_x, delta_y);
        self
    }

    /// Attaches keyboard key data to this dispatch result.
    #[inline]
    pub const fn with_key(mut self, key: Option<KeyCode>, is_pressed: bool) -> Self {
        self.key = key;
        self.is_key_pressed = is_pressed;
        self
    }

    /// Attaches typed or committed text input to this dispatch result.
    #[inline]
    pub fn with_text(mut self, text: Option<String>) -> Self {
        self.text_input = text;
        self
    }
}
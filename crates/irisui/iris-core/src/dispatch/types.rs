// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Event Dispatch Data Models & Interaction States (`iris-core::dispatch::types`)
//!
//! Defines the core data structures tracking user interaction states across frames
//! and capturing the outcomes of dispatching UI events through the hierarchy.

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

/// Default duration in milliseconds an element must remain hovered before an autonomous tooltip opens.
pub const TOOLTIP_DEFAULT_HOVER_DELAY_MS: u64 = 400;

/// Persistent tracking state of active pointer hover and autonomous tooltip display.
///
/// Records the hovered widget node, its semantic tag, the anchor coordinate where hover began,
/// and the timestamp of hover inception to enforce non-blocking delayed activation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TooltipState {
    /// Widget node identifier currently underneath the pointer.
    pub target_id: WidgetId,
    /// Semantic 64-bit tag of the target widget node, if assigned (`0` if none).
    pub target_tag: u64,
    /// Absolute cursor screen position when hover started or tooltip anchored.
    pub anchor_pos: Point,
    /// Timestamp when pointer hover over this widget began.
    pub hover_start: Instant,
    /// Configured duration threshold before tooltip is presented.
    pub delay: std::time::Duration,
    /// Whether the tooltip has passed its hover delay and is actively visible.
    pub is_open: bool,
}

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
    /// Persistent 64-bit semantic tag of the last confirmed click gesture, if assigned.
    pub last_click_tag: Option<u64>,
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
    /// Widget node whose active drag gesture was released during the current dispatch cycle, if any.
    pub frame_drag_released_id: Option<WidgetId>,
    /// Persistent 64-bit semantic tag whose active drag gesture was released during the current dispatch cycle, if any.
    pub frame_drag_released_tag: Option<u64>,
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
    /// Active autonomous inline numeric input editing session: `(tag, buffer, cursor_idx, is_all_selected)`.
    pub active_numeric_edit: Option<(u64, String, usize, bool)>,
    /// Active autonomous inline text input editing session: `(tag, buffer, cursor_byte_idx, is_all_selected)`.
    pub active_text_edit: Option<(u64, String, usize, bool)>,
    /// Whether the active pointer interaction has exceeded the drag threshold (>3px)
    /// and is actively recognized as an ongoing dragging gesture.
    pub is_pointer_drag_active: bool,
    /// Active or pending tooltip state tracking hover duration and anchor coordinates.
    pub active_tooltip: Option<TooltipState>,
    /// Active OS file being hovered over the window, if any: `(path, point)`.
    pub hovered_file: Option<(std::path::PathBuf, Point)>,
    /// OS files dropped onto a target widget during the current dispatch cycle: `(target_id, target_tag, paths, point)`.
    pub frame_dropped_files: Option<(Option<WidgetId>, u64, Vec<std::path::PathBuf>, Point)>,
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
            last_click_tag: None,
            last_click_pos: None,
            last_click_time: None,
            frame_clicked_id: None,
            frame_clicked_tag: None,
            frame_double_clicked_id: None,
            frame_double_clicked_tag: None,
            frame_secondary_clicked_id: None,
            frame_secondary_clicked_tag: None,
            frame_drag_released_id: None,
            frame_drag_released_tag: None,
            frame_clicked_pos: None,
            frame_drag_delta: None,
            last_pointer_pos: None,
            frame_key: None,
            frame_text_input: None,
            frame_key_pressed: false,
            active_context_menu: None,
            active_numeric_edit: None,
            active_text_edit: None,
            is_pointer_drag_active: false,
            active_tooltip: None,
            hovered_file: None,
            frame_dropped_files: None,
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
        self.last_click_tag = None;
        self.last_click_pos = None;
        self.last_click_time = None;
        self.frame_clicked_id = None;
        self.frame_clicked_tag = None;
        self.frame_double_clicked_id = None;
        self.frame_double_clicked_tag = None;
        self.frame_secondary_clicked_id = None;
        self.frame_secondary_clicked_tag = None;
        self.frame_drag_released_id = None;
        self.frame_drag_released_tag = None;
        self.frame_clicked_pos = None;
        self.frame_drag_delta = None;
        self.last_pointer_pos = None;
        self.frame_key = None;
        self.frame_text_input = None;
        self.frame_key_pressed = false;
        self.is_pointer_drag_active = false;
        self.active_tooltip = None;
        self.hovered_file = None;
        self.frame_dropped_files = None;
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
        self.frame_drag_released_id = None;
        self.frame_drag_released_tag = None;
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
        self.active_numeric_edit = None;
        self.active_text_edit = None;
    }

    /// Cleans any direct references to the specified widget ID if active in interaction tracking.
    ///
    /// Ensures that removed or replaced widgets do not leave dangling references within pointer capture,
    /// hover detection, keyboard focus, click records, active context menus, or tooltips.
    #[inline]
    pub fn untrack_widget(&mut self, id: WidgetId) {
        if self.pointer_capture == Some(id) {
            self.pointer_capture = None;
        }
        if self.hovered == Some(id) {
            self.hovered = None;
        }
        if self.pressed == Some(id) {
            self.pressed = None;
        }
        if self.focused == Some(id) {
            self.focused = None;
        }
        if self.last_click_id == Some(id) {
            self.last_click_id = None;
        }
        if self.frame_clicked_id == Some(id) {
            self.frame_clicked_id = None;
        }
        if self.frame_double_clicked_id == Some(id) {
            self.frame_double_clicked_id = None;
        }
        if self.frame_secondary_clicked_id == Some(id) {
            self.frame_secondary_clicked_id = None;
        }
        if self.frame_drag_released_id == Some(id) {
            self.frame_drag_released_id = None;
        }
        if let Some(ref menu) = self.active_context_menu
            && menu.owner_id == id
        {
            self.active_context_menu = None;
        }
        if let Some(ref tooltip) = self.active_tooltip
            && tooltip.target_id == id
        {
            self.active_tooltip = None;
        }
    }

    /// Activates an autonomous inline text editing session and synchronizes focus to the target widget and tag.
    ///
    /// Sets [`Self::focused`], [`Self::focused_tag`], and [`Self::active_text_edit`], while clearing any active numeric edit.
    #[inline]
    pub fn start_text_edit(
        &mut self,
        node_id: Option<WidgetId>,
        tag: u64,
        initial_text: &str,
        cursor_idx: usize,
        is_all_selected: bool,
    ) {
        if let Some(id) = node_id {
            self.focused = Some(id);
        }
        if tag != 0 {
            self.focused_tag = Some(tag);
        }
        self.active_text_edit = Some((tag, initial_text.to_string(), cursor_idx, is_all_selected));
        self.active_numeric_edit = None;
    }

    /// Returns the active text edit buffer for the specified semantic tag, if currently editing.
    #[inline]
    #[must_use]
    pub fn text_edit_buffer(&self, tag: u64) -> Option<&str> {
        if let Some((t, ref buf, ..)) = self.active_text_edit
            && t == tag
        {
            Some(buf.as_str())
        } else {
            None
        }
    }

    /// Returns the active text edit cursor byte index for the specified semantic tag, if currently editing.
    #[inline]
    #[must_use]
    pub fn text_edit_cursor(&self, tag: u64) -> Option<usize> {
        if let Some((t, _, cur, ..)) = self.active_text_edit
            && t == tag
        {
            Some(cur)
        } else {
            None
        }
    }

    /// Activates an autonomous inline numeric editing session and synchronizes focus to the target widget and tag.
    ///
    /// Sets [`Self::focused`], [`Self::focused_tag`], and [`Self::active_numeric_edit`], while clearing any active text edit.
    #[inline]
    pub fn start_numeric_edit(
        &mut self,
        node_id: Option<WidgetId>,
        tag: u64,
        buffer: String,
        cursor_idx: usize,
        is_all_selected: bool,
    ) {
        if let Some(id) = node_id {
            self.focused = Some(id);
        }
        if tag != 0 {
            self.focused_tag = Some(tag);
        }
        self.active_numeric_edit = Some((tag, buffer, cursor_idx, is_all_selected));
        self.active_text_edit = None;
    }

    /// Clears any active text editing session and releases focus if owned by the current edit tag.
    #[inline]
    pub fn clear_text_edit(&mut self) {
        if let Some((tag, ..)) = self.active_text_edit
            && self.focused_tag == Some(tag)
        {
            self.focused = None;
            self.focused_tag = None;
        }
        self.active_text_edit = None;
    }

    /// Clears any active numeric editing session and releases focus if owned by the current edit tag.
    #[inline]
    pub fn clear_numeric_edit(&mut self) {
        if let Some((tag, ..)) = self.active_numeric_edit
            && self.focused_tag == Some(tag)
        {
            self.focused = None;
            self.focused_tag = None;
        }
        self.active_numeric_edit = None;
    }

    /// Returns `true` if any widget node, semantic tag, numeric edit, or text edit currently holds keyboard focus.
    #[inline]
    #[must_use]
    pub fn has_focus(&self) -> bool {
        self.focused.is_some()
            || self.focused_tag.is_some()
            || self.active_numeric_edit.is_some()
            || self.active_text_edit.is_some()
    }

    /// Returns `true` if an active inline text editing buffer or numeric editing buffer currently holds keyboard focus.
    ///
    /// Distinct from generic widget focus (`has_focus`), which tracks keyboard navigation targets like buttons or tabs.
    #[inline]
    #[must_use]
    pub fn has_text_input_focus(&self) -> bool {
        self.active_text_edit.is_some() || self.active_numeric_edit.is_some()
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

    /// Closes any active or pending tooltip display.
    #[inline]
    pub fn close_tooltip(&mut self) {
        self.active_tooltip = None;
    }

    /// Returns `true` if a tooltip is currently active and passed its hover delay.
    #[inline]
    #[must_use]
    pub fn has_open_tooltip(&self) -> bool {
        self.active_tooltip.as_ref().is_some_and(|t| t.is_open)
    }

    /// Returns `true` if a tooltip is currently open for the given widget identifier or semantic tag.
    #[inline]
    #[must_use]
    pub fn is_tooltip_open_for(&self, target_id: WidgetId, target_tag: u64) -> bool {
        if let Some(ref tooltip) = self.active_tooltip
            && tooltip.is_open
        {
            return tooltip.target_id == target_id
                || (target_tag != 0 && tooltip.target_tag == target_tag);
        }
        false
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
    /// Optional timeout delay until the next required UI tick (e.g. for tooltip hover timer).
    pub next_timeout: Option<std::time::Duration>,
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
            next_timeout: None,
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
            next_timeout: None,
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

    /// Attaches a scheduled timer wake-up duration to this dispatch result.
    #[inline]
    pub const fn with_next_timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        self.next_timeout = timeout;
        self
    }
}
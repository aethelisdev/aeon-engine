// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Event Outcome and Interaction Results
//!
//! Encapsulates the complete result of dispatching an interaction event through
//! the autonomous [`Workbench`] coordinator, including event consumption,
//! requested platform cursor, and emitted type-safe application actions.

use crate::lifecycle::EventInvalidationRoute;
use irisui_core::geometry::Point;
use irisui_core::node::WidgetCursor;
use irisui_dock::notifier::UiNotifier;
use std::hash::Hash;

/// Summarized outcome returned after processing an event in [`crate::workbench::Workbench::handle_event`].
///
/// Designed to provide complete event telemetry to the hosting application without requiring
/// any internal knowledge of window borders, title bar dragging, dock splitters, or modal focus traps.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkbenchOutcome<Action, TabId = ()> {
    /// Whether the event was intercepted and consumed by the workbench shell,
    /// stopping propagation to underlying game engines, 3D canvases, or custom tools.
    pub consumed: bool,

    /// Requested platform cursor icon override (e.g. `ResizeNorth`, `ColResize`, `Grabbing`, `Pointer`),
    /// to be synchronized with the OS window via [`irisui_winit::frameless::WindowInterface::set_widget_cursor`].
    pub cursor: Option<WidgetCursor>,

    /// Most recently observed cursor coordinate in logical screen points.
    pub cursor_pos: Option<Point>,

    /// Whether the docking hierarchy, splitter ratios, or floating window boundaries changed,
    /// signaling that a layout rebuild or visual refresh is required.
    pub layout_changed: bool,

    /// List of decoupled application actions emitted during this event cycle
    /// (e.g. menu triggers, modal confirmations, titlebar minimize/maximize requests).
    pub actions: Vec<Action>,

    /// Dropped files intercepted during a native drag-and-drop file ingest event.
    pub dropped_files: Option<(Vec<std::path::PathBuf>, irisui_core::geometry::Point)>,

    /// UI zoom scale factor update requested via keyboard shortcuts (e.g. Ctrl +/-/0).
    pub zoom_changed: Option<f32>,

    /// Toggle preferences modal dialog requested via keyboard shortcuts (e.g. Ctrl+,).
    pub toggle_preferences: bool,

    /// Whether a platform window close control (e.g. titlebar/menubar close button) was triggered.
    pub close_requested: bool,

    /// Whether active workbench controls (modal dialogs or focused text inputs) require
    /// exclusive keyboard input, signaling the host application to suppress global hotkeys.
    pub wants_keyboard_input: bool,

    /// Whether an Escape key gesture was intercepted and handled by the workbench
    /// (e.g. dismissing modal dialogs, closing popups, or clearing focus traps).
    pub escape_handled: bool,

    /// Whether an outside-click gesture was detected and dismissed open popups or context menus.
    pub popups_dismissed: bool,

    /// Most recently resolved 64-bit semantic interaction tag under the cursor.
    pub hovered_tag: Option<u64>,

    /// Most recently resolved non-excluded dock panel tab under the cursor when unoccluded.
    pub hovered_panel: Option<TabId>,

    /// Whether the resolved hovered semantic tag transitioned during this dispatch pass.
    pub hover_target_changed: bool,

    /// Whether the hover target transition alters any visible text color.
    pub hover_affects_text: bool,

    /// Whether the hover target transition requires a full quad upload due to lacking boundaries.
    pub hover_requires_full_upload: bool,

    /// Optional targeted dock panel leaf container invalidated by an interaction event.
    pub invalidated_panel: Option<TabId>,

    /// Whether an interaction event invalidated the overall application shell.
    pub shell_dirty: bool,

    /// Detailed invalidation routing outcome, if determined.
    pub invalidation_route: Option<EventInvalidationRoute<TabId>>,
}

impl<Action, TabId> Default for WorkbenchOutcome<Action, TabId> {
    #[inline]
    fn default() -> Self {
        Self {
            consumed: false,
            cursor: None,
            cursor_pos: None,
            layout_changed: false,
            actions: Vec::new(),
            dropped_files: None,
            zoom_changed: None,
            toggle_preferences: false,
            close_requested: false,
            wants_keyboard_input: false,
            escape_handled: false,
            popups_dismissed: false,
            hovered_tag: None,
            hovered_panel: None,
            hover_target_changed: false,
            hover_affects_text: false,
            hover_requires_full_upload: false,
            invalidated_panel: None,
            shell_dirty: false,
            invalidation_route: None,
        }
    }
}

impl<Action, TabId> WorkbenchOutcome<Action, TabId> {
    /// Constructs an unconsumed, empty event outcome with no requested actions or layout alterations.
    #[inline]
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Constructs a consumed event outcome with an optional cursor override.
    #[inline]
    #[must_use]
    pub fn consumed(cursor: Option<WidgetCursor>) -> Self {
        Self {
            consumed: true,
            cursor,
            cursor_pos: None,
            layout_changed: false,
            actions: Vec::new(),
            dropped_files: None,
            zoom_changed: None,
            toggle_preferences: false,
            close_requested: false,
            wants_keyboard_input: false,
            escape_handled: false,
            popups_dismissed: false,
            hovered_tag: None,
            hovered_panel: None,
            hover_target_changed: false,
            hover_affects_text: false,
            hover_requires_full_upload: false,
            invalidated_panel: None,
            shell_dirty: false,
            invalidation_route: None,
        }
    }

    /// Marks whether active UI controls require exclusive keyboard focus.
    #[inline]
    #[must_use]
    pub const fn with_wants_keyboard_input(mut self, wants: bool) -> Self {
        self.wants_keyboard_input = wants;
        self
    }

    /// Attaches an emitted action to the outcome, marking the event as consumed.
    #[inline]
    pub fn with_action(mut self, action: Action) -> Self {
        self.consumed = true;
        self.actions.push(action);
        self
    }

    /// Attaches an observed logical cursor coordinate to the outcome.
    #[inline]
    #[must_use]
    pub fn with_cursor_pos(mut self, pos: Option<Point>) -> Self {
        self.cursor_pos = pos;
        self
    }

    /// Attaches a requested cursor shape override to the outcome.
    #[inline]
    #[must_use]
    pub fn with_cursor(mut self, cursor: Option<WidgetCursor>) -> Self {
        self.cursor = cursor;
        self
    }

    /// Marks whether an Escape key gesture was intercepted and handled by the workbench.
    #[inline]
    #[must_use]
    pub fn with_escape_handled(mut self, handled: bool) -> Self {
        self.escape_handled = handled;
        self
    }

    /// Marks whether open popups or context menus were dismissed by an outside-click gesture.
    #[inline]
    #[must_use]
    pub fn with_popups_dismissed(mut self, dismissed: bool) -> Self {
        self.popups_dismissed = dismissed;
        self
    }

    /// Attaches an invalidation route to the outcome, updating `invalidated_panel`
    /// and `shell_dirty` automatically.
    #[inline]
    #[must_use]
    pub fn with_invalidation_route(mut self, route: EventInvalidationRoute<TabId>) -> Self
    where
        TabId: Clone,
    {
        match &route {
            EventInvalidationRoute::Panel(tab) => {
                self.invalidated_panel = Some(tab.clone());
            }
            EventInvalidationRoute::Shell => {
                self.shell_dirty = true;
                self.layout_changed = true;
            }
            EventInvalidationRoute::None => {}
        }
        self.invalidation_route = Some(route);
        self
    }

    /// Merges `route` with any existing invalidation route using priority `Shell > Panel > None`.
    #[inline]
    #[must_use]
    pub fn merge_invalidation_route(mut self, route: EventInvalidationRoute<TabId>) -> Self
    where
        TabId: Clone,
    {
        let merged = match self.invalidation_route.take() {
            Some(existing) => existing.merge(route),
            None => route,
        };
        self.with_invalidation_route(merged)
    }

    /// Explicitly flags a specific panel tab as invalidated.
    #[inline]
    #[must_use]
    pub fn with_invalidated_panel(mut self, tab: Option<TabId>) -> Self {
        self.invalidated_panel = tab;
        self
    }

    /// Explicitly flags the overall shell as dirty.
    #[inline]
    #[must_use]
    pub fn with_shell_dirty(mut self, dirty: bool) -> Self {
        self.shell_dirty = dirty;
        if dirty {
            self.layout_changed = true;
        }
        self
    }

    /// Returns a reference to the invalidated dock panel tab identifier, if targeted.
    #[inline]
    #[must_use]
    pub const fn invalidated_panel(&self) -> Option<&TabId> {
        self.invalidated_panel.as_ref()
    }

    /// Returns whether the event invalidated the application shell.
    #[inline]
    #[must_use]
    pub const fn is_shell_dirty(&self) -> bool {
        self.shell_dirty
    }

    /// Returns the resolved event invalidation route, if present.
    #[inline]
    #[must_use]
    pub const fn invalidation_route(&self) -> Option<&EventInvalidationRoute<TabId>> {
        self.invalidation_route.as_ref()
    }

    /// Applies the invalidation routing decision directly to an external [`UiNotifier`].
    ///
    /// If an invalidation route is attached, it delegates directly to
    /// [`EventInvalidationRoute::apply_to_notifier`]. Otherwise, synchronizes
    /// `invalidated_panel` via [`UiNotifier::tag_redraw`] and `shell_dirty`
    /// via [`UiNotifier::tag_all`].
    #[inline]
    pub fn apply_to_notifier(&self, notifier: &mut UiNotifier<TabId>)
    where
        TabId: Copy + Eq + Hash + Send + Sync + 'static,
    {
        if let Some(route) = &self.invalidation_route {
            route.apply_to_notifier(notifier);
        } else {
            if self.shell_dirty {
                notifier.tag_all();
            }
            if let Some(tab) = self.invalidated_panel {
                notifier.tag_redraw(tab);
            }
        }
    }

    /// Returns the most recently resolved hovered 64-bit semantic interaction tag, if any.
    #[inline]
    #[must_use]
    pub const fn hovered_tag(&self) -> Option<u64> {
        self.hovered_tag
    }

    /// Returns the most recently resolved hovered non-excluded dock panel tab, if any.
    #[inline]
    #[must_use]
    pub const fn hovered_panel(&self) -> Option<&TabId> {
        self.hovered_panel.as_ref()
    }

    /// Returns whether the resolved hovered semantic interaction tag changed during this event pass.
    #[inline]
    #[must_use]
    pub const fn is_hover_target_changed(&self) -> bool {
        self.hover_target_changed
    }

    /// Discards panel invalidation telemetry, converting this outcome into an action-only outcome.
    #[inline]
    #[must_use]
    pub fn into_action_only(self) -> WorkbenchOutcome<Action, ()> {
        WorkbenchOutcome {
            consumed: self.consumed,
            cursor: self.cursor,
            cursor_pos: self.cursor_pos,
            layout_changed: self.layout_changed,
            actions: self.actions,
            dropped_files: self.dropped_files,
            zoom_changed: self.zoom_changed,
            toggle_preferences: self.toggle_preferences,
            close_requested: self.close_requested,
            wants_keyboard_input: self.wants_keyboard_input,
            escape_handled: self.escape_handled,
            popups_dismissed: self.popups_dismissed,
            hovered_tag: self.hovered_tag,
            hovered_panel: None,
            hover_target_changed: self.hover_target_changed,
            hover_affects_text: self.hover_affects_text,
            hover_requires_full_upload: self.hover_requires_full_upload,
            invalidated_panel: None,
            shell_dirty: self.shell_dirty,
            invalidation_route: match self.invalidation_route {
                Some(EventInvalidationRoute::Shell) => Some(EventInvalidationRoute::Shell),
                Some(EventInvalidationRoute::None) => Some(EventInvalidationRoute::None),
                _ => None,
            },
        }
    }
}
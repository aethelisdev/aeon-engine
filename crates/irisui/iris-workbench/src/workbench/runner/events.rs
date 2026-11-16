// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Event Handling and Interaction Pipeline
//!
//! Provides the primary event routing and invalidation coordination methods on [`Workbench`].
//! Handles frameless window resizing, titlebar dragging, dock splitters, modal keyboard traps,
//! global hotkeys, and targeted subtree invalidation.

use super::coordinator::Workbench;
use crate::lifecycle::EventInvalidationRoute;
use crate::workbench::outcome::WorkbenchOutcome;
use iris_core::dispatch::DispatchResult;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::Rect;
use iris_core::tree::UiTree;
use iris_dock::UiNotifier;
use iris_dock::area::runner::{DockArea, DockInteractionConfig};
use iris_dock::floating::{
    DEFAULT_RESIZE_MARGIN, FloatingInteractionResponse, FloatingWindowClampBounds,
};
use iris_dock::state::DockState;
use iris_winit::frameless::WindowInterface;
use std::fmt::Debug;
use std::hash::Hash;

struct TreeAndContentResult<TabId, Action> {
    consumed: bool,
    cursor: Option<iris_core::node::WidgetCursor>,
    needs_redraw: bool,
    wants_keyboard_input: bool,
    hover_target_changed: bool,
    route: Option<EventInvalidationRoute<TabId>>,
    extra_actions: Vec<Action>,
}

impl<TabId, Action> Default for TreeAndContentResult<TabId, Action> {
    fn default() -> Self {
        Self {
            consumed: false,
            cursor: None,
            needs_redraw: false,
            wants_keyboard_input: false,
            hover_target_changed: false,
            route: None,
            extra_actions: Vec::new(),
        }
    }
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + 'static,
    Action: Send + Sync + 'static,
{
    /// Dispatches a platform [`UiEvent`] through the autonomous workbench interaction pipeline
    /// using the internal docking state.
    pub fn handle_event<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
    ) -> WorkbenchOutcome<Action, TabId> {
        let mut dock_state = std::mem::take(&mut self.descriptor.dock_state);
        let outcome = self.handle_event_full(window, event, &mut dock_state, None);
        self.descriptor.dock_state = dock_state;
        outcome
    }

    /// Dispatches a platform [`UiEvent`] into the [`UiTree`], processes autonomous [`DialogHost`]
    /// modal tag clicks into `action_bus`, resolves hover targets, and computes the invalidation route.
    fn dispatch_tree_with_modals(
        &mut self,
        tree: &mut UiTree,
        event: &UiEvent,
    ) -> TreeAndContentResult<TabId, Action> {
        if let Some(pos) = self.last_cursor_pos {
            tree.set_cursor_position(pos);
        }
        let dispatch = tree.dispatch_event(event);
        let modal_click_consumed = tree
            .event_state
            .frame_clicked_tag
            .filter(|&tag| tag != 0)
            .is_some_and(|tag| {
                self.dialog_host
                    .handle_clicked_tag(tag, &mut self.action_bus)
            });
        if modal_click_consumed {
            tree.event_state.frame_clicked_tag = None;
        }
        let route = if modal_click_consumed {
            EventInvalidationRoute::Shell
        } else {
            self.route_event_invalidation(tree, event, &dispatch)
        };
        let hover = self.resolve_hover_targets(tree, self.last_cursor_pos);
        TreeAndContentResult {
            consumed: dispatch.consumed || modal_click_consumed,
            cursor: dispatch.cursor,
            needs_redraw: dispatch.needs_redraw || modal_click_consumed,
            wants_keyboard_input: tree.is_text_input_focused(),
            hover_target_changed: hover.hover_target_changed,
            route: Some(route),
            extra_actions: Vec::new(),
        }
    }

    /// Creates a base [`WorkbenchOutcome`] configured with consumption status, cursor shape,
    /// and the active pointer coordinate.
    #[inline]
    #[must_use]
    pub fn make_outcome(
        &self,
        consumed: bool,
        cursor: Option<iris_core::node::WidgetCursor>,
    ) -> WorkbenchOutcome<Action, TabId> {
        WorkbenchOutcome {
            consumed,
            cursor,
            cursor_pos: self.last_cursor_pos,
            layout_changed: false,
            actions: Vec::new(),
            dropped_files: None,
            zoom_changed: None,
            toggle_preferences: false,
            wants_keyboard_input: self.wants_keyboard_input(),
            escape_handled: false,
            popups_dismissed: false,
            hovered_tag: self.hovered_tag,
            hovered_panel: self.hovered_panel,
            hover_target_changed: false,
            invalidated_panel: None,
            shell_dirty: false,
            invalidation_route: None,
        }
    }

    /// Evaluates active floating/tab pointer captures and floating chrome interactions prior to
    /// content dispatch.
    fn try_handle_floating_priority<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
    ) -> Option<WorkbenchOutcome<Action, TabId>> {
        if dock_state.has_active_floating_drag()
            || dock_state.active_drag.is_some()
            || dock_state.pending_tab_drag.is_some()
        {
            let mut outcome = self.handle_shell_event_inner(window, event, dock_state, false, true);
            outcome.consumed = true;
            outcome.layout_changed = true;
            outcome.shell_dirty = true;
            return Some(outcome.with_invalidation_route(EventInvalidationRoute::Shell));
        }

        if let UiEvent::MouseDown {
            button: MouseButton::Left,
            ..
        } = event
        {
            let floating_res = self.handle_floating_event(window, event, dock_state);
            if floating_res.consumed {
                if let Some(pending) = floating_res.pending_tab_drag {
                    dock_state.pending_tab_drag = Some(pending);
                }
                let mut outcome = self.make_outcome(true, floating_res.cursor);
                outcome.layout_changed = floating_res.layout_changed;
                if floating_res.layout_changed {
                    outcome = outcome.with_invalidation_route(EventInvalidationRoute::Shell);
                }
                outcome.actions.extend(self.action_bus.drain());
                return Some(outcome);
            }
        }
        None
    }

    /// Merges tree/content dispatch telemetry with shell evaluation into a unified [`WorkbenchOutcome`].
    fn finalize_shell_outcome<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        content: TreeAndContentResult<TabId, Action>,
    ) -> WorkbenchOutcome<Action, TabId> {
        if let Some(cursor) = content.cursor {
            window.set_widget_cursor(cursor);
        }

        let mut outcome = self.handle_shell_event_inner(
            window,
            event,
            dock_state,
            content.consumed,
            content.wants_keyboard_input,
        );
        if !self.action_bus.is_empty() {
            outcome.actions.extend(self.action_bus.drain());
            outcome.layout_changed = true;
        }
        if !content.extra_actions.is_empty() {
            outcome.actions.extend(content.extra_actions);
            outcome.layout_changed = true;
        }
        if content.consumed {
            outcome.consumed = true;
        }
        if content.wants_keyboard_input {
            outcome.wants_keyboard_input = true;
        }
        if let UiEvent::FileDropped { paths, point } = event
            && !paths.is_empty()
            && outcome.dropped_files.is_none()
        {
            outcome.dropped_files = Some((paths.clone(), *point));
        }
        if content.needs_redraw {
            outcome.layout_changed = true;
        }
        outcome.hover_target_changed = content.hover_target_changed;
        if outcome.cursor.is_none() {
            outcome.cursor = content.cursor;
        }

        if let Some(route) = content.route {
            outcome = outcome.merge_invalidation_route(route);
        } else if self.lifecycle.is_panel_dirty() {
            if let Some(&dirty_tab) = self.lifecycle.dirty_tabs().iter().next() {
                outcome =
                    outcome.merge_invalidation_route(EventInvalidationRoute::Panel(dirty_tab));
            }
        } else if outcome.layout_changed || self.lifecycle.is_forced_shell_rebuild() {
            outcome = outcome.merge_invalidation_route(EventInvalidationRoute::Shell);
        }

        outcome
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous shell layer coordinating
    /// frameless window border resizing, titlebar dragging on empty menubar space, modal dialog
    /// keyboard traps, global shortcuts, and dock interaction while respecting content consumption.
    pub fn handle_shell_event<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        content_consumed: bool,
    ) -> WorkbenchOutcome<Action, TabId> {
        self.handle_shell_event_inner(window, event, dock_state, content_consumed, false)
    }

    fn handle_shell_event_inner<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        content_consumed: bool,
        suppress_global_shortcuts: bool,
    ) -> WorkbenchOutcome<Action, TabId> {
        let logical_size = window.logical_size();
        let screen_w = logical_size.width;
        let screen_h = logical_size.height;
        self.last_window_size = Some(logical_size);

        // Track cursor position and pointer states from events
        self.pointer_state.update_from_event(event);
        self.last_cursor_pos = self.pointer_state.position;

        // 1. Frameless Window 8-way Native Edge Resizing
        if self
            .frameless_host
            .handle_ui_event(window, event, logical_size)
        {
            let mut outcome = self.make_outcome(true, None);
            outcome.shell_dirty = true;
            outcome.layout_changed = true;
            outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
            return outcome;
        }

        // If the inner panel/overlay content already consumed the event, skip shell and dock interaction
        // UNLESS an active floating window drag or tab drag owns pointer capture, or pointer is over a floating window/border
        let has_active_drag = dock_state.has_active_floating_drag()
            || dock_state.active_drag.is_some()
            || dock_state.pending_tab_drag.is_some();
        let is_over_floating = match event {
            UiEvent::MouseMove { point } | UiEvent::MouseDown { point, .. } => {
                dock_state.is_point_over_floating_window_or_border(*point, DEFAULT_RESIZE_MARGIN)
            }
            _ => false,
        };
        if content_consumed && !has_active_drag && !is_over_floating {
            let mut outcome = self.make_outcome(true, None);
            outcome.actions.extend(self.action_bus.drain());
            return outcome;
        }

        // 2. Active framework modal dialog keyboard input or Escape key cancellation
        if let Some(outcome) = self.handle_modal_keyboard_event(event) {
            return outcome;
        }
        if let Some(outcome) = self.handle_escape_event(event, None, None) {
            return outcome;
        }

        // 3. Ingest dropped files directly into outcome
        if let UiEvent::FileDropped { paths, point } = event
            && !paths.is_empty()
        {
            let mut outcome = self.make_outcome(true, None);
            if let Some(builder) = &self.file_drop_action {
                outcome.actions.push(builder(paths.clone(), *point));
            }
            outcome.dropped_files = Some((paths.clone(), *point));
            outcome.layout_changed = true;
            outcome.shell_dirty = true;
            outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
            return outcome;
        }

        // 4. Native Title Bar Dragging & Maximize Double-Click on empty menubar space
        if self.descriptor.menubar.is_some() {
            let menubar_rect = Rect::new(0.0, 0.0, screen_w, self.menubar_height);
            if self.drag_host.handle_ui_event(window, event, menubar_rect) {
                let mut outcome = self.make_outcome(true, None);
                outcome.shell_dirty = true;
                outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
                return outcome;
            }
        }

        // 5. Global Keyboard Shortcuts (Ctrl +/-/0 for zoom, Ctrl+, for preferences)
        if let UiEvent::KeyboardInput {
            key,
            is_pressed,
            text,
            ctrl_held,
            ..
        } = event
            && *is_pressed
            && *ctrl_held
            && !suppress_global_shortcuts
            && !self.wants_keyboard_input()
        {
            let ch = match key {
                iris_core::event::KeyCode::Character(c) => Some(*c),
                _ => text.as_deref().and_then(|t| t.chars().next()),
            };

            if let Some(c) = ch {
                let zoom_target = match c {
                    '+' | '=' => Some(self.step_zoom(true)),
                    '-' | '_' => Some(self.step_zoom(false)),
                    '0' => Some(self.reset_zoom()),
                    _ => None,
                };

                if let Some(new_zoom) = zoom_target {
                    let mut outcome = self.make_outcome(true, None);
                    if let Some(builder) = &self.zoom_action {
                        outcome.actions.push(builder(new_zoom));
                    }
                    outcome.zoom_changed = Some(new_zoom);
                    outcome.layout_changed = true;
                    outcome.shell_dirty = true;
                    outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
                    return outcome;
                }

                if c == ',' {
                    let mut outcome = self.make_outcome(true, None);
                    if let Some(builder) = &self.preferences_action {
                        outcome.actions.push(builder());
                    }
                    outcome.toggle_preferences = true;
                    outcome.layout_changed = true;
                    outcome.shell_dirty = true;
                    outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
                    return outcome;
                }
            }
        }

        // 6. Autonomous Docking Interaction (Splitters, Tabs, Floating Windows)
        let workspace_rect = self.compute_workspace_rect(screen_w, screen_h);
        let clamp_bounds = FloatingWindowClampBounds::new(screen_w, screen_h)
            .with_min_y(self.menubar_height)
            .with_bottom_inset(self.statusbar_height);

        let dock_config = DockInteractionConfig {
            clamp_bounds,
            ..Default::default()
        };

        let tab_viewer = self.panel_registry.tab_viewer();
        let dock_res =
            DockArea::handle_event(dock_state, &tab_viewer, event, workspace_rect, &dock_config);

        if let Some(cursor) = dock_res.cursor_override {
            window.set_widget_cursor(cursor);
        }

        if dock_res.consumed {
            let mut outcome = self.make_outcome(true, dock_res.cursor_override);
            outcome.layout_changed = dock_res.layout_changed;
            if dock_res.layout_changed {
                outcome.shell_dirty = true;
                outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
            }
            outcome.actions.extend(self.action_bus.drain());
            return outcome;
        }

        let mut outcome = self.make_outcome(false, dock_res.cursor_override);
        outcome.layout_changed = dock_res.layout_changed;
        if dock_res.layout_changed {
            outcome.shell_dirty = true;
            outcome.invalidation_route = Some(EventInvalidationRoute::Shell);
        }
        outcome.actions.extend(self.action_bus.drain());
        outcome
    }

    /// Dispatches a platform [`UiEvent`] directly to the floating window interaction manager.
    ///
    /// Handles titlebar dragging, 8-directional border edge resizing, window close, dock-back,
    /// tab switching, and focus elevation across all detached floating windows.
    ///
    /// Automatically updates the cursor on the host window if requested by floating window hover or drag.
    pub fn handle_floating_event<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
    ) -> FloatingInteractionResponse<TabId> {
        let logical_size = window.logical_size();
        let screen_w = logical_size.width;
        let screen_h = logical_size.height;
        let clamp_bounds = FloatingWindowClampBounds::new(screen_w, screen_h)
            .with_min_y(self.menubar_height)
            .with_bottom_inset(self.statusbar_height);

        let tab_viewer = self.panel_registry.tab_viewer();
        let res = dock_state.handle_floating_event(&tab_viewer, event, clamp_bounds);
        if let Some(cursor) = res.cursor {
            window.set_widget_cursor(cursor);
        }
        res
    }

    /// Dispatches an event through both content and shell layers in industry-standard sequence.
    ///
    /// Evaluates content first. If content produces actions, requests cursor change, or consumes the event,
    /// those actions are aggregated and shell dragging/dock interaction is bypassed.
    ///
    /// Automatically forwards to [`Self::handle_event_with_content_tree`] with `tree = None`.
    pub fn handle_event_with_content<W, F>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        content_dispatcher: F,
    ) -> WorkbenchOutcome<Action, TabId>
    where
        W: WindowInterface,
        F: FnOnce(&UiEvent) -> (bool, Option<iris_core::node::WidgetCursor>, Vec<Action>),
    {
        self.handle_event_with_content_tree(window, event, dock_state, None, content_dispatcher)
    }

    /// Dispatches an event through both content and shell layers, optionally accepting an active [`UiTree`]
    /// for automatic targeted panel invalidation routing and lifecycle reconciliation.
    ///
    /// # Arguments
    /// * `window` - Host window implementing [`WindowInterface`] for cursor updates.
    /// * `event` - The platform [`UiEvent`] being processed.
    /// * `dock_state` - Mutable reference to the active docking layout state.
    /// * `tree` - Optional mutable reference to the active [`UiTree`] for cursor sync and invalidation routing.
    /// * `content_dispatcher` - Callback closure evaluating inner panel/overlay interaction.
    ///
    /// # Returns
    /// A complete [`WorkbenchOutcome`] including consumption, cursor requests, emitted actions,
    /// and targeted panel or shell invalidation routing.
    pub fn handle_event_with_content_tree<W, F>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        mut tree: Option<&mut UiTree>,
        content_dispatcher: F,
    ) -> WorkbenchOutcome<Action, TabId>
    where
        W: WindowInterface,
        F: FnOnce(&UiEvent) -> (bool, Option<iris_core::node::WidgetCursor>, Vec<Action>),
    {
        // Track pointer state immediately at entry so it is valid during content dispatch
        self.pointer_state.update_from_event(event);
        self.last_cursor_pos = self.pointer_state.position;

        // Active framework modal dialogues trap keyboard events before background content dispatch
        if let Some(outcome) = self.handle_modal_keyboard_event(event) {
            return outcome;
        }

        if let Some(outcome) = self.handle_escape_event(event, tree.as_deref_mut(), None) {
            return outcome;
        }

        if let Some(outcome) = self.handle_outside_click_event(event, tree.as_deref_mut()) {
            return outcome;
        }

        if let Some(outcome) = self.try_handle_floating_priority(window, event, dock_state) {
            return outcome;
        }

        let mut result = if let Some(t) = tree {
            self.dispatch_tree_with_modals(t, event)
        } else {
            TreeAndContentResult::default()
        };

        let (content_consumed, content_cursor, content_actions) = content_dispatcher(event);
        result.consumed = result.consumed || content_consumed;
        result.cursor = content_cursor.or(result.cursor);
        result.extra_actions = content_actions;
        result.needs_redraw = result.needs_redraw
            || matches!(
                event,
                UiEvent::MouseDown { .. }
                    | UiEvent::MouseUp { .. }
                    | UiEvent::KeyboardInput {
                        is_pressed: true,
                        ..
                    }
                    | UiEvent::Ime(_)
                    | UiEvent::FileDropped { .. }
            );
        self.finalize_shell_outcome(window, event, dock_state, result)
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous workbench interaction pipeline
    /// with an explicitly provided external docking state.
    pub fn handle_event_with_dock<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
    ) -> WorkbenchOutcome<Action, TabId> {
        self.handle_event_full(window, event, dock_state, None)
    }

    /// Dispatches a platform [`UiEvent`] with an explicit [`UiTree`], providing complete
    /// event dispatch and automatic targeted panel invalidation routing in the returned outcome.
    pub fn handle_event_with_tree<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        tree: &mut UiTree,
    ) -> WorkbenchOutcome<Action, TabId> {
        let mut dock_state = std::mem::take(&mut self.descriptor.dock_state);
        let outcome = self.handle_event_full(window, event, &mut dock_state, Some(tree));
        self.descriptor.dock_state = dock_state;
        outcome
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous workbench interaction pipeline
    /// coordinating frameless window management, titlebar dragging, modal dialogs, docking splitters,
    /// and declarative [`UiTree`] event routing in a single unified retained pass.
    ///
    /// Automatically routes discrete gestures and click events to their owning dock panel leaf
    /// container via [`Self::route_event_invalidation`], attaching the resulting [`EventInvalidationRoute`]
    /// directly to the returned [`WorkbenchOutcome`].
    pub fn handle_event_full<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        mut tree: Option<&mut UiTree>,
    ) -> WorkbenchOutcome<Action, TabId> {
        self.pointer_state.update_from_event(event);
        self.last_cursor_pos = self.pointer_state.position;

        if let Some(outcome) = self.handle_modal_keyboard_event(event) {
            return outcome;
        }

        if let Some(outcome) = self.handle_escape_event(event, tree.as_deref_mut(), None) {
            return outcome;
        }

        if let Some(outcome) = self.handle_outside_click_event(event, tree.as_deref_mut()) {
            return outcome;
        }

        if let Some(outcome) = self.try_handle_floating_priority(window, event, dock_state) {
            return outcome;
        }

        let result = if let Some(t) = tree {
            self.dispatch_tree_with_modals(t, event)
        } else {
            TreeAndContentResult::default()
        };

        self.finalize_shell_outcome(window, event, dock_state, result)
    }

    /// Routes an interaction event dispatch result to its owning dock panel leaf container,
    /// or marks the overall shell as dirty if the interaction occurred outside any leaf.
    ///
    /// Automatically updates [`Workbench::lifecycle`] by marking the resolved tab dirty
    /// via [`crate::lifecycle::WorkbenchLifecycle::mark_tab_dirty`], or marking the shell dirty via
    /// [`crate::lifecycle::WorkbenchLifecycle::mark_shell_dirty`].
    pub fn route_event_invalidation(
        &mut self,
        tree: &UiTree,
        event: &UiEvent,
        dispatch: &DispatchResult,
    ) -> EventInvalidationRoute<TabId> {
        let route = crate::lifecycle::route_event_invalidation(
            tree,
            self.dock_frame.as_ref(),
            event,
            dispatch,
        );
        match &route {
            EventInvalidationRoute::Panel(tab) => {
                self.lifecycle.mark_tab_dirty(*tab);
            }
            EventInvalidationRoute::Shell => {
                self.lifecycle.mark_shell_dirty();
            }
            EventInvalidationRoute::None => {}
        }
        route
    }

    /// Same as [`Self::route_event_invalidation`], but also synchronizes an external [`UiNotifier`].
    pub fn route_event_invalidation_with_notifier(
        &mut self,
        tree: &UiTree,
        event: &UiEvent,
        dispatch: &DispatchResult,
        notifier: &mut UiNotifier<TabId>,
    ) -> EventInvalidationRoute<TabId>
    where
        TabId: Send + Sync,
    {
        let route = self.route_event_invalidation(tree, event, dispatch);
        route.apply_to_notifier(notifier);
        route
    }

    /// Routes an interaction event dispatch result to its owning dock panel leaf container,
    /// accepting a custom fallback resolver before falling back to coarse shell invalidation.
    ///
    /// Automatically updates [`Workbench::lifecycle`] by marking the resolved tab dirty
    /// via [`crate::lifecycle::WorkbenchLifecycle::mark_tab_dirty`], or marking the shell dirty via
    /// [`crate::lifecycle::WorkbenchLifecycle::mark_shell_dirty`].
    pub fn route_event_invalidation_with_fallback<F>(
        &mut self,
        tree: &UiTree,
        event: &UiEvent,
        dispatch: &DispatchResult,
        custom_resolver: F,
    ) -> EventInvalidationRoute<TabId>
    where
        F: FnOnce(&DispatchResult, &UiEvent) -> Option<TabId>,
    {
        let route = crate::lifecycle::route_event_invalidation_with_fallback(
            tree,
            self.dock_frame.as_ref(),
            event,
            dispatch,
            custom_resolver,
        );
        match &route {
            EventInvalidationRoute::Panel(tab) => {
                self.lifecycle.mark_tab_dirty(*tab);
            }
            EventInvalidationRoute::Shell => {
                self.lifecycle.mark_shell_dirty();
            }
            EventInvalidationRoute::None => {}
        }
        route
    }
}
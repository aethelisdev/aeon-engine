// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Global Escape Key Resolution Subsystem
//!
//! Provides the centralized [`Workbench::handle_escape_key`] and [`Workbench::handle_escape_event`]
//! methods governing the dismissal of modal dialogues, contextual overlay popovers,
//! popup menus, text edit buffers, and keyboard focus traps in hierarchical priority order.

use super::outcome::WorkbenchOutcome;
use super::runner::Workbench;
use irisui_core::event::{KeyCode, UiEvent};
use std::fmt::Debug;
use std::hash::Hash;

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: 'static + Send + Sync,
{
    /// Evaluates whether any active modal dialog, overlay popover, popup menu, or text edit
    /// can be dismissed or cancelled via the keyboard Escape key.
    #[must_use]
    pub fn can_handle_escape(&self, app_state: Option<&AppState>) -> bool {
        if self.dialog_host.is_active() {
            return true;
        }

        if let Some(state) = app_state
            && self.overlay_host.has_escape_dismissible(state)
        {
            return true;
        }

        let t = &self.tree;
        t.is_any_modal_open()
            || t.has_open_popup()
            || t.has_open_tooltip()
            || t.event_state.has_text_input_focus()
            || t.event_state.focused.is_some()
    }

    /// Handles a global Escape key gesture, dismissing modal dialogues, popovers, and focus traps.
    ///
    /// In accordance with desktop IDE standards, cancellations are resolved in strict priority order:
    /// 1. Topmost modal dialog in [`DialogHost`]: dismissed if cancellable, or triggers attention animation.
    /// 2. Contextual overlay popovers in [`OverlayHost`]: dismissed if `dismiss_on_escape` is enabled.
    /// 3. Tagged modal dialogues in the retained `UiTree`: closed via `UiTree::close_all_modals`.
    /// 4. Open popups, cascading menus, and tooltips in the retained `UiTree`: closed via `UiTree::close_all_popups`.
    /// 5. Active text input sessions and keyboard focus in the retained `UiTree`: cleared via `UiTree::event_state`.
    pub fn handle_escape_key(&mut self, app_state: Option<&mut AppState>) -> bool {
        // 1. Topmost modal dialog in DialogHost
        if self.dialog_host.is_active() {
            let handled = self.dialog_host.handle_keyboard_input_with_modifiers(
                &KeyCode::Escape,
                true,
                false,
                &mut self.action_bus,
            );
            if handled {
                return true;
            }
        }

        // 2. Contextual overlays registered in OverlayHost
        if let Some(state) = app_state
            && self.overlay_host.handle_escape(state, &mut self.action_bus)
        {
            return true;
        }

        // 3. Tagged modal dialogs, popups, and focus states in owned UiTree
        let t = &mut self.tree;
        let mut handled = false;

        // 3a. Tagged modals (e.g. About modal, custom modal scopes)
        if t.is_any_modal_open() {
            t.close_all_modals();
            handled = true;
        }

        // 3b. Popups, context menus, cascading menus, comboboxes, and tooltips
        if t.has_open_popup() || t.has_open_tooltip() {
            t.close_all_popups();
            t.close_tooltip();
            handled = true;
        }

        // 3c. Text edit sessions and keyboard focus
        if t.event_state.has_text_input_focus() || t.event_state.focused.is_some() {
            t.event_state.clear_focus();
            t.event_state.clear_text_edit();
            handled = true;
        }

        t.event_state.clear_pressed();

        handled
    }

    /// Dispatches an event specifically for Escape cancellation, returning a complete [`WorkbenchOutcome`]
    /// if the event was an Escape key press that dismissed or cancelled any workbench state.
    pub fn handle_escape_event(
        &mut self,
        event: &UiEvent,
        app_state: Option<&mut AppState>,
    ) -> Option<WorkbenchOutcome<Action, TabId>> {
        if let UiEvent::KeyboardInput {
            key: KeyCode::Escape,
            is_pressed: true,
            ..
        } = event
            && self.handle_escape_key(app_state)
        {
            let mut outcome = self.make_outcome(true, None);
            outcome.layout_changed = true;
            outcome.escape_handled = true;
            outcome.shell_dirty = true;
            outcome.invalidation_route = Some(crate::lifecycle::EventInvalidationRoute::Shell);
            outcome.actions.extend(self.action_bus.drain());
            return Some(outcome);
        }
        None
    }

    /// Intercepts keyboard interaction when an elevated modal dialogue is active in [`DialogHost`].
    ///
    /// Modal dialogues trap keyboard navigation (`Tab`, `Shift+Tab`, `Enter`, `Escape`),
    /// resolving confirmation or cancellation gestures and preventing keystrokes from leaking
    /// down into background panel contents or dock splitters.
    ///
    /// # Arguments
    /// * `event` - Reference to the incoming [`UiEvent`].
    ///
    /// # Returns
    /// `Some(outcome)` with `consumed: true` if a modal consumed the keyboard event; `None` otherwise.
    pub fn handle_modal_keyboard_event(
        &mut self,
        event: &UiEvent,
    ) -> Option<WorkbenchOutcome<Action, TabId>> {
        if self.dialog_host.is_active()
            && let UiEvent::KeyboardInput {
                key,
                is_pressed: true,
                shift_held,
                ..
            } = event
            && self.dialog_host.handle_keyboard_input_with_modifiers(
                key,
                true,
                *shift_held,
                &mut self.action_bus,
            )
        {
            let mut outcome = self.make_outcome(true, None);
            outcome.layout_changed = true;
            outcome.escape_handled = *key == KeyCode::Escape;
            outcome.shell_dirty = true;
            outcome.invalidation_route = Some(crate::lifecycle::EventInvalidationRoute::Shell);
            outcome.actions.extend(self.action_bus.drain());
            return Some(outcome);
        }
        None
    }
}
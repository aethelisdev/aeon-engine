// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Focus synchronization, modifier query, and focus-coupled dispatch helpers for [`UiTree`].

use crate::dispatch::DispatchResult;
use crate::event::{FocusManager, InputModifiers, UiEvent};

use super::arena::UiTree;

impl UiTree {
    /// Automatically synchronizes keyboard focus with the active text editing state,
    /// dropping focus from the provided [`FocusManager`] when text editing concludes.
    ///
    /// Returns `true` if active focus was cleared during synchronization.
    #[inline]
    pub fn auto_sync_focus_state(&self, focus: &mut FocusManager) -> bool {
        focus.auto_sync_focus_state(self)
    }

    /// Dispatches an incoming [`UiEvent`] windowing input through the UI graph and
    /// automatically synchronizes the provided [`FocusManager`] with the resulting text edit state.
    #[inline]
    pub fn dispatch_event_with_focus(
        &mut self,
        event: &UiEvent,
        focus: &mut FocusManager,
    ) -> DispatchResult {
        let result = self.dispatch_event(event);
        focus.auto_sync_focus_state(self);
        result
    }

    /// Returns the persistent keyboard modifier flags (`ctrl` / `shift`) held on the tree.
    #[inline]
    #[must_use]
    pub const fn modifiers(&self) -> InputModifiers {
        self.event_state.modifiers
    }

    /// Assigns keyboard focus directly to the widget matching semantic `tag` (or clears focus when `tag == 0`).
    ///
    /// Keeps [`UiTree::event_state`] as the single source of truth by updating both
    /// `focused_tag` and resolved `focused` node ID.
    #[inline]
    pub fn set_focus_tag(&mut self, tag: u64) {
        if tag == 0 {
            self.event_state.clear_focus();
            return;
        }
        self.event_state.focused_tag = Some(tag);
        self.event_state.focused = self.find_node_by_tag(tag);
    }
}
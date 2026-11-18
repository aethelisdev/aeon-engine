// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Outside-Click Popup Dismissal Subsystem
//!
//! Provides the centralized [`Workbench::dismiss_popups_on_outside_click`] and
//! [`Workbench::handle_outside_click_event`] methods governing the declarative
//! dismissal of popup menus, elevated context menus, and dropdowns when an outside
//! click occurs on background canvases, panels, or splitters.

use super::outcome::WorkbenchOutcome;
use super::runner::Workbench;
use irisui_core::event::UiEvent;
use irisui_core::geometry::Point;
use std::fmt::Debug;
use std::hash::Hash;

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: 'static + Send + Sync,
{
    /// Evaluates whether an outside click should dismiss active popups or context menus.
    #[must_use]
    pub fn can_dismiss_popups_on_outside_click(&self, point: Point) -> bool {
        self.tree.has_open_popup() && !self.tree.is_point_inside_popup(point)
    }

    /// Dismisses active popups and context menus if `point` falls outside their interactive bounds.
    pub fn dismiss_popups_on_outside_click(&mut self, point: Point) -> bool {
        self.tree.dismiss_popups_on_outside_click(point)
    }

    /// Intercepts a [`UiEvent`] and automatically dismisses active popups if an outside
    /// [`UiEvent::MouseDown`] gesture is detected.
    pub fn handle_outside_click_event(
        &mut self,
        event: &UiEvent,
    ) -> Option<WorkbenchOutcome<Action, TabId>> {
        let UiEvent::MouseDown { point, .. } = event else {
            return None;
        };

        if self.dismiss_popups_on_outside_click(*point) {
            let mut outcome = self.make_outcome(true, None);
            outcome.popups_dismissed = true;
            outcome.layout_changed = true;
            outcome.shell_dirty = true;
            outcome.invalidation_route = Some(crate::lifecycle::EventInvalidationRoute::Shell);
            outcome.actions.extend(self.action_bus.drain());
            Some(outcome)
        } else {
            None
        }
    }
}
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
use iris_core::event::UiEvent;
use iris_core::geometry::Point;
use iris_core::tree::UiTree;
use std::fmt::Debug;
use std::hash::Hash;

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + 'static,
    Action: 'static + Send + Sync,
{
    /// Evaluates whether an outside click should dismiss active popups or context menus.
    ///
    /// # Arguments
    /// * `tree` - Optional reference to the active [`UiTree`].
    /// * `point` - Pointer coordinate where the mouse press occurred.
    ///
    /// # Returns
    /// `true` if an active popup exists and `point` is outside its bounds; `false` otherwise.
    #[must_use]
    pub fn can_dismiss_popups_on_outside_click(&self, tree: Option<&UiTree>, point: Point) -> bool {
        let Some(t) = tree else {
            return false;
        };

        if !t.has_open_popup() {
            return false;
        }

        !t.is_point_inside_popup(point)
    }

    /// Dismisses active popups and context menus if `point` falls outside their interactive bounds.
    ///
    /// Evaluates open popups in the provided [`UiTree`] against `point`. If `point` lies strictly
    /// outside all open popup hierarchies, dropdowns, and anchor tags, dismisses them and returns `true`.
    ///
    /// # Arguments
    /// * `tree` - Optional mutable reference to the active [`UiTree`].
    /// * `point` - Pointer coordinate where the mouse press occurred.
    ///
    /// # Returns
    /// `true` if an outside click was detected and popups were dismissed; `false` otherwise.
    pub fn dismiss_popups_on_outside_click(
        &mut self,
        tree: Option<&mut UiTree>,
        point: Point,
    ) -> bool {
        let Some(t) = tree else {
            return false;
        };

        t.dismiss_popups_on_outside_click(point)
    }

    /// Intercepts a [`UiEvent`] and automatically dismisses active popups if an outside
    /// [`UiEvent::MouseDown`] gesture is detected.
    ///
    /// If popups are dismissed, returns `Some(WorkbenchOutcome)` with `consumed = true`,
    /// `popups_dismissed = true`, and `layout_changed = true`. Background widgets are protected
    /// from receiving the click event.
    ///
    /// # Arguments
    /// * `event` - The platform [`UiEvent`] to evaluate.
    /// * `tree` - Optional mutable reference to the active [`UiTree`].
    ///
    /// # Returns
    /// `Some(WorkbenchOutcome)` if popups were dismissed; `None` otherwise.
    pub fn handle_outside_click_event(
        &mut self,
        event: &UiEvent,
        tree: Option<&mut UiTree>,
    ) -> Option<WorkbenchOutcome<Action>> {
        let UiEvent::MouseDown { point, .. } = event else {
            return None;
        };

        if self.dismiss_popups_on_outside_click(tree, *point) {
            let mut outcome = self.make_outcome(true, None);
            outcome.popups_dismissed = true;
            outcome.layout_changed = true;
            outcome.actions.extend(self.action_bus.drain());
            Some(outcome)
        } else {
            None
        }
    }
}
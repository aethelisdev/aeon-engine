// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Pointer Hit-Testing & Occlusion Subsystem
//!
//! Evaluates whether pointer coordinates fall over interactive application chrome,
//! elevated floating windows, active modal dialogs, or transparent 3D viewports.

use super::runner::Workbench;
use iris_core::event::{MouseButton, UiEvent};
use iris_core::geometry::{Point, Rect};
use iris_core::tree::UiTree;
use iris_dock::state::DockState;
use std::fmt::Debug;
use std::hash::Hash;

/// Tracks the state of the system pointer device across interaction events,
/// recording cursor coordinates and mouse button down states.
///
/// Designed to provide high-level, decoupled pointer telemetry to the workbench orchestrator
/// and application shells without leaking low-level window event structures.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PointerState {
    /// Most recently observed cursor coordinate in logical screen points.
    pub position: Option<Point>,

    /// Whether the primary (left) mouse button is currently held down.
    pub primary_down: bool,

    /// Whether the secondary (right) mouse button is currently held down.
    pub secondary_down: bool,

    /// Whether the middle mouse button is currently held down.
    pub middle_down: bool,
}

impl PointerState {
    /// Creates a default, uninitialized pointer state with no active buttons or coordinates.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: None,
            primary_down: false,
            secondary_down: false,
            middle_down: false,
        }
    }

    /// Updates internal cursor position and button pressed states from an incoming [`UiEvent`].
    pub fn update_from_event(&mut self, event: &UiEvent) {
        if let Some(pos) = event.pointer_position() {
            self.position = Some(pos);
        }
        match event {
            UiEvent::MouseDown { button, .. } => match button {
                MouseButton::Left => self.primary_down = true,
                MouseButton::Right => self.secondary_down = true,
                MouseButton::Middle => self.middle_down = true,
                MouseButton::Other(_) => {}
            },
            UiEvent::MouseUp { button, .. } => match button {
                MouseButton::Left => self.primary_down = false,
                MouseButton::Right => self.secondary_down = false,
                MouseButton::Middle => self.middle_down = false,
                MouseButton::Other(_) => {}
            },
            _ => {}
        }
    }

    /// Returns the most recently observed pointer position in logical points.
    #[must_use]
    #[inline]
    pub const fn position(&self) -> Option<Point> {
        self.position
    }

    /// Sets or overrides the recorded pointer position.
    #[inline]
    pub fn set_position(&mut self, position: Option<Point>) {
        self.position = position;
    }

    /// Returns whether the primary (left) mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_primary_down(&self) -> bool {
        self.primary_down
    }

    /// Returns whether the secondary (right) mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_secondary_down(&self) -> bool {
        self.secondary_down
    }

    /// Returns whether the middle mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_middle_down(&self) -> bool {
        self.middle_down
    }

    /// Returns whether any mouse button is currently held down.
    #[must_use]
    #[inline]
    pub const fn is_any_down(&self) -> bool {
        self.primary_down || self.secondary_down || self.middle_down
    }

    /// Resets all mouse button states to released.
    #[inline]
    pub fn release_all_buttons(&mut self) {
        self.primary_down = false;
        self.secondary_down = false;
        self.middle_down = false;
    }
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + 'static,
    Action: Send + Sync + 'static,
{
    /// Returns an immutable reference to the autonomous pointer state tracker.
    #[must_use]
    #[inline]
    pub fn pointer_state(&self) -> &PointerState {
        &self.pointer_state
    }

    /// Returns a mutable reference to the autonomous pointer state tracker.
    #[inline]
    pub fn pointer_state_mut(&mut self) -> &mut PointerState {
        &mut self.pointer_state
    }
    /// Evaluates whether the UI currently captures the pointer or if the pointer is positioned over
    /// any active workbench UI element (menubar, statusbar, panels, floating windows, popups).
    ///
    /// Returns `false` if the pointer is positioned over the interactive non-UI viewport canvas
    /// without any occluding UI layers.
    #[must_use]
    pub fn wants_pointer_input(
        &self,
        cursor_pos: Option<Point>,
        dock_state: &DockState<TabId>,
        tree: Option<&UiTree>,
    ) -> bool {
        let pt = cursor_pos.or(self.last_cursor_pos);
        let Some(point) = pt else {
            return false;
        };
        self.wants_pointer_at_full(
            [point.x, point.y],
            self.interactive_viewport,
            dock_state,
            tree,
        )
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, or the top menubar rather than transparent canvas/viewport areas.
    ///
    /// Uses the internal dock state from the descriptor and the configured interactive viewport.
    #[must_use]
    pub fn wants_pointer_at_default(&self, pos: [f32; 2], non_ui_viewport: Option<Rect>) -> bool {
        self.wants_pointer_at_with_dock(pos, non_ui_viewport, &self.descriptor.dock_state)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, floating windows, or elevated UI tree layers using the configured interactive viewport.
    #[must_use]
    pub fn wants_pointer_at(
        &self,
        pos: [f32; 2],
        dock_state: &DockState<TabId>,
        tree: Option<&UiTree>,
    ) -> bool {
        self.wants_pointer_at_full(pos, self.interactive_viewport, dock_state, tree)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, or floating windows using an explicitly provided docking state.
    #[must_use]
    pub fn wants_pointer_at_with_dock(
        &self,
        pos: [f32; 2],
        non_ui_viewport: Option<Rect>,
        dock_state: &DockState<TabId>,
    ) -> bool {
        self.wants_pointer_at_full(pos, non_ui_viewport, dock_state, None)
    }

    /// Evaluates whether the specified point is positioned over active workbench UI elements,
    /// modal dialogs, floating windows, or elevated UI tree layers.
    #[must_use]
    pub fn wants_pointer_at_full(
        &self,
        pos: [f32; 2],
        non_ui_viewport: Option<Rect>,
        dock_state: &DockState<TabId>,
        tree: Option<&UiTree>,
    ) -> bool {
        let point = Point::new(pos[0], pos[1]);

        // 1. Top menubar & bottom statusbar & active modal dialogs (always highest priority chrome)
        let is_over_statusbar = if self.descriptor.statusbar.is_some() {
            if let Some(win_size) = self.last_window_size {
                pos[1] >= (win_size.height - self.statusbar_height).max(0.0)
                    && pos[1] <= win_size.height
            } else {
                false
            }
        } else {
            false
        };

        if (self.descriptor.menubar.is_some() && pos[1] <= self.menubar_height)
            || is_over_statusbar
            || self.dialog_host.is_active()
        {
            return true;
        }

        // 2. Floating windows check (excluding floating window hosting the non_ui_tab)
        let in_occluding_floating = dock_state.floating_windows.iter().any(|w| {
            if let Some(non_ui) = self.non_ui_tab
                && w.tree.all_tabs().contains(&non_ui)
            {
                return false;
            }
            w.rect.contains_point(point)
        });

        // 3. Interactive non-UI viewport exclusion
        let effective_viewport = non_ui_viewport.or(self.interactive_viewport);
        if let Some(vp) = effective_viewport
            && vp.contains_point(point)
        {
            // If another floating window occludes this viewport point, UI captures pointer
            if in_occluding_floating {
                return true;
            }

            // If elevated popups, dropdowns, modals, or captured widgets in UiTree land over this point, UI captures pointer
            if let Some(t) = tree
                && t.wants_pointer_at(point)
            {
                return true;
            }

            return false;
        }

        if in_occluding_floating {
            return true;
        }

        true
    }
}
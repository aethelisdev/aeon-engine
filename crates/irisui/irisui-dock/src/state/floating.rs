// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Floating window lifecycle, dock-back origin restoration, clamping, and event delegation
//! for [`super::DockState`].

use super::DockState;
use crate::drag_drop::DropZone;
use crate::floating::FloatingWindow;
use crate::tab_viewer::TabViewer;
use crate::tree::{DockError, DockNode};
use irisui_core::Rect;
use irisui_core::event::UiEvent;

impl<T> DockState<T> {
    /// Docks a floating window back to its originating dock leaf or split anchor when available,
    /// falling back to the first active dock leaf if the original anchor no longer exists.
    pub fn dock_back_floating_window(&mut self, window_id: u64) -> Result<(), DockError> {
        let (origin_leaf, origin_split_fallback) = self
            .floating_windows
            .iter()
            .find(|w| w.id == window_id)
            .map(|w| (w.origin_leaf, w.origin_split_fallback))
            .ok_or(DockError::NodeNotFound)?;

        if let Some(leaf) = origin_leaf
            && matches!(self.tree.get(leaf), Some(DockNode::Leaf { .. }))
        {
            return self.dock_floating_window(window_id, leaf, DropZone::Center);
        }
        if let Some((sib_leaf, zone)) = origin_split_fallback
            && matches!(self.tree.get(sib_leaf), Some(DockNode::Leaf { .. }))
        {
            return self.dock_floating_window(window_id, sib_leaf, zone);
        }
        let fallback_leaf = self.tree.find_first_leaf().unwrap_or_default();
        self.dock_floating_window(window_id, fallback_leaf, DropZone::Center)
    }

    /// Checks whether the specified coordinate point falls inside any active floating window.
    #[inline]
    pub fn is_point_over_floating_window(&self, point: irisui_core::Point) -> bool {
        self.floating_windows
            .iter()
            .any(|w| w.contains_point(point))
    }

    /// Checks whether the specified coordinate point falls inside any active floating window
    /// or within its interactive border resize margin.
    #[inline]
    #[must_use]
    pub fn is_point_over_floating_window_or_border(
        &self,
        point: irisui_core::Point,
        margin: f32,
    ) -> bool {
        self.floating_windows.iter().any(|w| {
            let expanded = Rect::new(
                w.rect.x - margin,
                w.rect.y - margin,
                w.rect.width + margin * 2.0,
                w.rect.height + margin * 2.0,
            );
            expanded.contains_point(point)
        })
    }

    /// Returns `true` if a floating window is actively being dragged or resized.
    #[inline]
    #[must_use]
    pub const fn has_active_floating_drag(&self) -> bool {
        self.active_floating_drag.is_some()
    }

    /// Evaluates an incoming interaction event specifically against managed floating windows.
    ///
    /// Coordinates titlebar dragging, 8-directional border resizing, close button, dock-back button,
    /// tab selection, and Z-order focus promotion without requiring complete dock chrome construction.
    ///
    /// # Arguments
    /// * `viewer` - Reference to tab viewer providing tab titles and custom metrics.
    /// * `event` - Raw interaction event to process.
    /// * `clamp_bounds` - Boundary constraints ensuring windows remain accessible and visible.
    pub fn handle_floating_event<V: TabViewer<T>>(
        &mut self,
        viewer: &V,
        event: &UiEvent,
        clamp_bounds: crate::floating::FloatingWindowClampBounds,
    ) -> crate::events::floating::FloatingInteractionResponse<T>
    where
        T: Clone + Copy + PartialEq,
    {
        crate::events::floating::handle_floating_window_event(self, viewer, event, clamp_bounds)
    }

    /// Synchronizes active floating window bounding rectangles into the provided target buffer.
    ///
    /// Clears and repopulates `target` without reallocating underlying capacity when pre-allocated,
    /// enabling zero-allocation occlusion tracking and text culling on per-frame hot loops.
    #[inline]
    pub fn sync_floating_window_rects(&self, target: &mut Vec<Rect>) {
        target.clear();
        target.extend(self.floating_windows.iter().map(|w| w.rect));
    }

    /// Enforces boundary clamping constraints across all detached floating windows.
    ///
    /// Ensures title bars remain interactable and grab handles remain visible within screen extents.
    #[inline]
    pub fn clamp_floating_windows(&mut self, bounds: &crate::floating::FloatingWindowClampBounds) {
        crate::floating::clamp_floating_windows(&mut self.floating_windows, bounds);
    }

    /// Checks whether the specified tab is hosted inside any active floating window.
    ///
    /// Executes in O(N_windows * N_tabs) with zero heap allocations.
    #[inline]
    #[must_use]
    pub fn is_tab_in_floating_window(&self, tab: &T) -> bool
    where
        T: PartialEq,
    {
        self.floating_windows
            .iter()
            .any(|w| w.tree.find_tab(tab).is_some())
    }

    /// Resolves the content rectangle of the specified tab if it resides in an active floating window.
    #[inline]
    #[must_use]
    pub fn find_floating_tab_content_rect(&self, tab: &T, title_bar_height: f32) -> Option<Rect>
    where
        T: PartialEq,
    {
        crate::floating::find_active_tab_content_rect(&self.floating_windows, tab, title_bar_height)
    }

    /// Constructs an autonomous [`crate::floating::FloatingWindowManager`] mirroring current floating windows.
    #[must_use]
    pub fn floating_manager(
        &self,
        bounds: crate::floating::FloatingWindowClampBounds,
    ) -> crate::floating::FloatingWindowManager<T>
    where
        T: Clone,
    {
        crate::floating::FloatingWindowManager::new(bounds)
            .with_windows(self.floating_windows.clone())
    }

    /// Opens an autonomous floating window containing the specified tab or focuses it if already present.
    ///
    /// If the tab is already open within an existing floating window, that window
    /// is brought to the front of the z-order and its ID is returned without creating a new window.
    /// If not present, a new floating window is allocated, appended to the top of the z-order,
    /// and its newly assigned ID is returned.
    ///
    /// # Parameters
    /// - `tab`: The panel or tool data item to host in the floating window.
    /// - `title`: Window title display text shown on the floating window titlebar.
    /// - `rect`: Initial bounds rectangle of the floating window in workspace screen space.
    pub fn open_or_focus_floating_tab(
        &mut self,
        tab: T,
        title: impl Into<String>,
        rect: Rect,
    ) -> u64
    where
        T: PartialEq + Clone,
    {
        if let Some(win_id) = self
            .floating_windows
            .iter()
            .find(|w| w.tree.find_tab(&tab).is_some())
            .map(|w| w.id)
        {
            self.bring_floating_to_front(win_id);
            return win_id;
        }

        let next_id = self
            .floating_windows
            .iter()
            .map(|w| w.id)
            .max()
            .unwrap_or(0)
            + 1;
        let window = FloatingWindow::new(next_id, title, rect, vec![tab]);
        self.floating_windows.push(window);
        next_id
    }
}
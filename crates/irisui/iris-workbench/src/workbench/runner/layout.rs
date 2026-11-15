// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Layout and Partial Invalidation Pipeline
//!
//! Provides layout calculation, text preparation query utilities, and $O(M)$ partial
//! leaf container reconstruction routines on [`Workbench`].

use super::coordinator::Workbench;
use crate::layout_sync::dual_layout::SubtreeEngines;
use iris_core::WidgetId;
use iris_core::geometry::Rect;
use iris_dock::UiNotifier;
use iris_layout::LayoutError;
use std::fmt::Debug;
use std::hash::Hash;

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + 'static,
    Action: Send + Sync + 'static,
{
    /// Returns `true` if any partial subtree rebuild or layout pass triggered text preparation.
    #[inline]
    #[must_use]
    pub fn needs_text_prepare(&self) -> bool {
        self.dual_layout.needs_text_prepare()
    }

    /// Explicitly flags text preparation as required on the active frame.
    #[inline]
    pub fn mark_needs_text_prepare(&mut self) {
        self.dual_layout.mark_needs_text_prepare();
    }

    /// Resets the text preparation flag to `false`.
    #[inline]
    pub fn clear_needs_text_prepare(&mut self) {
        self.dual_layout.clear_needs_text_prepare();
    }

    /// Consumes and returns the current text preparation flag, resetting it to `false`.
    ///
    /// Useful for single-pass render loops to query and clear the flag atomically.
    #[inline]
    pub fn take_needs_text_prepare(&mut self) -> bool {
        self.dual_layout.take_needs_text_prepare()
    }

    /// Rebuilds all dirty dock leaf panels flagged in `notifier` using the workbench's
    /// internal [`crate::layout_sync::dual_layout::DualLayoutEliminator`] and registered [`crate::descriptors::panel_registry::WorkbenchPanelRegistry`].
    ///
    /// Automatically marks [`Self::needs_text_prepare`] if any panels were reconstructed.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn rebuild_dirty_panels<F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        context: &mut AppState,
        leaf_resolver: F,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
    {
        self.dual_layout.rebuild_dirty_panels(
            engines,
            notifier,
            &mut self.panel_registry,
            context,
            leaf_resolver,
        )
    }

    /// Computes the active dock workspace bounding rectangle excluding the menubar and statusbar.
    #[inline]
    #[must_use]
    pub fn compute_workspace_rect(&self, screen_w: f32, screen_h: f32) -> Rect {
        let top_inset = if self.descriptor.menubar.is_some() {
            self.menubar_height
        } else {
            0.0
        };
        let bottom_inset = if self.descriptor.statusbar.is_some() {
            self.statusbar_height
        } else {
            0.0
        };
        let height = (screen_h - top_inset - bottom_inset).max(0.0);
        Rect::new(0.0, top_inset, screen_w, height)
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Layout and Partial Invalidation Pipeline
//!
//! Provides layout calculation, text preparation query utilities, and $O(M)$ partial
//! leaf container reconstruction routines on [`Workbench`].

use super::coordinator::Workbench;
use crate::layout_sync::dual_layout::SubtreeEngines;
use irisui_core::WidgetId;
use irisui_core::geometry::Rect;
use irisui_dock::{DockState, DropZone, TabPlacement};
use irisui_layout::LayoutError;
use std::fmt::Debug;
use std::hash::Hash;

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
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

    /// Rebuilds all dirty dock leaf panels flagged in [`Self::notifier`] using the workbench's
    /// internal [`crate::layout_sync::dual_layout::DualLayoutEliminator`] and registered [`crate::descriptors::panel_registry::WorkbenchPanelRegistry`].
    pub fn rebuild_dirty_panels<F>(
        &mut self,
        context: &mut AppState,
        leaf_resolver: F,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
    {
        let mut engines = SubtreeEngines::new(
            &mut self.tree,
            &mut self.layout_engine,
            &mut self.text_system,
        );
        self.dual_layout.rebuild_dirty_panels(
            &mut engines,
            &mut self.notifier,
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

    /// Returns `true` if `tab` is open anywhere in the workbench docking tree or floating windows.
    #[inline]
    #[must_use]
    pub fn is_panel_open(&self, tab: &TabId) -> bool {
        self.descriptor.dock_state.contains_tab(tab)
    }

    /// Computes a 64-bit visibility bitmask over `tabs`, setting bit `i` when `tabs[i]` is open.
    #[inline]
    #[must_use]
    pub fn tab_visibility_mask(&self, tabs: &[TabId]) -> u64 {
        self.descriptor.dock_state.tab_visibility_mask(tabs)
    }

    /// Focuses `tab` if open, or opens it according to the registered [`PanelMetadata::placement`]
    /// rule, marking all panels for redraw.
    pub fn open_or_focus_panel(&mut self, tab: TabId) {
        let fallback_placement = TabPlacement::Docked {
            partner: None,
            anchor: None,
            zone: DropZone::Center,
        };
        let screen_size = self.screen_size();
        let meta_title: Option<&str>;
        let placement: TabPlacement<TabId>;
        if let Some(meta) = self.panel_registry.panel_metadata(&tab) {
            meta_title = Some(meta.title.as_str());
            placement = meta.placement.unwrap_or(fallback_placement);
        } else {
            meta_title = None;
            placement = fallback_placement;
        }
        let fallback_title = if meta_title.is_none() {
            format!("{tab:?}")
        } else {
            String::new()
        };
        let title = meta_title.unwrap_or(fallback_title.as_str());
        self.descriptor
            .dock_state
            .open_or_focus_tab(tab, title, &placement, screen_size);
        self.notifier.tag_all();
    }

    /// Closes `tab` from floating windows or the main dock tree and flags affected panels dirty.
    ///
    /// Returns `true` if the tab was open and removed.
    pub fn close_panel(&mut self, tab: &TabId) -> bool {
        let closed = self.descriptor.dock_state.close_tab_by_id(tab);
        if closed {
            self.notifier.tag_all();
        }
        closed
    }

    /// Toggles `tab` visibility: closes it when open, or opens/focuses it when closed.
    pub fn toggle_panel(&mut self, tab: TabId) {
        if !self.close_panel(&tab) {
            self.open_or_focus_panel(tab);
        }
    }

    /// Replaces the active docking state (e.g. resetting to factory defaults) and invalidates all panels.
    pub fn reset_dock_state(&mut self, dock_state: DockState<TabId>) {
        self.descriptor.dock_state = dock_state;
        self.notifier.tag_all();
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Workbench Layout and Partial Invalidation Pipeline
//!
//! Provides layout calculation, text preparation query utilities, and $O(M)$ partial
//! leaf container reconstruction routines on [`Workbench`].

use super::coordinator::Workbench;
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::layout_sync::dual_layout::SubtreeEngines;
use crate::lifecycle::LifecycleInputs;
use irisui_core::WidgetId;
use irisui_core::geometry::{Rect, Size};
use irisui_dock::floating::{
    FloatingWindowClampBounds, FloatingWindowStyle, MIN_VISIBLE_TITLE_BAR,
};
use irisui_dock::{
    DEFAULT_SPLITTER_THICKNESS, DEFAULT_TAB_BAR_HEIGHT, DockState, DropZone, TabPlacement,
    UiNotifier, compute_dock_layout_with_viewer,
};
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

    /// Constructs floating window clamp constraints matching the active menubar/statusbar insets.
    #[inline]
    #[must_use]
    pub fn floating_clamp_bounds(&self, logical_size: Size) -> FloatingWindowClampBounds {
        FloatingWindowClampBounds::new(logical_size.width, logical_size.height)
            .with_min_y(if self.descriptor.menubar.is_some() {
                self.menubar_height
            } else {
                0.0
            })
            .with_bottom_inset(if self.descriptor.statusbar.is_some() {
                self.statusbar_height
            } else {
                0.0
            })
            .with_title_bar_height(DEFAULT_TAB_BAR_HEIGHT.max(MIN_VISIBLE_TITLE_BAR))
    }

    /// Clamps detached floating windows within the active workspace and resolves the sanitized
    /// screen rectangle of [`Self::non_ui_tab`] (cached across steady-state frames), updating
    /// [`Self::interactive_viewport`] in place.
    pub fn prepare_viewport_layout(&mut self, logical_size: Size) -> Rect {
        if !logical_size.width.is_finite()
            || !logical_size.height.is_finite()
            || logical_size.width <= 0.0
            || logical_size.height <= 0.0
        {
            self.interactive_viewport = Some(Rect::ZERO);
            return Rect::ZERO;
        }

        let clamp_bounds = self.floating_clamp_bounds(logical_size);
        self.descriptor
            .dock_state
            .clamp_floating_windows(&clamp_bounds);

        let workspace_rect = self.compute_workspace_rect(logical_size.width, logical_size.height);
        let dock_state = &self.descriptor.dock_state;
        let floating_hash = LifecycleInputs::hash_floating_rects(
            dock_state.floating_windows.iter().map(|w| w.rect),
        );
        let dock_key = (
            dock_state.revision(),
            floating_hash,
            workspace_rect,
            dock_state.maximized_leaf.is_some(),
        );
        let is_dock_dragging =
            dock_state.active_splitter.is_some() || dock_state.active_drag.is_some();

        let viewport_rect =
            if !is_dock_dragging && self.cached_viewport_layout_key == Some(dock_key) {
                self.cached_non_ui_viewport_rect
            } else {
                let resolved = self.non_ui_tab.map_or(Rect::ZERO, |target_tab| {
                    let raw = dock_state
                        .find_floating_tab_content_rect(
                            &target_tab,
                            FloatingWindowStyle::DEFAULT_TITLE_BAR_HEIGHT,
                        )
                        .unwrap_or_else(|| {
                            let computed = compute_dock_layout_with_viewer(
                                &dock_state.tree,
                                workspace_rect,
                                DEFAULT_SPLITTER_THICKNESS,
                                DEFAULT_TAB_BAR_HEIGHT,
                                &self.panel_registry.tab_viewer(),
                            );
                            computed
                                .leaves
                                .iter()
                                .find(|leaf| leaf.tabs.get(leaf.active_tab) == Some(&target_tab))
                                .map_or(Rect::ZERO, |leaf| leaf.content_rect)
                        });
                    if raw.width > 0.0
                        && raw.height > 0.0
                        && raw.x.is_finite()
                        && raw.y.is_finite()
                        && raw.width.is_finite()
                        && raw.height.is_finite()
                    {
                        raw
                    } else {
                        Rect::ZERO
                    }
                });
                self.cached_viewport_layout_key = if is_dock_dragging {
                    None
                } else {
                    Some(dock_key)
                };
                self.cached_non_ui_viewport_rect = resolved;
                resolved
            };

        self.interactive_viewport = Some(viewport_rect);
        viewport_rect
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

    /// Opens or focuses `tab` inside a caller-supplied [`DockState`], marking panels dirty.
    pub(super) fn open_or_focus_panel_in(
        panel_registry: &WorkbenchPanelRegistry<TabId, AppState>,
        dock_state: &mut DockState<TabId>,
        notifier: &mut UiNotifier<TabId>,
        screen_size: Option<Size>,
        tab: TabId,
    ) {
        let fallback_placement = TabPlacement::Docked {
            partner: None,
            anchor: None,
            zone: DropZone::Center,
        };
        let meta_title: Option<&str>;
        let placement: TabPlacement<TabId>;
        if let Some(meta) = panel_registry.panel_metadata(&tab) {
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
        dock_state.open_or_focus_tab(tab, title, &placement, screen_size);
        notifier.tag_all();
    }

    /// Closes `tab` inside a caller-supplied [`DockState`] and flags panels dirty when removed.
    pub(super) fn close_panel_in(
        dock_state: &mut DockState<TabId>,
        notifier: &mut UiNotifier<TabId>,
        tab: &TabId,
    ) -> bool {
        let closed = dock_state.close_tab_by_id(tab);
        if closed {
            notifier.tag_all();
        }
        closed
    }

    /// Toggles `tab` visibility inside a caller-supplied [`DockState`].
    pub(super) fn toggle_panel_in(
        panel_registry: &WorkbenchPanelRegistry<TabId, AppState>,
        dock_state: &mut DockState<TabId>,
        notifier: &mut UiNotifier<TabId>,
        screen_size: Option<Size>,
        tab: TabId,
    ) {
        if !Self::close_panel_in(dock_state, notifier, &tab) {
            Self::open_or_focus_panel_in(panel_registry, dock_state, notifier, screen_size, tab);
        }
    }

    /// Focuses `tab` if open, or opens it according to the registered [`PanelMetadata::placement`]
    /// rule, marking all panels for redraw.
    pub fn open_or_focus_panel(&mut self, tab: TabId) {
        let screen_size = self.screen_size();
        Self::open_or_focus_panel_in(
            &self.panel_registry,
            &mut self.descriptor.dock_state,
            &mut self.notifier,
            screen_size,
            tab,
        );
    }

    /// Closes `tab` from floating windows or the main dock tree and flags affected panels dirty.
    ///
    /// Returns `true` if the tab was open and removed.
    pub fn close_panel(&mut self, tab: &TabId) -> bool {
        Self::close_panel_in(&mut self.descriptor.dock_state, &mut self.notifier, tab)
    }

    /// Toggles `tab` visibility: closes it when open, or opens/focuses it when closed.
    pub fn toggle_panel(&mut self, tab: TabId) {
        let screen_size = self.screen_size();
        Self::toggle_panel_in(
            &self.panel_registry,
            &mut self.descriptor.dock_state,
            &mut self.notifier,
            screen_size,
            tab,
        );
    }

    /// Replaces the active docking state (e.g. resetting to factory defaults) and invalidates all panels.
    pub fn reset_dock_state(&mut self, dock_state: DockState<TabId>) {
        self.descriptor.dock_state = dock_state;
        self.notifier.tag_all();
    }

    /// Synchronizes logical workbench screen bounds from physical pixel dimensions and the
    /// active UI scale factor, returning `Some((logical_w, logical_h))` when updated.
    ///
    /// Ignores zero-sized minimized windows and non-positive or non-finite scale factors.
    pub fn sync_physical_window_size(
        &mut self,
        physical_width: u32,
        physical_height: u32,
    ) -> Option<(f32, f32)> {
        let logical =
            compute_logical_screen_size(physical_width, physical_height, self.scale_factor())?;
        self.set_screen_size(logical);
        Some(logical)
    }

    /// Propagates shell or panel invalidation flags from a [`WorkbenchOutcome`](crate::WorkbenchOutcome)
    /// into [`Self::notifier`].
    #[inline]
    pub fn sync_outcome_invalidation<OAction>(
        &mut self,
        outcome: &crate::WorkbenchOutcome<OAction, TabId>,
    ) {
        if outcome.shell_dirty {
            self.notifier.tag_all();
        } else if let Some(panel) = outcome.invalidated_panel {
            self.notifier.tag_redraw(panel);
        }
    }
}

/// Computes logical screen dimensions from physical pixel extents and an active UI scale factor.
///
/// Returns `None` when either physical extent is zero (e.g. minimized window) or when the scale
/// factor is non-finite or non-positive.
#[inline]
#[must_use]
pub fn compute_logical_screen_size(
    physical_width: u32,
    physical_height: u32,
    scale_factor: f32,
) -> Option<(f32, f32)> {
    if physical_width == 0
        || physical_height == 0
        || !scale_factor.is_finite()
        || scale_factor <= 0.0
    {
        return None;
    }
    Some((
        physical_width as f32 / scale_factor,
        physical_height as f32 / scale_factor,
    ))
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Application Shell Assembly & Lifecycle Update Engine
//!
//! Provides the primary shell rendering and lifecycle synchronization pipelines for [`Workbench`].
//! Encapsulates full retained shell topology assembly, dock leaf synchronization,
//! modal dialog projection, contextual overlays, and granular $O(M)$ partial panel updates.

use std::fmt::Debug;
use std::hash::Hash;

use iris_core::WidgetId;
use iris_core::geometry::{Point, Rect, Size};
use iris_dock::UiNotifier;
use iris_dock::area::runner::DockArea;
use iris_dock::chrome::DockChromeFrame;
use iris_layout::LayoutError;
use iris_widgets::declarative::UiScope;

use super::runner::Workbench;
use crate::layout_sync::dual_layout::SubtreeEngines;
use crate::layout_sync::leaf_pipeline::synchronize_dock_frame;
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Renders the complete application shell into the retained widget tree.
    ///
    /// Constructs:
    /// 1. Top application menubar if configured in [`WorkbenchDescriptor`](crate::descriptors::workbench::WorkbenchDescriptor).
    /// 2. Docking area via [`DockArea`] and registered [`WorkbenchPanelRegistry`](crate::descriptors::panel_registry::WorkbenchPanelRegistry) panels.
    /// 3. Bottom statusbar if configured in [`WorkbenchDescriptor`](crate::descriptors::workbench::WorkbenchDescriptor).
    /// 4. Shapes typography across the entire shell tree.
    /// 5. Solves global Taffy flexbox layout.
    /// 6. Synchronizes and seals all dock leaf containers as Invalidation Boundaries.
    /// 7. Renders active modal dialogs from [`DialogHost`](crate::dialogs::host::DialogHost).
    /// 8. Renders active overlay hooks from [`OverlayHost`](crate::hooks::overlay::OverlayHost).
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation or subtree solving encounters invalid geometry.
    pub fn render_shell(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        context: &mut AppState,
        screen_size: Size,
        cursor: Option<Point>,
        is_cursor_occluded: bool,
    ) -> Result<DockChromeFrame<TabId>, LayoutError> {
        let root = engines.tree.create_root().map_err(|e| {
            LayoutError::Taffy(format!("Root widget creation failed for shell: {e:?}"))
        })?;

        let mut scope = UiScope::new(engines.tree, root);

        // 1. Top Application Menubar
        self.descriptor
            .render_menubar(&mut scope, screen_size.width);

        // 2. Docking Workspace Area
        let workspace_rect = self.compute_workspace_rect(screen_size.width, screen_size.height);
        let dock_frame = {
            let mut tab_viewer = self.panel_registry.active_tab_viewer(context);
            DockArea::from_ref(&self.descriptor.dock_state)
                .bounds(workspace_rect)
                .style(self.dock_style)
                .cursor_pos(cursor.unwrap_or_default())
                .cursor_occluded(is_cursor_occluded)
                .show(&mut scope, &mut tab_viewer)
        };

        // 3. Bottom Status Bar
        self.descriptor
            .render_statusbar(&mut scope, screen_size.width, screen_size.height);

        // Finalize coarse shell bounds
        scope.finish_layout(Rect::new(0.0, 0.0, screen_size.width, screen_size.height));

        // 4. Global Typography & Flexbox Layout
        engines.text_system.measure_subtree_text(engines.tree, root);
        engines
            .layout_engine
            .compute_layout(engines.tree, screen_size)?;

        // 5. Dock Leaf Boundary Sealing & Typography Measurement
        synchronize_dock_frame(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            &dock_frame,
        )?;

        // 6. Active Framework Modal Dialogs
        if self.dialog_host.is_active() {
            let mut modal_scope = UiScope::new(engines.tree, root);
            let screen_rect = Rect::new(0.0, 0.0, screen_size.width, screen_size.height);
            self.dialog_host
                .render(&mut modal_scope, screen_rect, false, &mut self.action_bus);
        }

        // 7. Active Contextual Overlay Hooks
        if self.overlay_host.has_active_overlays(context) {
            let mut overlay_scope = UiScope::new(engines.tree, root);
            let screen_rect = Rect::new(0.0, 0.0, screen_size.width, screen_size.height);
            self.overlay_host.render(
                &mut overlay_scope,
                screen_rect,
                context,
                &mut self.action_bus,
            );
        }

        self.dual_layout.mark_needs_text_prepare();
        self.dock_frame = Some(dock_frame.clone());

        Ok(dock_frame)
    }

    /// Autonomously rebuilds all dirty docking leaf panels flagged in [`UiNotifier`].
    ///
    /// Automatically detects internal boundary invalidations, maps dirty leaf containers
    /// to active tabs via the retained [`DockChromeFrame`], and invokes [`DualLayoutEliminator`](crate::layout_sync::dual_layout::DualLayoutEliminator)
    /// to recompute affected subtrees in $O(M)$ time with zero root destruction.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_dirty_panels_autonomous(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        context: &mut AppState,
    ) -> Result<usize, LayoutError> {
        let dock_frame_ref = self.dock_frame.as_ref();
        self.dual_layout.rebuild_dirty_panels_with_boundaries(
            engines,
            notifier,
            &mut self.panel_registry,
            context,
            |tab_id| dock_frame_ref.and_then(|df| df.leaf_container(tab_id)),
            |boundary_id| dock_frame_ref.and_then(|df| df.tab_for_leaf_container(boundary_id)),
        )
    }

    /// Rebuilds an individual dirty dock leaf panel in-place using [`DualLayoutEliminator`](crate::layout_sync::dual_layout::DualLayoutEliminator)
    /// and the registered [`WorkbenchPanelRegistry`](crate::descriptors::panel_registry::WorkbenchPanelRegistry).
    ///
    /// Clears the panel in `notifier` and flags text preparation if rebuilt.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_single_panel<F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        context: &mut AppState,
        panel_id: TabId,
        mut leaf_resolver: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnMut(TabId) -> Option<WidgetId>,
    {
        let Some(leaf_id) = leaf_resolver(panel_id) else {
            notifier.clear_panel(panel_id);
            return Ok(false);
        };
        self.dual_layout.rebuild_single_panel_notified(
            engines,
            notifier,
            &mut self.panel_registry,
            context,
            panel_id,
            leaf_id,
        )
    }

    /// Autonomously rebuilds an individual dirty dock leaf panel in-place.
    ///
    /// Resolves the leaf container for `panel_id` from the retained [`DockChromeFrame`],
    /// invokes the panel drawer via [`DualLayoutEliminator::rebuild_panel`](crate::layout_sync::dual_layout::DualLayoutEliminator::rebuild_panel),
    /// clears the panel in `notifier`, and flags text preparation.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn rebuild_single_panel_autonomous(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        context: &mut AppState,
        panel_id: TabId,
    ) -> Result<bool, LayoutError> {
        let Some(dock_frame) = &self.dock_frame else {
            return Ok(false);
        };
        let Some(leaf_id) = dock_frame.leaf_container(panel_id) else {
            notifier.clear_panel(panel_id);
            return Ok(false);
        };
        self.dual_layout.rebuild_single_panel_notified(
            engines,
            notifier,
            &mut self.panel_registry,
            context,
            panel_id,
            leaf_id,
        )
    }

    /// Rebuilds an individual leaf container directly using a custom declarative drawing closure.
    ///
    /// Delegates directly to [`DualLayoutEliminator::rebuild_single_panel_leaf`](crate::layout_sync::dual_layout::DualLayoutEliminator::rebuild_single_panel_leaf).
    ///
    /// # Errors
    /// Returns [`LayoutError`] if boundary layout computation encounters invalid geometry.
    pub fn rebuild_single_panel_leaf<F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        leaf_id: WidgetId,
        render_fn: F,
    ) -> Result<bool, LayoutError>
    where
        F: FnOnce(&mut UiScope<'_>),
    {
        engines.rebuild_single_panel_leaf_with_eliminator(&mut self.dual_layout, leaf_id, render_fn)
    }

    /// Executes the complete autonomous 3-branch lifecycle update pass for the workbench shell.
    ///
    /// Evaluates [`WorkbenchLifecycle`](crate::lifecycle::WorkbenchLifecycle):
    /// - **Branch 1 (Full Shell)**: Reconstructs root, menubar, docking hierarchy, statusbar,
    ///   modals, and overlays from scratch. Clears `notifier`.
    /// - **Branch 2 (Granular Subtree Invalidation)**: Rebuilds dirty dock leaf panels in-place
    ///   using $O(M)$ Taffy boundaries without touching root or clean sibling panels.
    /// - **Branch 3 (In-Place Volatile / Dormant)**: Skips retained tree mutations.
    ///
    /// # Returns
    /// The evaluated [`LifecycleBranch`].
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn update_shell(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        context: &mut AppState,
        inputs: &LifecycleInputs,
        is_cursor_occluded: bool,
    ) -> Result<LifecycleBranch<TabId>, LayoutError> {
        self.lifecycle.ingest_notifier(notifier);
        let branch = self.lifecycle.evaluate(*inputs);

        match &branch {
            LifecycleBranch::FullShellRebuild => {
                engines.tree.clear();
                let screen_size = Size::new(inputs.dimensions.0, inputs.dimensions.1);
                let cursor = self.last_cursor_pos;
                self.render_shell(engines, context, screen_size, cursor, is_cursor_occluded)?;
                notifier.clear_all();
            }
            LifecycleBranch::GranularPanels(_) => {
                let _ = self.rebuild_dirty_panels_autonomous(engines, notifier, context)?;
            }
            LifecycleBranch::InPlaceRefresh | LifecycleBranch::Dormant => {
                // Zero retained tree mutations
            }
        }

        Ok(branch)
    }

    /// Returns an immutable reference to the active docking interaction frame, if available.
    #[inline]
    #[must_use]
    pub fn active_dock_frame(&self) -> Option<&DockChromeFrame<TabId>> {
        self.dock_frame.as_ref()
    }

    /// Forces a full shell reconstruction on the subsequent update turn.
    #[inline]
    pub fn force_shell_rebuild(&mut self) {
        self.lifecycle.mark_shell_dirty();
    }
}
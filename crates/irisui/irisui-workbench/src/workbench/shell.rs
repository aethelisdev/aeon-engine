// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Application Shell Assembly & Lifecycle Update Engine
//!
//! Provides the primary shell rendering and lifecycle synchronization pipelines for [`Workbench`].
//! Encapsulates full retained shell topology assembly, dock leaf synchronization,
//! modal dialog projection, contextual overlays, and granular $O(M)$ partial panel updates.

use std::fmt::Debug;
use std::hash::Hash;

use irisui_core::geometry::{Point, Rect, Size};
use irisui_core::{UiTree, WidgetId};
use irisui_dock::UiNotifier;
use irisui_dock::area::runner::DockArea;
use irisui_dock::chrome::DockChromeFrame;
use irisui_layout::LayoutError;
use irisui_widgets::declarative::UiScope;

use super::runner::Workbench;
use crate::layout_sync::dual_layout::SubtreeEngines;
use crate::layout_sync::leaf_pipeline::synchronize_dock_frame;
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};

/// Parameters passed into [`Workbench::sync_frame_state`] to synchronize frame-level UI state.
pub struct WorkbenchFrameSyncParams<'a, TabId: Eq + std::hash::Hash + Copy + Send + Sync + 'static>
{
    /// Active docking state.
    pub dock: &'a irisui_dock::DockState<TabId>,
    /// Panel invalidation notifier updated with selection, domain, and lifecycle invalidations.
    pub notifier: &'a mut irisui_dock::UiNotifier<TabId>,
    /// Active UI scale factor.
    pub scale_factor: f32,
    /// Current opaque domain selection identifier.
    pub selection_id: Option<u64>,
    /// Current external domain/history revision counter.
    pub domain_revision: u64,
}

/// Outcome returned by [`Workbench::sync_frame_state`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchFrameSync<TabId> {
    /// Caret blink update produced by synchronizing [`irisui_text::TextSystem`].
    pub caret_blink: irisui_text::CaretBlinkUpdate,
    /// Whether the opaque domain selection identifier changed this frame.
    pub selection_changed: bool,
    /// Whether the external domain revision counter changed this frame.
    pub domain_revision_changed: bool,
    /// Evaluated 3-branch shell/panel lifecycle decision.
    pub lifecycle_branch: LifecycleBranch<TabId>,
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Creates the shell root widget bound to `screen_size` so out-of-flow
    /// `Position::Absolute` dock regions and overlays never collapse root bounds to `(0, 0)`.
    pub(super) fn create_viewport_root(
        tree: &mut UiTree,
        screen_size: Size,
    ) -> Result<WidgetId, LayoutError> {
        let root = tree.create_root().map_err(|e| {
            LayoutError::Taffy(format!("Root widget creation failed for shell: {e:?}"))
        })?;
        if screen_size.width > 0.0
            && screen_size.height > 0.0
            && let Some(root_node) = tree.get_mut(root)
        {
            root_node.style.width = Some(screen_size.width);
            root_node.style.height = Some(screen_size.height);
        }
        Ok(root)
    }

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
        if screen_size.width.is_finite() && screen_size.height.is_finite() {
            self.last_window_size = Some(Size::new(
                screen_size.width.max(0.0),
                screen_size.height.max(0.0),
            ));
        }
        engines.tree.set_theme(self.theme);

        let root = Self::create_viewport_root(engines.tree, screen_size)?;

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
        if engines.tree.root().is_none() || self.dock_frame.is_none() {
            self.lifecycle.mark_shell_dirty();
        }
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
                engines.tree.set_theme(self.theme);
                let _ = self.rebuild_dirty_panels_autonomous(engines, notifier, context)?;
            }
            LifecycleBranch::InPlaceRefresh => {
                let volatile_res = self.volatile_hooks.refresh_in_place(engines.tree, context);
                if volatile_res.needs_text_prepare {
                    self.dual_layout.mark_needs_text_prepare();
                }
            }
            LifecycleBranch::Dormant => {
                // Zero retained tree mutations
            }
        }

        let _ = self.sync_caret_blink(engines.text_system, engines.tree);

        Ok(branch)
    }

    /// Synchronizes `notifier` and evaluates [`WorkbenchLifecycle`](crate::lifecycle::WorkbenchLifecycle)
    /// directly from a [`irisui_dock::DockState`] plus host-provided lifecycle flags.
    pub fn evaluate_dock_lifecycle(
        &mut self,
        dock: &irisui_dock::DockState<TabId>,
        notifier: &irisui_dock::UiNotifier<TabId>,
        scale_factor: f32,
        host_inputs: impl FnOnce(LifecycleInputs) -> LifecycleInputs,
    ) -> LifecycleBranch<TabId> {
        let screen = self.screen_size().unwrap_or_default();
        let base = LifecycleInputs::from_dock(
            dock,
            (screen.width, screen.height),
            scale_factor,
            self.theme_revision,
        );
        self.lifecycle.ingest_notifier(notifier);
        self.lifecycle.evaluate(host_inputs(base))
    }

    /// Synchronizes caret blink, opaque selection, domain revision, and dock lifecycle in a single
    /// library call, forwarding any resulting shell or panel invalidations into `params.notifier`.
    pub fn sync_frame_state(
        &mut self,
        text_system: &mut irisui_text::TextSystem,
        tree: &mut irisui_core::UiTree,
        params: WorkbenchFrameSyncParams<'_, TabId>,
        host_inputs: impl FnOnce(LifecycleInputs) -> LifecycleInputs,
    ) -> WorkbenchFrameSync<TabId> {
        let caret_blink = self.sync_caret_blink(text_system, tree);
        let selection_changed =
            self.sync_selection_with_notifier(params.selection_id, params.notifier);
        let domain_revision_changed =
            self.sync_domain_revision_with_notifier(params.domain_revision, params.notifier);
        let lifecycle_branch = self.evaluate_dock_lifecycle(
            params.dock,
            params.notifier,
            params.scale_factor,
            |inputs| host_inputs(inputs.with_domain_revision(params.domain_revision)),
        );
        match &lifecycle_branch {
            LifecycleBranch::FullShellRebuild => params.notifier.tag_all(),
            LifecycleBranch::GranularPanels(tabs) => {
                for &tab in tabs {
                    params.notifier.tag_redraw(tab);
                }
            }
            LifecycleBranch::InPlaceRefresh | LifecycleBranch::Dormant => {}
        }
        WorkbenchFrameSync {
            caret_blink,
            selection_changed,
            domain_revision_changed,
            lifecycle_branch,
        }
    }

    /// Synchronizes an opaque domain selection identifier (`Option<u64>`) against the workbench.
    ///
    /// When the selection changes, all registered domain panels (in [`Self::panel_registry`] and
    /// [`Self::lifecycle`]) are automatically marked dirty in [`Self::lifecycle`]. Returns `true`
    /// if the selection changed.
    pub fn sync_selection(&mut self, selection_id: Option<u64>) -> bool {
        if self.last_selection_id == selection_id {
            return false;
        }
        self.last_selection_id = selection_id;
        self.mark_domain_tabs_dirty(None);
        true
    }

    /// Synchronizes an opaque domain selection identifier (`Option<u64>`) and forwards dirty
    /// notifications for all registered domain panels into `notifier` when selection changes.
    pub fn sync_selection_with_notifier(
        &mut self,
        selection_id: Option<u64>,
        notifier: &mut irisui_dock::UiNotifier<TabId>,
    ) -> bool {
        if self.last_selection_id == selection_id {
            return false;
        }
        self.last_selection_id = selection_id;
        self.mark_domain_tabs_dirty(Some(notifier));
        true
    }

    /// Synchronizes an external domain/history revision counter (`u64`) against the workbench.
    ///
    /// When the revision changes, all registered domain panels (in [`Self::panel_registry`] and
    /// [`Self::lifecycle`]) are automatically marked dirty in [`Self::lifecycle`]. Returns `true`
    /// if the revision changed.
    pub fn sync_domain_revision(&mut self, revision: u64) -> bool {
        if self.last_domain_revision == revision {
            return false;
        }
        self.last_domain_revision = revision;
        self.mark_domain_tabs_dirty(None);
        true
    }

    /// Synchronizes an external domain/history revision counter (`u64`) and forwards dirty
    /// notifications for all registered domain panels into `notifier` when the revision changes.
    pub fn sync_domain_revision_with_notifier(
        &mut self,
        revision: u64,
        notifier: &mut irisui_dock::UiNotifier<TabId>,
    ) -> bool {
        if self.last_domain_revision == revision {
            return false;
        }
        self.last_domain_revision = revision;
        self.mark_domain_tabs_dirty(Some(notifier));
        true
    }

    /// Returns the most recently synchronized opaque selection identifier.
    #[inline]
    #[must_use]
    pub const fn last_selection_id(&self) -> Option<u64> {
        self.last_selection_id
    }

    /// Returns the most recently synchronized external domain revision counter.
    #[inline]
    #[must_use]
    pub const fn last_domain_revision(&self) -> u64 {
        self.last_domain_revision
    }

    fn mark_domain_tabs_dirty(
        &mut self,
        mut notifier: Option<&mut irisui_dock::UiNotifier<TabId>>,
    ) {
        for &tab in self.panel_registry.domain_panels() {
            self.lifecycle.mark_tab_dirty(tab);
            if let Some(ref mut n) = notifier {
                n.tag_redraw(tab);
            }
        }
        self.lifecycle.mark_domain_panels_dirty();
        if let Some(n) = notifier {
            for &tab in self.lifecycle.domain_panels() {
                n.tag_redraw(tab);
            }
        }
    }

    /// Synchronizes [`irisui_text::TextSystem`]'s internal caret blink animation timer against `tree`
    /// using the monotonic system clock, marks focused nodes dirty on visibility flips, and caches
    /// the latest [`irisui_text::CaretBlinkUpdate`].
    #[inline]
    pub fn sync_caret_blink(
        &mut self,
        text_system: &mut irisui_text::TextSystem,
        tree: &mut irisui_core::UiTree,
    ) -> irisui_text::CaretBlinkUpdate {
        let update = text_system.sync_caret_blink(tree);
        self.finish_caret_blink_sync(tree, update)
    }

    /// Synchronizes [`irisui_text::TextSystem`]'s internal caret blink animation timer against `tree`
    /// using an explicit elapsed frame `delta`, marks focused nodes dirty on visibility flips, and
    /// caches the latest [`irisui_text::CaretBlinkUpdate`].
    #[inline]
    pub fn sync_caret_blink_with_delta(
        &mut self,
        text_system: &mut irisui_text::TextSystem,
        tree: &mut irisui_core::UiTree,
        delta: std::time::Duration,
    ) -> irisui_text::CaretBlinkUpdate {
        let update = text_system.sync_caret_blink_with_delta(tree, delta);
        self.finish_caret_blink_sync(tree, update)
    }

    #[inline]
    fn finish_caret_blink_sync(
        &mut self,
        tree: &mut irisui_core::UiTree,
        update: irisui_text::CaretBlinkUpdate,
    ) -> irisui_text::CaretBlinkUpdate {
        self.last_caret_blink = update;
        if update.needs_redraw() {
            self.dual_layout.mark_needs_text_prepare();
            if let Some(focused_id) = tree.event_state.focused {
                tree.mark_dirty(focused_id, irisui_core::DirtyFlags::PAINT);
            }
        }
        update
    }

    /// Registers a [`VolatileCanvasHook`](crate::hooks::VolatileCanvasHook) for
    /// `LifecycleBranch::InPlaceRefresh` frames.
    pub fn register_volatile_hook<H: crate::hooks::VolatileCanvasHook<AppState> + 'static>(
        &mut self,
        hook: H,
    ) {
        self.volatile_hooks.register(hook);
    }

    /// Registers a closure-based volatile hook under `id` for `LifecycleBranch::InPlaceRefresh`.
    pub fn register_volatile_fn<F>(&mut self, id: crate::hooks::OverlayId, callback: F)
    where
        F: FnMut(&mut irisui_core::UiTree, &mut AppState) -> crate::hooks::VolatileRefreshResult
            + Send
            + Sync
            + 'static,
    {
        self.volatile_hooks.register_fn(id, callback);
    }

    /// Unregisters the volatile hook identified by `id`, returning `true` if one existed.
    pub fn unregister_volatile_hook(&mut self, id: crate::hooks::OverlayId) -> bool {
        self.volatile_hooks.unregister(id)
    }

    /// Runs all registered volatile hooks in-place and forwards text preparation needs to `dual_layout`.
    pub fn refresh_volatile_hooks(
        &mut self,
        tree: &mut irisui_core::UiTree,
        context: &mut AppState,
    ) -> crate::hooks::VolatileRefreshResult {
        let res = self.volatile_hooks.refresh_in_place(tree, context);
        if res.needs_text_prepare {
            self.dual_layout.mark_needs_text_prepare();
        }
        res
    }

    /// Returns the most recently evaluated caret blink synchronization outcome.
    #[inline]
    #[must_use]
    pub const fn caret_blink_state(&self) -> irisui_text::CaretBlinkUpdate {
        self.last_caret_blink
    }

    /// Returns `true` when any text input currently holds focus and its caret is in the visible blink phase.
    #[inline]
    #[must_use]
    pub const fn is_caret_visible(&self) -> bool {
        self.last_caret_blink.is_visible
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
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unified Per-Frame Workbench Orchestration
//!
//! Combines floating window clamping, selection/domain synchronization, 3-branch lifecycle
//! execution (`FullShellRebuild`, `GranularPanels`, `InPlaceRefresh`, `Dormant`), caret blink
//! synchronization, cursor resolution, and end-of-frame cleanup into one library call.

use std::fmt::Debug;
use std::hash::Hash;

use irisui_core::{Rect, Size, UiTree, WidgetCursor};
use irisui_dock::{DockArea, DockState, UiNotifier};
use irisui_layout::LayoutError;
use irisui_text::CaretBlinkUpdate;
use irisui_widgets::UiScope;

use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::hooks::VolatileRefreshResult;
use crate::layout_sync::{SubtreeEngines, synchronize_dock_frame};
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};
use crate::workbench::Workbench;

/// Parameters passed to [`Workbench::update_frame_with`] for a single UI frame.
pub struct WorkbenchFrameUpdateParams<'a, TabId: Eq + Hash + Copy + Send + Sync + 'static> {
    /// Active docking hierarchy and floating windows.
    pub dock: &'a DockState<TabId>,
    /// Panel invalidation notifier cleared at the end of the frame.
    pub notifier: &'a mut UiNotifier<TabId>,
    /// Active UI scale factor.
    pub scale_factor: f32,
    /// Current opaque domain selection identifier.
    pub selection_id: Option<u64>,
    /// Current external domain/history revision counter.
    pub domain_revision: u64,
    /// Whether the pointer cursor is occluded by an external modal/capture surface.
    pub is_cursor_occluded: bool,
}

/// External panel registry and per-frame context passed to
/// [`Workbench::update_frame_live_with_registry`].
///
/// Allows hosts whose [`Workbench`] `AppState` differs from the per-frame panel context
/// (e.g. lifetime-borrowing frame view contexts) to run all 4 lifecycle branches through
/// a single library call without exposing [`UiTree`].
pub struct WorkbenchExternalContext<'a, TabId: Eq + Hash + Copy + Send + Sync + 'static, Ctx> {
    /// External panel registry containing panel drawers bound to `Ctx`.
    pub registry: &'a mut WorkbenchPanelRegistry<TabId, Ctx>,
    /// Mutable per-frame context passed to panel drawers, chrome builder, and live refresh.
    pub context: &'a mut Ctx,
}

/// Complete outcome produced by [`Workbench::update_frame_with`] for GPU & window synchronization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchFrameOutcome<TabId> {
    /// Evaluated lifecycle branch executed this frame.
    pub lifecycle_branch: LifecycleBranch<TabId>,
    /// Caret blink update produced by synchronizing [`irisui_text::TextSystem`].
    pub caret_blink: CaretBlinkUpdate,
    /// Whether the opaque domain selection identifier changed this frame.
    pub selection_changed: bool,
    /// Whether the external domain revision counter changed this frame.
    pub domain_revision_changed: bool,
    /// Aggregated volatile refresh flags when `lifecycle_branch` is [`LifecycleBranch::InPlaceRefresh`].
    pub volatile_refresh: VolatileRefreshResult,
    /// Whether glyph shaping / text section preparation is required before rendering.
    pub needs_text_prepare: bool,
    /// Whether retained quad draw commands / vertex buffers must be rebuilt and uploaded.
    pub needs_quad_reupload: bool,
    /// Effective cursor icon resolved at the workbench's latest pointer position.
    pub active_cursor: WidgetCursor,
}

struct FrameOutcomeSummary<TabId> {
    lifecycle_branch: LifecycleBranch<TabId>,
    selection_changed: bool,
    domain_revision_changed: bool,
    volatile_refresh: VolatileRefreshResult,
    needs_text_prepare: bool,
    needs_quad_reupload: bool,
}

impl<TabId> WorkbenchFrameOutcome<TabId> {
    /// Returns `true` when this frame performed a full shell reconstruction.
    #[inline]
    #[must_use]
    pub const fn is_full_shell_rebuild(&self) -> bool {
        matches!(self.lifecycle_branch, LifecycleBranch::FullShellRebuild)
    }
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Executes the full unified per-frame workbench lifecycle using default no-op volatile hook.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        context: &mut AppState,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: impl FnOnce(LifecycleInputs) -> LifecycleInputs,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError> {
        self.update_frame_with(engines, context, params, host_inputs, |_, _| {
            VolatileRefreshResult::none()
        })
    }

    /// Executes the full unified per-frame workbench lifecycle using a declarative [`UiScope`]
    /// callback for `LifecycleBranch::InPlaceRefresh`, never exposing [`UiTree`] or manual
    /// [`VolatileRefreshResult`] flags to the caller.
    ///
    /// Live canvas/text updates executed via [`irisui_widgets::UiScope::update_live_canvas`] or
    /// [`irisui_widgets::UiScope::update_live_text`] automatically propagate `needs_text_prepare`
    /// and `needs_quad_reupload` flags through [`WorkbenchFrameOutcome`].
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_live<FInputs, FLive>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        context: &mut AppState,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: FInputs,
        on_live_refresh: FLive,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FLive: FnOnce(&mut irisui_widgets::UiScope<'_>, &mut AppState),
    {
        let has_live = engines.tree.has_live_nodes();
        self.update_frame_with(
            engines,
            context,
            params,
            |inputs| host_inputs(inputs.with_volatile_updates(has_live)),
            |tree, ctx| {
                let _ = tree.take_live_refresh_flags();
                if let Some(root) = tree.root() {
                    let mut scope = irisui_widgets::UiScope::new(tree, root);
                    on_live_refresh(&mut scope, ctx);
                }
                let (needs_text_prepare, needs_quad_reupload) = tree.take_live_refresh_flags();
                VolatileRefreshResult {
                    needs_text_prepare,
                    needs_quad_reupload,
                }
            },
        )
    }

    /// Executes the full unified per-frame workbench lifecycle:
    /// 1. Clamps floating windows when `params.floating_clamp_bounds` is provided.
    /// 2. Synchronizes selection and domain revision invalidations into `params.notifier`.
    /// 3. Evaluates [`LifecycleBranch`] and executes `FullShellRebuild`, `GranularPanels`,
    ///    `InPlaceRefresh` (registered [`VolatileHookRegistry`](crate::hooks::VolatileHookRegistry) +
    ///    `on_volatile_refresh`), or `Dormant`.
    /// 4. Synchronizes caret blink animation and resolves the effective cursor icon.
    /// 5. Clears `params.notifier` and `engines.tree` frame click flags.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_with<FInputs, FVolatile>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        context: &mut AppState,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: FInputs,
        on_volatile_refresh: FVolatile,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FVolatile: FnOnce(&mut UiTree, &mut AppState) -> VolatileRefreshResult,
    {
        let selection_changed =
            self.sync_selection_with_notifier(params.selection_id, params.notifier);
        let domain_revision_changed =
            self.sync_domain_revision_with_notifier(params.domain_revision, params.notifier);

        if engines.tree.root().is_none() || self.dock_frame.is_none() {
            self.lifecycle.mark_shell_dirty();
        }

        let lifecycle_branch = self.evaluate_dock_lifecycle(
            params.dock,
            params.notifier,
            params.scale_factor,
            |inputs| host_inputs(inputs.with_domain_revision(params.domain_revision)),
        );

        let mut volatile_refresh = VolatileRefreshResult::none();
        let (mut needs_text_prepare, needs_quad_reupload) = match &lifecycle_branch {
            LifecycleBranch::FullShellRebuild => {
                engines.tree.clear();
                let screen_size = self.screen_size().unwrap_or_default();
                let cursor = self.last_cursor_pos;
                self.render_shell(
                    engines,
                    context,
                    screen_size,
                    cursor,
                    params.is_cursor_occluded,
                )?;
                (true, true)
            }
            LifecycleBranch::GranularPanels(tabs) => {
                for &tab in tabs {
                    params.notifier.tag_redraw(tab);
                }
                engines.tree.set_theme(self.theme);
                let rebuilt =
                    self.rebuild_dirty_panels_autonomous(engines, params.notifier, context)?;
                (
                    rebuilt > 0,
                    rebuilt > 0 || engines.tree.has_dirty_boundaries(),
                )
            }
            LifecycleBranch::InPlaceRefresh => {
                let reg_res = self.volatile_hooks.refresh_in_place(engines.tree, context);
                let cb_res = on_volatile_refresh(engines.tree, context);
                volatile_refresh = reg_res.merge(cb_res);
                if volatile_refresh.needs_text_prepare {
                    self.dual_layout.mark_needs_text_prepare();
                }
                (
                    volatile_refresh.needs_text_prepare,
                    volatile_refresh.needs_quad_reupload || engines.tree.has_dirty_boundaries(),
                )
            }
            LifecycleBranch::Dormant => (false, engines.tree.has_dirty_boundaries()),
        };

        let caret_blink = self.sync_caret_blink(engines.text_system, engines.tree);
        if caret_blink.needs_redraw() || self.dual_layout.take_needs_text_prepare() {
            needs_text_prepare = true;
        }

        let active_cursor = self
            .last_cursor_pos
            .map_or(WidgetCursor::Default, |pos| engines.tree.cursor_at(pos));
        params.notifier.clear_all();
        engines.tree.clear_frame_clicks();

        Ok(WorkbenchFrameOutcome {
            lifecycle_branch,
            caret_blink,
            selection_changed,
            domain_revision_changed,
            volatile_refresh,
            needs_text_prepare,
            needs_quad_reupload,
            active_cursor,
        })
    }

    /// Executes the full unified per-frame workbench lifecycle using an external
    /// [`WorkbenchPanelRegistry`] and per-frame context `Ctx` without exposing [`UiTree`].
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_live_with_registry<Ctx, FInputs, FChrome, FLive>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        mut params: WorkbenchFrameUpdateParams<'_, TabId>,
        external: WorkbenchExternalContext<'_, TabId, Ctx>,
        host_inputs: FInputs,
        on_shell_chrome: FChrome,
        on_live_refresh: FLive,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
        FLive: FnOnce(&mut UiScope<'_>, &mut Ctx),
    {
        let (selection_changed, domain_revision_changed, lifecycle_branch) = self
            .begin_frame_lifecycle(engines.tree, &mut params, |inputs| {
                host_inputs(inputs.with_volatile_updates(engines.tree.has_live_nodes()))
            });

        let mut volatile_refresh = VolatileRefreshResult::none();
        let (needs_text_prepare, needs_quad_reupload) = match &lifecycle_branch {
            LifecycleBranch::FullShellRebuild => {
                self.rebuild_shell_with_external_registry(
                    engines,
                    params.dock,
                    params.is_cursor_occluded,
                    external.registry,
                    external.context,
                    on_shell_chrome,
                )?;
                (true, true)
            }
            LifecycleBranch::GranularPanels(tabs) => {
                for &tab in tabs {
                    params.notifier.tag_redraw(tab);
                }
                engines.tree.set_theme(self.theme);
                let dock_frame_ref = self.dock_frame.as_ref();
                let rebuilt = self.dual_layout.rebuild_dirty_panels_with_boundaries(
                    engines,
                    params.notifier,
                    external.registry,
                    external.context,
                    |tab_id| dock_frame_ref.and_then(|df| df.leaf_container(tab_id)),
                    |boundary_id| {
                        dock_frame_ref.and_then(|df| df.tab_for_leaf_container(boundary_id))
                    },
                )?;
                (
                    rebuilt > 0,
                    rebuilt > 0 || engines.tree.has_dirty_boundaries(),
                )
            }
            LifecycleBranch::InPlaceRefresh => {
                let _ = engines.tree.take_live_refresh_flags();
                if let Some(root) = engines.tree.root() {
                    let mut scope = UiScope::new(engines.tree, root);
                    on_live_refresh(&mut scope, external.context);
                }
                let (needs_text, needs_quad) = engines.tree.take_live_refresh_flags();
                volatile_refresh = VolatileRefreshResult {
                    needs_text_prepare: needs_text,
                    needs_quad_reupload: needs_quad,
                };
                if needs_text {
                    self.dual_layout.mark_needs_text_prepare();
                }
                (
                    needs_text,
                    needs_quad || engines.tree.has_dirty_boundaries(),
                )
            }
            LifecycleBranch::Dormant => (false, engines.tree.has_dirty_boundaries()),
        };

        Ok(self.finish_frame_outcome(
            engines,
            params.notifier,
            FrameOutcomeSummary {
                lifecycle_branch,
                selection_changed,
                domain_revision_changed,
                volatile_refresh,
                needs_text_prepare,
                needs_quad_reupload,
            },
        ))
    }

    fn begin_frame_lifecycle(
        &mut self,
        tree: &UiTree,
        params: &mut WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: impl FnOnce(LifecycleInputs) -> LifecycleInputs,
    ) -> (bool, bool, LifecycleBranch<TabId>) {
        let selection_changed =
            self.sync_selection_with_notifier(params.selection_id, params.notifier);
        let domain_revision_changed =
            self.sync_domain_revision_with_notifier(params.domain_revision, params.notifier);
        if tree.root().is_none() || self.dock_frame.is_none() {
            self.lifecycle.mark_shell_dirty();
        }
        let domain_revision = params.domain_revision;
        let lifecycle_branch = self.evaluate_dock_lifecycle(
            params.dock,
            params.notifier,
            params.scale_factor,
            |inputs| host_inputs(inputs.with_domain_revision(domain_revision)),
        );
        (selection_changed, domain_revision_changed, lifecycle_branch)
    }

    fn rebuild_shell_with_external_registry<Ctx, FChrome>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        dock: &DockState<TabId>,
        is_cursor_occluded: bool,
        registry: &mut WorkbenchPanelRegistry<TabId, Ctx>,
        context: &mut Ctx,
        on_shell_chrome: FChrome,
    ) -> Result<(), LayoutError>
    where
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
    {
        engines.tree.clear();
        engines.layout_engine.clear();
        let screen_size = self.screen_size().unwrap_or_default();
        engines.tree.set_theme(self.theme);
        let root = Self::create_viewport_root(engines.tree, screen_size)?;
        let dock_frame = {
            let mut scope = UiScope::new(engines.tree, root);
            if self
                .descriptor
                .menubar
                .as_ref()
                .is_some_and(|m| !m.is_empty())
            {
                self.descriptor
                    .render_menubar(&mut scope, screen_size.width);
            }
            let workspace_rect = self.compute_workspace_rect(screen_size.width, screen_size.height);
            let dock_frame = {
                let mut tv = registry.active_tab_viewer(context);
                DockArea::from_ref(dock)
                    .bounds(workspace_rect)
                    .style(self.dock_style)
                    .cursor_pos(self.last_cursor_pos.unwrap_or_default())
                    .cursor_occluded(is_cursor_occluded)
                    .show(&mut scope, &mut tv)
            };
            if self
                .descriptor
                .statusbar
                .as_ref()
                .is_some_and(|s| !s.is_default_empty())
            {
                self.descriptor
                    .render_statusbar(&mut scope, screen_size.width, screen_size.height);
            }
            on_shell_chrome(&mut scope, context, screen_size);
            scope.finish_layout(Rect::new(0.0, 0.0, screen_size.width, screen_size.height));
            dock_frame
        };
        engines.text_system.measure_subtree_text(engines.tree, root);
        engines
            .layout_engine
            .compute_layout(engines.tree, screen_size)?;
        synchronize_dock_frame(
            engines.tree,
            engines.layout_engine,
            engines.text_system,
            &dock_frame,
        )?;
        if self.dialog_host.is_active() {
            let mut modal_scope = UiScope::new(engines.tree, root);
            let screen_rect = Rect::new(0.0, 0.0, screen_size.width, screen_size.height);
            self.dialog_host
                .render(&mut modal_scope, screen_rect, false, &mut self.action_bus);
        }
        self.dual_layout.mark_needs_text_prepare();
        self.dock_frame = Some(dock_frame);
        Ok(())
    }

    fn finish_frame_outcome(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        mut summary: FrameOutcomeSummary<TabId>,
    ) -> WorkbenchFrameOutcome<TabId> {
        let caret_blink = self.sync_caret_blink(engines.text_system, engines.tree);
        if caret_blink.needs_redraw() || self.dual_layout.take_needs_text_prepare() {
            summary.needs_text_prepare = true;
        }
        let active_cursor = self
            .last_cursor_pos
            .map_or(WidgetCursor::Default, |pos| engines.tree.cursor_at(pos));
        notifier.clear_all();
        engines.tree.clear_frame_clicks();
        WorkbenchFrameOutcome {
            lifecycle_branch: summary.lifecycle_branch,
            caret_blink,
            selection_changed: summary.selection_changed,
            domain_revision_changed: summary.domain_revision_changed,
            volatile_refresh: summary.volatile_refresh,
            needs_text_prepare: summary.needs_text_prepare,
            needs_quad_reupload: summary.needs_quad_reupload,
            active_cursor,
        }
    }

    /// Executes `on_live` inside a root [`UiScope`] when `tree` has registered live nodes,
    /// returning the aggregated [`VolatileRefreshResult`] without exposing [`UiTree`] to the callback.
    pub fn refresh_live_scope(
        &mut self,
        tree: &mut UiTree,
        on_live: impl FnOnce(&mut irisui_widgets::UiScope<'_>),
    ) -> VolatileRefreshResult {
        if !tree.has_live_nodes() {
            return VolatileRefreshResult::none();
        }
        let _ = tree.take_live_refresh_flags();
        if let Some(root) = tree.root() {
            let mut scope = irisui_widgets::UiScope::new(tree, root);
            on_live(&mut scope);
        }
        let (needs_text_prepare, needs_quad_reupload) = tree.take_live_refresh_flags();
        if needs_text_prepare {
            self.dual_layout.mark_needs_text_prepare();
        }
        VolatileRefreshResult {
            needs_text_prepare,
            needs_quad_reupload,
        }
    }

    /// Builds [`irisui_text::TextCollectionOptions`] directly from the retained `tree` focus/hover
    /// state, the workbench's latest caret blink visibility, and caller-provided `extra_occluders`.
    #[must_use]
    pub fn build_text_collection_options(
        &self,
        tree: &UiTree,
        extra_occluders: Vec<irisui_core::Rect>,
    ) -> irisui_text::TextCollectionOptions {
        irisui_text::TextCollectionOptions {
            hovered_tag: tree
                .event_state
                .hovered
                .and_then(|id| tree.get(id))
                .map(|n| n.tag),
            hovered_id: tree.event_state.hovered,
            focused_tag: tree.event_state.focused_tag,
            focused_id: tree.event_state.focused,
            blink_caret: self.is_caret_visible(),
            extra_occluders,
            ..Default::default()
        }
    }
}
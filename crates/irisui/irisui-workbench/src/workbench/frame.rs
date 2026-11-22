// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unified Per-Frame Workbench Orchestration
//!
//! Combines floating window clamping, selection/domain synchronization, 3-branch lifecycle
//! execution (`FullShellRebuild`, `GranularPanels`, `InPlaceRefresh`, `Dormant`), caret blink
//! synchronization, cursor resolution, and end-of-frame cleanup into one library call.

use std::fmt::Debug;
use std::hash::Hash;

use irisui_core::{IrisSurfaceFrameFlags, Rect, Size, SurfaceRefreshMode, UiTree, WidgetCursor};
use irisui_dock::DockState;
use irisui_layout::LayoutError;
use irisui_text::CaretBlinkUpdate;
use irisui_widgets::UiScope;

use crate::descriptors::panel_catalog::PanelRegistryTarget;
use crate::hooks::VolatileRefreshResult;
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};
use crate::workbench::Workbench;
use crate::workbench::frame_services::{BranchExecTarget, WorkbenchDomSlice};

/// Parameters passed to [`Workbench::update_frame_with`] for a single UI frame.
pub struct WorkbenchFrameUpdateParams<'a, TabId: Eq + Hash + Copy + Send + Sync + 'static> {
    /// Active docking hierarchy and floating windows.
    pub dock: &'a DockState<TabId>,
    /// Active UI scale factor.
    pub scale_factor: f32,
    /// Current opaque domain selection identifier.
    pub selection_id: Option<u64>,
    /// Current external domain/history revision counter.
    pub domain_revision: u64,
    /// Whether the pointer cursor is occluded by an external modal/capture surface.
    pub is_cursor_occluded: bool,
}

/// Frame parameters for [`Workbench::update_frame_with_registry`] using the workbench's own
/// internal [`DockState`] without maintaining a duplicate external dock copy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkbenchOwnedFrameParams {
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
pub struct WorkbenchExternalContext<'a, Ctx, R: ?Sized> {
    /// External panel registry containing panel drawers bound to `Ctx`.
    pub registry: &'a mut R,
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

pub(super) struct FrameOutcomeSummary<TabId> {
    pub lifecycle_branch: LifecycleBranch<TabId>,
    pub selection_changed: bool,
    pub domain_revision_changed: bool,
    pub volatile_refresh: VolatileRefreshResult,
    pub needs_text_prepare: bool,
    pub needs_quad_reupload: bool,
}

impl<TabId> WorkbenchFrameOutcome<TabId> {
    /// Returns `true` when this frame performed a full shell reconstruction.
    #[inline]
    #[must_use]
    pub const fn is_full_shell_rebuild(&self) -> bool {
        matches!(self.lifecycle_branch, LifecycleBranch::FullShellRebuild)
    }

    /// Translates this lifecycle outcome into decoupled [`IrisSurfaceFrameFlags`] for GPU surface
    /// text preparation and incremental quad buffer updates.
    #[must_use]
    pub fn to_surface_frame_flags(&self) -> IrisSurfaceFrameFlags {
        let mode = match &self.lifecycle_branch {
            LifecycleBranch::FullShellRebuild => SurfaceRefreshMode::StructuralRebuild {
                is_full_shell: true,
            },
            LifecycleBranch::GranularPanels(_) => SurfaceRefreshMode::StructuralRebuild {
                is_full_shell: false,
            },
            LifecycleBranch::InPlaceRefresh => SurfaceRefreshMode::InPlaceRefresh,
            LifecycleBranch::Dormant => SurfaceRefreshMode::Dormant,
        };
        IrisSurfaceFrameFlags {
            mode,
            caret_blink_redraw: self.caret_blink.needs_redraw(),
            needs_text_prepare: self.needs_text_prepare,
            volatile_needs_text_prepare: self.volatile_refresh.needs_text_prepare,
            needs_quad_reupload: self.needs_quad_reupload,
        }
    }
}

impl<TabId> From<&WorkbenchFrameOutcome<TabId>> for IrisSurfaceFrameFlags {
    #[inline]
    fn from(outcome: &WorkbenchFrameOutcome<TabId>) -> Self {
        outcome.to_surface_frame_flags()
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
        context: &mut AppState,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: impl FnOnce(LifecycleInputs) -> LifecycleInputs,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError> {
        self.update_frame_with(context, params, host_inputs, |_, _| {
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
        context: &mut AppState,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: FInputs,
        on_live_refresh: FLive,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FLive: FnOnce(&mut irisui_widgets::UiScope<'_>, &mut AppState),
    {
        let has_live = self.tree.has_live_nodes();
        self.update_frame_with(
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
    /// 1. Synchronizes selection and domain revision invalidations into [`Self::notifier`].
    /// 2. Evaluates [`LifecycleBranch`] and executes `FullShellRebuild`, `GranularPanels`,
    ///    `InPlaceRefresh` (registered [`VolatileHookRegistry`](crate::hooks::VolatileHookRegistry) +
    ///    `on_volatile_refresh`), or `Dormant`.
    /// 3. Synchronizes caret blink animation and resolves the effective cursor icon.
    /// 4. Clears [`Self::notifier`] and retained frame click flags.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_with<FInputs, FVolatile>(
        &mut self,
        context: &mut AppState,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: FInputs,
        on_volatile_refresh: FVolatile,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FVolatile: FnOnce(&mut UiTree, &mut AppState) -> VolatileRefreshResult,
    {
        let selection_changed = self.sync_selection_with_notifier(params.selection_id);
        let domain_revision_changed =
            self.sync_domain_revision_with_notifier(params.domain_revision);

        if self.tree.root().is_none() || self.dock_frame.is_none() {
            self.lifecycle.mark_shell_dirty();
        }

        let lifecycle_branch =
            self.evaluate_dock_lifecycle(params.dock, params.scale_factor, |inputs| {
                host_inputs(inputs.with_domain_revision(params.domain_revision))
            });
        self.tree.caret_blink = self.text_system.is_caret_visible();

        let mut volatile_refresh = VolatileRefreshResult::none();
        let (mut needs_text_prepare, needs_quad_reupload) = match &lifecycle_branch {
            LifecycleBranch::FullShellRebuild => {
                self.tree.clear();
                let screen_size = self.screen_size().unwrap_or_default();
                let cursor = self.last_cursor_pos;
                self.render_shell(context, screen_size, cursor, params.is_cursor_occluded)?;
                (true, true)
            }
            LifecycleBranch::GranularPanels(tabs) => {
                for &tab in tabs {
                    self.notifier.tag_redraw(tab);
                }
                self.tree.set_theme(self.theme);
                let rebuilt = self.rebuild_dirty_panels_autonomous(context)?;
                (rebuilt > 0, rebuilt > 0 || self.tree.has_dirty_boundaries())
            }
            LifecycleBranch::InPlaceRefresh => {
                let reg_res = self
                    .volatile_hooks
                    .refresh_in_place(&mut self.tree, context);
                let cb_res = on_volatile_refresh(&mut self.tree, context);
                volatile_refresh = reg_res.merge(cb_res);
                if volatile_refresh.needs_text_prepare {
                    self.dual_layout.mark_needs_text_prepare();
                }
                (
                    volatile_refresh.needs_text_prepare,
                    volatile_refresh.needs_quad_reupload || self.tree.has_dirty_boundaries(),
                )
            }
            LifecycleBranch::Dormant => (false, self.tree.has_dirty_boundaries()),
        };

        let caret_blink = self.sync_caret_blink();
        if caret_blink.needs_redraw() || self.dual_layout.take_needs_text_prepare() {
            needs_text_prepare = true;
        }

        let active_cursor = self
            .last_cursor_pos
            .map_or(WidgetCursor::Default, |pos| self.tree.cursor_at(pos));
        self.notifier.clear_all();
        self.tree.clear_frame_clicks();

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

    /// Executes the full unified per-frame workbench lifecycle using the workbench's own owned
    /// [`DockState`] and an external [`WorkbenchPanelRegistry`] without exposing [`UiTree`].
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_with_registry<Ctx, R, FInputs, FChrome>(
        &mut self,
        params: WorkbenchOwnedFrameParams,
        external: WorkbenchExternalContext<'_, Ctx, R>,
        host_inputs: FInputs,
        on_shell_chrome: FChrome,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
    {
        let dock = std::mem::take(&mut self.descriptor.dock_state);
        let result = self.update_frame_live_with_registry(
            WorkbenchFrameUpdateParams {
                dock: &dock,
                scale_factor: params.scale_factor,
                selection_id: params.selection_id,
                domain_revision: params.domain_revision,
                is_cursor_occluded: params.is_cursor_occluded,
            },
            external,
            host_inputs,
            on_shell_chrome,
            |_scope, _ctx| {},
        );
        self.descriptor.dock_state = dock;
        result
    }

    /// Executes the full unified per-frame workbench lifecycle using an external
    /// [`WorkbenchPanelRegistry`] and per-frame context `Ctx` without exposing [`UiTree`].
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_live_with_registry<Ctx, R, FInputs, FChrome, FLive>(
        &mut self,
        params: WorkbenchFrameUpdateParams<'_, TabId>,
        external: WorkbenchExternalContext<'_, Ctx, R>,
        host_inputs: FInputs,
        on_shell_chrome: FChrome,
        on_live_refresh: FLive,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
        FLive: FnOnce(&mut UiScope<'_>, &mut Ctx),
    {
        let has_live = self.tree.has_live_nodes()
            || self
                .live_scheduler
                .has_active_live_panel(self.dock_frame.as_ref(), external.registry);
        let (selection_changed, domain_revision_changed, lifecycle_branch) = self
            .begin_frame_lifecycle(&params, |inputs| {
                host_inputs(inputs.with_volatile_updates(has_live))
            });

        let screen_size = self.screen_size().unwrap_or_default();
        let workspace_rect = self.compute_workspace_rect(screen_size.width, screen_size.height);
        self.tree.caret_blink = self.text_system.is_caret_visible();
        let (needs_text_prepare, needs_quad_reupload, volatile_refresh) = {
            let mut slice = WorkbenchDomSlice {
                tree: &mut self.tree,
                layout_engine: &mut self.layout_engine,
                text_system: &mut self.text_system,
                notifier: &mut self.notifier,
                dual_layout: &mut self.dual_layout,
                live_scheduler: &mut self.live_scheduler,
                dock_frame: &mut self.dock_frame,
                descriptor: &self.descriptor,
                dock_style: self.dock_style,
                theme: self.theme,
                last_cursor_pos: self.last_cursor_pos,
                screen_size,
                workspace_rect,
            };
            slice.execute_branch(
                &lifecycle_branch,
                BranchExecTarget {
                    dock: params.dock,
                    is_cursor_occluded: params.is_cursor_occluded,
                    registry: external.registry,
                    context: external.context,
                    on_shell_chrome,
                    on_live_refresh,
                },
            )?
        };

        if matches!(lifecycle_branch, LifecycleBranch::FullShellRebuild)
            && self.dialog_host.is_active()
            && let Some(root) = self.tree.root()
        {
            let mut modal_scope = UiScope::new(&mut self.tree, root);
            let screen_rect = Rect::new(0.0, 0.0, screen_size.width, screen_size.height);
            self.dialog_host
                .render(&mut modal_scope, screen_rect, false, &mut self.action_bus);
        }

        Ok(self.finish_frame_outcome(FrameOutcomeSummary {
            lifecycle_branch,
            selection_changed,
            domain_revision_changed,
            volatile_refresh,
            needs_text_prepare,
            needs_quad_reupload,
        }))
    }

    pub(super) fn begin_frame_lifecycle(
        &mut self,
        params: &WorkbenchFrameUpdateParams<'_, TabId>,
        host_inputs: impl FnOnce(LifecycleInputs) -> LifecycleInputs,
    ) -> (bool, bool, LifecycleBranch<TabId>) {
        let selection_changed = self.sync_selection_with_notifier(params.selection_id);
        let domain_revision_changed =
            self.sync_domain_revision_with_notifier(params.domain_revision);
        if self.tree.root().is_none() || self.dock_frame.is_none() {
            self.lifecycle.mark_shell_dirty();
        }
        let domain_revision = params.domain_revision;
        let lifecycle_branch =
            self.evaluate_dock_lifecycle(params.dock, params.scale_factor, |inputs| {
                host_inputs(inputs.with_domain_revision(domain_revision))
            });
        (selection_changed, domain_revision_changed, lifecycle_branch)
    }

    /// Merges dialogs staged by panel drawers during a frame pass into [`Self::dialog_host`] and
    /// immediately renders the active dialog overlay onto the retained shell root.
    pub fn append_dialogs(&mut self, staged: &mut crate::dialogs::DialogHost<Action>) {
        if !staged.is_active() {
            return;
        }
        self.dialog_host.append(staged);
        if let Some(root) = self.tree.root() {
            let screen_size = self.screen_size().unwrap_or_default();
            let screen_rect = Rect::new(0.0, 0.0, screen_size.width, screen_size.height);
            let mut modal_scope = UiScope::new(&mut self.tree, root);
            self.dialog_host
                .render(&mut modal_scope, screen_rect, false, &mut self.action_bus);
            self.dual_layout.mark_needs_text_prepare();
        }
    }

    pub(super) fn finish_frame_outcome(
        &mut self,
        mut summary: FrameOutcomeSummary<TabId>,
    ) -> WorkbenchFrameOutcome<TabId> {
        let caret_blink = self.sync_caret_blink();
        if caret_blink.needs_redraw() || self.dual_layout.take_needs_text_prepare() {
            summary.needs_text_prepare = true;
        }
        let active_cursor = self
            .last_cursor_pos
            .map_or(WidgetCursor::Default, |pos| self.tree.cursor_at(pos));
        self.notifier.clear_all();
        self.tree.clear_frame_clicks();
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

    /// Executes `on_live` inside a root [`UiScope`] when the retained tree has registered live
    /// nodes, returning the aggregated [`VolatileRefreshResult`] without exposing [`UiTree`].
    pub fn refresh_live_scope(
        &mut self,
        on_live: impl FnOnce(&mut irisui_widgets::UiScope<'_>),
    ) -> VolatileRefreshResult {
        if !self.tree.has_live_nodes() {
            return VolatileRefreshResult::none();
        }
        let _ = self.tree.take_live_refresh_flags();
        if let Some(root) = self.tree.root() {
            let mut scope = irisui_widgets::UiScope::new(&mut self.tree, root);
            on_live(&mut scope);
        }
        let (needs_text_prepare, needs_quad_reupload) = self.tree.take_live_refresh_flags();
        if needs_text_prepare {
            self.dual_layout.mark_needs_text_prepare();
        }
        VolatileRefreshResult {
            needs_text_prepare,
            needs_quad_reupload,
        }
    }

    /// Builds [`irisui_text::TextCollectionOptions`] directly from the workbench's retained tree
    /// focus/hover state, caret blink visibility, and caller-provided `extra_occluders`.
    #[must_use]
    pub fn build_text_collection_options(
        &self,
        extra_occluders: Vec<irisui_core::Rect>,
    ) -> irisui_text::TextCollectionOptions {
        irisui_text::TextCollectionOptions {
            hovered_tag: self
                .tree
                .event_state
                .hovered
                .and_then(|id| self.tree.get(id))
                .map(|n| n.tag),
            hovered_id: self.tree.event_state.hovered,
            focused_tag: self.tree.event_state.focused_tag,
            focused_id: self.tree.event_state.focused,
            blink_caret: self.is_caret_visible(),
            extra_occluders,
            ..Default::default()
        }
    }
}
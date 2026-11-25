// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Borrow-Split Frame Services for Scoped Panel Registries
//!
//! Exposes [`WorkbenchFrameServices`] (`&mut ActionBus<Action>` and `&mut DialogHost<Action>`)
//! directly to scoped panel rendering contexts during [`Workbench::update_frame_with_services`],
//! eliminating temporary `std::mem::take(&mut workbench.action_bus)` and intermediate staged dialog hosts.

use std::fmt::Debug;
use std::hash::Hash;

use irisui_core::{Point, Rect, Size, Theme, UiTree};
use irisui_dock::chrome::DockChromeFrame;
use irisui_dock::style::DockChromeStyle;
use irisui_dock::{DeferredNotifier, DockArea, DockState, UiNotifier};
use irisui_layout::{LayoutEngine, LayoutError};
use irisui_text::TextSystem;
use irisui_widgets::UiScope;

use crate::action_bus::ActionBus;
use crate::descriptors::panel_catalog::{PanelRegistryTarget, RegistryActiveTabViewer};
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::dialogs::host::DialogHost;
use crate::hooks::VolatileRefreshResult;
use crate::layout_sync::{DualLayoutEliminator, SubtreeEngines, synchronize_dock_frame};
use crate::lifecycle::{LifecycleBranch, LifecycleInputs};
use crate::workbench::Workbench;
use crate::workbench::frame::{
    FrameOutcomeSummary, WorkbenchFrameOutcome, WorkbenchFrameUpdateParams,
    WorkbenchOwnedFrameParams,
};
use crate::workbench::live_scheduler::LivePanelScheduler;

/// Mutable framework services borrowed directly from [`Workbench`] for a single frame pass.
///
/// Passed alongside [`WorkbenchFrameRunner`] into [`Workbench::update_frame_with_services`] so
/// menubar chrome and panel renderers emit actions and stage dialogs in-place without
/// copying or draining temporary buses.
pub struct WorkbenchFrameServices<
    'a,
    Action: 'static + Send + Sync,
    TabId: Eq + Hash + Copy + Send + Sync + 'static,
> {
    /// Central action bus collecting UI commands directly during the frame pass.
    pub action_bus: &'a mut ActionBus<Action>,
    /// Central dialog host staging modals directly onto the workbench without staging buffers.
    pub dialog_host: &'a mut DialogHost<Action>,
    /// Frame-scoped deferred dock mutation buffer applied automatically when the pass ends.
    pub notifier: &'a mut DeferredNotifier<TabId>,
    /// Current caret blink visibility sampled from the workbench text system.
    pub blink_caret: bool,
    /// Effective logical pointer coordinate tracked by the workbench.
    pub cursor_pos: Point,
    /// Current logical workbench dimensions.
    pub screen_size: Size,
    /// Active UI scale factor applied by the workbench.
    pub zoom_factor: f32,
    /// Active timed status spans managed by the workbench, if any.
    pub status_spans: Option<&'a [(String, irisui_core::Color)]>,
}

/// Borrow-split DOM and layout state view used to build shell or panel trees while
/// [`Workbench::action_bus`] and [`Workbench::dialog_host`] are borrowed by [`WorkbenchFrameServices`].
pub(super) struct WorkbenchDomSlice<'a, TabId: Eq + Hash + Copy + Send + Sync + 'static> {
    pub tree: &'a mut UiTree,
    pub layout_engine: &'a mut LayoutEngine,
    pub text_system: &'a mut TextSystem,
    pub notifier: &'a mut UiNotifier<TabId>,
    pub dual_layout: &'a mut DualLayoutEliminator,
    pub live_scheduler: &'a mut LivePanelScheduler<TabId>,
    pub dock_frame: &'a mut Option<DockChromeFrame<TabId>>,
    pub descriptor: &'a WorkbenchDescriptor<TabId>,
    pub dock_style: DockChromeStyle,
    pub theme: Theme,
    pub last_cursor_pos: Option<Point>,
    pub screen_size: Size,
    pub workspace_rect: Rect,
}

/// Per-branch execution arguments bundling registry and transient panel callbacks.
pub(super) struct BranchExecTarget<
    'a,
    TabId: Eq + Hash + Copy + Send + Sync + 'static,
    Ctx,
    R: ?Sized,
    FChrome,
    FLive,
> {
    pub dock: &'a DockState<TabId>,
    pub is_cursor_occluded: bool,
    pub registry: &'a mut R,
    pub context: &'a mut Ctx,
    pub on_shell_chrome: FChrome,
    pub on_live_refresh: FLive,
}

/// Scoped frame execution handle exposed inside [`Workbench::update_frame_with_services`].
///
/// Executes the active lifecycle branch (`FullShellRebuild`, `GranularPanels`, `InPlaceRefresh`,
/// or `Dormant`) using caller-assembled context `Ctx` that borrows directly from [`WorkbenchFrameServices`].
pub struct WorkbenchFrameRunner<'a, TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static> {
    branch: &'a LifecycleBranch<TabId>,
    dock: &'a DockState<TabId>,
    is_cursor_occluded: bool,
    slice: WorkbenchDomSlice<'a, TabId>,
}

/// Internal per-branch execution metrics returned by [`WorkbenchFrameRunner::execute`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkbenchFramePassResult {
    needs_text_prepare: bool,
    needs_quad_reupload: bool,
    volatile_refresh: VolatileRefreshResult,
}

impl<TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static> WorkbenchFrameRunner<'_, TabId> {
    /// Executes the active lifecycle branch against `registry`, `context`, and `on_shell_chrome`.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn execute<Ctx, R, FChrome>(
        &mut self,
        registry: &mut R,
        context: &mut Ctx,
        on_shell_chrome: FChrome,
    ) -> Result<WorkbenchFramePassResult, LayoutError>
    where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
    {
        let (needs_text_prepare, needs_quad_reupload, volatile_refresh) =
            self.slice.execute_branch(
                self.branch,
                BranchExecTarget {
                    dock: self.dock,
                    is_cursor_occluded: self.is_cursor_occluded,
                    registry,
                    context,
                    on_shell_chrome,
                    on_live_refresh: |_scope: &mut UiScope<'_>, _ctx: &mut Ctx| {},
                },
            )?;
        Ok(WorkbenchFramePassResult {
            needs_text_prepare,
            needs_quad_reupload,
            volatile_refresh,
        })
    }
}

impl<TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static> WorkbenchDomSlice<'_, TabId> {
    pub(super) fn execute_branch<Ctx, R, FChrome, FLive>(
        &mut self,
        branch: &LifecycleBranch<TabId>,
        target: BranchExecTarget<'_, TabId, Ctx, R, FChrome, FLive>,
    ) -> Result<(bool, bool, VolatileRefreshResult), LayoutError>
    where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
        FLive: FnOnce(&mut UiScope<'_>, &mut Ctx),
    {
        match branch {
            LifecycleBranch::FullShellRebuild => {
                self.live_scheduler.clear_all();
                self.rebuild_shell_dom(
                    target.dock,
                    target.is_cursor_occluded,
                    target.registry,
                    target.context,
                    target.on_shell_chrome,
                )?;
                Ok((true, true, VolatileRefreshResult::none()))
            }
            LifecycleBranch::GranularPanels(tabs) => {
                self.live_scheduler.clear_tabs(tabs);
                for &tab in tabs {
                    self.notifier.tag_redraw(tab);
                }
                self.tree.set_theme(self.theme);
                let dock_frame_ref = self.dock_frame.as_ref();
                let mut engines = SubtreeEngines::new(
                    &mut *self.tree,
                    &mut *self.layout_engine,
                    &mut *self.text_system,
                );
                let rebuilt = self.dual_layout.rebuild_dirty_panels_with_boundaries(
                    &mut engines,
                    self.notifier,
                    target.registry,
                    target.context,
                    |tab_id| dock_frame_ref.and_then(|df| df.leaf_container(tab_id)),
                    |boundary_id| {
                        dock_frame_ref.and_then(|df| df.tab_for_leaf_container(boundary_id))
                    },
                )?;
                Ok((
                    rebuilt > 0,
                    rebuilt > 0 || self.tree.has_dirty_boundaries(),
                    VolatileRefreshResult::none(),
                ))
            }
            LifecycleBranch::InPlaceRefresh => {
                let _ = self.tree.take_live_refresh_flags();
                if let Some(root) = self.tree.root() {
                    let mut scope = UiScope::new(self.tree, root);
                    self.live_scheduler.dispatch(
                        &mut scope,
                        self.dock_frame.as_ref(),
                        target.registry,
                        target.context,
                    );
                    (target.on_live_refresh)(&mut scope, target.context);
                }
                let (needs_text, needs_quad) = self.tree.take_live_refresh_flags();
                if needs_text {
                    self.dual_layout.mark_needs_text_prepare();
                }
                Ok((
                    needs_text,
                    needs_quad || self.tree.has_dirty_boundaries(),
                    VolatileRefreshResult {
                        needs_text_prepare: needs_text,
                        needs_quad_reupload: needs_quad,
                    },
                ))
            }
            LifecycleBranch::Dormant => Ok((
                false,
                self.tree.has_dirty_boundaries(),
                VolatileRefreshResult::none(),
            )),
        }
    }

    fn rebuild_shell_dom<Ctx, R, FChrome>(
        &mut self,
        dock: &DockState<TabId>,
        is_cursor_occluded: bool,
        registry: &mut R,
        context: &mut Ctx,
        on_shell_chrome: FChrome,
    ) -> Result<(), LayoutError>
    where
        R: PanelRegistryTarget<TabId, Ctx> + ?Sized,
        FChrome: FnOnce(&mut UiScope<'_>, &mut Ctx, Size),
    {
        self.tree.clear();
        self.layout_engine.clear();
        self.tree.set_theme(self.theme);
        let root = Workbench::<TabId, (), ()>::create_viewport_root(self.tree, self.screen_size)?;
        let dock_frame = {
            let mut scope = UiScope::new(self.tree, root);
            if self
                .descriptor
                .menubar
                .as_ref()
                .is_some_and(|m| !m.is_empty())
            {
                self.descriptor
                    .render_menubar(&mut scope, self.screen_size.width);
            }
            let dock_frame = {
                let mut tv = RegistryActiveTabViewer::new(registry, context);
                DockArea::from_ref(dock)
                    .bounds(self.workspace_rect)
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
                self.descriptor.render_statusbar(
                    &mut scope,
                    self.screen_size.width,
                    self.screen_size.height,
                );
            }
            on_shell_chrome(&mut scope, context, self.screen_size);
            scope.finish_layout(Rect::new(
                0.0,
                0.0,
                self.screen_size.width,
                self.screen_size.height,
            ));
            dock_frame
        };
        self.text_system.measure_subtree_text(self.tree, root);
        self.layout_engine
            .compute_layout(self.tree, self.screen_size)?;
        synchronize_dock_frame(self.tree, self.layout_engine, self.text_system, &dock_frame)?;
        self.dual_layout.mark_needs_text_prepare();
        *self.dock_frame = Some(dock_frame);
        Ok(())
    }
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Executes the full unified per-frame workbench lifecycle using the workbench's own owned
    /// [`DockState`] while borrowing [`Workbench::action_bus`] and [`Workbench::dialog_host`]
    /// directly via [`WorkbenchFrameServices`] without temporary bus swaps or staging hosts.
    ///
    /// `has_live_panels` tracks whether any visible live-refresh panel is active (`false` otherwise).
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn update_frame_with_services<FInputs, FRun>(
        &mut self,
        params: WorkbenchOwnedFrameParams,
        has_live_panels: bool,
        host_inputs: FInputs,
        on_run: FRun,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FRun: FnOnce(
            &mut WorkbenchFrameRunner<'_, TabId>,
            WorkbenchFrameServices<'_, Action, TabId>,
        ) -> Result<WorkbenchFramePassResult, LayoutError>,
    {
        self.expire_status_message_if_needed();
        let dock = std::mem::take(&mut self.descriptor.dock_state);
        let result = self.run_services_frame(&dock, params, has_live_panels, host_inputs, on_run);
        self.descriptor.dock_state = dock;
        result
    }

    /// Returns `true` when retained live nodes or a scheduled registry panel require `InPlaceRefresh`.
    #[inline]
    #[must_use]
    pub fn has_active_live_for<Ctx, R: PanelRegistryTarget<TabId, Ctx> + ?Sized>(
        &self,
        registry: &R,
    ) -> bool {
        self.tree.has_live_nodes()
            || self
                .live_scheduler
                .has_active_live_panel(self.dock_frame.as_ref(), registry)
    }

    fn run_services_frame<FInputs, FRun>(
        &mut self,
        dock: &DockState<TabId>,
        params: WorkbenchOwnedFrameParams,
        has_live_panels: bool,
        host_inputs: FInputs,
        on_run: FRun,
    ) -> Result<WorkbenchFrameOutcome<TabId>, LayoutError>
    where
        FInputs: FnOnce(LifecycleInputs) -> LifecycleInputs,
        FRun: FnOnce(
            &mut WorkbenchFrameRunner<'_, TabId>,
            WorkbenchFrameServices<'_, Action, TabId>,
        ) -> Result<WorkbenchFramePassResult, LayoutError>,
    {
        let update_params = WorkbenchFrameUpdateParams {
            dock,
            scale_factor: params.scale_factor,
            selection_id: params.selection_id,
            domain_revision: params.domain_revision,
            is_cursor_occluded: params.is_cursor_occluded,
        };
        let has_live = self.tree.has_live_nodes() || has_live_panels;
        let (selection_changed, domain_revision_changed, lifecycle_branch) = self
            .begin_frame_lifecycle(&update_params, |inputs| {
                host_inputs(inputs.with_volatile_updates(has_live))
            });

        let screen_size = self.screen_size().unwrap_or_default();
        let workspace_rect = self.compute_workspace_rect(screen_size.width, screen_size.height);
        let blink_caret = self.text_system.is_caret_visible();
        self.tree.caret_blink = blink_caret;
        let cursor_pos = self.effective_cursor_pos();
        let prev_dialog_depth = self.dialog_host.stack_depth();
        self.deferred_notifier = DeferredNotifier::new();

        let status_spans = self
            .status_message
            .as_ref()
            .filter(|msg| !msg.is_expired_at(std::time::Instant::now()))
            .map(crate::workbench::runner::TimedStatusMessage::spans);
        let zoom_factor = self.zoom_factor;

        let pass = {
            let mut runner = WorkbenchFrameRunner {
                branch: &lifecycle_branch,
                dock,
                is_cursor_occluded: params.is_cursor_occluded,
                slice: WorkbenchDomSlice {
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
                },
            };
            on_run(
                &mut runner,
                WorkbenchFrameServices {
                    action_bus: &mut self.action_bus,
                    dialog_host: &mut self.dialog_host,
                    notifier: &mut self.deferred_notifier,
                    blink_caret,
                    cursor_pos,
                    screen_size,
                    zoom_factor,
                    status_spans,
                },
            )?
        };
        let needs_text_prepare = pass.needs_text_prepare;
        let needs_quad_reupload = pass.needs_quad_reupload;
        let should_render_dialog = match &lifecycle_branch {
            LifecycleBranch::FullShellRebuild => self.dialog_host.is_active(),
            LifecycleBranch::GranularPanels(_) => {
                self.dialog_host.stack_depth() > prev_dialog_depth
            }
            _ => false,
        };
        let (needs_text_prepare, needs_quad_reupload) =
            if should_render_dialog && let Some(root) = self.tree.root() {
                let screen_rect = Rect::new(0.0, 0.0, screen_size.width, screen_size.height);
                let mut modal_scope = UiScope::new(&mut self.tree, root);
                self.dialog_host
                    .render(&mut modal_scope, screen_rect, false, &mut self.action_bus);
                self.dual_layout.mark_needs_text_prepare();
                (true, true)
            } else {
                (needs_text_prepare, needs_quad_reupload)
            };

        let outcome = self.finish_frame_outcome(FrameOutcomeSummary {
            lifecycle_branch,
            selection_changed,
            domain_revision_changed,
            volatile_refresh: pass.volatile_refresh,
            needs_text_prepare,
            needs_quad_reupload,
        });
        // Replay panel-emitted deferred invalidations after `finish_frame_outcome` clears the
        // completed frame's dirty set so they schedule work on the subsequent frame.
        self.notifier.apply_deferred(&mut self.deferred_notifier);
        Ok(outcome)
    }
}
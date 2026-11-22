// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Unified Application Workbench Host (`WorkbenchHost`)
//!
//! Combines [`Workbench`], [`ScopedPanelRegistry`], GPU [`IrisSurfaceHost`], and an
//! optional retained shell chrome builder into a single retained UI host so consumer
//! applications need no manual wiring glue, `pending_actions` buffers, or per-frame
//! cursor/surface flag plumbing.

use irisui_core::{ExternalTextureId, Rect, Size, Theme, UiEvent};
use irisui_dock::DockState;
use irisui_dock::style::DockChromeStyle;
use irisui_layout::LayoutError;
use irisui_wgpu::IrisSurfaceHost;
use irisui_widgets::UiScope;
use irisui_winit::frameless::WindowInterface;

use super::Workbench;
use super::frame::WorkbenchOwnedFrameParams;
use super::frame_services::{
    WorkbenchFramePassResult, WorkbenchFrameRunner, WorkbenchFrameServices,
};
use super::runner::{WorkbenchSurfaceFrameParams, WorkbenchSurfaceRender};
use crate::descriptors::panel::{WorkbenchAppData, WorkbenchAppScope, WorkbenchPanelContext};
use crate::descriptors::scoped_registry::ScopedPanelRegistry;
use crate::dialogs::ProgressModalDescriptor;

/// Boxed higher-ranked shell chrome callback rendered alongside dock panes inside [`WorkbenchHost`].
pub type WorkbenchChromeBuilderFn<A> = Box<
    dyn for<'scope, 'frame> FnMut(&mut UiScope<'scope>, &mut WorkbenchPanelContext<'frame, A>, Size)
        + Send
        + Sync,
>;

/// Summary returned from [`WorkbenchHost::handle_event`] after forwarding emitted UI actions
/// directly into the internal [`crate::ActionBus`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkbenchHostEventResult {
    /// Whether the UI consumed the input event (blocking underlying 3D/game input).
    pub consumed: bool,
    /// Whether a frameless window close control triggered an exit request.
    pub close_requested: bool,
    /// New UI zoom scale factor if a keyboard zoom shortcut changed it on this event.
    pub zoom_changed: Option<f32>,
}

/// Frame execution and WGPU target descriptor passed into [`WorkbenchHost::render_frame_with`].
pub struct WorkbenchHostFrameParams<'a, W: WindowInterface> {
    /// Active WGPU device handle.
    pub device: &'a wgpu::Device,
    /// Active WGPU submission queue.
    pub queue: &'a wgpu::Queue,
    /// Active WGPU command encoder recording overlay passes.
    pub encoder: &'a mut wgpu::CommandEncoder,
    /// Target window swapchain/framebuffer texture view.
    pub target_view: &'a wgpu::TextureView,
    /// Native platform window handle used for cursor synchronization.
    pub window: &'a mut W,
    /// Physical target surface dimensions `(width, height)`.
    pub physical_size: (u32, u32),
    /// Currently selected application entity/object identifier used for invalidation.
    pub selection_id: Option<u64>,
    /// Monotonic application domain history revision triggering domain-bound panel rebuilds.
    pub domain_revision: u64,
    /// Whether an external drag-and-drop operation is currently active.
    pub drag_active: bool,
    /// External surface texture revision (e.g. viewport RTT active/inactive).
    pub surface_revision: u64,
}

/// Scoped frame execution handle exposed inside [`WorkbenchHost::render_frame_with`].
pub struct WorkbenchHostPass<'pass, 'runner, 'wb, A: WorkbenchAppData> {
    runner: &'pass mut WorkbenchFrameRunner<'runner, A::TabId>,
    services: WorkbenchFrameServices<'wb, A::Action, A::TabId>,
    registry: &'pass mut ScopedPanelRegistry<A::TabId, WorkbenchAppScope<A>>,
    chrome_builder: &'pass mut Option<WorkbenchChromeBuilderFn<A>>,
    /// Logical non-UI viewport rectangle resolved for the active frame.
    pub viewport_rect: Rect,
}

impl<'pass, 'runner, 'wb, A: WorkbenchAppData> WorkbenchHostPass<'pass, 'runner, 'wb, A> {
    /// Executes the active lifecycle branch against the host's panel registry and chrome builder
    /// using frame-borrowed application `data`.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn execute(self, data: A::Data<'wb>) -> Result<WorkbenchFramePassResult, LayoutError> {
        let chrome_builder = self.chrome_builder;
        self.runner.execute(
            self.registry,
            &mut WorkbenchPanelContext::from_services(data, self.services),
            |scope, ctx, size| {
                if let Some(chrome) = chrome_builder.as_mut() {
                    chrome(scope, ctx, size);
                }
            },
        )
    }

    /// Evaluates `make_data` with [`Self::viewport_rect`] and immediately executes the frame pass.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn execute_with(
        self,
        make_data: impl FnOnce(Rect) -> A::Data<'wb>,
    ) -> Result<WorkbenchFramePassResult, LayoutError> {
        let viewport_rect = self.viewport_rect;
        self.execute(make_data(viewport_rect))
    }
}

/// Unified retained UI workbench host uniting [`Workbench`], [`ScopedPanelRegistry`],
/// [`IrisSurfaceHost`], and shell chrome rendering.
pub struct WorkbenchHost<A: WorkbenchAppData> {
    /// Autonomous dock, event, lifecycle, modal, and action-bus coordinator.
    pub workbench: Workbench<A::TabId, A::Action, ()>,
    /// Persistent self-owned panel registry storing per-panel state across rebuilds.
    pub registry: ScopedPanelRegistry<A::TabId, WorkbenchAppScope<A>>,
    /// Hardware SDF/text GPU surface host.
    pub surface: IrisSurfaceHost,
    chrome_builder: Option<WorkbenchChromeBuilderFn<A>>,
    last_viewport_rect: Rect,
}

impl<A: WorkbenchAppData> WorkbenchHost<A> {
    /// Creates a new [`WorkbenchHost`], automatically synchronizing domain-bound panels and
    /// metadata from `registry` into `workbench`.
    #[must_use]
    pub fn new(
        mut workbench: Workbench<A::TabId, A::Action, ()>,
        registry: ScopedPanelRegistry<A::TabId, WorkbenchAppScope<A>>,
        surface: IrisSurfaceHost,
    ) -> Self {
        workbench.sync_domain_panels_from_registry(&registry);
        Self {
            workbench,
            registry,
            surface,
            chrome_builder: None,
            last_viewport_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// Attaches a retained shell chrome builder invoked on full shell rebuilds (menubar,
    /// statusbar, application modals).
    #[must_use]
    pub fn with_chrome_builder(
        mut self,
        builder: impl for<'scope, 'frame> FnMut(
            &mut UiScope<'scope>,
            &mut WorkbenchPanelContext<'frame, A>,
            Size,
        ) + Send
        + Sync
        + 'static,
    ) -> Self {
        self.chrome_builder = Some(Box::new(builder));
        self
    }

    /// Applies initial visual [`Theme`] and [`DockChromeStyle`] across the workbench and retained tree.
    #[must_use]
    pub fn with_theme_and_dock_style(mut self, theme: Theme, dock_style: DockChromeStyle) -> Self {
        self.workbench.set_theme_and_dock_style(theme, dock_style);
        self
    }

    /// Runs one-time initialization against [`Self::surface`] during builder assembly.
    #[must_use]
    pub fn with_surface_init(mut self, init: impl FnOnce(&mut IrisSurfaceHost)) -> Self {
        init(&mut self.surface);
        self
    }

    /// Displays temporary color-coded spans inside the workbench status bar until expired.
    pub fn set_status_message(
        &mut self,
        spans: Vec<(String, irisui_core::Color)>,
        duration: std::time::Duration,
    ) {
        self.workbench.set_status_message(spans, duration);
    }

    /// Displays a single temporary status bar message for the default 2.5s timeout.
    pub fn set_status_text(&mut self, text: impl Into<String>, color: irisui_core::Color) {
        self.workbench.set_status_text(text, color);
    }

    /// Clears any active timed status notification immediately.
    #[inline]
    pub fn clear_status_message(&mut self) {
        self.workbench.clear_status_message();
    }

    /// Emits a custom application action onto the internal [`ActionBus`].
    #[inline]
    pub fn emit_action(&mut self, action: A::Action) {
        self.workbench.action_bus.emit(action);
    }

    /// Updates [`Theme`] and [`DockChromeStyle`] at runtime and marks the shell for redraw.
    pub fn set_theme_and_dock_style(&mut self, theme: Theme, dock_style: DockChromeStyle) {
        self.workbench.set_theme_and_dock_style(theme, dock_style);
        self.workbench.notifier.tag_all();
    }

    /// Dispatches a translated [`UiEvent`], updates GPU surface hover flags, pushes emitted
    /// actions directly onto [`Workbench::action_bus`], and propagates invalidations.
    pub fn handle_event<W: WindowInterface>(
        &mut self,
        window: &mut W,
        physical_size: (u32, u32),
        event: &UiEvent,
    ) -> WorkbenchHostEventResult {
        let outcome = self.workbench.handle_event_with_surface(
            &mut self.surface,
            window,
            physical_size,
            event,
        );
        if !outcome.actions.is_empty() {
            self.workbench.action_bus.emit_batch(outcome.actions);
        }
        WorkbenchHostEventResult {
            consumed: outcome.consumed,
            close_requested: outcome.close_requested,
            zoom_changed: outcome.zoom_changed,
        }
    }

    /// Executes one full frame pass: viewport layout preparation, cursor sync, lifecycle evaluation,
    /// panel & chrome rendering, deferred invalidation replay, and WGPU surface compositing.
    ///
    /// Returns the resolved logical non-UI viewport rectangle (e.g. 3D scene target bounds).
    ///
    /// # Errors
    /// Returns [`LayoutError`] if shell or dirty panel layout solving fails.
    pub fn render_frame_with<W, FRun>(
        &mut self,
        params: WorkbenchHostFrameParams<'_, W>,
        on_pass: FRun,
    ) -> Result<Rect, LayoutError>
    where
        W: WindowInterface,
        FRun: FnOnce(
            WorkbenchHostPass<'_, '_, '_, A>,
        ) -> Result<WorkbenchFramePassResult, LayoutError>,
    {
        let (surface_w, surface_h) = params.physical_size;
        if surface_w == 0 || surface_h == 0 {
            self.workbench.sync_cursor(params.window);
            return Ok(self.last_viewport_rect);
        }
        let zoom = self.workbench.scale_factor();
        let viewport_rect = self
            .workbench
            .prepare_viewport_layout(Size::new(surface_w as f32 / zoom, surface_h as f32 / zoom));
        self.last_viewport_rect = viewport_rect;

        let is_cursor_occluded = self.workbench.is_cursor_occluded();
        let has_live_panels = self.workbench.has_active_live_for(&self.registry);
        let registry = &mut self.registry;
        let chrome_builder = &mut self.chrome_builder;

        let _ = self.workbench.render_frame_to_surface(
            &mut self.surface,
            WorkbenchSurfaceFrameParams {
                frame: WorkbenchOwnedFrameParams {
                    scale_factor: zoom,
                    selection_id: params.selection_id,
                    domain_revision: params.domain_revision,
                    is_cursor_occluded,
                },
                surface: WorkbenchSurfaceRender {
                    device: params.device,
                    queue: params.queue,
                    encoder: params.encoder,
                    target_view: params.target_view,
                    physical_screen_size: params.physical_size,
                    zoom_factor: zoom,
                },
                window: params.window,
                has_live_panels,
            },
            |inputs| {
                inputs
                    .with_drag_active(params.drag_active)
                    .with_surface_revision(params.surface_revision)
            },
            |runner, services| {
                on_pass(WorkbenchHostPass {
                    runner,
                    services,
                    registry,
                    chrome_builder,
                    viewport_rect,
                })
            },
        )?;

        Ok(viewport_rect)
    }

    /// Drains all queued actions from [`Workbench::action_bus`] and passes each alongside `&mut Self`
    /// to `on_action`, allowing handlers to mutate dock layout, theme, or panel state without borrow conflicts.
    pub fn dispatch_actions<F>(&mut self, mut on_action: F)
    where
        F: FnMut(&mut Self, A::Action),
    {
        if self.workbench.action_bus.is_empty() {
            return;
        }
        let drained: Vec<A::Action> = self.workbench.action_bus.drain().collect();
        for action in drained {
            on_action(self, action);
        }
    }

    /// Renders an offscreen snapshot of the current retained UI tree using [`Self::surface`].
    pub fn render_offscreen_snapshot(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
    ) {
        self.workbench.render_to_surface(
            &mut self.surface,
            WorkbenchSurfaceRender {
                device,
                queue,
                encoder,
                target_view,
                physical_screen_size,
                zoom_factor,
            },
        );
    }

    /// Synchronizes an autonomous progress modal dialog via [`Workbench::set_loading_progress`].
    #[inline]
    pub fn set_loading_progress<F>(&mut self, active: bool, make_descriptor: F)
    where
        F: FnOnce() -> ProgressModalDescriptor<A::Action>,
    {
        self.workbench.set_loading_progress(active, make_descriptor);
    }

    /// Synchronizes logical workbench bounds from physical pixel dimensions.
    #[inline]
    pub fn resize(&mut self, physical_width: u32, physical_height: u32) {
        self.workbench
            .sync_physical_window_size(physical_width, physical_height);
    }

    /// Returns an immutable reference to persistent panel state owned by [`Self::registry`].
    #[inline]
    #[must_use]
    pub fn panel_state<State: Send + Sync + 'static>(&self, tab: &A::TabId) -> Option<&State> {
        self.registry.panel_state::<State>(tab)
    }

    /// Returns a mutable reference to persistent panel state owned by [`Self::registry`].
    #[inline]
    #[must_use]
    pub fn panel_state_mut<State: Send + Sync + 'static>(
        &mut self,
        tab: &A::TabId,
    ) -> Option<&mut State> {
        self.registry.panel_state_mut::<State>(tab)
    }

    /// Registers an external WGPU texture view onto [`Self::surface`].
    #[inline]
    pub fn set_external_texture(
        &mut self,
        device: &wgpu::Device,
        id: ExternalTextureId,
        view: Option<&wgpu::TextureView>,
    ) {
        self.surface.set_external_texture(device, id, view);
    }

    /// Returns the most recently resolved logical non-UI viewport rectangle.
    #[inline]
    #[must_use]
    pub const fn last_viewport_rect(&self) -> Rect {
        self.last_viewport_rect
    }

    /// Computes a 64-bit visibility bitmask for `tabs` in the active dock/floating layout.
    #[inline]
    #[must_use]
    pub fn tab_visibility_mask(&self, tabs: &[A::TabId]) -> u64 {
        self.workbench.tab_visibility_mask(tabs)
    }

    /// Marks `tab` dirty for granular rebuild on the next frame.
    #[inline]
    pub fn tag_redraw(&mut self, tab: A::TabId) {
        self.workbench.notifier.tag_redraw(tab);
    }

    /// Marks all panels and shell chrome dirty for full rebuild.
    #[inline]
    pub fn invalidate_all(&mut self) {
        self.workbench.notifier.tag_all();
    }

    /// Opens, focuses, or floats `tab` according to its registered [`irisui_dock::TabPlacement`].
    #[inline]
    pub fn open_or_focus_panel(&mut self, tab: A::TabId) {
        self.workbench.open_or_focus_panel(tab);
    }

    /// Toggles visibility or active focus of `tab` inside the dock hierarchy.
    #[inline]
    pub fn toggle_panel(&mut self, tab: A::TabId) {
        self.workbench.toggle_panel(tab);
    }

    /// Closes any active modal dialog hosted by [`Workbench::dialog_host`].
    #[inline]
    pub fn close_modal(&mut self) {
        self.workbench.dialog_host.close();
    }

    /// Resets the dock hierarchy to `dock_state` and triggers a full shell rebuild.
    #[inline]
    pub fn reset_dock_state(&mut self, dock_state: DockState<A::TabId>) {
        self.workbench.reset_dock_state(dock_state);
    }

    /// Returns `true` when `tab` is currently open in either docked leaves or floating windows.
    #[inline]
    #[must_use]
    pub fn is_panel_open(&self, tab: &A::TabId) -> bool {
        self.workbench.is_panel_open(tab)
    }

    /// Returns the active clamped UI scale factor (`[0.6, 2.0]`).
    #[inline]
    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        self.workbench.scale_factor()
    }

    /// Steps UI zoom up or down across predefined scale presets.
    #[inline]
    pub fn step_zoom(&mut self, increase: bool) -> f32 {
        self.workbench.step_zoom(increase)
    }

    /// Resets UI zoom to `1.0`.
    #[inline]
    pub fn reset_zoom(&mut self) -> f32 {
        self.workbench.reset_zoom()
    }

    /// Returns whether the pointer currently rests over interactive UI rather than the viewport.
    #[inline]
    #[must_use]
    pub fn wants_pointer_input(&self) -> bool {
        self.workbench.wants_pointer_self()
    }

    /// Evaluates whether logical coordinate `pos` hits interactive UI chrome or panels.
    #[inline]
    #[must_use]
    pub fn wants_pointer_at(&self, pos: [f32; 2]) -> bool {
        self.workbench.wants_pointer_at_self(pos)
    }

    /// Returns whether modals or retained text inputs currently capture keyboard focus.
    #[inline]
    #[must_use]
    pub fn wants_keyboard_input(&self) -> bool {
        self.workbench.wants_keyboard_input()
    }

    /// Returns whether an OS file drop is currently hovering over the window.
    #[inline]
    #[must_use]
    pub const fn is_file_drop_hovered(&self) -> bool {
        self.workbench.is_file_drop_hovered()
    }

    /// Returns an immutable reference to the authoritative [`DockState`].
    #[inline]
    #[must_use]
    pub fn dock_state(&self) -> &DockState<A::TabId> {
        self.workbench.dock_state()
    }

    /// Returns a mutable reference to the authoritative [`DockState`].
    #[inline]
    pub fn dock_state_mut(&mut self) -> &mut DockState<A::TabId> {
        self.workbench.dock_state_mut()
    }

    /// Returns a concise read-only telemetry summary of the retained UI without exposing [`irisui_core::UiTree`].
    #[inline]
    #[must_use]
    pub fn ui_inspection_summary(&self) -> super::inspect::UiInspectionSummary {
        self.workbench.ui_inspection_summary()
    }

    /// Captures a filtered node & tag-bounds snapshot of the active UI for automation or diagnostics.
    #[inline]
    #[must_use]
    pub fn inspect_ui_nodes(
        &self,
        tag_filter: Option<u64>,
        name_filter: Option<&str>,
        only_popups: bool,
    ) -> super::inspect::UiTreeDump {
        self.workbench
            .inspect_ui_nodes(tag_filter, name_filter, only_popups)
    }

    /// Resolves the bounding rectangle for a semantic widget `tag`, if laid out with non-zero size.
    #[inline]
    #[must_use]
    pub fn resolve_tag_rect(&self, tag: u64) -> Option<Rect> {
        self.workbench.resolve_tag_rect(tag)
    }

    /// Resolves the non-zero bounding rectangle for the `index`-th widget matching `name` or text.
    #[inline]
    #[must_use]
    pub fn resolve_widget_rect(&self, name: &str, index: usize) -> Option<Rect> {
        self.workbench.resolve_widget_rect(name, index)
    }

    /// Returns logical workbench screen bounds `[width, height]`, if synchronized.
    #[inline]
    #[must_use]
    pub fn logical_screen_size(&self) -> Option<[f32; 2]> {
        self.workbench.screen_size().map(|s| [s.width, s.height])
    }
}
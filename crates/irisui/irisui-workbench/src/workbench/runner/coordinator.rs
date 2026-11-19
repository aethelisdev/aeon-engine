// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Primary Workbench Coordinator and Struct Definition
//!
//! Encapsulates the core [`Workbench`] struct definition, constructor, and fluent builder
//! methods governing application descriptors, chrome dimensions, and overlay registrations.

use super::action_types::{FileDropActionBuilder, PreferencesActionBuilder, ZoomActionBuilder};
use crate::action_bus::ActionBus;
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::dialogs::host::DialogHost;
use crate::hooks::overlay::{OverlayHook, OverlayHost, OverlayId};
use crate::hooks::volatile::VolatileHookRegistry;
use crate::layout_sync::dual_layout::DualLayoutEliminator;
use crate::lifecycle::WorkbenchLifecycle;
use crate::workbench::pointer::PointerState;
use irisui_core::geometry::{Point, Rect, Size};
use irisui_core::node::{UiLayer, WidgetCursor};
use irisui_core::theme::Theme;
use irisui_core::tree::UiTree;
use irisui_dock::UiNotifier;
use irisui_dock::chrome::DockChromeFrame;
use irisui_dock::state::DockState;
use irisui_dock::style::DockChromeStyle;
use irisui_layout::LayoutEngine;
use irisui_text::TextSystem;
use irisui_winit::frameless::{FramelessWindowHost, WindowDragHost, WindowInterface};
use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;

/// Autonomous desktop IDE and workbench shell orchestrator.
///
/// Encapsulates all desktop application management:
/// - Retained multi-pane docking hierarchy (`DockState<TabId>`).
/// - 8-directional frameless window border resizing (`FramelessWindowHost`).
/// - Titlebar and menubar window dragging and double-click maximize (`WindowDragHost`).
/// - Framework-level modal dialogues and confirmations (`DialogHost<Action>`).
/// - Decoupled asynchronous action bus (`ActionBus<Action>`).
/// - Autonomous panel registry delegating content drawing (`WorkbenchPanelRegistry<TabId, AppState>`).
pub struct Workbench<
    TabId: Eq + Hash + Copy + Send + Sync + 'static,
    Action: 'static + Send + Sync,
    AppState,
> {
    /// Declarative configuration descriptor governing application shell chrome.
    pub descriptor: WorkbenchDescriptor<TabId>,
    /// Autonomous modal dialog host managing confirmations, text inputs, and focus traps.
    pub dialog_host: DialogHost<Action>,
    /// Central decoupled action bus collecting emitted application actions.
    pub action_bus: ActionBus<Action>,
    /// Autonomous panel registry storing panels and tab viewers.
    pub panel_registry: WorkbenchPanelRegistry<TabId, AppState>,
    /// Autonomous overlay layer host managing registered floating HUDs and contextual popovers.
    pub overlay_host: OverlayHost<AppState, Action>,
    /// Frameless window host managing 8-way border hit-testing and hardware resize operations.
    pub frameless_host: FramelessWindowHost,
    /// Window drag host managing titlebar/menubar dragging and double-click maximize behavior.
    pub drag_host: WindowDragHost,
    /// Autonomous dual-layout eliminator coordinating $O(M)$ Taffy subtree solves and text preparation.
    pub dual_layout: DualLayoutEliminator,
    /// Standard menubar height in logical pixels. Defaults to 30.0.
    pub menubar_height: f32,
    /// Standard statusbar height in logical pixels. Defaults to 24.0.
    pub statusbar_height: f32,
    /// Dock chrome visual style configuration.
    pub dock_style: DockChromeStyle,
    /// Active visual theme tokens propagated to retained UI trees during shell and panel builds.
    pub theme: Theme,
    /// Monotonic revision counter incremented whenever [`Self::set_theme`] or [`Self::set_dock_style`] changes styling.
    pub theme_revision: u64,
    /// UI zoom scaling factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
    /// Bounding rectangle of the interactive non-UI viewport canvas (e.g. 3D scene), if any.
    pub interactive_viewport: Option<Rect>,

    /// Most recently observed cursor position in logical points.
    pub last_cursor_pos: Option<Point>,

    /// Autonomous pointer state tracker recording cursor position and button down states.
    pub pointer_state: PointerState,

    /// Most recently observed host window size in logical pixels.
    pub last_window_size: Option<Size>,

    /// Optional identifier of the non-UI interactive viewport tab (e.g. 3D scene canvas).
    pub non_ui_tab: Option<TabId>,

    /// Optional action mapping hook invoked when external files are dropped onto the host window.
    pub file_drop_action: Option<FileDropActionBuilder<Action>>,

    /// Optional action mapping hook invoked when UI scale factor is adjusted via hotkeys.
    pub zoom_action: Option<ZoomActionBuilder<Action>>,

    /// Optional action mapping hook invoked when application preferences modal is toggled.
    pub preferences_action: Option<PreferencesActionBuilder<Action>>,

    /// Optional dock tab identifier toggled directly on `Ctrl+,` preferences hotkey.
    pub preferences_panel: Option<TabId>,

    /// Optional `(minimize, maximize, close)` semantic tags for native window control dispatching.
    pub window_control_tags: Option<(u64, u64, u64)>,

    /// Active docking interaction frame from the most recent shell assembly pass.
    pub dock_frame: Option<DockChromeFrame<TabId>>,

    /// Currently hovered 64-bit semantic interaction tag resolved by layered hit-testing.
    pub hovered_tag: Option<u64>,

    /// Previously evaluated hovered 64-bit semantic interaction tag for transition detection.
    pub last_hovered_tag: Option<u64>,

    /// Previously evaluated hovered widget identifier for detecting untagged hover transitions.
    pub last_hit_widget: Option<irisui_core::WidgetId>,

    /// Currently hovered non-excluded dock panel tab resolved under the cursor, if not occluded by overlays.
    pub hovered_panel: Option<TabId>,

    /// Previously evaluated hovered dock panel tab for transition detection.
    pub last_hovered_panel: Option<TabId>,

    /// Autonomous 3-branch lifecycle state machine tracking shell vs panel invalidation.
    pub lifecycle: WorkbenchLifecycle<TabId>,

    /// Most recent caret blink animation evaluation produced by synchronizing [`irisui_text::TextSystem`].
    pub last_caret_blink: irisui_text::CaretBlinkUpdate,

    /// Most recently synchronized opaque domain selection identifier (e.g. entity/node ID).
    pub last_selection_id: Option<u64>,

    /// Most recently synchronized external domain/history revision counter.
    pub last_domain_revision: u64,

    /// Registry of volatile in-place canvas/telemetry refresh hooks executed during `InPlaceRefresh`.
    pub volatile_hooks: VolatileHookRegistry<AppState>,

    /// Retained UI widget tree owned directly by this workbench.
    pub tree: UiTree,

    /// Persistent flexbox layout solver synchronized with [`Self::tree`].
    pub layout_engine: LayoutEngine,

    /// Persistent typography shaping and caret blink engine owned by this workbench.
    pub text_system: TextSystem,

    /// Selective panel and shell invalidation notifier owned by this workbench.
    pub notifier: UiNotifier<TabId>,

    /// Last widget cursor dispatched to the platform window to suppress redundant OS IPC.
    last_synced_window_cursor: Option<WidgetCursor>,

    /// Scratch buffer storing active floating window bounds for zero-allocation cursor evaluation.
    cached_floating_rects: Vec<Rect>,

    /// Cached `(dock_revision, floating_hash, workspace_rect, is_maximized)` key for viewport resolution.
    pub(super) cached_viewport_layout_key: Option<(u64, u64, Rect, bool)>,

    /// Cached sanitized screen rectangle of `non_ui_tab` paired with `cached_viewport_layout_key`.
    pub(super) cached_non_ui_viewport_rect: Rect,
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + Send + Sync + 'static,
    Action: Send + Sync + 'static,
{
    /// Creates a new autonomous workbench shell with default configurations.
    #[must_use]
    pub fn new(descriptor: WorkbenchDescriptor<TabId>) -> Self {
        Self {
            descriptor,
            dialog_host: DialogHost::new(),
            action_bus: ActionBus::new(),
            panel_registry: WorkbenchPanelRegistry::new(),
            overlay_host: OverlayHost::new(),
            frameless_host: FramelessWindowHost::new(),
            drag_host: WindowDragHost::new().with_title_bar_height(30.0),
            dual_layout: DualLayoutEliminator::new(),
            menubar_height: 30.0,
            statusbar_height: 24.0,
            dock_style: DockChromeStyle::default(),
            theme: Theme::default(),
            theme_revision: 0,
            zoom_factor: 1.0,
            interactive_viewport: None,
            last_cursor_pos: None,
            pointer_state: PointerState::new(),
            last_window_size: None,
            non_ui_tab: None,
            file_drop_action: None,
            zoom_action: None,
            preferences_action: None,
            preferences_panel: None,
            window_control_tags: None,
            dock_frame: None,
            hovered_tag: None,
            last_hovered_tag: None,
            last_hit_widget: None,
            hovered_panel: None,
            last_hovered_panel: None,
            lifecycle: WorkbenchLifecycle::new(),
            last_caret_blink: irisui_text::CaretBlinkUpdate::default(),
            last_selection_id: None,
            last_domain_revision: 0,
            volatile_hooks: VolatileHookRegistry::new(),
            tree: UiTree::new(),
            layout_engine: LayoutEngine::new(),
            text_system: TextSystem::new(),
            notifier: UiNotifier::new(),
            last_synced_window_cursor: None,
            cached_floating_rects: Vec::new(),
            cached_viewport_layout_key: None,
            cached_non_ui_viewport_rect: Rect::ZERO,
        }
    }

    /// Configures an action builder to emit whenever external files are dropped into the window.
    #[must_use]
    pub fn with_file_drop_action<F>(mut self, builder: F) -> Self
    where
        F: Fn(Vec<PathBuf>, Point) -> Action + 'static + Send + Sync,
    {
        self.file_drop_action = Some(Box::new(builder));
        self
    }

    /// Configures an action builder to emit whenever the UI scale is altered via hotkeys (Ctrl +/-/0).
    #[must_use]
    pub fn with_zoom_action<F>(mut self, builder: F) -> Self
    where
        F: Fn(f32) -> Action + 'static + Send + Sync,
    {
        self.zoom_action = Some(Box::new(builder));
        self
    }

    /// Configures an action builder to emit whenever preferences are toggled via hotkey (Ctrl+,).
    #[must_use]
    pub fn with_preferences_action<F>(mut self, builder: F) -> Self
    where
        F: Fn() -> Action + 'static + Send + Sync,
    {
        self.preferences_action = Some(Box::new(builder));
        self
    }

    /// Configures a dock tab identifier to toggle autonomously whenever `Ctrl+,` is pressed.
    #[must_use]
    pub fn with_preferences_panel(mut self, panel: TabId) -> Self {
        self.preferences_panel = Some(panel);
        self
    }

    /// Configures 64-bit semantic tags for chrome window control buttons (`minimize`, `maximize`, `close`).
    ///
    /// When clicked during event dispatch, [`Self::drag_host`] executes the matching
    /// [`irisui_widgets::TitleBarAction`] immediately onto the platform [`WindowInterface`].
    #[must_use]
    pub fn with_window_control_tags(
        mut self,
        minimize_tag: u64,
        maximize_tag: u64,
        close_tag: u64,
    ) -> Self {
        self.window_control_tags = Some((minimize_tag, maximize_tag, close_tag));
        self
    }

    /// Configures the identifier of the non-UI interactive viewport tab.
    #[must_use]
    pub fn with_non_ui_tab(mut self, tab: TabId) -> Self {
        self.non_ui_tab = Some(tab);
        self
    }

    /// Registers an overlay hook into the workbench's autonomous overlay layer.
    #[must_use]
    pub fn with_overlay<H>(mut self, hook: H) -> Self
    where
        H: OverlayHook<AppState, Action> + 'static,
    {
        self.overlay_host.register(hook);
        self
    }

    /// Registers an overlay hook into the workbench's autonomous overlay layer.
    pub fn register_overlay<H>(&mut self, hook: H)
    where
        H: OverlayHook<AppState, Action> + 'static,
    {
        self.overlay_host.register(hook);
    }

    /// Removes an overlay hook by its unique identifier.
    pub fn remove_overlay(
        &mut self,
        id: OverlayId,
    ) -> Option<Box<dyn OverlayHook<AppState, Action>>> {
        self.overlay_host.remove(id)
    }

    /// Evaluates whether any registered overlay hook is currently active.
    #[must_use]
    pub fn has_active_overlays(&self, state: &AppState) -> bool {
        self.overlay_host.has_active_overlays(state)
    }

    /// Sets the top application menubar height in logical pixels.
    #[must_use]
    pub fn with_menubar_height(mut self, height: f32) -> Self {
        self.menubar_height = height;
        self.drag_host = self.drag_host.with_title_bar_height(height);
        self
    }

    /// Sets the bottom status bar height in logical pixels.
    #[must_use]
    pub fn with_statusbar_height(mut self, height: f32) -> Self {
        self.statusbar_height = height;
        self
    }

    /// Sets the dock chrome visual style configuration via fluent builder pattern.
    #[must_use]
    pub fn with_dock_style(mut self, style: DockChromeStyle) -> Self {
        self.dock_style = style;
        self
    }

    /// Updates the dock chrome visual style, bumping `theme_revision` and marking the shell
    /// dirty if the style changed.
    #[inline]
    pub fn set_dock_style(&mut self, style: DockChromeStyle) {
        if self.dock_style != style {
            self.dock_style = style;
            self.theme_revision = self.theme_revision.wrapping_add(1);
            self.lifecycle.mark_shell_dirty();
        }
    }

    /// Returns an immutable reference to the active dock chrome visual style configuration.
    #[inline]
    #[must_use]
    pub const fn dock_style(&self) -> &DockChromeStyle {
        &self.dock_style
    }

    /// Configures the initial visual [`Theme`] tokens via fluent builder pattern.
    #[must_use]
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Updates the active visual [`Theme`] tokens for the workbench.
    ///
    /// Comparing against the current theme guarantees zero-cost idempotence when invoked per-frame
    /// with unchanged tokens. When tokens differ, `theme_revision` increments and the lifecycle
    /// state machine flags a full shell rebuild so chrome and panels re-style cleanly.
    #[inline]
    pub fn set_theme(&mut self, theme: Theme) {
        if self.theme != theme {
            self.theme = theme;
            self.theme_revision = self.theme_revision.wrapping_add(1);
            self.lifecycle.mark_shell_dirty();
        }
    }

    /// Returns an immutable reference to the active visual [`Theme`] tokens.
    #[inline]
    #[must_use]
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Returns the monotonic styling revision counter incremented on theme or dock style changes.
    #[inline]
    #[must_use]
    pub const fn theme_revision(&self) -> u64 {
        self.theme_revision
    }

    /// Configures the initial logical screen size via fluent builder pattern.
    ///
    /// Non-finite values are ignored; negative finite extents are clamped to `0.0`.
    #[must_use]
    pub fn with_screen_size(mut self, size: impl Into<Size>) -> Self {
        self.set_screen_size(size);
        self
    }

    /// Updates the logical screen dimensions tracked by the workbench.
    ///
    /// Accepts [`Size`], `(f32, f32)`, or `[f32; 2]`. Ignores non-finite (`NaN` / infinite) values
    /// and clamps negative extents to `0.0`. When the resolved size differs from the previous
    /// value, the shell lifecycle is marked dirty so subsequent passes reconstruct layout cleanly.
    #[inline]
    pub fn set_screen_size(&mut self, size: impl Into<Size>) {
        let size = size.into();
        if !size.width.is_finite() || !size.height.is_finite() {
            return;
        }
        let sanitized = Size::new(size.width.max(0.0), size.height.max(0.0));
        if self.last_window_size != Some(sanitized) {
            self.last_window_size = Some(sanitized);
            self.lifecycle.mark_shell_dirty();
        }
    }

    /// Returns the most recently configured or observed logical screen size, if available.
    #[inline]
    #[must_use]
    pub const fn screen_size(&self) -> Option<Size> {
        self.last_window_size
    }

    /// Sets the bounding rectangle of the interactive non-UI viewport canvas (in logical points).
    #[inline]
    pub fn set_interactive_viewport(&mut self, viewport: Option<Rect>) {
        self.interactive_viewport = viewport;
    }

    /// Configures the interactive non-UI viewport bounding rectangle via builder pattern.
    #[inline]
    #[must_use]
    pub fn with_interactive_viewport(mut self, viewport: Option<Rect>) -> Self {
        self.interactive_viewport = viewport;
        self
    }

    /// Returns the most recently observed cursor position in logical points, if available.
    #[inline]
    #[must_use]
    pub fn cursor_pos(&self) -> Option<Point> {
        self.last_cursor_pos
    }

    /// Sets or updates the internally tracked cursor position.
    #[inline]
    pub fn set_cursor_pos(&mut self, point: Option<Point>) {
        self.last_cursor_pos = point;
        self.pointer_state.set_position(point);
    }

    /// Returns the most recently observed window size in logical pixels, if available.
    #[inline]
    #[must_use]
    pub fn window_size(&self) -> Option<Size> {
        self.last_window_size
    }

    /// Returns the most recently resolved hovered 64-bit semantic interaction tag, if any.
    #[inline]
    #[must_use]
    pub const fn hovered_tag(&self) -> Option<u64> {
        self.hovered_tag
    }

    /// Returns the most recently resolved hovered dock panel tab identifier, if any.
    #[inline]
    #[must_use]
    pub const fn hovered_panel(&self) -> Option<TabId> {
        self.hovered_panel
    }

    /// Evaluates whether the workbench currently requires active text keyboard focus
    /// (e.g. when a modal dialog text input or retained tree text input is active).
    #[inline]
    #[must_use]
    pub fn wants_keyboard_input(&self) -> bool {
        self.dialog_host.is_active() || self.tree.is_text_input_focused()
    }

    /// Returns an immutable reference to the owned retained [`UiTree`].
    #[inline]
    #[must_use]
    pub const fn tree(&self) -> &UiTree {
        &self.tree
    }

    /// Returns a mutable reference to the owned retained [`UiTree`].
    #[inline]
    pub fn tree_mut(&mut self) -> &mut UiTree {
        &mut self.tree
    }

    /// Returns an immutable reference to the owned [`UiNotifier`].
    #[inline]
    #[must_use]
    pub const fn notifier(&self) -> &UiNotifier<TabId> {
        &self.notifier
    }

    /// Returns a mutable reference to the owned [`UiNotifier`].
    #[inline]
    pub fn notifier_mut(&mut self) -> &mut UiNotifier<TabId> {
        &mut self.notifier
    }

    /// Returns an immutable reference to the owned [`TextSystem`].
    #[inline]
    #[must_use]
    pub const fn text_system(&self) -> &TextSystem {
        &self.text_system
    }

    /// Returns the effective logical pointer coordinate resolved from the retained tree
    /// or last tracked pointer position.
    #[inline]
    #[must_use]
    pub fn effective_cursor_pos(&self) -> Point {
        self.tree
            .cursor_position()
            .or(self.last_cursor_pos)
            .unwrap_or_default()
    }

    /// Returns `true` if `point` hits an elevated overlay node (`UiLayer::Floating` or higher).
    #[inline]
    #[must_use]
    pub fn is_point_over_overlay(&self, point: Point) -> bool {
        self.tree
            .hit_test_layered(point)
            .is_some_and(|id| self.tree.effective_layer(id) >= UiLayer::Floating)
    }

    /// Returns `true` when the active pointer position is occluded by either an elevated overlay
    /// node or any detached floating dock window.
    #[inline]
    #[must_use]
    pub fn is_cursor_occluded(&self) -> bool {
        let pos = self.effective_cursor_pos();
        self.is_point_over_overlay(pos)
            || self
                .descriptor
                .dock_state
                .is_point_over_floating_window(pos)
    }

    /// Returns the sanitized screen rectangle of [`Self::non_ui_tab`], or [`Rect::ZERO`] if none is active.
    #[inline]
    #[must_use]
    pub fn non_ui_viewport_rect(&self) -> Rect {
        self.interactive_viewport.unwrap_or(Rect::ZERO)
    }

    /// Returns `true` when the active pointer resides inside the resolved non-UI viewport canvas
    /// without being occluded by elevated overlays or detached floating dock windows.
    #[inline]
    #[must_use]
    pub fn is_cursor_over_non_ui_viewport(&self) -> bool {
        let vp = self.non_ui_viewport_rect();
        let pos = self.effective_cursor_pos();
        vp.width > 0.0
            && vp.height > 0.0
            && pos.x >= vp.x
            && pos.x <= vp.right()
            && pos.y >= vp.y
            && pos.y <= vp.bottom()
            && !self.is_cursor_occluded()
    }

    /// Resolves the desired OS widget cursor across elevated overlays, floating resize borders,
    /// and retained tree widgets for the supplied docking state without heap allocation.
    ///
    /// Returns `None` when the pointer rests over unoccluded transparent/non-UI canvas.
    pub fn resolve_window_cursor(&mut self, dock: &DockState<TabId>) -> Option<WidgetCursor> {
        let p = self.effective_cursor_pos();
        let is_overlay_hit = self.is_point_over_overlay(p);

        if is_overlay_hit {
            let tree_cursor = self.tree.cursor_at(p);
            if tree_cursor != WidgetCursor::Default {
                return Some(tree_cursor);
            }
        }

        if p.y > self.menubar_height && !is_overlay_hit {
            dock.sync_floating_window_rects(&mut self.cached_floating_rects);
            if let Some(cur) = irisui_dock::evaluate_floating_resize_cursor(
                &self.cached_floating_rects,
                p,
                irisui_dock::floating::DEFAULT_RESIZE_MARGIN,
            ) {
                return Some(cur.to_widget_cursor());
            }
        }

        let tree_cursor = self.tree.cursor_at(p);
        if tree_cursor != WidgetCursor::Default {
            Some(tree_cursor)
        } else if is_overlay_hit {
            Some(WidgetCursor::Default)
        } else {
            None
        }
    }

    /// Synchronizes the platform window cursor using the workbench's owned docking state,
    /// suppressing redundant OS IPC calls when the resolved cursor shape is unchanged.
    pub fn sync_cursor<W: WindowInterface>(&mut self, window: &mut W) {
        let dock = std::mem::take(&mut self.descriptor.dock_state);
        self.sync_window_cursor(window, &dock);
        self.descriptor.dock_state = dock;
    }

    /// Synchronizes the platform window cursor only when the resolved cursor icon changes.
    pub fn sync_window_cursor<W: WindowInterface>(
        &mut self,
        window: &mut W,
        dock: &DockState<TabId>,
    ) {
        let target = self.resolve_window_cursor(dock);
        if let Some(cursor) = target {
            if self.last_synced_window_cursor != Some(cursor) {
                window.set_widget_cursor(cursor);
                self.last_synced_window_cursor = Some(cursor);
            }
        } else {
            self.last_synced_window_cursor = None;
        }
    }

    /// Returns a mutable reference to the underlying docking state.
    #[inline]
    pub fn dock_state_mut(&mut self) -> &mut DockState<TabId> {
        &mut self.descriptor.dock_state
    }

    /// Returns an immutable reference to the underlying docking state.
    #[inline]
    #[must_use]
    pub fn dock_state(&self) -> &DockState<TabId> {
        &self.descriptor.dock_state
    }
}
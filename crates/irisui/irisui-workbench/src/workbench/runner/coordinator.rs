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
use crate::layout_sync::dual_layout::DualLayoutEliminator;
use crate::lifecycle::WorkbenchLifecycle;
use crate::workbench::pointer::PointerState;
use irisui_core::geometry::{Point, Rect, Size};
use irisui_core::theme::Theme;
use irisui_dock::chrome::DockChromeFrame;
use irisui_dock::state::DockState;
use irisui_dock::style::DockChromeStyle;
use irisui_winit::frameless::{FramelessWindowHost, WindowDragHost};
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
pub struct Workbench<TabId: Eq + Hash + Clone, Action: 'static + Send + Sync, AppState> {
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
}

impl<TabId, Action, AppState> Workbench<TabId, Action, AppState>
where
    TabId: Clone + Copy + Eq + Hash + Debug + 'static,
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
            dock_frame: None,
            hovered_tag: None,
            last_hovered_tag: None,
            last_hit_widget: None,
            hovered_panel: None,
            last_hovered_panel: None,
            lifecycle: WorkbenchLifecycle::new(),
            last_caret_blink: irisui_text::CaretBlinkUpdate::default(),
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
    /// (e.g. when a modal dialog text input is active).
    #[inline]
    #[must_use]
    pub fn wants_keyboard_input(&self) -> bool {
        self.dialog_host.is_active()
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
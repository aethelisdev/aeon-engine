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
use iris_core::geometry::{Point, Rect, Size};
use iris_dock::chrome::DockChromeFrame;
use iris_dock::state::DockState;
use iris_dock::style::DockChromeStyle;
use iris_winit::frameless::{FramelessWindowHost, WindowDragHost};
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

    /// Autonomous 3-branch lifecycle state machine tracking shell vs panel invalidation.
    pub lifecycle: WorkbenchLifecycle<TabId>,
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
            lifecycle: WorkbenchLifecycle::new(),
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

    /// Sets the dock chrome visual style configuration.
    #[must_use]
    pub fn with_dock_style(mut self, style: DockChromeStyle) -> Self {
        self.dock_style = style;
        self
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
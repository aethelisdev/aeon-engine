// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Autonomous Workbench Application Orchestrator
//!
//! Provides the primary [`Workbench`] struct combining retained-mode docking,
//! frameless window management, modal dialogs, and decoupled action dispatching
//! into a unified, crates.io-grade desktop IDE framework component.

use super::outcome::WorkbenchOutcome;
use super::pointer::PointerState;
use crate::action_bus::ActionBus;
use crate::descriptors::panel_registry::WorkbenchPanelRegistry;
use crate::descriptors::workbench::WorkbenchDescriptor;
use crate::dialogs::host::DialogHost;
use crate::hooks::overlay::{OverlayHook, OverlayHost, OverlayId};
use crate::layout_sync::dual_layout::{DualLayoutEliminator, SubtreeEngines};
use crate::lifecycle::WorkbenchLifecycle;
use iris_core::WidgetId;
use iris_core::event::UiEvent;
use iris_core::geometry::{Point, Rect, Size};
use iris_core::tree::UiTree;
use iris_dock::UiNotifier;
use iris_dock::area::runner::{DockArea, DockInteractionConfig};
use iris_dock::chrome::DockChromeFrame;
use iris_dock::floating::FloatingWindowClampBounds;
use iris_dock::state::DockState;
use iris_dock::style::DockChromeStyle;
use iris_layout::LayoutError;
use iris_winit::frameless::{FramelessWindowHost, WindowDragHost, WindowInterface};
use std::fmt::Debug;
use std::hash::Hash;

/// Type alias for external file drag-and-drop action handler closure.
pub type FileDropActionBuilder<Action> =
    Box<dyn Fn(Vec<std::path::PathBuf>, Point) -> Action + Send + Sync>;

/// Type alias for UI zoom alteration action handler closure.
pub type ZoomActionBuilder<Action> = Box<dyn Fn(f32) -> Action + Send + Sync>;

/// Type alias for preferences modal toggle action handler closure.
pub type PreferencesActionBuilder<Action> = Box<dyn Fn() -> Action + Send + Sync>;

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
        F: Fn(Vec<std::path::PathBuf>, Point) -> Action + 'static + Send + Sync,
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

    /// Rebuilds all dirty dock leaf panels flagged in `notifier` using the workbench's
    /// internal [`DualLayoutEliminator`] and registered [`WorkbenchPanelRegistry`].
    ///
    /// Automatically marks [`Self::needs_text_prepare`] if any panels were reconstructed.
    ///
    /// # Errors
    /// Returns [`LayoutError`] if layout computation encounters invalid geometry.
    pub fn rebuild_dirty_panels<F>(
        &mut self,
        engines: &mut SubtreeEngines<'_>,
        notifier: &mut UiNotifier<TabId>,
        context: &mut AppState,
        leaf_resolver: F,
    ) -> Result<usize, LayoutError>
    where
        TabId: Eq + Hash + Copy + Debug + Send + Sync + 'static,
        F: FnMut(TabId) -> Option<WidgetId>,
    {
        self.dual_layout.rebuild_dirty_panels(
            engines,
            notifier,
            &mut self.panel_registry,
            context,
            leaf_resolver,
        )
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

    /// Returns the active UI scale factor clamped to the valid range `[0.6, 2.0]`.
    #[inline]
    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        if self.zoom_factor.is_finite() && self.zoom_factor > 0.1 {
            self.zoom_factor.clamp(0.6, 2.0)
        } else {
            1.0
        }
    }

    /// Steps the active UI zoom factor up or down across the predefined scale presets.
    ///
    /// Presets: `[0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50]`.
    pub fn step_zoom(&mut self, increase: bool) -> f32 {
        const PRESETS: [f32; 7] = [0.75, 0.80, 0.90, 1.00, 1.10, 1.25, 1.50];
        let current = self.scale_factor();
        let target = if increase {
            PRESETS
                .iter()
                .copied()
                .find(|&s| s > current + 0.01)
                .unwrap_or(*PRESETS.last().unwrap_or(&1.50))
        } else {
            PRESETS
                .iter()
                .copied()
                .rfind(|&s| s < current - 0.01)
                .unwrap_or(*PRESETS.first().unwrap_or(&0.75))
        };
        self.zoom_factor = target;
        target
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

    /// Resets the UI zoom factor to default 100% (1.0).
    #[inline]
    pub fn reset_zoom(&mut self) -> f32 {
        self.zoom_factor = 1.0;
        1.0
    }

    /// Evaluates whether the workbench currently requires active text keyboard focus
    /// (e.g. when a modal dialog text input is active).
    #[inline]
    #[must_use]
    pub fn wants_keyboard_input(&self) -> bool {
        self.dialog_host.is_active()
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous workbench interaction pipeline
    /// using the internal docking state.
    pub fn handle_event<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
    ) -> WorkbenchOutcome<Action> {
        let mut dock_state = std::mem::take(&mut self.descriptor.dock_state);
        let outcome = self.handle_event_full(window, event, &mut dock_state, None);
        self.descriptor.dock_state = dock_state;
        outcome
    }

    /// Creates a base [`WorkbenchOutcome`] configured with consumption status, cursor shape,
    /// and the active pointer coordinate.
    #[inline]
    #[must_use]
    pub fn make_outcome(
        &self,
        consumed: bool,
        cursor: Option<iris_core::node::WidgetCursor>,
    ) -> WorkbenchOutcome<Action> {
        WorkbenchOutcome {
            consumed,
            cursor,
            cursor_pos: self.last_cursor_pos,
            layout_changed: false,
            actions: Vec::new(),
            dropped_files: None,
            zoom_changed: None,
            toggle_preferences: false,
        }
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous shell layer coordinating
    /// frameless window border resizing, titlebar dragging on empty menubar space, modal dialog
    /// keyboard traps, global shortcuts, and dock interaction while respecting content consumption.
    pub fn handle_shell_event<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        content_consumed: bool,
    ) -> WorkbenchOutcome<Action> {
        let logical_size = window.logical_size();
        let screen_w = logical_size.width;
        let screen_h = logical_size.height;
        self.last_window_size = Some(logical_size);

        // Track cursor position and pointer states from events
        self.pointer_state.update_from_event(event);
        self.last_cursor_pos = self.pointer_state.position;

        // 1. Frameless Window 8-way Native Edge Resizing
        if self
            .frameless_host
            .handle_ui_event(window, event, logical_size)
        {
            return self.make_outcome(true, None);
        }

        // If the inner panel/overlay content already consumed the event, skip shell and dock interaction
        if content_consumed {
            let mut outcome = self.make_outcome(true, None);
            outcome.actions.extend(self.action_bus.drain());
            return outcome;
        }

        // 2. Active framework modal dialog keyboard input
        if let UiEvent::KeyboardInput {
            key,
            is_pressed,
            shift_held,
            ..
        } = event
            && self.dialog_host.is_active()
        {
            let handled = self.dialog_host.handle_keyboard_input_with_modifiers(
                key,
                *is_pressed,
                *shift_held,
                &mut self.action_bus,
            );
            if handled {
                let mut outcome = self.make_outcome(true, None);
                outcome.layout_changed = true;
                outcome.actions.extend(self.action_bus.drain());
                return outcome;
            }
        }

        // 3. Ingest dropped files directly into outcome
        if let UiEvent::FileDropped { paths, point } = event
            && !paths.is_empty()
        {
            let mut outcome = self.make_outcome(true, None);
            if let Some(builder) = &self.file_drop_action {
                outcome.actions.push(builder(paths.clone(), *point));
            }
            outcome.dropped_files = Some((paths.clone(), *point));
            outcome.layout_changed = true;
            return outcome;
        }

        // 4. Native Title Bar Dragging & Maximize Double-Click on empty menubar space
        if self.descriptor.menubar.is_some() {
            let menubar_rect = Rect::new(0.0, 0.0, screen_w, self.menubar_height);
            if self.drag_host.handle_ui_event(window, event, menubar_rect) {
                return self.make_outcome(true, None);
            }
        }

        // 5. Global Keyboard Shortcuts (Ctrl +/-/0 for zoom, Ctrl+, for preferences)
        if let UiEvent::KeyboardInput {
            key,
            is_pressed,
            text,
            ctrl_held,
            ..
        } = event
            && *is_pressed
            && *ctrl_held
            && !self.wants_keyboard_input()
        {
            let ch = match key {
                iris_core::event::KeyCode::Character(c) => Some(*c),
                _ => text.as_deref().and_then(|t| t.chars().next()),
            };

            if let Some(c) = ch {
                let zoom_target = match c {
                    '+' | '=' => Some(self.step_zoom(true)),
                    '-' | '_' => Some(self.step_zoom(false)),
                    '0' => Some(self.reset_zoom()),
                    _ => None,
                };

                if let Some(new_zoom) = zoom_target {
                    let mut outcome = self.make_outcome(true, None);
                    if let Some(builder) = &self.zoom_action {
                        outcome.actions.push(builder(new_zoom));
                    }
                    outcome.zoom_changed = Some(new_zoom);
                    outcome.layout_changed = true;
                    return outcome;
                }

                if c == ',' {
                    let mut outcome = self.make_outcome(true, None);
                    if let Some(builder) = &self.preferences_action {
                        outcome.actions.push(builder());
                    }
                    outcome.toggle_preferences = true;
                    outcome.layout_changed = true;
                    return outcome;
                }
            }
        }

        // 6. Autonomous Docking Interaction (Splitters, Tabs, Floating Windows)
        let workspace_rect = self.compute_workspace_rect(screen_w, screen_h);
        let clamp_bounds = FloatingWindowClampBounds::new(screen_w, screen_h)
            .with_min_y(self.menubar_height)
            .with_bottom_inset(self.statusbar_height);

        let dock_config = DockInteractionConfig {
            clamp_bounds,
            ..Default::default()
        };

        let tab_viewer = self.panel_registry.tab_viewer();
        let dock_res =
            DockArea::handle_event(dock_state, &tab_viewer, event, workspace_rect, &dock_config);

        if let Some(cursor) = dock_res.cursor_override {
            window.set_widget_cursor(cursor);
        }

        if dock_res.consumed {
            let mut outcome = self.make_outcome(true, dock_res.cursor_override);
            outcome.layout_changed = dock_res.layout_changed;
            outcome.actions.extend(self.action_bus.drain());
            return outcome;
        }

        let mut outcome = self.make_outcome(false, dock_res.cursor_override);
        outcome.layout_changed = dock_res.layout_changed;
        outcome.actions.extend(self.action_bus.drain());
        outcome
    }

    /// Dispatches an event through both content and shell layers in industry-standard sequence.
    ///
    /// Evaluates content first. If content produces actions, requests cursor change, or consumes the event,
    /// those actions are aggregated and shell dragging/dock interaction is bypassed.
    pub fn handle_event_with_content<W, F>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        content_dispatcher: F,
    ) -> WorkbenchOutcome<Action>
    where
        W: WindowInterface,
        F: FnOnce(&UiEvent) -> (bool, Option<iris_core::node::WidgetCursor>, Vec<Action>),
    {
        let (content_consumed, content_cursor, content_actions) = content_dispatcher(event);
        if let Some(cursor) = content_cursor {
            window.set_widget_cursor(cursor);
        }

        let mut outcome = self.handle_shell_event(window, event, dock_state, content_consumed);
        if !content_actions.is_empty() {
            outcome.actions.extend(content_actions);
            outcome.layout_changed = true;
        }
        if content_consumed {
            outcome.consumed = true;
        }
        if matches!(
            event,
            UiEvent::MouseUp { .. }
                | UiEvent::KeyboardInput {
                    is_pressed: true,
                    ..
                }
                | UiEvent::Ime(_)
                | UiEvent::FileDropped { .. }
        ) {
            outcome.layout_changed = true;
        }
        if outcome.cursor.is_none() {
            outcome.cursor = content_cursor;
        }
        outcome
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous workbench interaction pipeline
    /// with an explicitly provided external docking state.
    pub fn handle_event_with_dock<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
    ) -> WorkbenchOutcome<Action> {
        self.handle_event_full(window, event, dock_state, None)
    }

    /// Dispatches a platform [`UiEvent`] through the autonomous workbench interaction pipeline
    /// coordinating frameless window management, titlebar dragging, modal dialogs, docking splitters,
    /// and declarative [`UiTree`] event routing in a single unified retained pass.
    pub fn handle_event_full<W: WindowInterface>(
        &mut self,
        window: &mut W,
        event: &UiEvent,
        dock_state: &mut DockState<TabId>,
        tree: Option<&mut UiTree>,
    ) -> WorkbenchOutcome<Action> {
        let (content_consumed, content_cursor, needs_redraw) = if let Some(t) = tree {
            let dispatch = t.dispatch_event(event);
            (dispatch.consumed, dispatch.cursor, dispatch.needs_redraw)
        } else {
            (false, None, false)
        };

        if let Some(cursor) = content_cursor {
            window.set_widget_cursor(cursor);
        }

        let mut outcome = self.handle_shell_event(window, event, dock_state, content_consumed);
        if content_consumed {
            outcome.consumed = true;
        }
        if needs_redraw {
            outcome.layout_changed = true;
        }
        if outcome.cursor.is_none() {
            outcome.cursor = content_cursor;
        }
        outcome
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
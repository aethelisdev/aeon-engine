// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type definitions, actions, and event response structures for the Iris UI editor bridge.

pub use super::assets::{AssetsPanelState, AssetsState};
pub use super::console::{ConsolePanelState, ConsoleState};
pub use super::hierarchy::{HierarchyPanelState, HierarchyState};
pub use super::inspector::{
    InspectorColorDragMode, InspectorNumberDragState, InspectorNumberInputSession,
    InspectorPanelState, InspectorState,
};
pub use super::material::{MaterialPanelState, MaterialState};
use super::modals::ModalsOverlayState;
use super::preferences::PreferencesDialogState;
pub use super::stats::{StatsPanelState, StatsState};
pub use super::timeline::{TimelinePanelState, TimelineState};
pub use super::ui_designer::UiDesignerPanelState;
pub use super::viewport_hud::ViewportHudState;
use crate::ui::EngineUiAction;
use crate::ui::panel_layout::PanelId;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSystem};
use irisui::widgets::controls::{FocusState, FocusTarget};
use irisui::widgets::declarative::hash_label;

/// Top menu bar categories for active open dropdown menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveMenu {
    /// File operations (New, Load, Save, Save As, Exit).
    File,
    /// Edit actions (Undo, Redo, Preferences).
    Edit,
    /// View layout and tool panel visibility toggles.
    View,
    /// Tool windows and workspace resets.
    Window,
    /// Documentation, engine information, and shortcuts.
    Help,
}

impl ActiveMenu {
    /// Converts this menu category to its deterministic hashed widget tag.
    ///
    /// Evaluates the tag via [`hash_label`] ensuring  alignment with
    /// [`UiScope::menu_bar_item`].
    #[inline]
    pub fn to_tag(self) -> u64 {
        match self {
            Self::File => hash_label("File"),
            Self::Edit => hash_label("Edit"),
            Self::View => hash_label("View"),
            Self::Window => hash_label("Window"),
            Self::Help => hash_label("Help"),
        }
    }

    /// Resolves an active menu category from its numeric widget tag.
    ///
    /// Accepts both string label hash tags and legacy designated tag space (`10..=14`).
    #[inline]
    pub fn from_tag(tag: u64) -> Option<Self> {
        if tag == hash_label("File") || tag == 10 || tag == 0 {
            Some(Self::File)
        } else if tag == hash_label("Edit") || tag == 11 || tag == 1 {
            Some(Self::Edit)
        } else if tag == hash_label("View") || tag == 12 || tag == 2 {
            Some(Self::View)
        } else if tag == hash_label("Window") || tag == 13 || tag == 3 {
            Some(Self::Window)
        } else if tag == hash_label("Help") || tag == 14 || tag == 4 {
            Some(Self::Help)
        } else {
            None
        }
    }
}

/// Action payload dispatched from clicking a dropdown menu item.
#[derive(Debug, Clone)]
pub enum DropdownAction {
    /// Dispatches an event-bus UI action.
    UiAction(EngineUiAction),
    /// Toggles tool panel visibility.
    TogglePanel(PanelId),
    /// Resets docking layout to default preset.
    ResetLayout,
    /// Opens preferences modal dialog.
    OpenPreferences,
    /// Opens about engine modal dialog.
    OpenAbout,
    /// Minimizes the application window to the taskbar.
    WindowMinimize,
    /// Toggles the application window between maximized and normal restored size.
    WindowMaximize,
    /// Requests the application window to close.
    WindowClose,
}

/// Event handling response payload returned from `IrisEditorOverlay::handle_event`.
#[derive(Debug, Default, Clone)]
pub struct IrisOverlayEventResult {
    /// Whether the event was intercepted and consumed by the Iris UI overlay.
    pub consumed: bool,
    /// UI action payload to enqueue.
    pub ui_action: Option<EngineUiAction>,
    /// Panel toggle request.
    pub toggle_panel: Option<PanelId>,
    /// Reset layout request.
    pub reset_layout: bool,
    /// Open preferences dialog request.
    pub open_preferences: bool,
    /// Open about dialog request.
    pub open_about: bool,
    /// Close about dialog request.
    pub close_about: bool,
    /// Close preferences dialog request.
    pub close_preferences: bool,
    /// Preferences action payload.
    pub preferences_action: Option<super::preferences::PreferencesAction>,
    /// Confirm delete file request.
    pub confirm_delete: bool,
    /// Cancel delete file request.
    pub cancel_delete: bool,
    /// Create new folder request with specified folder name.
    pub create_folder: Option<String>,
    /// Cancel new folder dialog request.
    pub cancel_new_folder: bool,
    /// Apply rename request with specified new name.
    pub apply_rename: Option<String>,
    /// Window minimize request.
    pub window_minimize: bool,
    /// Window maximize/restore toggle request.
    pub window_maximize: bool,
    /// Window close request.
    pub window_close: bool,
    /// Cancel rename dialog request.
    pub cancel_rename: bool,
    /// Whether to clear the cached console log entries in the editor UI state.
    pub clear_console_entries: bool,
    /// Dock tab activation request specifying leaf identifier and target tab index.
    pub activate_dock_tab: Option<(irisui::dock::DockNodeId, usize)>,
}

impl IrisOverlayEventResult {
    /// Constructs an event result initialized with the specified consumption flag.
    #[must_use]
    #[inline]
    pub fn consumed(consumed: bool) -> Self {
        Self {
            consumed,
            ..Default::default()
        }
    }
}

/// Standardized interaction and hit-testing container for Iris UI editor panels.
///
/// Encapsulates the recurring panel-level fields (transient hit-test targets,
/// scroll offset, search filter query, focus state, and dispatched action queue)
/// to eliminate boilerplate bloat across panel definitions.
#[derive(Debug, Clone)]
pub struct PanelInteractionState<TTargets, TAction> {
    /// Cached interaction and hit-testing targets from the latest layout/render pass.
    pub targets: Option<TTargets>,
    /// Encapsulated interactive text input editing state with selection, caret navigation and clipboard.
    pub search_input: irisui::prelude::TextInputState,
    /// Queue of dispatched actions waiting to be consumed by the editor workbench.
    pub actions: Vec<TAction>,
}

impl<TTargets, TAction> Default for PanelInteractionState<TTargets, TAction> {
    fn default() -> Self {
        Self {
            targets: None,
            search_input: irisui::prelude::TextInputState::default(),
            actions: Vec::new(),
        }
    }
}

impl<TTargets, TAction> PanelInteractionState<TTargets, TAction> {
    /// Consumes and returns all pending dispatched actions.
    pub fn take_actions(&mut self) -> Vec<TAction> {
        std::mem::take(&mut self.actions)
    }

    /// Appends a dispatched action to the pending queue.
    #[inline]
    pub fn push_action(&mut self, action: TAction) {
        self.actions.push(action);
    }

    /// Resets transient per-frame interaction targets.
    pub fn clear_targets(&mut self) {
        self.targets = None;
    }

    /// Returns `true` if there are pending actions queued in this panel interaction state.
    ///
    /// Enables O(1) reactive dirty invalidation without cloning or draining the action queue.
    #[inline]
    #[must_use]
    pub fn has_pending_actions(&self) -> bool {
        !self.actions.is_empty()
    }
}

/// Top menubar and floating dropdown interaction state.
#[derive(Debug, Default, Clone)]
pub struct MenubarOverlayState {
    /// Currently open dropdown menu category.
    pub active_menu: Option<ActiveMenu>,
    /// Widget identifiers of active top menu header buttons for dynamic coordinate resolution.
    pub button_ids: Vec<(ActiveMenu, WidgetId)>,
    /// Dispatched action callbacks indexed by numeric tag from active dropdown items.
    pub actions: Vec<DropdownAction>,
    /// Cached bounding box of the active floating dropdown.
    pub dropdown_rect: Option<Rect>,
}

/// Window chrome, dock frame, cursor tracking, and invalidation state.
#[derive(Debug, Clone)]
pub struct IrisChromeState {
    /// Current mouse cursor coordinates in logical pixels.
    pub cursor_pos: Point,
    /// Last recorded cursor position for detecting interactive hover transitions and invalidation.
    pub last_cursor_pos: Point,
    /// Native dock chrome interaction frame from the last layout reconstruction.
    pub native_dock_frame: Option<super::native_dock::NativeDockFrame>,
    /// Active bounding rectangles of all independent floating windows for solid occlusion and text culling.
    pub floating_window_rects: Vec<Rect>,
    /// Whether Shift modifier key is currently held down.
    pub shift_held: bool,
    /// Whether Alt modifier key is currently held down.
    pub alt_held: bool,
    /// Whether Control modifier key is currently held down.
    pub ctrl_held: bool,
    /// Last recorded screen dimensions.
    pub last_dimensions: (f32, f32),
    /// Last recorded UI Zoom factor.
    pub last_zoom_factor: f32,
    /// Last recorded count of floating windows.
    pub last_floating_count: usize,
    /// Last recorded presence of the 3D viewport rendered texture for reactive viewport binding.
    pub last_has_viewport_texture: bool,
    /// Last recorded presence of an active asset drag payload to trigger immediate overlay rebuild upon completion or cancellation.
    pub last_has_drag_payload: bool,
    /// Explicit flag requesting full layout reconstruction on invalidation.
    pub needs_layout_rebuild: bool,
    /// Explicit flag requesting in-place scroll offset synchronization without tree rebuilding.
    pub needs_scroll_sync: bool,
    /// Explicit flag requesting full GPU instanced quad buffer upload bypassing boundary slicing cache.
    ///
    /// When `true`, instructs [`irisui_wgpu::BoundaryLayerCache`] to upload all compiled draw quads
    /// to GPU VRAM (e.g. on shell reconstruction or non-boundary global visual changes).
    /// When `false`, enables high-performance partial boundary uploads targeting only dirty panels.
    pub needs_full_quad_upload: bool,
    /// Currently open dock tab overflow dropdown menu, storing the parent leaf node ID and anchor button rectangle.
    pub active_dock_overflow: Option<(irisui::dock::DockNodeId, Rect)>,
    /// Currently hovered 64-bit semantic tag resolved from hit testing in the active frame.
    pub hovered_tag: Option<u64>,
    /// Last hovered 64-bit semantic tag used for reactive hover state invalidation.
    pub last_hovered_tag: Option<u64>,
    /// Currently hovered dock panel identifier resolved from dock leaf hit testing.
    pub hovered_panel: Option<PanelId>,
    /// Last hovered dock panel identifier used to detect panel-level hover transitions and tag redraws.
    pub last_hovered_panel: Option<PanelId>,
    /// Last focused 64-bit semantic tag used for reactive focus state invalidation.
    pub last_focused_tag: Option<u64>,
    /// Last recorded caret blink state to detect in-place caret redraw cycles.
    pub last_blink_caret: bool,
    /// Active pointer capture source locking mouse move and release events to a specific panel during drag operations.
    pub pointer_capture: Option<PanelId>,
    /// Last recorded presence of an active top menubar dropdown.
    pub last_menu_active: bool,
    /// Last recorded presence of inspector floating menus/popups.
    pub last_inspector_popups: bool,
    /// Last recorded presence of hierarchy floating menus/popups.
    pub last_hierarchy_popups: bool,
    /// Last recorded presence of preferences modal dialog.
    pub last_preferences_active: bool,
    /// When set to `true`, disables retained diffing/sleeping cache and forces a full UI tree reconstruction
    /// every frame (pure immediate / always-rebuild mode).
    ///
    /// Useful for smooth animation playback and responsive UI iteration prior to modular sub-tree adoption.
    pub always_rebuild: bool,
    /// Explicit flag requesting text section re-collection and GPU glyph font shaping.
    ///
    /// When `false`, the text renderer bypasses Glyphon `prepare` and directly replays
    /// already-prepared GPU font buffers in the render pass for 1000+ FPS retained throughput.
    pub needs_text_prepare: bool,
    /// Currently active workspace theme preset (Obsidian Deep or Cold Slate).
    pub theme_preset: super::theme::ThemePreset,
    /// Last recorded workspace theme preset to trigger full shell rebuild on theme changes.
    pub last_theme_preset: super::theme::ThemePreset,
    /// Last recorded structural dock revision counter for O(1) zero-allocation tab/layout change detection.
    pub last_dock_revision: u64,
    /// Last instant when the performance stats telemetry text was refreshed in the UI tree.
    pub last_stats_refresh: Option<std::time::Instant>,
    /// Last recorded selected entity to trigger targeted panel redraws on selection transitions.
    pub last_selected_entity: Option<hecs::Entity>,
}

impl Default for IrisChromeState {
    fn default() -> Self {
        Self {
            cursor_pos: Point::default(),
            last_cursor_pos: Point::new(-1000.0, -1000.0),
            native_dock_frame: None,
            floating_window_rects: Vec::new(),
            shift_held: false,
            alt_held: false,
            ctrl_held: false,
            last_dimensions: (0.0, 0.0),
            last_zoom_factor: 1.0,
            last_floating_count: 0,
            last_has_viewport_texture: false,
            last_has_drag_payload: false,
            needs_layout_rebuild: false,
            needs_scroll_sync: false,
            needs_full_quad_upload: true,
            active_dock_overflow: None,
            hovered_tag: None,
            last_hovered_tag: None,
            hovered_panel: None,
            last_hovered_panel: None,
            last_focused_tag: None,
            last_blink_caret: false,
            pointer_capture: None,
            last_menu_active: false,
            last_inspector_popups: false,
            last_hierarchy_popups: false,
            last_preferences_active: false,
            always_rebuild: false,
            needs_text_prepare: true,
            theme_preset: super::theme::ThemePreset::default(),
            last_theme_preset: super::theme::ThemePreset::default(),
            last_dock_revision: 0,
            last_stats_refresh: None,
            last_selected_entity: None,
        }
    }
}

impl IrisChromeState {
    /// Evaluates whether the window chrome, dock frame, or root shell requires reconstruction.
    ///
    /// Rebuilding is required when dimensions, display scaling, floating windows, active modals,
    /// or explicit layout rebuild flags change, or when [`Self::always_rebuild`] is enabled.
    #[must_use]
    pub fn is_dirty(
        &self,
        dimensions: (f32, f32),
        zoom_factor: f32,
        floating_count: usize,
        has_drag_payload: bool,
    ) -> bool {
        self.always_rebuild
            || self.needs_layout_rebuild
            || self.last_theme_preset != self.theme_preset
            || self.last_dimensions != dimensions
            || (self.last_zoom_factor - zoom_factor).abs() > 1e-4
            || self.last_floating_count != floating_count
            || self.active_dock_overflow.is_some()
            || self.last_has_drag_payload != has_drag_payload
            || has_drag_payload
    }
}

/// Central coordinator governing Iris UI editor overlays, chrome, modals, and panel subsystems.
pub struct IrisEditorOverlay {
    /// Generational UI tree storing active overlay widget nodes.
    pub tree: UiTree,
    /// Taffy-powered flexbox layout computation engine.
    pub layout_engine: LayoutEngine,
    /// GPU SDF quad and geometry renderer.
    pub renderer: IrisRenderer,
    /// Typography layout and shaping engine.
    pub text_system: TextSystem,
    /// GPU text atlas and glyphon text renderer.
    pub text_renderer: Option<TextRenderer>,
    /// Active frame drawing command stream.
    pub command_list: DrawCommandList,
    /// Selective redraw and change notification engine.
    pub notifier: UiNotifier<PanelId>,
    /// Creation instant used for smooth sub-second continuous UI animations.
    pub start_time: std::time::Instant,
    /// Target surface texture format.
    pub target_format: wgpu::TextureFormat,
    /// Texture resources and bind group for the editor tools icon atlas (`editor_tools.png`).
    pub tools_texture: Option<(wgpu::Texture, wgpu::TextureView, wgpu::BindGroup)>,
    /// Last measured screen width.
    pub screen_width: f32,
    /// Last measured screen height.
    pub screen_height: f32,
    /// Whether the editor overlays are visible.
    pub is_visible: bool,

    // --- Modular Subsystem States ---
    /// Window chrome, dock frame, cursor tracking, and invalidation state.
    pub chrome: IrisChromeState,
    /// Top menu bar and dropdown interaction state.
    pub menubar: MenubarOverlayState,
    /// Modal dialogs and asset loading splash overlays.
    pub modals: ModalsOverlayState,
    /// Preferences modal dialog interactive state.
    pub preferences: PreferencesDialogState,
    /// 3D Viewport HUD controls and dropdown state.
    pub viewport_hud: ViewportHudState,
    /// Performance Stats & Telemetry profiler panel state.
    pub stats: StatsPanelState,
    /// Scene Hierarchy tree panel state.
    pub hierarchy: HierarchyPanelState,
    /// Developer Console and log filter panel state.
    pub console: ConsolePanelState,
    /// Content / Asset Browser panel state.
    pub assets: AssetsPanelState,
    /// Animation Timeline Studio panel state.
    pub timeline: TimelinePanelState,
    /// Material & Surface Studio panel state.
    pub material: MaterialPanelState,
    /// 2D Visual UI Designer panel state.
    pub ui_designer: UiDesignerPanelState,
    /// Scene Inspector component editor panel state.
    pub inspector: InspectorPanelState,
    /// Central focus manager tracking keyboard input focus across widgets and panels.
    pub focus_manager: FocusManager,
    /// Autonomous focus state machine providing SSOT for panel inputs and outside-click clearing.
    pub focus_state: FocusState<FocusTarget>,
    /// Centralized queue of pending engine UI actions emitted by panels and overlays.
    pub pending_actions: Vec<EngineUiAction>,
}

impl IrisEditorOverlay {
    /// Returns `true` if there are pending UI actions queued in the central action queue.
    #[inline]
    #[must_use]
    pub fn has_pending_actions(&self) -> bool {
        !self.pending_actions.is_empty()
            || self.inspector.has_pending_actions()
            || !self.assets.actions.is_empty()
    }

    /// Returns the current logical cursor position.
    #[inline]
    pub fn cursor_pos(&self) -> Point {
        self.chrome.cursor_pos
    }

    /// Returns the active pointer capture panel, if any.
    ///
    /// When dragging starts (e.g. timeline scrubbing, asset dragging), pointer motion and release
    /// events are routed exclusively to the captured panel even when the cursor leaves its bounds.
    #[inline]
    #[must_use]
    pub fn pointer_capture(&self) -> Option<crate::ui::panel_layout::PanelId> {
        self.chrome.pointer_capture
    }

    /// Locks subsequent pointer motion and release events to the specified panel.
    #[inline]
    pub fn set_pointer_capture(&mut self, panel_id: crate::ui::panel_layout::PanelId) {
        self.chrome.pointer_capture = Some(panel_id);
    }

    /// Releases any active pointer capture, restoring standard hit-test routing.
    #[inline]
    pub fn release_pointer_capture(&mut self) {
        self.chrome.pointer_capture = None;
    }

    /// Returns `true` if the specified point is occluded by any active top-level overlay layer.
    ///
    /// Evaluates bounding geometry against the elevated layers of the primary retained [`UiTree`].
    #[must_use]
    #[inline]
    pub fn is_point_over_overlay(&self, point: Point) -> bool {
        if let Some(hit_id) = self.tree.hit_test_layered(point) {
            self.tree.effective_layer(hit_id) >= UiLayer::Modal
        } else {
            false
        }
    }

    /// Returns true if the specified 64-bit semantic tag currently holds keyboard input focus.
    #[must_use]
    #[inline]
    pub fn is_tag_focused(&self, tag: u64) -> bool {
        self.focus_state.is_tag_focused(tag)
    }

    /// Assigns keyboard input focus to the specified 64-bit semantic tag and synchronizes the core focus manager.
    #[inline]
    pub fn set_focus_tag(&mut self, tag: u64) {
        if self.focus_state.set_focus_tag(tag) {
            self.focus_state.sync_to_core(&mut self.focus_manager);
        }
    }

    /// Clears any active keyboard input focus from both the autonomous focus state and the core focus manager.
    ///
    /// Returns `true` if an active focus target was cleared.
    #[inline]
    pub fn clear_focus(&mut self) -> bool {
        let changed = self.focus_state.clear_focus();
        if changed {
            self.focus_state.sync_to_core(&mut self.focus_manager);
        }
        changed
    }

    /// Checks if any search box, text field, or modal rename input currently has keyboard focus.
    pub fn is_any_text_input_focused(&self) -> bool {
        self.tree.is_text_input_focused()
            || self.inspector.active_number_input.is_some()
            || self.inspector.active_text_input.is_some()
            || self.inspector.rename_input.is_some()
            || self.inspector.hex_buffer.is_some()
            || self.modals.is_new_folder_active
            || self.modals.is_rename_active
    }

    /// Configures whether the editor UI overlay reconstructs the tree on every frame (immediate mode)
    /// or uses dirty invalidation caching (retained mode).
    ///
    /// When set to `true`, the layout tree and command buffer are rebuilt each frame, ensuring that
    /// live animations, timelines, and inputs update smoothly without requiring mouse movement.
    #[inline]
    pub fn set_always_rebuild(&mut self, enabled: bool) {
        self.chrome.always_rebuild = enabled;
    }
}

pub use super::params::*;
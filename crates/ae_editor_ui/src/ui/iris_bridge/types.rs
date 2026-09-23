// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type definitions, actions, and event response structures for the Iris UI editor bridge.

use super::assets::AssetsPanelState;
use super::console::ConsolePanelState;
use super::hierarchy::HierarchyPanelState;
use super::inspector::InspectorPanelState;
pub use super::inspector::{
    InspectorColorDragMode, InspectorNumberDragState, InspectorNumberInputSession,
};
use super::material::MaterialPanelState;
use super::modals::ModalsOverlayState;
use super::preferences::PreferencesDialogState;
use super::stats::StatsPanelState;
use super::timeline::TimelinePanelState;
use super::ui_designer::UiDesignerPanelState;
use super::viewport_hud::ViewportHudState;
use crate::ui::EngineUiAction;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use ae_core::modules::EngineModule;
use ae_editor::editor_state::EditorConfig;
use ae_editor::snapping::SnapSettings;
use ae_renderer::graphics_settings::GraphicsSettings;
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSystem};
use std::collections::HashSet;
use std::path::Path;

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
    /// Converts this menu category to its numeric widget tag.
    #[inline]
    pub const fn to_tag(self) -> u64 {
        match self {
            Self::File => 0,
            Self::Edit => 1,
            Self::View => 2,
            Self::Window => 3,
            Self::Help => 4,
        }
    }

    /// Resolves an active menu category from its numeric widget tag.
    #[inline]
    pub const fn from_tag(tag: u64) -> Option<Self> {
        match tag {
            0 => Some(Self::File),
            1 => Some(Self::Edit),
            2 => Some(Self::View),
            3 => Some(Self::Window),
            4 => Some(Self::Help),
            _ => None,
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
    /// Cancel rename dialog request.
    pub cancel_rename: bool,
    /// Whether to clear the cached console log entries in the editor UI state.
    pub clear_console_entries: bool,
    /// Dock tab activation request specifying leaf identifier and target tab index.
    pub activate_dock_tab: Option<(irisui::dock::DockNodeId, usize)>,
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
    /// Content area vertical scroll offset.
    pub scroll_y: f32,
    /// Whether the panel's search input field is currently focused for text editing.
    pub is_search_focused: bool,
    /// Active text query typed in the search filter input.
    pub search_query: String,
    /// Queue of dispatched actions waiting to be consumed by the editor workbench.
    pub actions: Vec<TAction>,
}

impl<TTargets, TAction> Default for PanelInteractionState<TTargets, TAction> {
    fn default() -> Self {
        Self {
            targets: None,
            scroll_y: 0.0,
            is_search_focused: false,
            search_query: String::new(),
            actions: Vec::new(),
        }
    }
}

impl<TTargets, TAction> PanelInteractionState<TTargets, TAction> {
    /// Consumes and returns all pending dispatched actions.
    pub fn take_actions(&mut self) -> Vec<TAction> {
        std::mem::take(&mut self.actions)
    }

    /// Resets transient per-frame interaction targets.
    pub fn clear_targets(&mut self) {
        self.targets = None;
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
    /// Currently open dock tab overflow dropdown menu, storing the parent leaf node ID and anchor button rectangle.
    pub active_dock_overflow: Option<(irisui::dock::DockNodeId, Rect)>,
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
            active_dock_overflow: None,
        }
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
    pub notifier: UiNotifier,
    /// Central registry of dockable tool and workspace panels.
    pub panels: PanelRegistry,
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
}

impl IrisEditorOverlay {
    /// Returns the current logical cursor position.
    #[inline]
    pub fn cursor_pos(&self) -> Point {
        self.chrome.cursor_pos
    }

    /// Checks if any search box, text field, or modal rename input currently has keyboard focus.
    pub fn is_any_text_input_focused(&self) -> bool {
        self.hierarchy.is_search_focused
            || self.console.is_search_focused
            || self.assets.is_search_focused
            || self.viewport_hud.is_search_focused
            || self.inspector.active_number_input.is_some()
            || self.inspector.active_text_input.is_some()
            || self.inspector.rename_buffer.is_some()
            || self.inspector.hex_buffer.is_some()
            || self.modals.is_new_folder_active
            || self.modals.is_rename_active
    }
}

/// Global editor context, dimension, and layout parameters for overlay updates.
pub struct EditorContextParams<'a> {
    /// Screen dimensions (width, height) in physical pixels.
    pub dimensions: (f32, f32),
    /// Current display/UI zoom factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
    /// Whether the editor is currently in Edit mode.
    pub is_editing: bool,
    /// Whether the editor and project are running in 2D dimension mode.
    pub is_2d_mode: bool,
    /// Active panel layout state reference.
    pub layout_state: &'a PanelLayoutState,
    /// Whether undo is available.
    pub can_undo: bool,
    /// Whether redo is available.
    pub can_redo: bool,
    /// Whether live hot-reload editor updates are active.
    pub enable_live_updates: bool,
    /// Optional status notification message spans with text color.
    pub status_spans: Option<&'a [(String, Color)]>,
}

/// 3D Viewport canvas bounds, camera, and navigation state.
pub struct ViewportParams<'a> {
    /// Whether the resolved 3D viewport render target texture is present.
    pub has_viewport_texture: bool,
    /// Screen rectangle bounding the 3D viewport canvas.
    pub viewport_rect: Rect,
    /// Reference to the active 3D camera.
    pub camera: &'a ae_renderer::camera::Camera,
    /// Whether wireframe rendering is currently enabled.
    pub wireframe_enabled: bool,
    /// Whether the viewport coordinate grid is enabled.
    pub grid_enabled: bool,
    /// Currently active gizmo manipulation mode (Translate, Rotate, Scale).
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    /// Currently active gizmo coordinate space (World, Local).
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
}

/// Active ECS scene, selection handle, and entity population telemetry.
pub struct SceneParams<'a> {
    /// Active ECS world reference for entity queries.
    pub world: &'a hecs::World,
    /// Currently selected entity in the scene, if any.
    pub selected_entity: Option<hecs::Entity>,
    /// Count of active entities in the ECS world.
    pub active_entities_count: usize,
}

/// Modal dialog display flags and file system operation targets.
pub struct DialogParams<'a> {
    /// Whether the About Aeon Engine modal dialogue is currently visible.
    pub show_about: bool,
    /// Whether the Preferences modal dialogue is currently visible.
    pub show_preferences: bool,
    /// Optional target path pending delete confirmation.
    pub delete_target: Option<&'a Path>,
    /// Optional new folder parent path.
    pub new_folder_parent: Option<&'a Path>,
    /// Optional rename target path and is_folder flag.
    pub rename_target: Option<(&'a Path, bool)>,
    /// Whether background assets are currently being loaded.
    pub is_loading_assets: bool,
}

/// Engine configuration, snapping, and preferences settings references.
pub struct OverlayPreferencesParams<'a> {
    /// Reference to graphics settings for Preferences rendering.
    pub graphics_settings: &'a GraphicsSettings,
    /// Reference to snapping settings for Preferences rendering.
    pub snapping_settings: &'a SnapSettings,
    /// Reference to editor configuration for Preferences rendering.
    pub editor_config: &'a EditorConfig,
    /// Set of enabled engine core modules for Preferences rendering.
    pub enabled_modules: &'a HashSet<EngineModule>,
}

/// Engine performance telemetry, frame pacing, and hardware statistics.
pub struct TelemetryParams<'a> {
    /// Real-time engine frames per second (FPS) rate.
    pub fps: f32,
    /// Historical frame pacing ring buffer.
    pub frame_pacing: &'a ae_core::telemetry::FrameRingBuffer<240>,
    /// Calculated frametime variance, 1% low, and 0.1% low stats.
    pub frame_pacing_stats: &'a ae_core::telemetry::FramePacingStats,
    /// CPU thread synchronization timings breakdown.
    pub cpu_timings: &'a ae_core::telemetry::CpuSyncTimings,
    /// GPU render pass profiling breakdown.
    pub gpu_pass_timings: &'a ae_core::telemetry::GpuPassTimings,
    /// GPU draw calls, pipeline binds, and primitive counts.
    pub draw_call_stats: &'a ae_core::telemetry::DrawCallBreakdown,
    /// Video memory (VRAM) budget allocation breakdown.
    pub vram_stats: &'a ae_core::telemetry::VramStats,
    /// Total rendered triangles count.
    pub render_triangles: usize,
    /// Total rendered vertices count.
    pub render_vertices: usize,
    /// Name of active GPU hardware adapter.
    pub gpu_adapter_name: &'a str,
    /// Active rendering backend identifier (e.g. Vulkan, DX12, Metal).
    pub gpu_backend: &'a str,
}

/// Bounding rectangles computed by dock system for active overlay panels.
#[derive(Debug, Default, Clone, Copy)]
pub struct OverlayPanelRects {
    /// Bounds of active Stats panel, if visible.
    pub stats: Option<Rect>,
    /// Bounds of active Hierarchy panel, if visible.
    pub hierarchy: Option<Rect>,
    /// Bounds of active Inspector panel, if visible.
    pub inspector: Option<Rect>,
    /// Bounds of active Developer Console panel, if visible.
    pub console: Option<Rect>,
    /// Bounds of active Content Browser panel, if visible.
    pub assets: Option<Rect>,
    /// Bounds of active Timeline panel, if visible.
    pub timeline: Option<Rect>,
    /// Bounds of active Material panel, if visible.
    pub material: Option<Rect>,
    /// Bounds of active 2D Visual UI Designer panel, if visible.
    pub ui_designer: Option<Rect>,
}

/// Subsystem data repositories and live editor caches consumed by panel builders.
pub struct OverlayPanelData<'a> {
    /// Reference to persistent UI Designer state for canvas and toolbar rendering.
    pub ui_designer_state: &'a ae_uidesign::UiDesignerState,
    /// Reference to persistent asset browser state for Content Browser rendering.
    pub asset_browser: &'a crate::assets::AssetBrowserState,
    /// Slice of active in-memory log entries for Developer Console rendering.
    pub console_entries: &'a [crate::ui::types::ConsoleEntry],
    /// GPU texture asset repository for material panel inspection.
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    /// GPU 3D model asset repository for material panel inspection.
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Euler angle cache for rotation editing: `[yaw, pitch, roll]` in degrees.
    pub inspector_euler: &'a [f32; 3],
    /// Hex color string cache for object appearance editing (e.g. `"#6699cc"`).
    pub inspector_color_hex: &'a str,
    /// Saved swatches palette: list of RGBA float arrays `[r, g, b, a]`.
    pub saved_swatches: &'a [[f32; 4]],
}

/// Parameters required for reconstructing and resolving all Iris UI editor overlays.
///
/// Composed of domain-specific parameter sub-structures to eliminate tight coupling.
pub struct OverlayUpdateParams<'a> {
    /// Global editor context and window properties.
    pub context: EditorContextParams<'a>,
    /// 3D Viewport canvas bounds, camera, and navigation state.
    pub viewport: ViewportParams<'a>,
    /// Scene graph, active entity, and ECS world access.
    pub scene: SceneParams<'a>,
    /// Modal dialog flags and file operations.
    pub dialogs: DialogParams<'a>,
    /// Engine configuration and preferences data.
    pub preferences: OverlayPreferencesParams<'a>,
    /// Performance statistics, frame pacing, and hardware telemetry.
    pub telemetry: TelemetryParams<'a>,
    /// Layout rectangles computed by the dock system for active panels.
    pub panel_rects: OverlayPanelRects,
    /// Subsystem data repositories and caches consumed by panel builders.
    pub panel_data: OverlayPanelData<'a>,
}
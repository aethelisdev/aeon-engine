// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Type definitions, actions, and event response structures for the Iris UI editor bridge.

use super::about::AboutDialogTargets;
use super::hierarchy::{AddSubmenuId, HierarchyAction, HierarchyPanelTargets, HierarchyRow};
use super::modals::*;
use super::preferences::{PreferencesDropdownId, PreferencesSliderId, PreferencesTargets};
use super::stats::{StatsPanelAction, StatsPanelNodes, StatsPanelTargets};
use super::viewport_hud::{ViewportHudAction, ViewportHudDropdownId, ViewportHudTargets};
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
}

/// Central state manager governing Iris UI editor overlays, menu bar, modal dialogs, and status bar rendering.
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
    /// Current mouse cursor coordinates.
    pub cursor_pos: Point,
    /// Currently open dropdown menu category.
    pub active_menu: Option<ActiveMenu>,
    /// Interactive dropdown item hit-testing targets.
    pub dropdown_items: Vec<(Rect, DropdownAction)>,
    /// Cached bounding box of the active floating dropdown.
    pub dropdown_rect: Option<Rect>,
    /// Cached bounding box and close button hit targets of the active About dialog.
    pub about_targets: Option<AboutDialogTargets>,
    /// Cached bounding box and button targets of the active Delete Confirmation modal.
    pub delete_targets: Option<DeleteModalTargets>,
    /// Cached bounding box and input targets of the active New Folder modal.
    pub new_folder_targets: Option<NewFolderModalTargets>,
    /// Cached bounding box and input targets of the active Rename modal.
    pub rename_targets: Option<RenameModalTargets>,
    /// Cached bounding box targets of the active Asset Loading splash screen.
    pub loading_targets: Option<LoadingOverlayTargets>,
    /// Cached bounding box and interactive widget targets of the active Preferences dialog.
    pub preferences_targets: Option<PreferencesTargets>,
    /// Cached interaction targets for 3D Viewport HUD.
    pub viewport_hud_targets: Option<ViewportHudTargets>,
    /// Currently open dropdown menu in Viewport HUD.
    pub viewport_hud_dropdown: Option<ViewportHudDropdownId>,
    /// Dispatched action queue for Viewport HUD interactions.
    pub viewport_hud_actions: Vec<ViewportHudAction>,
    /// Cached interaction targets for Performance Stats & Telemetry panel.
    pub stats_targets: Option<StatsPanelTargets>,
    /// Persistent node handles for the Stats & Profiler panel in retained mode.
    pub stats_nodes: Option<StatsPanelNodes>,
    /// Last bounding rectangle allocated for the Stats & Profiler panel.
    pub last_stats_rect: Option<Rect>,
    /// Cached interaction targets for Scene Hierarchy panel.
    pub hierarchy_targets: Option<HierarchyPanelTargets>,
    /// Persistent pre-allocated row cache for Scene Hierarchy to eliminate per-frame allocations.
    pub hierarchy_rows_cache: Vec<HierarchyRow>,
    /// Content area vertical scroll offset for Scene Hierarchy panel.
    pub hierarchy_scroll_y: f32,
    /// Active search filter query for Scene Hierarchy panel.
    pub hierarchy_search_query: String,
    /// Whether the `➕` Add Menu is open in Scene Hierarchy.
    pub hierarchy_is_add_menu_open: bool,
    /// Currently open cascading submenu in Scene Hierarchy Add Menu.
    pub hierarchy_active_submenu: Option<AddSubmenuId>,
    /// Currently open right-click context menu in Scene Hierarchy.
    pub hierarchy_active_context_menu: Option<(hecs::Entity, Point)>,
    /// Whether search input box is focused in Scene Hierarchy.
    pub hierarchy_is_search_focused: bool,
    /// Dispatched action queue for Scene Hierarchy panel interactions.
    pub hierarchy_actions: Vec<HierarchyAction>,
    /// Cached interaction targets for Scene Inspector panel.
    pub inspector_targets: Option<super::inspector::InspectorPanelTargets>,
    /// Content area vertical scroll offset for Scene Inspector panel.
    pub inspector_scroll_y: f32,
    /// Whether `➕ Add Component` menu is open in Inspector.
    pub inspector_is_add_menu_open: bool,
    /// Currently open category submenu in Add Component menu.
    pub inspector_active_submenu: Option<super::inspector::ComponentCategory>,
    /// Currently open dropdown in Inspector.
    pub inspector_active_dropdown: Option<super::inspector::InspectorDropdownId>,
    /// Currently active number input editing state in Inspector: `(id, buffer)`.
    pub inspector_active_number_input: Option<(super::inspector::InspectorNumberInputId, String)>,
    /// Active continuous horizontal mouse drag state for Inspector numeric fields.
    pub inspector_drag_number: Option<InspectorNumberDragState>,
    /// Live entity rename text buffer if currently focused.
    pub inspector_rename_buffer: Option<String>,
    /// Live HEX color text input editing buffer if currently focused.
    pub inspector_hex_buffer: Option<String>,
    /// Live HSV color cache: `[hue (0..360), saturation (0..1), value (0..1)]`.
    pub inspector_hsv: [f32; 3],
    /// Active mouse dragging mode on the 2D HSV color picker.
    pub inspector_color_drag_mode: Option<InspectorColorDragMode>,
    /// Whether the floating Color Picker popup is currently open.
    pub inspector_is_color_picker_open: bool,
    /// Dispatched action queue for Inspector panel interactions.
    pub inspector_actions: Vec<super::inspector::InspectorAction>,
    /// Last recorded screen dimensions.
    pub last_dimensions: (f32, f32),
    /// Last recorded UI Zoom factor.
    pub last_zoom_factor: f32,
    /// Explicit flag requesting full layout reconstruction on invalidation.
    pub needs_layout_rebuild: bool,
    /// Content area vertical scroll offset for Stats & Telemetry panel.
    pub stats_scroll_y: f32,
    /// Dispatched action queue for Stats & Telemetry panel interactions.
    pub stats_actions: Vec<StatsPanelAction>,
    /// Custom floating position coordinates for the Preferences panel.
    pub preferences_pos: Option<Point>,
    /// Active drag offset from window top-left when dragging the title bar.
    pub preferences_drag_offset: Option<Point>,
    /// Currently selected tab index in the Preferences dialog (0..=9).
    pub preferences_tab: u8,
    /// Content area vertical scroll offset for Preferences dialog.
    pub preferences_scroll_y: f32,
    /// Currently open dropdown ComboBox in the Preferences dialog.
    pub preferences_dropdown: Option<PreferencesDropdownId>,
    /// Currently active slider drag descriptor: `(slider_id, track_rect, min_val, max_val)`.
    pub active_slider_drag: Option<(PreferencesSliderId, Rect, f32, f32)>,
    /// Dispatched action queue for Preferences dialog interactions.
    pub preferences_actions: Vec<super::preferences::PreferencesAction>,
    /// Live typing input buffer for the new folder modal.
    pub new_folder_buffer: String,
    /// Live typing input buffer for the rename modal.
    pub rename_buffer: String,
    /// Set of currently collapsed card/section identifiers in the Preferences dialog.
    pub collapsed_sections: HashSet<&'static str>,
    /// Currently active inline number input editing state in Preferences: `(slider_id, typed_buffer)`.
    pub active_number_input: Option<(PreferencesSliderId, String)>,
    /// Last measured screen width.
    pub screen_width: f32,
    /// Last measured screen height.
    pub screen_height: f32,
    /// Whether the editor overlays are visible.
    pub is_visible: bool,
    /// Target surface texture format.
    pub target_format: wgpu::TextureFormat,
    /// Creation instant used for smooth sub-second continuous UI animations.
    pub start_time: std::time::Instant,
    /// Active search filter text query in Viewport Add Object popup.
    pub viewport_search_query: String,
    /// Whether search input box is focused in Viewport Add Object popup.
    pub viewport_is_search_focused: bool,
}

impl IrisEditorOverlay {
    /// Determines the appropriate mouse cursor icon based on current hover targets.
    pub fn requested_cursor_icon(&self) -> winit::window::CursorIcon {
        let p = self.cursor_pos;
        if self.inspector_drag_number.is_some() {
            return winit::window::CursorIcon::EwResize;
        }
        if let Some(mode) = self.inspector_color_drag_mode {
            return match mode {
                InspectorColorDragMode::SaturationValue => winit::window::CursorIcon::Crosshair,
                InspectorColorDragMode::Hue => winit::window::CursorIcon::NsResize,
            };
        }
        if let Some(ref targets) = self.inspector_targets {
            if targets
                .color_picker_sv_box_rect
                .is_some_and(|r| r.contains_point(p))
            {
                return winit::window::CursorIcon::Crosshair;
            }
            if targets
                .color_picker_hue_bar_rect
                .is_some_and(|r| r.contains_point(p))
            {
                return winit::window::CursorIcon::NsResize;
            }
            if targets
                .number_inputs
                .iter()
                .any(|(_, r, _, _, _)| r.contains_point(p))
            {
                return winit::window::CursorIcon::EwResize;
            }
            if targets
                .dropdowns
                .iter()
                .any(|(_, r, _)| r.contains_point(p))
                || targets
                    .component_delete_btns
                    .iter()
                    .any(|(_, r)| r.contains_point(p))
                || targets.add_component_btn_rect.contains_point(p)
                || targets.save_prefab_btn_rect.contains_point(p)
                || targets
                    .color_swatch_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets.hex_input_rect.is_some_and(|r| r.contains_point(p))
                || targets
                    .add_palette_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets
                    .clear_palette_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets.preset_btn_rect.is_some_and(|r| r.contains_point(p))
                || targets
                    .checkboxes
                    .iter()
                    .any(|(_, r, _)| r.contains_point(p))
                || targets
                    .palette_swatches
                    .iter()
                    .any(|(_, r, _)| r.contains_point(p))
                || targets
                    .color_picker_close_btn_rect
                    .is_some_and(|r| r.contains_point(p))
            {
                return winit::window::CursorIcon::Pointer;
            }
            if let Some(popup_r) = targets.active_dropdown_popup_rect
                && popup_r.contains_point(p)
            {
                return winit::window::CursorIcon::Pointer;
            }
            if let Some(picker_r) = targets.color_picker_popup_rect
                && picker_r.contains_point(p)
            {
                return winit::window::CursorIcon::Pointer;
            }
        }

        // 3. Hierarchy interactive items
        if let Some(ref targets) = self.hierarchy_targets
            && (targets.add_btn_rect.contains_point(p)
                || targets.delete_btn_rect.is_some_and(|r| r.contains_point(p))
                || targets
                    .search_clear_btn_rect
                    .is_some_and(|r| r.contains_point(p))
                || targets
                    .entity_rows
                    .iter()
                    .any(|(_, row_r, _, _)| row_r.contains_point(p)))
        {
            return winit::window::CursorIcon::Pointer;
        }

        // 4. Viewport HUD interactive items
        if let Some(ref hud) = self.viewport_hud_targets
            && (hud.buttons.iter().any(|(_, r)| r.contains_point(p))
                || hud
                    .dropdown_triggers
                    .iter()
                    .any(|(_, r)| r.contains_point(p))
                || hud.billboard_icons.iter().any(|(_, r)| r.contains_point(p)))
        {
            return winit::window::CursorIcon::Pointer;
        }

        winit::window::CursorIcon::Default
    }
}

/// Parameters required for reconstructing and resolving all Iris UI editor overlays.
pub struct OverlayUpdateParams<'a> {
    /// Screen dimensions (width, height) in physical pixels.
    pub dimensions: (f32, f32),
    /// Whether the editor is currently in Edit mode.
    pub is_editing: bool,
    /// Active panel layout state reference.
    pub layout_state: &'a PanelLayoutState,
    /// Whether undo is available.
    pub can_undo: bool,
    /// Whether redo is available.
    pub can_redo: bool,
    /// Whether the About Aeon Engine modal dialogue is currently visible.
    pub show_about: bool,
    /// Whether the Preferences modal dialogue is currently visible.
    pub show_preferences: bool,
    /// Reference to graphics settings for Preferences rendering.
    pub graphics_settings: &'a GraphicsSettings,
    /// Reference to snapping settings for Preferences rendering.
    pub snapping_settings: &'a SnapSettings,
    /// Reference to editor configuration for Preferences rendering.
    pub editor_config: &'a EditorConfig,
    /// Whether live hot-reload editor updates are active.
    pub enable_live_updates: bool,
    /// Set of enabled engine core modules for Preferences rendering.
    pub enabled_modules: &'a HashSet<EngineModule>,
    /// Current display/UI zoom factor (e.g. 1.0 = 100%).
    pub zoom_factor: f32,
    /// Optional target path pending delete confirmation.
    pub delete_target: Option<&'a Path>,
    /// Optional new folder parent path.
    pub new_folder_parent: Option<&'a Path>,
    /// Optional rename target path and is_folder flag.
    pub rename_target: Option<(&'a Path, bool)>,
    /// Whether background assets are currently being loaded.
    pub is_loading_assets: bool,
    /// Optional status notification message spans with text color.
    pub status_spans: Option<&'a [(String, Color)]>,
    /// Screen rectangle bounding the 3D viewport canvas.
    pub viewport_rect: Rect,
    /// Reference to the active 3D camera.
    pub camera: &'a ae_renderer::camera::Camera,
    /// Whether wireframe rendering is currently enabled.
    pub wireframe_enabled: bool,
    /// Currently active gizmo manipulation mode (Translate, Rotate, Scale).
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    /// Currently active gizmo coordinate space (World, Local).
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
    /// Currently selected entity in the editor, if any.
    pub selected_entity: Option<hecs::Entity>,
    /// Active ECS world reference for billboard entity query.
    pub world: &'a hecs::World,
    /// Bounding rectangle allocated for the Performance Stats panel, if active.
    pub stats_panel_rect: Option<Rect>,
    /// Bounding rectangle allocated for the Scene Hierarchy panel, if active.
    pub hierarchy_panel_rect: Option<Rect>,
    /// Bounding rectangle allocated for the Scene Inspector panel, if active.
    pub inspector_panel_rect: Option<Rect>,
    /// Euler angle cache for rotation editing: `[yaw, pitch, roll]` in degrees.
    pub inspector_euler: &'a [f32; 3],
    /// Hex color string cache for object appearance editing (e.g. `"#6699cc"`).
    pub inspector_color_hex: &'a str,
    /// Saved swatches palette: list of RGBA float arrays `[r, g, b, a]`.
    pub saved_swatches: &'a [[f32; 4]],
    /// Whether the viewport coordinate grid is enabled.
    pub grid_enabled: bool,
    /// Smoothed FPS rate.
    pub fps: f32,
    /// Historical frame pacing ring buffer.
    pub frame_pacing: &'a ae_core::telemetry::FrameRingBuffer<240>,
    /// Calculated frametime variance, 1% low, and 0.1% low stats.
    pub frame_pacing_stats: &'a ae_core::telemetry::FramePacingStats,
    /// CPU thread synchronization timings breakdown.
    pub cpu_timings: &'a ae_core::telemetry::CpuSyncTimings,
    /// GPU render pass execution durations.
    pub gpu_pass_timings: &'a ae_core::telemetry::GpuPassTimings,
    /// Granular draw call metrics and batch counts.
    pub draw_call_stats: &'a ae_core::telemetry::DrawCallBreakdown,
    /// Categorized VRAM memory consumption.
    pub vram_stats: &'a ae_core::telemetry::VramStats,
    /// Total rendered triangles in current frame.
    pub render_triangles: u64,
    /// Total rendered vertices in current frame.
    pub render_vertices: u64,
    /// Hardware GPU adapter device name.
    pub gpu_adapter_name: &'a str,
    /// Active graphics API backend (e.g. Vulkan, Metal, DX12).
    pub gpu_backend: &'a str,
    /// Count of active entities in the ECS world.
    pub active_entities_count: usize,
}

/// Active horizontal mouse drag state for interactive Inspector numeric inputs.
#[derive(Debug, Clone, Copy)]
pub struct InspectorNumberDragState {
    /// Target numeric input identifier.
    pub id: super::inspector::InspectorNumberInputId,
    /// Starting X coordinate of the cursor when mouse was pressed.
    pub start_x: f32,
    /// Starting value of the numeric field before dragging began.
    pub start_val: f32,
    /// Lower clamp bound.
    pub min_val: f32,
    /// Upper clamp bound.
    pub max_val: f32,
    /// Value delta per dragged pixel.
    pub sensitivity: f32,
    /// Whether mouse has dragged beyond threshold.
    pub has_dragged: bool,
}

/// Dragging mode on the 2D HSV color picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectorColorDragMode {
    /// Dragging on the 2D Saturation-Value box.
    SaturationValue,
    /// Dragging on the vertical Rainbow Hue spectrum bar.
    Hue,
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::types::{ConsoleEntry, EngineUiAction};
use irisui::prelude::{Color, Point, Rect};
use winit::window::Window;

/// Action payload sent from async native file dialog threads to the main UI thread.
pub enum SceneDialogAction {
    SaveTo(std::path::PathBuf),
    LoadFrom(std::path::PathBuf),
}

/// Identifies the border or corner being resized on a floating window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatingResizeEdge {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// Active drag state on a detached floating window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatingDragState {
    Title { offset: Point },
    Resize(FloatingResizeEdge),
}

/// The main UI management system for the Aeon Engine.
/// Owns the Iris UI overlay pipeline, docking layout, and all persistent
/// editor state (selection, inspector, preferences, console).
pub struct EngineUi {
    pub selected_entity: Option<hecs::Entity>,
    pub status_message: Option<(Vec<(String, Color)>, std::time::Instant)>,
    pub inspector_euler: [f32; 3],
    pub last_selected_entity: Option<hecs::Entity>,
    pub wireframe_enabled: bool,
    /// Controls whether the editor grid is visible in the viewport.
    pub grid_enabled: bool,
    pub is_loading_assets: bool,
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    /// Controls whether gizmo axes are aligned to world or entity-local orientation.
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
    pub inspector_color_hex: String,
    pub saved_swatches: Vec<[f32; 4]>,
    pub show_preferences: bool,
    pub show_about: bool,
    pub preferences_tab: u8,
    /// Pending preferences actions to execute on the engine state.
    pub pending_preferences_actions: Vec<crate::ui::iris_bridge::PreferencesAction>,
    pub should_save_scene: bool,
    pub should_load_scene: bool,
    pub active_scene_path: String,
    pub pending_save_path: Option<std::path::PathBuf>,
    pub pending_load_path: Option<std::path::PathBuf>,
    pub scene_dialog_receivers: Vec<std::sync::mpsc::Receiver<SceneDialogAction>>,
    pub should_exit: bool,
    /// Modular docking panel and tab layout state.
    pub layout_state: PanelLayoutState,
    pub hierarchy_search_query: String,
    /// Snapshot of log entries (updated at most once per frame, only when count changed).
    pub(crate) console_entries: Vec<ConsoleEntry>,
    /// The log count we last snapshotted from – used for change detection.
    pub(crate) console_last_count: u64,
    /// All UI rects from the last frame (panels + floating windows)
    pub(crate) ui_rects: Vec<Rect>,
    /// Profiler snapshot (ms) – updated by engine before render
    pub profiler_ecs_ms: f32,
    pub profiler_physics_ms: f32,
    pub profiler_render_ms: f32,
    /// VSync/swapchain present blocking time (ms) – separated from render for accurate profiling.
    pub profiler_present_ms: f32,
    pub profiler_ui_ms: f32,
    pub profiler_frame_ms: f32,
    /// Detailed CPU thread execution and synchronization stage timings
    pub cpu_timings: ae_core::telemetry::CpuSyncTimings,
    /// Detailed GPU pass execution timings (Shadow, Main Opaque, Post-Process, UI)
    pub gpu_pass_timings: ae_core::telemetry::GpuPassTimings,
    /// Live 240-frame ring buffer for real-time frame pacing analysis
    pub frame_pacing: ae_core::telemetry::FrameRingBuffer<240>,
    /// Precalculated statistical pacing metrics (1% Low, 0.1% Low, Jitter Variance, Spikes)
    pub frame_pacing_stats: ae_core::telemetry::FramePacingStats,
    /// Detailed draw calls and culling breakdown
    pub draw_call_stats: ae_core::telemetry::DrawCallBreakdown,
    /// Granular Video RAM (VRAM) consumption metrics
    pub vram_stats: ae_core::telemetry::VramStats,
    /// Memory usage snapshot (MB) – updated by engine before render
    pub memory_models_mb: f32,
    pub memory_textures_mb: f32,
    /// Live rendering geometry metrics
    pub render_draw_calls: u32,
    pub render_triangles: u64,
    pub render_vertices: u64,
    /// Physical GPU adapter information
    pub gpu_adapter_name: String,
    pub gpu_backend: String,
    /// Smoothed FPS value for readable, flicker-free presentation in the Stats panel.
    pub smoothed_fps: f32,
    /// The actual displayed FPS value in the UI panel, updated periodically (every 100ms) for high readability.
    pub displayed_fps: f32,
    /// The instant of the last FPS counter refresh.
    pub last_fps_update: std::time::Instant,
    /// Last registered viewport texture width.
    pub viewport_rect_width: f32,
    /// Last registered viewport texture height.
    pub viewport_rect_height: f32,
    /// Last recorded 3D viewport screen rectangle in logical coordinates.
    pub last_viewport_rect: Rect,
    /// Active UI Zoom / Scaling factor (e.g. 1.0 = 100%, 0.8 = 80%, 1.25 = 125%).
    pub ui_zoom_factor: f32,
    /// Persistent Content / Asset Browser state (directory path, search query, active category filter).
    pub asset_browser: crate::ui::panels::assets::AssetBrowserState,
    /// Persistent 2D UI Designer canvas state (aspect ratio, zoom, pan, grid snap).
    pub ui_designer_state: ae_uidesign::UiDesignerState,
    /// Pending UI actions queued from window event dispatchers.
    pub pending_actions: Vec<EngineUiAction>,
    /// Iris UI retained-mode overlay manager (SDF shaders, menubar, docking).
    pub iris_overlay: IrisEditorOverlay,
    /// Active floating window drag mode: `(window_id, drag_state)`.
    pub active_floating_drag: Option<(u64, FloatingDragState)>,
    /// Pending tab drag awaiting distance threshold to activate native dock drag.
    pub pending_tab_drag: Option<PendingTabDrag>,
}

/// Tracks a pending tab drag before the cursor moves past the activation distance threshold.
#[derive(Debug, Clone, Copy)]
pub struct PendingTabDrag {
    /// Leaf owning the tab.
    pub leaf: irisui::dock::DockNodeId,
    /// Tab index within the owning leaf.
    pub tab_index: usize,
    /// Associated panel identifier.
    pub panel: PanelId,
    /// Screen-space position where mouse was pressed.
    pub press_pos: Point,
    /// Bounding rectangle of the source leaf.
    pub leaf_rect: Rect,
    /// Optional floating window identifier if dragged from a floating window.
    pub floating_window_id: Option<u64>,
}

impl EngineUi {
    /// Initializes the native Iris UI overlay manager and editor subsystems.
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        _window: &Window,
    ) -> Self {
        let iris_overlay = IrisEditorOverlay::new(device, output_color_format);

        Self {
            iris_overlay,
            pending_actions: Vec::new(),
            selected_entity: None,
            status_message: None,
            inspector_euler: [0.0; 3],
            last_selected_entity: None,
            wireframe_enabled: false,
            grid_enabled: true,
            is_loading_assets: false,
            gizmo_mode: ae_editor::gizmo::GizmoMode::Select,
            gizmo_space: ae_editor::gizmo::GizmoSpace::Local,
            inspector_color_hex: String::new(),
            saved_swatches: vec![
                [1.0, 1.0, 1.0, 1.0],
                [0.5, 0.5, 0.5, 1.0],
                [0.1, 0.1, 0.1, 1.0],
                [1.0, 0.2, 0.2, 1.0],
                [0.2, 1.0, 0.2, 1.0],
                [0.2, 0.2, 1.0, 1.0],
                [1.0, 1.0, 0.1, 1.0],
            ],
            show_preferences: false,
            show_about: false,
            preferences_tab: 1,
            pending_preferences_actions: Vec::new(),
            should_save_scene: false,
            should_load_scene: false,
            active_scene_path: "scene.aee".to_string(),
            pending_save_path: None,
            pending_load_path: None,
            scene_dialog_receivers: Vec::new(),
            should_exit: false,
            layout_state: PanelLayoutState::new_default(),
            hierarchy_search_query: String::new(),
            console_entries: Vec::new(),
            console_last_count: 0,
            ui_rects: Vec::new(),
            profiler_ecs_ms: 0.0,
            profiler_physics_ms: 0.0,
            profiler_render_ms: 0.0,
            profiler_present_ms: 0.0,
            profiler_ui_ms: 0.0,
            profiler_frame_ms: 0.0,
            cpu_timings: ae_core::telemetry::CpuSyncTimings::default(),
            gpu_pass_timings: ae_core::telemetry::GpuPassTimings::default(),
            frame_pacing: ae_core::telemetry::FrameRingBuffer::new(),
            frame_pacing_stats: ae_core::telemetry::FramePacingStats::default(),
            draw_call_stats: ae_core::telemetry::DrawCallBreakdown::default(),
            vram_stats: ae_core::telemetry::VramStats::default(),
            memory_models_mb: 0.0,
            memory_textures_mb: 0.0,
            render_draw_calls: 0,
            render_triangles: 0,
            render_vertices: 0,
            gpu_adapter_name: String::new(),
            gpu_backend: String::new(),
            smoothed_fps: 60.0,
            displayed_fps: 60.0,
            last_fps_update: std::time::Instant::now(),
            viewport_rect_width: 0.0,
            viewport_rect_height: 0.0,
            last_viewport_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            ui_zoom_factor: 1.0,
            asset_browser: crate::ui::panels::assets::AssetBrowserState::new(),
            ui_designer_state: ae_uidesign::UiDesignerState::default(),
            active_floating_drag: None,
            pending_tab_drag: None,
        }
    }

    /// Active UI scaling factor.
    pub fn scale_factor(&self) -> f32 {
        if self.ui_zoom_factor.is_finite() && self.ui_zoom_factor > 0.1 {
            self.ui_zoom_factor.clamp(0.6, 2.0)
        } else {
            1.0
        }
    }

    /// Steps the active UI scale up or down across the predefined scale presets.
    pub fn step_ui_scale(&mut self, increase: bool) -> f32 {
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
        self.ui_zoom_factor = target;
        target
    }

    /// Resets the UI scale back to default 100% (1.0).
    pub fn reset_ui_scale(&mut self) -> f32 {
        self.ui_zoom_factor = 1.0;
        1.0
    }

    /// Sets a temporary status message displayed at the bottom status bar.
    pub fn set_status_message(&mut self, text: impl Into<String>, color: Color) {
        self.status_message = Some((vec![(text.into(), color)], std::time::Instant::now()));
    }

    /// Returns whether any modal dialog, search input, or inspector field currently captures keyboard events.
    pub fn wants_keyboard_input(&self) -> bool {
        self.iris_overlay.hierarchy_is_search_focused
            || self.iris_overlay.console_is_search_focused
            || self.iris_overlay.assets_is_search_focused
            || self.iris_overlay.viewport_is_search_focused
            || self.iris_overlay.inspector_active_number_input.is_some()
            || self.iris_overlay.inspector_active_text_input.is_some()
            || self.iris_overlay.inspector_rename_buffer.is_some()
            || self.iris_overlay.inspector_hex_buffer.is_some()
            || self.iris_overlay.new_folder_targets.is_some()
            || self.iris_overlay.rename_targets.is_some()
    }

    /// Polls asynchronous native file dialog receivers and applies their actions.
    pub fn poll_dialog_receivers(&mut self) {
        let mut completed_indices = Vec::new();
        let mut actions = Vec::new();

        for (idx, rx) in self.scene_dialog_receivers.iter().enumerate() {
            if let Ok(action) = rx.try_recv() {
                actions.push(action);
                completed_indices.push(idx);
            }
        }

        for &idx in completed_indices.iter().rev() {
            self.scene_dialog_receivers.swap_remove(idx);
        }

        for action in actions {
            match action {
                SceneDialogAction::SaveTo(path) => {
                    self.active_scene_path = path.to_string_lossy().to_string();
                    self.pending_save_path = Some(path);
                    self.should_save_scene = true;
                }
                SceneDialogAction::LoadFrom(path) => {
                    self.active_scene_path = path.to_string_lossy().to_string();
                    self.pending_load_path = Some(path);
                    self.should_load_scene = true;
                    self.is_loading_assets = true;
                }
            }
        }
    }

    /// Called once per frame before rendering to snapshot the global log buffer.
    pub fn sync_console(&mut self) {
        if !self.layout_state.is_panel_visible(PanelId::Console) {
            return;
        }

        let current_total = ae_editor::editor_logger::LOGGER
            .log_count
            .load(std::sync::atomic::Ordering::Relaxed);

        if current_total != self.console_last_count {
            if let Ok(lock) = ae_editor::editor_logger::LOGGER.logs.try_lock() {
                self.console_entries = lock
                    .iter()
                    .map(|e| ConsoleEntry {
                        level: e.level,
                        target: e.target.clone(),
                        msg: e.msg.clone(),
                        timestamp: e.timestamp.clone(),
                    })
                    .collect();
                self.console_last_count = current_total;
            }
        } else if current_total == 0 && !self.console_entries.is_empty() {
            self.console_entries.clear();
            self.console_last_count = 0;
        }
    }
}
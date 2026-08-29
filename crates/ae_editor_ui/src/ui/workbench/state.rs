// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::style;
use crate::ui::types::{ConsoleEntry, EngineUiAction};
use egui::Context;
use egui_wgpu::Renderer;
use egui_winit::State;
use winit::window::Window;

/// Action payload sent from async native file dialog threads to the main UI thread.
pub enum SceneDialogAction {
    SaveTo(std::path::PathBuf),
    LoadFrom(std::path::PathBuf),
}

/// The main UI management system for the Aeon Engine.
/// Owns the egui context, winit state adapter, WGPU renderer, panel docking layout,
/// and all persistent editor UI state (selection, inspector, preferences, console).
pub struct EngineUi {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
    pub selected_entity: Option<hecs::Entity>,
    pub status_message: Option<(Vec<(String, egui::Color32)>, std::time::Instant)>,
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
    pub(crate) ui_rects: Vec<egui::Rect>,
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
    /// Pre-built flat snapshot of the scene hierarchy, rebuilt only when entity count changes.
    /// Avoids O(N × k) random hecs lookups every frame; replaced by a single O(N) pass
    /// on change, then zero-cost virtual-scrolled drawing on subsequent frames.
    pub hierarchy_cache: crate::ui::panels::hierarchy::HierarchyCache,
    /// Egui texture ID of the registered WGPU viewport texture.
    pub viewport_texture_id: Option<egui::TextureId>,
    /// Last registered viewport texture width.
    pub viewport_rect_width: f32,
    /// Last registered viewport texture height.
    pub viewport_rect_height: f32,
    /// Last recorded 3D viewport screen rectangle in logical coordinates.
    pub last_viewport_rect: egui::Rect,
    /// Active UI Zoom / Scaling factor (e.g. 1.0 = 100%, 0.8 = 80%, 1.25 = 125%).
    pub ui_zoom_factor: f32,
    /// Persistent Content / Asset Browser state (directory path, search query, active category filter).
    pub asset_browser: crate::ui::panels::assets::AssetBrowserState,
    /// Persistent 2D UI Designer canvas state (aspect ratio, zoom, pan, grid snap).
    pub ui_designer_state: crate::ui::panels::UiDesignerState,
    /// Pending UI actions queued from window event dispatchers.
    pub pending_actions: Vec<EngineUiAction>,
    /// Iris UI retained-mode overlay manager (SDF shaders, menubar, docking).
    pub iris_overlay: IrisEditorOverlay,
}

impl EngineUi {
    /// Initializes the egui context with custom fonts (NotoSans, symbols, math, and emoji), dark theme, and WGPU renderer.
    /// Extends font rendering definitions by loading and binding custom TTF font streams, including
    /// `NotoSans` for standard text, symbol variants for icons/culling glyphs, `NotoSansMath` for formula symbols,
    /// and `NotoEmoji` to provide universal monochrome emoji symbol visibility throughout all engine UI panels.
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let context = Context::default();

        style::load_fonts(&context);
        style::setup_custom_style(&context);

        let state = State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = Renderer::new(
            device,
            output_color_format,
            egui_wgpu::RendererOptions::default(),
        );

        let iris_overlay = IrisEditorOverlay::new(device, output_color_format);

        Self {
            context,
            state,
            renderer,
            iris_overlay,
            pending_actions: Vec::new(),
            selected_entity: None,
            status_message: None,
            inspector_euler: [0.0; 3],
            last_selected_entity: None,
            wireframe_enabled: false,
            grid_enabled: true,
            is_loading_assets: false,
            gizmo_mode: ae_editor::gizmo::GizmoMode::Translate,
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
            hierarchy_cache: crate::ui::panels::hierarchy::HierarchyCache::new(),
            smoothed_fps: 60.0,
            displayed_fps: 60.0,
            last_fps_update: std::time::Instant::now(),
            viewport_texture_id: None,
            viewport_rect_width: 0.0,
            viewport_rect_height: 0.0,
            last_viewport_rect: egui::Rect::ZERO,
            ui_zoom_factor: 1.0,
            asset_browser: crate::ui::panels::assets::AssetBrowserState::new(),
            ui_designer_state: crate::ui::panels::UiDesignerState::default(),
        }
    }

    /// Drains any finished async scene file dialog tasks and sets the pending save/load paths.
    pub fn process_scene_dialogs(&mut self) {
        let mut actions = Vec::new();
        for rx in &self.scene_dialog_receivers {
            while let Ok(action) = rx.try_recv() {
                actions.push(action);
            }
        }
        self.scene_dialog_receivers.retain(|rx| {
            !matches!(
                rx.try_recv(),
                Err(std::sync::mpsc::TryRecvError::Disconnected)
            )
        });

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

    /// Called once per frame, BEFORE Egui rendering.
    /// Snapshots the global log buffer only when new entries exist.
    /// This avoids holding the Mutex during the Egui render pass.
    pub fn sync_console(&mut self) {
        // Fast-path: skip any processing if Console is not currently visible
        if !self.layout_state.is_panel_visible(PanelId::Console) {
            return;
        }

        // Use atomic load for cheap change detection without locking the mutex
        let current_total = ae_editor::editor_logger::LOGGER
            .log_count
            .load(std::sync::atomic::Ordering::Relaxed);

        if current_total != self.console_last_count {
            // Only lock if we actually have new work to do
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
        }
    }
}
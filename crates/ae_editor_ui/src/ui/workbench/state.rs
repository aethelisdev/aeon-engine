// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::iris_bridge::IrisEditorOverlay;
use crate::ui::panel_layout::{PanelId, PanelLayoutState};
use crate::ui::types::{ConsoleEntry, EngineUiAction};
use irisui::prelude::{Color, Rect};
use winit::window::Window;

/// Action payload sent from async native file dialog threads to the main UI thread.
pub enum SceneDialogAction {
    SaveTo(std::path::PathBuf),
    LoadFrom(std::path::PathBuf),
}

/// The main UI management system for the Aeon Engine.
///
/// Owns the Iris UI overlay pipeline, docking layout, and all persistent
/// editor state (selection, inspector, preferences, console).
pub struct EngineUi {
    pub selected_entity: Option<hecs::Entity>,
    pub status_message: Option<(Vec<(String, Color)>, std::time::Instant)>,
    pub inspector_euler: [f32; 3],
    pub last_selected_entity: Option<hecs::Entity>,
    pub wireframe_enabled: bool,
    pub grid_enabled: bool,
    pub is_loading_assets: bool,
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
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
    /// Detailed CPU command buffer recording and pass orchestration timings
    pub pass_recording_timings: ae_core::telemetry::RenderPassRecordingTimings,
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
    /// Real-time frames per second (FPS) calculation updated per-frame directly from engine timing.
    pub fps: f32,
    /// Last registered viewport texture width.
    pub viewport_rect_width: f32,
    /// Last registered viewport texture height.
    pub viewport_rect_height: f32,
    /// Last recorded 3D viewport screen rectangle in logical coordinates.
    pub last_viewport_rect: Rect,
    /// Active UI Zoom / Scaling factor (e.g. 1.0 = 100%, 0.8 = 80%, 1.25 = 125%).
    pub ui_zoom_factor: f32,
    /// Persistent Content / Asset Browser state (directory path, search query, active category filter).
    pub asset_browser: crate::assets::AssetBrowserState,
    /// Persistent 2D UI Designer canvas state (aspect ratio, zoom, pan, grid snap).
    pub ui_designer_state: ae_uidesign::UiDesignerState,
    /// Pending UI actions queued from window event dispatchers.
    pub pending_actions: Vec<EngineUiAction>,
    /// Framework modal dialog host managing modal confirmations, textual prompts, and keyboard focus traps.
    pub dialog_host: irisui::workbench::DialogHost<EngineUiAction>,
    /// Global UI event and action bus managing decoupled FIFO dispatch.
    pub action_bus: irisui::workbench::ActionBus<EngineUiAction>,
    /// Autonomous workbench orchestrator coordinating dock hierarchy, frameless window border resize, and modal dialogs.
    pub workbench: irisui::workbench::Workbench<PanelId, EngineUiAction, ()>,
    /// Iris UI retained-mode overlay manager (SDF shaders, menubar, docking).
    pub iris_overlay: IrisEditorOverlay,
}

impl EngineUi {
    /// Initializes the native Iris UI overlay manager and editor subsystems.
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        _window: &Window,
    ) -> Self {
        let iris_overlay = IrisEditorOverlay::new(device, output_color_format);
        let layout_state = PanelLayoutState::new_default();

        let workbench_desc = irisui::workbench::WorkbenchDescriptor::new(
            "Aeon Engine Studio",
            layout_state.dock_state.clone(),
        )
        .with_menubar(irisui::workbench::MenuBarDescriptor::new())
        .with_statusbar(irisui::workbench::StatusBarDescriptor::new());

        let mut workbench: irisui::workbench::Workbench<PanelId, EngineUiAction, ()> =
            irisui::workbench::Workbench::new(workbench_desc)
                .with_menubar_height(IrisEditorOverlay::MENUBAR_HEIGHT)
                .with_statusbar_height(IrisEditorOverlay::STATUS_BAR_HEIGHT)
                .with_non_ui_tab(PanelId::Viewport)
                .with_file_drop_action(|paths, pt| EngineUiAction::ImportDroppedFiles {
                    paths,
                    screen_pos: Some([pt.x, pt.y]),
                })
                .with_zoom_action(EngineUiAction::SetUiScale)
                .with_preferences_action(|| EngineUiAction::OpenPreferences);

        for panel in PanelId::all() {
            let mut meta = irisui::workbench::PanelMetadata::new(*panel, panel.title())
                .with_icon(panel.icon());
            if let Some(atlas) = panel.atlas_icon() {
                meta = meta.with_atlas_icon(atlas);
            }
            workbench.panel_registry.register_metadata(meta);
        }

        Self {
            iris_overlay,
            workbench,
            dialog_host: irisui::workbench::DialogHost::new(),
            action_bus: irisui::workbench::ActionBus::new(),
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
            active_scene_path: "scene.ae3d".to_string(),
            pending_save_path: None,
            pending_load_path: None,
            scene_dialog_receivers: Vec::new(),
            should_exit: false,
            layout_state,
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
            pass_recording_timings: ae_core::telemetry::RenderPassRecordingTimings::default(),
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
            fps: 60.0,
            viewport_rect_width: 0.0,
            viewport_rect_height: 0.0,
            last_viewport_rect: Rect::new(0.0, 0.0, 0.0, 0.0),
            ui_zoom_factor: 1.0,
            asset_browser: crate::assets::AssetBrowserState::new(),
            ui_designer_state: ae_uidesign::UiDesignerState::default(),
        }
    }

    /// Active UI scaling factor.
    pub fn scale_factor(&self) -> f32 {
        self.workbench.scale_factor()
    }

    /// Steps the active UI scale up or down across predefined scale presets.
    pub fn step_ui_scale(&mut self, increase: bool) -> f32 {
        self.ui_zoom_factor = self.workbench.step_zoom(increase);
        self.ui_zoom_factor
    }

    /// Resets the UI scale back to default 100% (1.0).
    pub fn reset_ui_scale(&mut self) -> f32 {
        self.ui_zoom_factor = self.workbench.reset_zoom();
        self.ui_zoom_factor
    }

    /// Sets a temporary status message displayed at the bottom status bar.
    ///
    /// Triggers an immediate reactive overlay redraw so that the status message
    /// is instantly reflected in the bottom status bar.
    pub fn set_status_message(&mut self, text: impl Into<String>, color: Color) {
        self.status_message = Some((vec![(text.into(), color)], std::time::Instant::now()));
        self.iris_overlay.notifier.tag_all();
    }

    /// Checks status message expiration and clears it if 2.5 seconds have elapsed.
    ///
    /// If expired, clears the message and flags the overlay notifier dirty to
    /// revert the status bar back to "● Ready".
    pub fn expire_status_message(&mut self) {
        if let Some((_, timestamp)) = self.status_message
            && is_status_message_expired(timestamp)
        {
            self.status_message = None;
            self.iris_overlay.notifier.tag_all();
        }
    }

    /// Returns whether any modal dialog, search input, or inspector field currently captures keyboard events.
    pub fn wants_keyboard_input(&self) -> bool {
        self.iris_overlay.is_any_text_input_focused()
    }

    /// Polls asynchronous native file dialog receivers and applies their actions.
    pub fn poll_dialog_receivers(&mut self) {
        let mut completed = Vec::new();
        let mut actions = Vec::new();
        for (idx, rx) in self.scene_dialog_receivers.iter().enumerate() {
            if let Ok(action) = rx.try_recv() {
                actions.push(action);
                completed.push(idx);
            }
        }
        for &idx in completed.iter().rev() {
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

    /// Returns whether the UI currently captures the pointer or if the pointer is positioned over
    /// any active workbench UI element (menubar, statusbar, panels, floating windows, popups).
    ///
    /// Delegates directly to [`irisui::workbench::Workbench::wants_pointer_input`].
    #[must_use]
    #[inline]
    pub fn wants_pointer_input(&self) -> bool {
        self.workbench.wants_pointer_input(
            Some(self.iris_overlay.cursor_pos()),
            &self.layout_state.dock_state,
            Some(&self.iris_overlay.tree),
        )
    }

    /// Evaluates whether the specified screen-space coordinate lands over active workbench UI elements
    /// rather than the interactive 3D viewport canvas.
    ///
    /// Delegates directly to [`irisui::workbench::Workbench::wants_pointer_at`].
    #[must_use]
    #[inline]
    pub fn wants_pointer_at(&self, pos: [f32; 2]) -> bool {
        self.workbench.wants_pointer_at(
            pos,
            &self.layout_state.dock_state,
            Some(&self.iris_overlay.tree),
        )
    }

    /// Compatibility alias forwarding to [`Self::wants_pointer_at`].
    #[must_use]
    #[inline]
    pub fn is_point_over_ui_rects(&self, pos: [f32; 2]) -> bool {
        self.wants_pointer_at(pos)
    }

    /// Dispatches declarative UI interaction events to panel contents and the autonomous
    /// [`irisui::workbench::Workbench`] orchestrator.
    pub fn handle_event<W: iris_winit::frameless::WindowInterface>(
        &mut self,
        mut window: W,
        event: &irisui::prelude::UiEvent,
        _world: Option<&hecs::World>,
    ) -> bool {
        let overlay = &mut self.iris_overlay;
        let outcome = self.workbench.handle_event_with_content(
            &mut window,
            event,
            &mut self.layout_state.dock_state,
            |ev| {
                let res = overlay.handle_event(ev);
                let actions = res.ui_action.into_iter().collect();
                let cursor = if res.consumed {
                    Some(overlay.active_cursor())
                } else {
                    None
                };
                (res.consumed, cursor, actions)
            },
        );

        if !outcome.actions.is_empty() {
            self.pending_actions.extend(outcome.actions);
        }
        if let Some(zoom) = outcome.zoom_changed {
            self.ui_zoom_factor = zoom;
        }
        if outcome.toggle_preferences {
            self.show_preferences = !self.show_preferences;
        }
        if outcome.layout_changed {
            self.iris_overlay.chrome.needs_layout_rebuild = true;
            self.iris_overlay.notifier.tag_all();
        }

        outcome.consumed
    }
}

/// Returns whether a status message has expired based on the standard 2.5s duration.
pub fn is_status_message_expired(timestamp: std::time::Instant) -> bool {
    const STATUS_MESSAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(2500);
    timestamp.elapsed() >= STATUS_MESSAGE_TIMEOUT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_message_expiration_timing() {
        let now = std::time::Instant::now();
        assert!(!is_status_message_expired(now));

        let expired = now - std::time::Duration::from_millis(2600);
        assert!(is_status_message_expired(expired));
    }
}
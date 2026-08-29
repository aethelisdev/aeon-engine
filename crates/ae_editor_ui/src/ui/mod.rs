// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use egui::Context;
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use winit::{event::WindowEvent, window::Window};

pub mod iris_bridge;
pub mod panel_layout;
pub mod panels;

mod dialogs;
mod docking;
pub mod menubar;
mod preferences;
mod status_bar;
mod style;
mod types;
mod viewport_hud;

// Re-exports
pub use iris_bridge::IrisEditorOverlay;
pub use menubar::*;
pub use panel_layout::{PanelId, PanelLayoutState};
pub use panels::hierarchy::{HierarchyCache, HierarchyRow};
pub use types::{ConsoleEntry, EngineUiAction, UiElementType};

/// Action payload sent from async native file dialog threads to the main UI thread.
pub enum SceneDialogAction {
    SaveTo(std::path::PathBuf),
    LoadFrom(std::path::PathBuf),
}

/// Parameters for rendering the entire Editor UI frame.
pub struct EditorUiRenderParams<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a Window,
    pub window_surface_view: &'a wgpu::TextureView,
    pub viewport_texture_view: Option<&'a wgpu::TextureView>,
    pub fps: f32,
    pub world: &'a hecs::World,
    pub mode: &'a ae_core::modules::EngineMode,
    pub undo_stack: &'a [ae_editor::undo_redo::Command],
    pub redo_stack: &'a [ae_editor::undo_redo::Command],
    pub graphics_settings: &'a ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping: &'a ae_editor::snapping::SnapSettings,
    pub editor_state: &'a ae_editor::editor_state::EditorState,
    pub camera: &'a ae_renderer::camera::Camera,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub shaders: &'a ae_renderer::asset::AssetStorage<ae_renderer::asset::ShaderAsset>,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub ui_actions: &'a mut Vec<EngineUiAction>,
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
    console_entries: Vec<ConsoleEntry>,
    /// The log count we last snapshotted from – used for change detection.
    console_last_count: u64,
    /// All UI rects from the last frame (panels + floating windows)
    ui_rects: Vec<egui::Rect>,
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

    /// Forwards winit window events to egui and Iris UI for input processing.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let iris_res = self.iris_overlay.handle_event(event);
        if let Some(act) = iris_res.ui_action {
            self.pending_actions.push(act);
        }
        if let Some(panel) = iris_res.toggle_panel {
            self.layout_state.activate_or_open(panel);
        }
        if iris_res.reset_layout {
            self.layout_state.reset_to_default();
        }
        if iris_res.open_preferences {
            self.show_preferences = true;
        }
        if iris_res.open_about {
            self.show_about = true;
        }
        if iris_res.consumed {
            window.set_cursor(winit::window::CursorIcon::Default);
            return true;
        }

        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    /// Returns true if the point is over any UI panel, floating modal dialog, or outside the 3D viewport.
    pub fn is_point_over_ui_rects(&self, pos: egui::Pos2) -> bool {
        if pos.y <= IrisEditorOverlay::MENUBAR_HEIGHT
            || self
                .iris_overlay
                .is_point_over_overlay(irisui::prelude::Point::new(pos.x, pos.y))
        {
            return true;
        }

        // 1. Outside 3D viewport -> 100% over an editor UI panel (Hierarchy, Inspector, Assets, Menus, etc.)
        if !self.last_viewport_rect.contains(pos) {
            return true;
        }

        // 2. Open popups / context menus ( /  style)
        if egui::Popup::is_any_open(&self.context) {
            return true;
        }

        // 3. Floating dialogs (Preferences, About, Loading overlay, etc.)
        self.ui_rects.iter().any(|rect| rect.contains(pos))
    }

    /// Orchestrates the drawing of all editor panels, toolbar menus, preference views,
    /// hierarchy snapshots, interactive HUD nodes, and overlay dialogs for the frame.
    pub fn render(&mut self, params: EditorUiRenderParams<'_>) -> egui::Rect {
        let device = params.device;
        let queue = params.queue;
        let encoder = params.encoder;
        let window = params.window;
        let window_surface_view = params.window_surface_view;
        let viewport_texture_view = params.viewport_texture_view;
        let fps = params.fps;
        let world = params.world;
        let mode = params.mode;
        let undo_stack = params.undo_stack;
        let redo_stack = params.redo_stack;
        let graphics_settings = params.graphics_settings;
        let snapping = params.snapping;
        let editor_state = params.editor_state;
        let camera = params.camera;
        let models = params.models;
        let textures = params.textures;
        let shaders = params.shaders;
        let enabled_modules = params.enabled_modules;
        let ui_actions = params.ui_actions;
        ui_actions.append(&mut self.pending_actions);

        let alpha = 0.08f32;
        self.smoothed_fps = alpha * fps + (1.0 - alpha) * self.smoothed_fps;

        let now = std::time::Instant::now();
        if now.duration_since(self.last_fps_update).as_secs_f32() >= 0.10 {
            self.displayed_fps = self.smoothed_fps;
            self.last_fps_update = now;
        }

        let raw_input = self.state.take_egui_input(window);

        // Register/Update viewport texture if provided
        if let Some(view) = viewport_texture_view {
            if let Some(id) = self.viewport_texture_id {
                self.renderer.update_egui_texture_from_wgpu_texture(
                    device,
                    view,
                    wgpu::FilterMode::Linear,
                    id,
                );
            } else {
                let id =
                    self.renderer
                        .register_native_texture(device, view, wgpu::FilterMode::Linear);
                self.viewport_texture_id = Some(id);
            }
        } else if let Some(old_id) = self.viewport_texture_id.take() {
            self.renderer.free_texture(&old_id);
        }

        // Handle Ctrl + / - / 0 UI Zoom Shortcuts and apply active UI scaling
        let zoom_delta = self.context.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    0.05f32
                } else if i.key_pressed(egui::Key::Minus) {
                    -0.05f32
                } else if i.key_pressed(egui::Key::Num0) {
                    100.0f32
                } else {
                    0.0f32
                }
            } else {
                0.0f32
            }
        });

        if zoom_delta == 100.0 {
            self.ui_zoom_factor = 1.0;
        } else if zoom_delta != 0.0 {
            self.ui_zoom_factor = (self.ui_zoom_factor + zoom_delta).clamp(0.6, 2.0);
        }

        self.context.set_zoom_factor(self.ui_zoom_factor);

        // Destructure self fields to allow split borrows in the closure
        let show_preferences = &mut self.show_preferences;
        let show_about = &mut self.show_about;
        let _should_save_scene = &mut self.should_save_scene;
        let _should_load_scene = &mut self.should_load_scene;
        let preferences_tab = &mut self.preferences_tab;
        let selected_entity = &mut self.selected_entity;
        let last_selected_entity = &mut self.last_selected_entity;
        let inspector_euler = &mut self.inspector_euler;
        let inspector_color_hex = &mut self.inspector_color_hex;
        let saved_swatches = &mut self.saved_swatches;
        let wireframe_enabled = &mut self.wireframe_enabled;
        let grid_enabled = &mut self.grid_enabled;
        let is_loading_assets = self.is_loading_assets;
        let gizmo_mode = &mut self.gizmo_mode;
        let gizmo_space = &mut self.gizmo_space;
        let status_message = &mut self.status_message;
        let layout_state = &mut self.layout_state;
        let hierarchy_search_query = &mut self.hierarchy_search_query;
        let console_entries = &self.console_entries;
        let render_triangles = self.render_triangles;
        let render_vertices = self.render_vertices;
        let gpu_adapter_name = &self.gpu_adapter_name;
        let gpu_backend = &self.gpu_backend;
        let smoothed_fps = self.displayed_fps;
        let frame_pacing = &self.frame_pacing;
        let frame_pacing_stats = self.frame_pacing_stats;
        let cpu_timings = self.cpu_timings;
        let gpu_pass_timings = self.gpu_pass_timings;
        let draw_call_stats = self.draw_call_stats;
        let vram_stats = self.vram_stats;
        let hierarchy_cache = &mut self.hierarchy_cache;
        let ui_designer_state = &mut self.ui_designer_state;

        let viewport_rect = std::cell::Cell::new(egui::Rect::ZERO);

        let ui_rects_collector = std::cell::RefCell::new(Vec::new());

        let full_output = self.context.run_ui(raw_input, |ui| {
            let ctx = ui.ctx().clone();
            let is_editing = *mode == ae_core::modules::EngineMode::Edit;

            // 1. Top Menu Bar
            Self::draw_menu_bar(
                ui,
                menubar::MenuBarDrawParams {
                    show_preferences,
                    show_about,
                    layout_state,
                    undo_stack,
                    redo_stack,
                    is_editing,
                    ui_actions,
                },
            );

            // 1.5 PREFERENCES WINDOW
            if *show_preferences {
                let mut temp_gs = (*graphics_settings).clone();
                let mut temp_snap = *snapping;
                let mut temp_live_updates = editor_state.enable_live_editor_updates;
                let mut temp_cfg = editor_state.config.clone();

                let pref_resp =
                    Self::draw_preferences_window(preferences::PreferencesWindowParams {
                        show_preferences,
                        preferences_tab,
                        ctx: &ctx,
                        graphics_settings: &mut temp_gs,
                        snapping_settings: &mut temp_snap,
                        enable_live_updates: &mut temp_live_updates,
                        editor_config: &mut temp_cfg,
                        enabled_modules,
                        ui_actions,
                        status_message,
                    });

                if temp_gs != *graphics_settings {
                    ui_actions.push(EngineUiAction::UpdateGraphicsSettings(temp_gs));
                }
                ui_actions.push(EngineUiAction::UpdateSnapSettings(temp_snap));
                ui_actions.push(EngineUiAction::SetLiveEditorUpdates(temp_live_updates));
                ui_actions.push(EngineUiAction::UpdateEditorConfig(temp_cfg));

                if let Some(rect) = pref_resp {
                    ui_rects_collector.borrow_mut().push(rect);
                }
            }

            // 2. Bottom Status Bar
            if let Some(rect) = Self::draw_utility_bar(layout_state, status_message, ui) {
                ui_rects_collector.borrow_mut().push(rect);
            }

            // 3. Central Tree Docking System (egui_dock - Tree-based split tab layout)
            let mut tab_viewer = docking::EditorTabViewer {
                world,
                hierarchy_cache,
                hierarchy_search_query,
                selected_entity,
                last_selected_entity,
                inspector_euler,
                inspector_color_hex,
                saved_swatches,
                is_editing,
                ui_actions,
                editor_state,
                camera,
                asset_browser: &mut self.asset_browser,
                models,
                textures,
                shaders,
                console_entries,
                wireframe_enabled,
                grid_enabled,
                fps: smoothed_fps,
                frame_pacing,
                frame_pacing_stats,
                cpu_timings,
                gpu_pass_timings,
                draw_call_stats,
                vram_stats,
                render_triangles,
                render_vertices,
                gpu_adapter_name,
                gpu_backend,
                viewport_texture_id: self.viewport_texture_id,
                viewport_rect_out: &viewport_rect,
                enabled_modules,
                gizmo_mode,
                gizmo_space,
                ui_designer_state,
            };

            egui::CentralPanel::default()
                .frame(egui::Frame::new().inner_margin(egui::Margin::ZERO))
                .show(ui, |ui| {
                    Self::draw_docking_system(ui, layout_state, &mut tab_viewer);
                });

            // Top-Level Modal Dialogs & Floating Overlays
            let mut collected_rects = ui_rects_collector.borrow_mut();
            dialogs::draw_dialogs(&ctx, is_loading_assets, &mut collected_rects);
            menubar::help::draw_about_dialog(&ctx, show_about, &mut collected_rects);

            // Asset Browser Modals (New Folder, Rename, Delete confirmation)
            if let Some(rect) = crate::ui::panels::assets::file_ops::draw_file_operations_dialogs(
                &ctx,
                &mut self.asset_browser,
            ) {
                collected_rects.push(rect);
            }

            // Quick Asset Inspector Modal (Only in Edit mode)
            if is_editing {
                if let Some(rect) =
                    crate::ui::panels::assets::preview_modal::draw_asset_preview_modal(
                        &ctx,
                        &mut self.asset_browser,
                        models,
                        textures,
                        shaders,
                        ui_actions,
                    )
                {
                    collected_rects.push(rect);
                }
            } else {
                self.asset_browser.preview_modal = None;
            }

            // Clean up status message after duration
            if let Some((_, start_time)) = status_message
                && start_time.elapsed().as_secs() >= 5
            {
                *status_message = None;
            }
        });

        // Store collected rects for hit testing
        self.ui_rects = ui_rects_collector.into_inner();

        self.state
            .handle_platform_output(window, full_output.platform_output);

        if self
            .iris_overlay
            .is_point_over_overlay(self.iris_overlay.cursor_pos)
        {
            window.set_cursor(winit::window::CursorIcon::Default);
        }
        let clipped_primitives = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, image_deltas) in &full_output.textures_delta.set {
            for image_delta in image_deltas {
                self.renderer
                    .update_texture(device, queue, *id, image_delta);
            }
        }
        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: window_surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let mut render_pass = render_pass.forget_lifetime();
            self.renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }
        for id in &full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        // 4. Iris UI Overlay Render Pass (Menubar and active SDF UI overlays)
        let win_size = window.inner_size();
        if win_size.width > 0 && win_size.height > 0 {
            self.iris_overlay.update_menu_bar(
                win_size.width as f32,
                *mode == ae_core::modules::EngineMode::Edit,
                &self.layout_state,
                !undo_stack.is_empty(),
                !redo_stack.is_empty(),
            );
            self.iris_overlay.render(
                device,
                queue,
                encoder,
                window_surface_view,
                (win_size.width, win_size.height),
            );
        }

        // Check if viewport size changed to trigger re-registration on next frame
        let mut new_rect = viewport_rect.get();
        if !new_rect.is_positive()
            || !new_rect.min.x.is_finite()
            || !new_rect.min.y.is_finite()
            || !new_rect.max.x.is_finite()
            || !new_rect.max.y.is_finite()
        {
            new_rect = egui::Rect::ZERO;
        }

        if new_rect.width() != self.viewport_rect_width
            || new_rect.height() != self.viewport_rect_height
        {
            if let Some(old_id) = self.viewport_texture_id.take() {
                self.renderer.free_texture(&old_id);
            }
            self.viewport_rect_width = new_rect.width();
            self.viewport_rect_height = new_rect.height();
        }

        self.last_viewport_rect = new_rect;

        new_rect
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use egui::Context;
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use winit::{event::WindowEvent, window::Window};

mod hierarchy;
mod inspector;
mod menubar;
mod preferences;
mod workspace;

// New modular submodules
mod dialogs;
mod style;
mod types;
mod viewport_hud;

// Re-exports
pub use types::{ConsoleEntry, EngineUiAction};

/// The main UI management system for the Aeon Engine.
/// Owns the egui context, winit state adapter, WGPU renderer, and all
/// persistent editor UI state (selection, inspector, preferences, console).
/// Action payload sent from async native file dialog threads to the main UI thread.
pub enum SceneDialogAction {
    SaveTo(std::path::PathBuf),
    LoadFrom(std::path::PathBuf),
}

pub struct EngineUi {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
    pub selected_entity: Option<hecs::Entity>,
    stress_cube_counter: usize,
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
    pub workspace_tab: usize,
    pub show_workspace: bool,
    /// Snapshot of log entries (updated at most once per frame, only when count changed).
    console_entries: Vec<ConsoleEntry>,
    /// The log count we last snapshotted from – used for change detection.
    console_last_count: u64,
    /// All UI rects from the last frame (panels + floating windows)
    ui_rects: Vec<egui::Rect>,
    /// Profiler snapshot (ms) – updated by engine before render
    pub profiler_ecs_ms: f32,
    pub profiler_render_ms: f32,
    /// VSync/swapchain present blocking time (ms) – separated from render for accurate profiling.
    pub profiler_present_ms: f32,
    pub profiler_ui_ms: f32,
    pub profiler_frame_ms: f32,
    /// Memory usage snapshot (MB) – updated by engine before render
    pub memory_models_mb: f32,
    pub memory_textures_mb: f32,
    /// Smoothed FPS value for readable, flicker-free presentation in the Stats panel.
    pub smoothed_fps: f32,
    /// The actual displayed FPS value in the UI panel, updated periodically (every 100ms) for high readability.
    pub displayed_fps: f32,
    /// The instant of the last FPS counter refresh.
    pub last_fps_update: std::time::Instant,
    /// Pre-built flat snapshot of the scene hierarchy, rebuilt only when entity count changes.
    /// Avoids O(N × k) random hecs lookups every frame; replaced by a single O(N) pass
    /// on change, then zero-cost virtual-scrolled drawing on subsequent frames.
    pub hierarchy_cache: crate::ui::hierarchy::HierarchyCache,
    /// Egui texture ID of the registered WGPU viewport texture.
    pub viewport_texture_id: Option<egui::TextureId>,
    /// Last registered viewport texture width.
    pub viewport_rect_width: f32,
    /// Last registered viewport texture height.
    pub viewport_rect_height: f32,
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
            &window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = Renderer::new(
            device,
            output_color_format,
            egui_wgpu::RendererOptions::default(),
        );

        Self {
            context,
            state,
            renderer,
            selected_entity: None,
            stress_cube_counter: 0,
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
            workspace_tab: 1, // Default to Console
            show_workspace: false,
            console_entries: Vec::new(),
            console_last_count: 0,
            ui_rects: Vec::new(),
            profiler_ecs_ms: 0.0,
            profiler_render_ms: 0.0,
            profiler_present_ms: 0.0,
            profiler_ui_ms: 0.0,
            profiler_frame_ms: 0.0,
            memory_models_mb: 0.0,
            memory_textures_mb: 0.0,
            hierarchy_cache: crate::ui::hierarchy::HierarchyCache::new(),
            smoothed_fps: 60.0,
            displayed_fps: 60.0,
            last_fps_update: std::time::Instant::now(),
            viewport_texture_id: None,
            viewport_rect_width: 0.0,
            viewport_rect_height: 0.0,
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
        // Fast-path: skip any processing if panel is hidden
        if !self.show_workspace || self.workspace_tab != 1 {
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

    /// Forwards winit window events to egui for input processing.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let response = self.state.on_window_event(window, event);
        response.consumed
    }

    /// Returns true if point is over any UI rect from last frame
    pub fn is_point_over_ui_rects(&self, pos: egui::Pos2) -> bool {
        self.ui_rects.iter().any(|rect| rect.contains(pos))
    }

    /// Orchestrates the drawing of all editor panels, toolbar menus, preference views,
    /// hierarchy snapshots, interactive HUD nodes, and overlay dialogs for the frame.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        window_surface_view: &wgpu::TextureView,
        viewport_texture_view: Option<&wgpu::TextureView>,
        fps: f32,
        world: &hecs::World,
        mode: &ae_core::modules::EngineMode,
        undo_stack: &[ae_editor::undo_redo::Command],
        redo_stack: &[ae_editor::undo_redo::Command],
        _view_matrix: cgmath::Matrix4<f32>,
        _proj_matrix: cgmath::Matrix4<f32>,
        graphics_settings: &ae_renderer::graphics_settings::GraphicsSettings,
        snapping: &ae_editor::snapping::SnapSettings,
        editor_state: &ae_editor::editor_state::EditorState,
        camera: &ae_renderer::camera::Camera,
        models: &ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
        textures: &ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
        enabled_modules: &std::collections::HashSet<ae_core::modules::EngineModule>,
        ui_actions: &mut Vec<EngineUiAction>,
    ) -> egui::Rect {
        // Calculate the exponential moving average (EMA) EVERY single frame at full engine throughput.
        // Running EMA per-frame ensures instantaneous responsiveness (converges in milliseconds at 1000+ FPS)
        // while filtering out sub-millisecond high-frequency noise.
        let alpha = 0.08f32;
        self.smoothed_fps = alpha * fps + (1.0 - alpha) * self.smoothed_fps;

        // We also maintain a displayed FPS value that we update only every 100ms to prevent
        // rapid numeric flickering in the UI text, making it highly readable.
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
        } else {
            if let Some(old_id) = self.viewport_texture_id.take() {
                self.renderer.free_texture(&old_id);
            }
        }

        // Destructure self fields to allow split borrows in the closure
        let show_preferences = &mut self.show_preferences;
        let show_about = &mut self.show_about;
        let should_save_scene = &mut self.should_save_scene;
        let should_load_scene = &mut self.should_load_scene;
        let preferences_tab = &mut self.preferences_tab;
        let selected_entity = &mut self.selected_entity;
        let last_selected_entity = &mut self.last_selected_entity;
        let inspector_euler = &mut self.inspector_euler;
        let inspector_color_hex = &mut self.inspector_color_hex;
        let saved_swatches = &mut self.saved_swatches;
        let _stress_cube_counter = &mut self.stress_cube_counter;
        let wireframe_enabled = &mut self.wireframe_enabled;
        let grid_enabled = &mut self.grid_enabled;
        let is_loading_assets = self.is_loading_assets;
        let gizmo_mode = &mut self.gizmo_mode;
        let gizmo_space = &mut self.gizmo_space;
        let status_message = &mut self.status_message;

        let show_workspace = &mut self.show_workspace;
        let workspace_tab = &mut self.workspace_tab;
        let console_entries = &self.console_entries;
        let profiler_ecs_ms = self.profiler_ecs_ms;
        let profiler_render_ms = self.profiler_render_ms;
        let profiler_present_ms = self.profiler_present_ms;
        let profiler_ui_ms = self.profiler_ui_ms;
        let profiler_frame_ms = self.profiler_frame_ms;
        let memory_models_mb = self.memory_models_mb;
        let memory_textures_mb = self.memory_textures_mb;
        let smoothed_fps = self.displayed_fps;
        let hierarchy_cache = &mut self.hierarchy_cache;

        let viewport_rect = std::cell::Cell::new(egui::Rect::EVERYTHING);
        let ui_rects_collector = std::cell::RefCell::new(Vec::new());

        let full_output = self.context.run_ui(raw_input, |ui| {
            let ctx = ui.ctx().clone();
            let is_editing = *mode == ae_core::modules::EngineMode::Edit;

            // 1. Top Menu Bar
            Self::draw_menu_bar(
                show_preferences,
                show_about,
                should_save_scene,
                should_load_scene,
                show_workspace,
                workspace_tab,
                ui,
                world,
                mode,
                undo_stack,
                redo_stack,
                is_editing,
                ui_actions,
            );
            // Collect menu bar rect if visible
            if let Some(menu_rect) = ctx.memory(|mem| mem.area_rect(egui::Id::new("menu_bar"))) {
                ui_rects_collector.borrow_mut().push(menu_rect);
            }

            // 1.5 PREFERENCES WINDOW
            if *show_preferences {
                let mut temp_gs = (*graphics_settings).clone();
                let mut temp_snap = (*snapping).clone();
                let mut temp_live_updates = editor_state.enable_live_editor_updates;
                let mut temp_cfg = editor_state.config.clone();

                let pref_resp = Self::draw_preferences_window(
                    show_preferences,
                    preferences_tab,
                    &ctx,
                    &mut temp_gs,
                    &mut temp_snap,
                    &mut temp_live_updates,
                    &mut temp_cfg,
                    enabled_modules,
                    ui_actions,
                    status_message,
                );

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

            // 2. Bottom Utility Bar (Toggle buttons + Status)
            // Call this first to anchor it to the absolute bottom of the window.
            if let Some(rect) =
                Self::draw_utility_bar(show_workspace, workspace_tab, status_message, ui)
            {
                ui_rects_collector.borrow_mut().push(rect);
            }

            // 2.1 Right Panel (Inspector Panel)
            // Occupies the right side from top bar down to utility bar.
            let mut inspector_snapshot: Option<ae_editor::undo_redo::EntitySnapshot> = None;
            let inspector_resp = Self::draw_inspector_panel(
                selected_entity,
                last_selected_entity,
                inspector_euler,
                inspector_color_hex,
                saved_swatches,
                &mut inspector_snapshot,
                ui,
                world,
                undo_stack,
                redo_stack,
                is_editing,
                ui_actions,
                editor_state,
                camera,
                models,
            );
            if let Some(rect) = inspector_resp {
                ui_rects_collector.borrow_mut().push(rect);
            }

            // 2.2 Bottom Workspace Panel (Console / Asset Browser)
            // Docks at bottom of remaining central space (strictly to the left of inspector_panel).
            if let Some(rect) = Self::draw_workspace_panel(
                show_workspace,
                workspace_tab,
                console_entries,
                ui,
                models,
                textures,
                ui_actions,
            ) {
                ui_rects_collector.borrow_mut().push(rect);
            }

            // 2.7 Central Viewport Area (Render-to-Texture Display)
            // Captured as a CentralPanel that consumes the remaining empty space in the middle.
            let central_rect = egui::CentralPanel::default()
                .frame(egui::Frame::new().inner_margin(egui::Margin::ZERO))
                .show(ui, |ui| {
                    let rect = ui.available_rect_before_wrap();
                    if let Some(texture_id) = self.viewport_texture_id {
                        ui.image(egui::load::SizedTexture {
                            id: texture_id,
                            size: rect.size(),
                        });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Rendering viewport...");
                        });
                    }
                    rect
                })
                .inner;
            viewport_rect.set(central_rect);

            // 3. Viewport Toolbar, Scene Gizmo & Billboard Icons (Visible in Edit Mode & when Render module is ENABLED)
            let is_render_active =
                enabled_modules.contains(&ae_core::modules::EngineModule::Render);
            if is_editing && is_render_active {
                let available_rect = central_rect;
                viewport_hud::draw_viewport_toolbar(
                    &ctx,
                    available_rect,
                    wireframe_enabled,
                    gizmo_mode,
                    gizmo_space,
                    camera,
                    ui_actions,
                );

                // 3.6 Camera Position HUD & 3D Scene Navigation Gizmo
                viewport_hud::draw_camera_hud(&ctx, available_rect, camera);
                viewport_hud::draw_scene_navigation_gizmo(&ctx, available_rect, camera, ui_actions);

                // 3.7 3D Viewport Billboard Icons (Light 💡, Audio 🔊, Ear 👂, Camera 🎥)
                viewport_hud::draw_billboard_icons(
                    &ctx,
                    available_rect,
                    world,
                    camera,
                    *selected_entity,
                    ui_actions,
                );
            }

            // 4. Decoupled Hierarchy and Stats Panels (Groundwork for Modular Placements)
            let hierarchy_resp = Self::draw_hierarchy_panel(
                selected_entity,
                &ctx,
                world,
                is_editing,
                ui_actions,
                hierarchy_cache,
            );
            if let Some(rect) = hierarchy_resp {
                ui_rects_collector.borrow_mut().push(rect);
            }

            let stats_resp = Self::draw_stats_panel(
                wireframe_enabled,
                grid_enabled,
                &ctx,
                smoothed_fps,
                profiler_ecs_ms,
                profiler_render_ms,
                profiler_present_ms,
                profiler_ui_ms,
                profiler_frame_ms,
                memory_models_mb,
                memory_textures_mb,
            );
            if let Some(rect) = stats_resp {
                ui_rects_collector.borrow_mut().push(rect);
            }

            // Loading Overlay and Dialogs
            let mut collected_rects = ui_rects_collector.borrow_mut();
            dialogs::draw_dialogs(&ctx, is_loading_assets, &mut *collected_rects);
            menubar::help::draw_about_dialog(&ctx, show_about, &mut *collected_rects);

            // Clean up status message if it should no longer be handled by caller
            // (The utility bar handles duration now)
            if let Some((_, start_time)) = status_message {
                if start_time.elapsed().as_secs() >= 5 {
                    *status_message = None;
                }
            }
        });

        // Store collected rects for hit testing
        self.ui_rects = ui_rects_collector.into_inner();

        self.state
            .handle_platform_output(window, full_output.platform_output);
        let clipped_primitives = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [window.inner_size().width, window.inner_size().height],
            pixels_per_point: window.scale_factor() as f32,
        };

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
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

        // Check if viewport size changed to trigger re-registration on next frame
        let new_rect = viewport_rect.get();
        if new_rect.width() != self.viewport_rect_width
            || new_rect.height() != self.viewport_rect_height
        {
            if let Some(old_id) = self.viewport_texture_id.take() {
                self.renderer.free_texture(&old_id);
            }
            self.viewport_rect_width = new_rect.width();
            self.viewport_rect_height = new_rect.height();
        }

        viewport_rect.get()
    }
}
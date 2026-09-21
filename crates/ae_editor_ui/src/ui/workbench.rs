// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Editor Workbench Subsystem
//!
//! Orchestrates the editor lifecycle, window events, persistent UI state, and Iris UI WGPU rendering.
//!

use crate::ui::panel_layout::{PanelId, PanelLayoutState};
pub use crate::ui::types::{
    ConsoleEntry, EditorUiRenderParams, EngineUiAction, SceneDialogAction,
    map_widget_cursor_to_winit,
};
use irisui::prelude::*;
use irisui::text::{TextRenderer, TextSystem};
use irisui::wgpu_backend::{IrisRenderer, compile_tree_draw_commands_into};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use winit::window::Window;

/// The main UI management system for the Aeon Engine editor.
///
/// Owns the Iris UI retained-mode tree, GPU renderer, and editor state.
pub struct EngineUi {
    // Iris UI Subsystems
    /// Retained-mode widget tree.
    pub tree: UiTree,
    /// Flexbox layout engine.
    pub layout_engine: LayoutEngine,
    /// GPU SDF quad renderer.
    pub renderer: IrisRenderer,
    /// Hardware text system.
    pub text_system: TextSystem,
    /// Hardware text GPU renderer.
    pub text_renderer: Option<TextRenderer>,
    /// Docking panel and tab layout state.
    pub layout_state: PanelLayoutState,
    /// Iris UI draw command buffer.
    pub command_list: DrawCommandList,

    // Editor Core State
    /// Selected entity in the active scene.
    pub selected_entity: Option<hecs::Entity>,
    /// Active temporary status message and its timestamp.
    pub status_message: Option<(Vec<(String, Color)>, Instant)>,
    /// Inspector Euler angle editing cache.
    pub inspector_euler: [f32; 3],
    /// Previously selected entity.
    pub last_selected_entity: Option<hecs::Entity>,
    /// Controls whether wireframe rendering is enabled.
    pub wireframe_enabled: bool,
    /// Controls whether the editor grid is visible in the viewport.
    pub grid_enabled: bool,
    /// Whether asset loading is in progress.
    pub is_loading_assets: bool,
    /// Active gizmo mode (Select, Translate, Rotate, Scale).
    pub gizmo_mode: ae_editor::gizmo::GizmoMode,
    /// Controls whether gizmo axes are aligned to world or entity-local orientation.
    pub gizmo_space: ae_editor::gizmo::GizmoSpace,
    /// Active color hex buffer in the inspector.
    pub inspector_color_hex: String,
    /// User-saved color swatches.
    pub saved_swatches: Vec<[f32; 4]>,
    /// Whether the preferences dialog is visible.
    pub show_preferences: bool,
    /// Whether the about dialog is visible.
    pub show_about: bool,
    /// Active tab in the preferences dialog.
    pub preferences_tab: u8,
    /// Flag requesting the engine to save the active scene.
    pub should_save_scene: bool,
    /// Flag requesting the engine to load a scene.
    pub should_load_scene: bool,
    /// Active filesystem path of the current scene.
    pub active_scene_path: String,
    /// Pending save path from dialog.
    pub pending_save_path: Option<PathBuf>,
    /// Pending load path from dialog.
    pub pending_load_path: Option<PathBuf>,
    /// Asynchronous file dialog receivers.
    pub scene_dialog_receivers: Vec<Receiver<SceneDialogAction>>,
    /// Flag requesting editor shutdown.
    pub should_exit: bool,
    /// Search query in the hierarchy panel.
    pub hierarchy_search_query: String,
    /// Active UI scaling factor.
    pub ui_zoom_factor: f32,
    /// Last recorded 3D viewport screen rectangle in logical coordinates.
    pub last_viewport_rect: Rect,
    /// Viewport width in logical pixels.
    pub viewport_rect_width: f32,
    /// Viewport height in logical pixels.
    pub viewport_rect_height: f32,
    /// Live frames per second.
    pub fps: f32,
    /// Current mouse cursor position in logical pixels.
    pub cursor_pos: Point,

    // Telemetry & Profiler
    /// Profiler snapshot: ECS update duration (ms).
    pub profiler_ecs_ms: f32,
    /// Profiler snapshot: Physics simulation duration (ms).
    pub profiler_physics_ms: f32,
    /// Profiler snapshot: Render duration (ms).
    pub profiler_render_ms: f32,
    /// Profiler snapshot: VSync present wait duration (ms).
    pub profiler_present_ms: f32,
    /// Profiler snapshot: UI update duration (ms).
    pub profiler_ui_ms: f32,
    /// Profiler snapshot: Total frame duration (ms).
    pub profiler_frame_ms: f32,
    /// CPU execution breakdown timings.
    pub cpu_timings: ae_core::telemetry::CpuSyncTimings,
    /// GPU pass execution timings.
    pub gpu_pass_timings: ae_core::telemetry::GpuPassTimings,
    /// Frame pacing 240-sample history.
    pub frame_pacing: ae_core::telemetry::FrameRingBuffer<240>,
    /// Frame pacing statistical metrics.
    pub frame_pacing_stats: ae_core::telemetry::FramePacingStats,
    /// Draw call breakdown metrics.
    pub draw_call_stats: ae_core::telemetry::DrawCallBreakdown,
    /// Granular Video RAM metrics.
    pub vram_stats: ae_core::telemetry::VramStats,
    /// Models memory usage (MB).
    pub memory_models_mb: f32,
    /// Textures memory usage (MB).
    pub memory_textures_mb: f32,
    /// Live render draw calls.
    pub render_draw_calls: u32,
    /// Live render triangles.
    pub render_triangles: u64,
    /// Live render vertices.
    pub render_vertices: u64,
    /// GPU adapter name string.
    pub gpu_adapter_name: String,
    /// GPU backend string.
    pub gpu_backend: String,

    // Console Logging Buffer
    /// Snapshot of log entries.
    pub console_entries: Vec<ConsoleEntry>,
    /// Log count snapshot tracker for change detection.
    pub console_last_count: u64,

    /// Active engine execution mode snapshot (Edit, Play, Pause).
    pub active_mode: ae_core::modules::EngineMode,
    /// Pending UI action commands queued from interactive widgets to be dispatched to the engine.
    pub pending_actions: Vec<EngineUiAction>,

    /// Surface output color format for lazy text renderer initialization.
    pub output_color_format: wgpu::TextureFormat,
    /// Retained top application menu bar and dropdown overlay state.
    pub menubar: crate::ui::menubar::MenuBarState,
    /// Asset browser state.
    pub asset_browser: crate::assets::AssetBrowserState,
    /// UI designer state.
    pub ui_designer_state: ae_uidesign::UiDesignerState,
}

impl EngineUi {
    /// Initializes the native Iris UI renderer and workbench subsystems.
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let renderer = IrisRenderer::new(device, output_color_format);
        let text_system = TextSystem::new();
        let layout_engine = LayoutEngine::new();
        let tree = UiTree::new();
        let layout_state = PanelLayoutState::new_default();
        let command_list = DrawCommandList::new();

        Self {
            tree,
            layout_engine,
            renderer,
            text_system,
            text_renderer: None,
            layout_state,
            command_list,
            output_color_format,
            asset_browser: crate::assets::AssetBrowserState::new(),
            ui_designer_state: ae_uidesign::UiDesignerState::default(),
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
            should_save_scene: false,
            should_load_scene: false,
            active_scene_path: "scene.ae3d".to_string(),
            pending_save_path: None,
            pending_load_path: None,
            scene_dialog_receivers: Vec::new(),
            should_exit: false,
            hierarchy_search_query: String::new(),
            ui_zoom_factor: 1.0,
            menubar: crate::ui::menubar::MenuBarState::new(),
            last_viewport_rect: Rect::new(
                0.0,
                crate::ui::menubar::MENUBAR_HEIGHT,
                size.width as f32 / scale_factor,
                ((size.height as f32 / scale_factor) - crate::ui::menubar::MENUBAR_HEIGHT)
                    .max(10.0),
            ),
            viewport_rect_width: size.width as f32 / scale_factor,
            viewport_rect_height: size.height as f32 / scale_factor,
            fps: 60.0,
            cursor_pos: Point::new(0.0, 0.0),
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
            console_entries: Vec::new(),
            console_last_count: 0,
            active_mode: ae_core::modules::EngineMode::Edit,
            pending_actions: Vec::new(),
        }
    }

    /// Active UI scaling factor.
    #[inline]
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
        self.status_message = Some((vec![(text.into(), color)], Instant::now()));
    }

    /// Returns whether any text input widget currently captures keyboard events.
    pub fn wants_keyboard_input(&self) -> bool {
        self.tree
            .find_node(|n| n.role == WidgetRole::TextInput)
            .is_some()
    }

    /// Returns whether a screen coordinate is over interactive UI elements.
    ///
    /// Evaluates the active [`UiTree`] via [`UiTree::hit_test_target`], respecting layer stacking
    /// contexts and element interactivity with zero manual coordinate heuristics.
    pub fn is_point_over_ui(&self, pos: [f32; 2]) -> bool {
        let p = Point::new(pos[0], pos[1]);
        self.tree.hit_test_target(p).is_some()
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

    /// Dispatches window input events to Iris UI.
    pub fn handle_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let scale = self.scale_factor();
                self.cursor_pos = Point::new(position.x as f32 / scale, position.y as f32 / scale);
                let cur = self.tree.cursor_at(self.cursor_pos);
                window.set_cursor(map_widget_cursor_to_winit(cur));
            }
            winit::event::WindowEvent::Resized(size) => {
                let scale = self.scale_factor();
                self.viewport_rect_width = size.width as f32 / scale;
                self.viewport_rect_height = size.height as f32 / scale;
                self.last_viewport_rect = Rect::new(
                    0.0,
                    crate::ui::menubar::MENUBAR_HEIGHT,
                    self.viewport_rect_width,
                    (self.viewport_rect_height - crate::ui::menubar::MENUBAR_HEIGHT).max(10.0),
                );
            }
            _ => {}
        }

        let mut menubar_ctx = crate::ui::menubar::MenuBarEventContext {
            cursor_pos: self.cursor_pos,
            pending_actions: &mut self.pending_actions,
            layout_state: &mut self.layout_state,
            show_preferences: &mut self.show_preferences,
            show_about: &mut self.show_about,
            is_editing: self.active_mode == ae_core::modules::EngineMode::Edit,
            should_exit: &mut self.should_exit,
        };
        if crate::ui::menubar::handle_menubar_event(
            &mut self.menubar,
            &self.tree,
            event,
            &mut menubar_ctx,
        ) {
            return true;
        }

        if let winit::event::WindowEvent::MouseInput {
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Left,
            ..
        } = event
        {
            return self.handle_mouse_pressed();
        }
        false
    }

    /// Handles left mouse click dispatching actions for interactive widgets.
    fn handle_mouse_pressed(&mut self) -> bool {
        if let Some(target) = self.tree.hit_test_target(self.cursor_pos) {
            if let Some(hud_action) = evaluate_viewport_hud_tag(target.tag) {
                self.handle_hud_action(hud_action);
            }
            true
        } else {
            false
        }
    }

    /// Dispatches an action originating from Viewport HUD overlay controls.
    pub fn handle_hud_action(&mut self, action: ViewportHudAction) {
        match action {
            ViewportHudAction::SetGizmoMode(mode) => {
                self.gizmo_mode = match mode {
                    ViewportGizmoMode::Select => ae_editor::gizmo::GizmoMode::Select,
                    ViewportGizmoMode::Translate => ae_editor::gizmo::GizmoMode::Translate,
                    ViewportGizmoMode::Rotate => ae_editor::gizmo::GizmoMode::Rotate,
                    ViewportGizmoMode::Scale => ae_editor::gizmo::GizmoMode::Scale,
                };
            }
            ViewportHudAction::ToggleGizmoSpace => {
                self.gizmo_space = match self.gizmo_space {
                    ae_editor::gizmo::GizmoSpace::World => ae_editor::gizmo::GizmoSpace::Local,
                    ae_editor::gizmo::GizmoSpace::Local => ae_editor::gizmo::GizmoSpace::World,
                };
            }
            ViewportHudAction::ToggleSnapping => {
                self.pending_actions
                    .push(EngineUiAction::UpdateSnapSettings(
                        ae_editor::snapping::SnapSettings {
                            mode: ae_editor::snapping::SnapMode::Toggle,
                            current_enabled: true,
                            ..Default::default()
                        },
                    ));
            }
            ViewportHudAction::ToggleGrid => {
                self.grid_enabled = !self.grid_enabled;
            }
            ViewportHudAction::TogglePlayPause => {
                let next_mode = if self.active_mode == ae_core::modules::EngineMode::Play {
                    ae_core::modules::EngineMode::Edit
                } else {
                    ae_core::modules::EngineMode::Play
                };
                self.pending_actions
                    .push(EngineUiAction::ChangeMode(next_mode));
            }
            ViewportHudAction::StopSimulation => {
                self.pending_actions.push(EngineUiAction::ChangeMode(
                    ae_core::modules::EngineMode::Edit,
                ));
            }
            ViewportHudAction::ToggleCameraProjection => {
                self.pending_actions
                    .push(EngineUiAction::ToggleCameraProjection);
            }
            ViewportHudAction::ToggleWireframe => {
                self.wireframe_enabled = !self.wireframe_enabled;
            }
        }
    }

    /// Primary render pass executing Iris UI GPU drawing commands.
    pub fn render(
        &mut self,
        params: EditorUiRenderParams<'_>,
    ) -> ae_renderer::render::ViewportRect {
        self.active_mode = *params.mode;
        // Drain any pending interactive actions into outgoing queue
        params.ui_actions.append(&mut self.pending_actions);

        let win_size = params.window.inner_size();
        let scale = self.scale_factor();
        let logical_w = win_size.width as f32 / scale;
        let logical_h = win_size.height as f32 / scale;
        self.viewport_rect_width = logical_w;
        self.viewport_rect_height = logical_h;
        self.last_viewport_rect = Rect::new(
            0.0,
            crate::ui::menubar::MENUBAR_HEIGHT,
            logical_w,
            (logical_h - crate::ui::menubar::MENUBAR_HEIGHT).max(10.0),
        );

        // 1. Rebuild UI Tree cleanly with standardized root canvas
        let root = self
            .tree
            .reset_with_canvas(Rect::new(0.0, 0.0, logical_w, logical_h), "WorkbenchRoot");

        // 2. Top Application Menu Bar
        let is_editing = *params.mode == ae_core::modules::EngineMode::Edit;
        let menu_output = crate::ui::menubar::build_top_menu_bar(
            &mut self.tree,
            Some(root),
            logical_w,
            &self.menubar,
            is_editing,
        );
        self.menubar.button_ids = menu_output.menu_button_ids.to_vec();

        // 3. Viewport 3D Render Texture Binding via iris-widgets ViewportCanvasBuilder
        if let Some(vp_tex) = params.viewport_texture_view {
            let tex_id = ExternalTextureId(1);
            self.renderer.external_textures.set(
                params.device,
                &self.renderer.external_pipeline,
                tex_id,
                vp_tex,
            );

            let _canvas_id = ViewportCanvasBuilder::new(&mut self.tree, self.last_viewport_rect)
                .external_texture(tex_id)
                .background(Color::rgba(0.08, 0.08, 0.10, 1.0))
                .layer(UiLayer::Background)
                .cursor(WidgetCursor::Default)
                .interactive(false)
                .name("Main3DViewport")
                .build(Some(root));

            // Floating Viewport HUD Controls via iris-widgets ViewportHudBuilder
            let vp_gizmo_mode = match self.gizmo_mode {
                ae_editor::gizmo::GizmoMode::Select => ViewportGizmoMode::Select,
                ae_editor::gizmo::GizmoMode::Translate => ViewportGizmoMode::Translate,
                ae_editor::gizmo::GizmoMode::Rotate => ViewportGizmoMode::Rotate,
                ae_editor::gizmo::GizmoMode::Scale => ViewportGizmoMode::Scale,
            };
            let vp_gizmo_space = match self.gizmo_space {
                ae_editor::gizmo::GizmoSpace::World => ViewportGizmoSpace::World,
                ae_editor::gizmo::GizmoSpace::Local => ViewportGizmoSpace::Local,
            };
            let vp_cam_mode = match params.camera.mode {
                ae_renderer::camera::ProjectionMode::Perspective => ViewportCameraMode::Perspective,
                ae_renderer::camera::ProjectionMode::Orthographic => {
                    ViewportCameraMode::Orthographic
                }
            };
            let vp_eng_mode = match *params.mode {
                ae_core::modules::EngineMode::Edit => ViewportEngineMode::Editing,
                ae_core::modules::EngineMode::Play => ViewportEngineMode::Playing,
            };
            let frame_ms = if params.fps > 0.1 {
                1000.0 / params.fps
            } else {
                16.67
            };

            let _hud_frame = ViewportHudBuilder::new(self.last_viewport_rect)
                .gizmo_mode(vp_gizmo_mode)
                .gizmo_space(vp_gizmo_space)
                .snapping(params.snapping.current_enabled)
                .grid(self.grid_enabled)
                .wireframe(self.wireframe_enabled)
                .camera_mode(vp_cam_mode)
                .engine_mode(vp_eng_mode)
                .diagnostics(params.fps, frame_ms)
                .build(&mut self.tree, root);
        }

        // 4. Content Text Measurement and Flexbox Layout Pass
        self.layout_engine.clear();
        self.text_system.measure_tree(&mut self.tree, root);
        let _ = self
            .layout_engine
            .compute_layout(&mut self.tree, Size::new(logical_w, logical_h));

        // 5. Floating Dropdown Popup Overlay (Rendered as topmost overlay with anchor aligned to menu button)
        if let Some(active) = self.menubar.active_menu {
            let anchor_x = self
                .menubar
                .button_ids
                .iter()
                .find(|(m, _)| *m == active)
                .and_then(|(_, id)| self.tree.get(*id))
                .map(|n| n.computed_rect.x)
                .unwrap_or(0.0);
            let (dd_id, actions, dd_rect) = crate::ui::menubar::build_floating_dropdown(
                &mut self.tree,
                Some(root),
                crate::ui::menubar::DropdownBuildParams {
                    active,
                    anchor_x,
                    cursor_pos: self.cursor_pos,
                    layout_state: &self.layout_state,
                    can_undo: !params.undo_stack.is_empty(),
                    can_redo: !params.redo_stack.is_empty(),
                },
            );
            self.text_system.measure_tree(&mut self.tree, dd_id);
            self.menubar.actions = actions;
            self.menubar.dropdown_rect = Some(dd_rect);
        }

        // 6. Compile Draw Commands
        self.command_list.clear();
        let mut options = TreeCompilerOptions::default();
        compile_tree_draw_commands_into(&self.tree, root, &mut options, &mut self.command_list);

        // 7. Prepare GPU buffers
        self.renderer.prepare_command_list(
            params.device,
            params.queue,
            [win_size.width as f32, win_size.height as f32],
            &self.command_list,
        );

        // 8. Lazy initialize text renderer and upload glyphs to GPU text atlas
        if self.text_renderer.is_none() {
            self.text_renderer = Some(TextRenderer::new(
                params.device,
                params.queue,
                self.output_color_format,
            ));
        }

        let text_sections = irisui::text::collect_text_sections(&self.tree);
        if let Some(ref mut txt_renderer) = self.text_renderer {
            txt_renderer.prepare(
                params.device,
                params.queue,
                &mut self.text_system,
                (win_size.width, win_size.height),
                scale,
                &text_sections,
            );
        }

        // 9. Execute Render Pass via ae_renderer helper
        ae_renderer::render::iris_render_pass(ae_renderer::render::IrisRenderPassParams {
            device: params.device,
            queue: params.queue,
            encoder: params.encoder,
            target_view: params.window_surface_view,
            renderer: &mut self.renderer,
            command_list: &self.command_list,
            text_renderer: self.text_renderer.as_ref(),
            screen_size: (win_size.width, win_size.height),
        });

        ae_renderer::render::ViewportRect {
            min_x: self.last_viewport_rect.x,
            min_y: self.last_viewport_rect.y,
            max_x: self.last_viewport_rect.right(),
            max_y: self.last_viewport_rect.bottom(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_point_over_ui_tree_driven_and_hud_isolation() {
        let mut tree = UiTree::new();
        let root = tree.reset_with_canvas(Rect::new(0.0, 0.0, 1920.0, 1080.0), "WorkbenchRoot");

        // 1. Viewport canvas (interactive = false)
        let _canvas_id = ViewportCanvasBuilder::new(&mut tree, Rect::new(0.0, 0.0, 1920.0, 1080.0))
            .background(Color::rgba(0.08, 0.08, 0.10, 1.0))
            .layer(UiLayer::Background)
            .interactive(false)
            .name("Main3DViewport")
            .build(Some(root));

        // 2. Viewport HUD overlay (interactive = true buttons)
        let _hud_frame = ViewportHudBuilder::new(Rect::new(0.0, 0.0, 1920.0, 1080.0))
            .gizmo_mode(ViewportGizmoMode::Translate)
            .build(&mut tree, root);

        // Inside empty 3D viewport canvas: hit_test_target must return None
        let center_canvas_hit = tree.hit_test_target(Point::new(960.0, 540.0));
        assert!(center_canvas_hit.is_none());

        // Over Viewport HUD top-left toolbar button (e.g. Move at x=75.0, y=20.0):
        let hud_btn_hit = tree.hit_test_target(Point::new(75.0, 20.0));
        assert!(hud_btn_hit.is_some());
        let info = hud_btn_hit.unwrap();
        assert_eq!(info.role, WidgetRole::Button);
        assert_eq!(info.tag, iris_widgets::VIEWPORT_HUD_TAG_GIZMO_TRANSLATE);
    }

    #[test]
    fn test_map_widget_cursor_to_winit() {
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Default),
            winit::window::CursorIcon::Default
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Pointer),
            winit::window::CursorIcon::Pointer
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::Grab),
            winit::window::CursorIcon::Grab
        );
        assert_eq!(
            map_widget_cursor_to_winit(WidgetCursor::ColResize),
            winit::window::CursorIcon::ColResize
        );
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Overlay Render Pass
//!
//! Handles cursor state resolution, overlay parameter construction, layout update,
//! and Iris UI GPU SDF WGPU render pass execution.

use crate::ui::iris_bridge;
use crate::ui::workbench::state::EngineUi;
use irisui::prelude::Rect;
use winit::window::Window;

/// Parameters descriptor for the Iris UI overlay update and rendering pass.
pub struct IrisPassParams<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a Window,
    pub window_surface_view: &'a wgpu::TextureView,
    pub viewport_texture_view: Option<&'a wgpu::TextureView>,
    pub viewport_rect: Rect,
    pub is_editing: bool,
    pub undo_stack: &'a [ae_editor::undo_redo::Command],
    pub redo_stack: &'a [ae_editor::undo_redo::Command],
    pub graphics_settings: &'a ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping_settings: &'a ae_editor::snapping::SnapSettings,
    pub editor_config: &'a ae_editor::editor_state::EditorConfig,
    pub enable_live_updates: bool,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub camera: &'a ae_renderer::camera::Camera,
    pub world: &'a hecs::World,
    pub stats_panel_rect: Option<Rect>,
    pub hierarchy_panel_rect: Option<Rect>,
    pub inspector_panel_rect: Option<Rect>,
    pub console_panel_rect: Option<Rect>,
    pub assets_panel_rect: Option<Rect>,
    pub timeline_panel_rect: Option<Rect>,
    pub material_panel_rect: Option<Rect>,
    pub ui_designer_panel_rect: Option<Rect>,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Whether the editor is running in 2D dimension mode.
    pub is_2d_mode: bool,
}

impl EngineUi {
    /// Updates Iris UI overlays (Menubar, Toolbar, Inspector, Hierarchy, Modals) and renders them to WGPU.
    pub fn execute_iris_pass(&mut self, params: IrisPassParams<'_>) {
        // Synchronize active floating window boundaries with IrisEditorOverlay for occlusion testing
        self.iris_overlay.chrome.floating_window_rects = self
            .layout_state
            .dock_state
            .floating_windows
            .iter()
            .map(|w| Rect::new(w.rect.x, w.rect.y, w.rect.width, w.rect.height))
            .collect();

        // Resolve active window cursor icon
        let requested_cursor = self.iris_overlay.requested_cursor_icon();
        if requested_cursor != winit::window::CursorIcon::Default {
            params.window.set_cursor(requested_cursor);
        } else if self
            .iris_overlay
            .is_point_over_overlay(self.iris_overlay.cursor_pos())
        {
            params.window.set_cursor(winit::window::CursorIcon::Default);
        }

        let win_size = params.window.inner_size();
        if win_size.width == 0 || win_size.height == 0 {
            return;
        }

        self.expire_status_message();

        let zoom = self.scale_factor();
        let logical_w = win_size.width as f32 / zoom;
        let logical_h = win_size.height as f32 / zoom;

        let delete_target = self.asset_browser.delete_confirmation.as_deref();

        if self.asset_browser.new_folder_parent.is_some()
            && self.iris_overlay.modals.new_folder_buffer.is_empty()
            && !self.asset_browser.new_folder_name.is_empty()
        {
            self.iris_overlay.modals.new_folder_buffer = self.asset_browser.new_folder_name.clone();
        }

        let new_folder_parent = self.asset_browser.new_folder_parent.as_deref();

        if let Some(ref ren) = self.asset_browser.rename_state
            && self.iris_overlay.modals.rename_buffer.is_empty()
            && !ren.current_name.is_empty()
        {
            self.iris_overlay.modals.rename_buffer = ren.current_name.clone();
        }

        let rename_target = self
            .asset_browser
            .rename_state
            .as_ref()
            .map(|r| (r.target_path.as_path(), r.is_folder));

        self.iris_overlay
            .ensure_tools_texture(params.device, params.queue);
        self.iris_overlay
            .ensure_asset_thumbnails(params.queue, &self.asset_browser.cached_items);
        self.iris_overlay
            .set_viewport_texture(params.device, params.viewport_texture_view);

        let status_spans = self
            .status_message
            .as_ref()
            .map(|(spans, _)| spans.as_slice());

        if let Some(entity) = self.selected_entity
            && self.iris_overlay.inspector.color_drag_mode.is_none()
        {
            let col = params
                .world
                .get::<&ae_core::ecs::Color>(entity)
                .map(|c| *c)
                .unwrap_or(ae_core::ecs::Color {
                    r: 0.60,
                    g: 0.75,
                    b: 0.95,
                    a: 1.0,
                });
            let (h, s, v) = irisui::prelude::rgb_to_hsv(col.r, col.g, col.b);
            self.iris_overlay.inspector.hsv = [h, s, v];
            let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
            self.inspector_color_hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        }

        self.iris_overlay
            .update_overlays(iris_bridge::OverlayUpdateParams {
                context: iris_bridge::EditorContextParams {
                    dimensions: (logical_w, logical_h),
                    zoom_factor: zoom,
                    is_editing: params.is_editing,
                    is_2d_mode: params.is_2d_mode,
                    layout_state: &self.layout_state,
                    can_undo: !params.undo_stack.is_empty(),
                    can_redo: !params.redo_stack.is_empty(),
                    enable_live_updates: params.enable_live_updates,
                    status_spans,
                },
                viewport: iris_bridge::ViewportParams {
                    has_viewport_texture: params.viewport_texture_view.is_some(),
                    viewport_rect: params.viewport_rect,
                    camera: params.camera,
                    wireframe_enabled: self.wireframe_enabled,
                    grid_enabled: self.grid_enabled,
                    gizmo_mode: self.gizmo_mode,
                    gizmo_space: self.gizmo_space,
                },
                scene: iris_bridge::SceneParams {
                    selected_entity: self.selected_entity,
                    world: params.world,
                    active_entities_count: params.world.len() as usize,
                },
                dialogs: iris_bridge::DialogParams {
                    show_about: self.show_about,
                    show_preferences: self.show_preferences,
                    delete_target,
                    new_folder_parent,
                    rename_target,
                    is_loading_assets: self.is_loading_assets,
                },
                preferences: iris_bridge::OverlayPreferencesParams {
                    graphics_settings: params.graphics_settings,
                    snapping_settings: params.snapping_settings,
                    editor_config: params.editor_config,
                    enabled_modules: params.enabled_modules,
                },
                telemetry: iris_bridge::TelemetryParams {
                    fps: self.fps,
                    frame_pacing: &self.frame_pacing,
                    frame_pacing_stats: &self.frame_pacing_stats,
                    cpu_timings: &self.cpu_timings,
                    gpu_pass_timings: &self.gpu_pass_timings,
                    draw_call_stats: &self.draw_call_stats,
                    vram_stats: &self.vram_stats,
                    render_triangles: self.render_triangles as usize,
                    render_vertices: self.render_vertices as usize,
                    gpu_adapter_name: &self.gpu_adapter_name,
                    gpu_backend: &self.gpu_backend,
                },
                panel_rects: iris_bridge::OverlayPanelRects {
                    stats: params.stats_panel_rect,
                    hierarchy: params.hierarchy_panel_rect,
                    inspector: params.inspector_panel_rect,
                    console: params.console_panel_rect,
                    assets: params.assets_panel_rect,
                    timeline: params.timeline_panel_rect,
                    material: params.material_panel_rect,
                    ui_designer: params.ui_designer_panel_rect,
                },
                panel_data: iris_bridge::OverlayPanelData {
                    ui_designer_state: &self.ui_designer_state,
                    asset_browser: &self.asset_browser,
                    console_entries: &self.console_entries,
                    textures: params.textures,
                    models: params.models,
                    inspector_euler: &self.inspector_euler,
                    inspector_color_hex: &self.inspector_color_hex,
                    saved_swatches: &self.saved_swatches,
                },
            });

        self.iris_overlay.render(
            params.device,
            params.queue,
            params.encoder,
            params.window_surface_view,
            (win_size.width, win_size.height),
            zoom,
        );
    }
}
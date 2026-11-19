// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Overlay Render Pass
//!
//! Handles cursor state resolution, overlay parameter construction, layout update,
//! and Iris UI GPU SDF WGPU render pass execution.

use crate::ui::iris_bridge;
use crate::ui::workbench::{EditorPanelContext, EngineUi};
use irisui::prelude::Rect;
use winit::window::Window;

/// Parameters descriptor for the Iris UI overlay update and rendering pass.
pub struct IrisPassParams<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a Window,
    pub window_surface_view: &'a wgpu::TextureView,
    /// Authoritative physical pixel dimensions `(width, height)` of the bound `window_surface_view`.
    pub surface_size: (u32, u32),
    pub viewport_texture_view: Option<&'a wgpu::TextureView>,
    pub viewport_rect: Rect,
    pub is_editing: bool,
    pub undo_stack: &'a [ae_editor::undo_redo::Command],
    pub redo_stack: &'a [ae_editor::undo_redo::Command],
    pub history_revision: u64,
    pub graphics_settings: &'a mut ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping_settings: &'a mut ae_editor::snapping::SnapSettings,
    pub editor_config: &'a mut ae_editor::editor_state::EditorConfig,
    pub enable_live_updates: &'a mut bool,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub camera: &'a ae_renderer::camera::Camera,
    pub world: &'a hecs::World,
    pub textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
    pub models: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::ModelAsset>,
    /// Whether the editor is running in 2D dimension mode.
    pub is_2d_mode: bool,
}

impl EngineUi {
    /// Updates Iris UI overlays (Menubar, Toolbar, Inspector, Hierarchy, Modals) and renders them to WGPU.
    pub fn execute_iris_pass(&mut self, params: IrisPassParams<'_>) {
        // Resolve and synchronize OS cursor icon through Workbench (cached to suppress redundant OS IPC)
        let mut win = params.window;
        self.workbench.sync_cursor(&mut win);

        let (surface_w, surface_h) = params.surface_size;
        if surface_w == 0 || surface_h == 0 {
            return;
        }

        self.expire_status_message();

        let zoom = self.scale_factor();
        let logical_w = surface_w as f32 / zoom;
        let logical_h = surface_h as f32 / zoom;

        iris_bridge::icons::ensure_editor_icon_atlas(
            &mut self.surface,
            params.device,
            params.queue,
        );
        self.iris_overlay.assets.ensure_thumbnails(
            &self.surface,
            params.queue,
            &self.asset_browser.cached_items,
        );
        self.surface.set_external_texture(
            params.device,
            iris_bridge::VIEWPORT_TEXTURE_ID,
            params.viewport_texture_view,
        );

        let status_spans = self
            .status_message
            .as_ref()
            .map(|(spans, _)| spans.as_slice());

        let view_panel_mask = self
            .workbench
            .tab_visibility_mask(&iris_bridge::menubar::VIEW_PANEL_IDS);
        let cursor_pos = self.workbench.effective_cursor_pos();
        let is_cursor_occluded = self.workbench.is_cursor_occluded();
        let selection_id = self.selected_entity.map(|e| e.to_bits().get());
        let is_drag_active = self.asset_browser.drag_payload.is_some();

        let mut overlay_params = iris_bridge::OverlayUpdateParams {
            context: iris_bridge::EditorContextParams {
                dimensions: (logical_w, logical_h),
                zoom_factor: zoom,
                is_editing: params.is_editing,
                is_2d_mode: params.is_2d_mode,
                view_panel_mask,
                can_undo: !params.undo_stack.is_empty(),
                can_redo: !params.redo_stack.is_empty(),
                enable_live_updates: *params.enable_live_updates,
                status_spans,
                history_revision: params.history_revision,
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
            preferences: iris_bridge::OverlayPreferencesParams {
                graphics_settings: params.graphics_settings,
                snapping_settings: params.snapping_settings,
                editor_config: params.editor_config,
                enable_live_updates: params.enable_live_updates,
                enabled_modules: params.enabled_modules,
            },
            telemetry: iris_bridge::TelemetryParams {
                fps: self.fps,
                frame_pacing: &self.telemetry.frame_pacing,
                frame_pacing_stats: &self.telemetry.frame_pacing_stats,
                cpu_timings: &self.telemetry.cpu_timings,
                pass_recording_timings: &self.telemetry.pass_recording_timings,
                draw_call_stats: &self.telemetry.draw_call_stats,
                vram_stats: &self.telemetry.vram_stats,
                render_triangles: self.telemetry.render_triangles as usize,
                render_vertices: self.telemetry.render_vertices as usize,
                gpu_adapter_name: &self.gpu_adapter_name,
                gpu_backend: &self.gpu_backend,
            },
            panel_data: iris_bridge::OverlayPanelData {
                ui_designer_state: &self.ui_designer_state,
                asset_browser: &mut self.asset_browser,
                console_entries: &self.console_entries,
                textures: params.textures,
                models: params.models,
            },
        };

        let mut menubar_bus = std::mem::take(&mut self.workbench.action_bus);
        let mut staged_dialogs = irisui::workbench::DialogHost::new();
        self.deferred_notifier.clear();
        let blink_caret = self.workbench.text_system.is_caret_visible();
        let frame_outcome = {
            let mut panel_ctx = EditorPanelContext {
                overlay: &mut self.iris_overlay,
                notifier: &mut self.deferred_notifier,
                action_bus: &mut menubar_bus,
                dialog_host: &mut staged_dialogs,
                params: &mut overlay_params,
                blink_caret,
                cursor_pos,
            };
            self.workbench.update_frame_with_registry(
                irisui::workbench::WorkbenchOwnedFrameParams {
                    scale_factor: zoom,
                    selection_id,
                    domain_revision: params.history_revision,
                    is_cursor_occluded,
                },
                irisui::workbench::WorkbenchExternalContext {
                    registry: &mut self.panel_registry,
                    context: &mut panel_ctx,
                },
                |inputs| {
                    inputs
                        .with_drag_active(is_drag_active)
                        .with_surface_revision(u64::from(params.viewport_texture_view.is_some()))
                },
                |scope, ctx, size| {
                    iris_bridge::menubar::build_top_menu_bar(
                        scope,
                        &iris_bridge::menubar::MenuBarParams {
                            screen_width: size.width,
                            view_panel_mask: ctx.params.context.view_panel_mask,
                            can_undo: ctx.params.context.can_undo,
                            can_redo: ctx.params.context.can_redo,
                            is_editing: ctx.params.context.is_editing,
                        },
                        ctx.action_bus,
                    );
                    iris_bridge::export_modal::render_export_project_modal(
                        scope,
                        &mut ctx.overlay.export_modal,
                        ctx.action_bus,
                    );
                    iris_bridge::status_bar::render_workbench_status_bar(
                        scope,
                        size.width,
                        size.height,
                        ctx.params.context.status_spans,
                    );
                },
            )
        };
        self.workbench.append_dialogs(&mut staged_dialogs);
        self.workbench
            .notifier
            .apply_deferred(&mut self.deferred_notifier);
        for action in menubar_bus.drain() {
            self.workbench.action_bus.emit(action);
        }
        if let Ok(ref outcome) = frame_outcome {
            let hovered_tag = self.workbench.hovered_tag();
            let blink_caret = self.workbench.text_system.is_caret_visible();
            self.surface.apply_frame_flags(
                &mut self.workbench.tree,
                blink_caret,
                hovered_tag,
                outcome.to_surface_frame_flags(),
                None,
            );
        }

        let hovered_tag = self.workbench.hovered_tag();
        self.surface.render_to_view(
            &mut self.workbench.tree,
            &mut self.workbench.text_system,
            hovered_tag,
            irisui::IrisSurfaceRenderParams {
                device: params.device,
                queue: params.queue,
                encoder: params.encoder,
                target_view: params.window_surface_view,
                physical_screen_size: (surface_w, surface_h),
                zoom_factor: zoom,
                extra_text_occluders: &[],
            },
        );
    }

    /// Uploads editor icon atlas if needed and renders the retained UI tree into `target_view`.
    pub fn render_offscreen_snapshot(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        physical_screen_size: (u32, u32),
        zoom_factor: f32,
    ) {
        crate::ui::workbench::icons::ensure_editor_icon_atlas(&mut self.surface, device, queue);
        self.workbench.render_to_surface(
            &mut self.surface,
            irisui::workbench::WorkbenchSurfaceRender {
                device,
                queue,
                encoder,
                target_view,
                physical_screen_size,
                zoom_factor,
            },
        );
    }
}
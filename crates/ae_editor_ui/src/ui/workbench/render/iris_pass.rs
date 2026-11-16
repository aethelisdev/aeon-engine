// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Iris UI Overlay Render Pass
//!
//! Handles cursor state resolution, overlay parameter construction, layout update,
//! and Iris UI GPU SDF WGPU render pass execution.

use crate::ui::iris_bridge;
use crate::ui::workbench::state::EngineUi;
use irisui::prelude::{HsvColorPickerState, Rect};
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
    pub history_revision: u64,
    pub graphics_settings: &'a mut ae_renderer::graphics_settings::GraphicsSettings,
    pub snapping_settings: &'a mut ae_editor::snapping::SnapSettings,
    pub editor_config: &'a mut ae_editor::editor_state::EditorConfig,
    pub enable_live_updates: bool,
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
        // Synchronize active floating window boundaries with IrisEditorOverlay for occlusion testing (zero allocation)
        self.layout_state
            .dock_state
            .sync_floating_window_rects(&mut self.iris_overlay.chrome.floating_window_rects);

        // Resolve active window cursor icon
        let active_widget_cursor = self.iris_overlay.active_cursor();
        if active_widget_cursor != irisui::prelude::WidgetCursor::Default {
            params
                .window
                .set_cursor(irisui::winit::widget_cursor_to_winit(active_widget_cursor));
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

        if let Some(entity) = self.selected_entity {
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
            let r = (col.r.clamp(0.0, 1.0) * 255.0) as u8;
            let g = (col.g.clamp(0.0, 1.0) * 255.0) as u8;
            let b = (col.b.clamp(0.0, 1.0) * 255.0) as u8;
            const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
            let hex_bytes = [
                b'#',
                HEX_DIGITS[(r >> 4) as usize],
                HEX_DIGITS[(r & 0xf) as usize],
                HEX_DIGITS[(g >> 4) as usize],
                HEX_DIGITS[(g & 0xf) as usize],
                HEX_DIGITS[(b >> 4) as usize],
                HEX_DIGITS[(b & 0xf) as usize],
            ];
            if self.inspector_color_hex.as_bytes() != hex_bytes {
                let (h, s, v) = HsvColorPickerState::rgb_to_hsv(col.r, col.g, col.b);
                self.iris_overlay.inspector.hsv = [h, s, v];
                self.inspector_color_hex.clear();
                if let Ok(s) = std::str::from_utf8(&hex_bytes) {
                    self.inspector_color_hex.push_str(s);
                }
            }
        }

        let signals = self.iris_overlay.lifecycle_signals(
            self.layout_state
                .is_panel_visible(crate::ui::panel_layout::PanelId::Preferences),
        );
        let is_assets_search_focused = self
            .iris_overlay
            .is_tag_focused(iris_bridge::assets::ASSETS_TAG_SEARCH_INPUT);
        let is_cursor_occluded = self
            .iris_overlay
            .is_point_over_overlay(self.iris_overlay.cursor_pos())
            || self
                .layout_state
                .dock_state
                .is_point_over_floating_window(self.iris_overlay.cursor_pos());
        let selection_id = self.selected_entity.map(|e| e.to_bits().get());
        let is_drag_active = self.asset_browser.drag_payload.is_some();

        let overlay_params = iris_bridge::OverlayUpdateParams {
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
            dialogs: iris_bridge::DialogParams {
                show_preferences: false,
                delete_target: None,
                new_folder_parent: None,
                rename_target: None,
                is_loading_assets: false,
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
                pass_recording_timings: &self.pass_recording_timings,
                draw_call_stats: &self.draw_call_stats,
                vram_stats: &self.vram_stats,
                render_triangles: self.render_triangles as usize,
                render_vertices: self.render_vertices as usize,
                gpu_adapter_name: &self.gpu_adapter_name,
                gpu_backend: &self.gpu_backend,
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
            dialog_host: None,
            action_bus: None,
        };

        let mut tree = std::mem::take(&mut self.iris_overlay.tree);
        let mut layout_engine = std::mem::take(&mut self.iris_overlay.layout_engine);
        let mut text_system = std::mem::take(&mut self.iris_overlay.text_system);
        let mut notifier = std::mem::replace(
            &mut self.iris_overlay.notifier,
            irisui::dock::UiNotifier::clean(),
        );
        self.iris_overlay.assets.sync_dirty_invalidation(
            &self.asset_browser,
            params.is_2d_mode,
            is_assets_search_focused,
            &mut notifier,
        );
        let mut menubar_bus = std::mem::take(&mut self.workbench.action_bus);
        let frame_outcome = {
            let mut registry = crate::ui::workbench::create_editor_panel_registry();
            let mut panel_ctx = crate::ui::workbench::EditorPanelContext {
                overlay: &mut self.iris_overlay,
                params: &overlay_params,
            };
            let mut engines = irisui::workbench::SubtreeEngines {
                tree: &mut tree,
                layout_engine: &mut layout_engine,
                text_system: &mut text_system,
            };
            self.workbench.update_frame_live_with_registry(
                &mut engines,
                irisui::workbench::WorkbenchFrameUpdateParams {
                    dock: &self.layout_state.dock_state,
                    notifier: &mut notifier,
                    scale_factor: zoom,
                    selection_id,
                    domain_revision: params.history_revision,
                    is_cursor_occluded,
                },
                irisui::workbench::WorkbenchExternalContext {
                    registry: &mut registry,
                    context: &mut panel_ctx,
                },
                |inputs| {
                    inputs
                        .with_menu_active(signals.is_menu_active)
                        .with_modal_active(signals.is_modal_active)
                        .with_popup_active(signals.is_popup_active)
                        .with_drag_active(is_drag_active)
                        .with_surface_revision(u64::from(params.viewport_texture_view.is_some()))
                        .with_overlay_revision(signals.overlay_revision)
                },
                |scope, ctx, size| {
                    iris_bridge::menubar::build_top_menu_bar(
                        scope,
                        &iris_bridge::menubar::MenuBarParams {
                            screen_width: size.width,
                            layout_state: ctx.params.context.layout_state,
                            can_undo: ctx.params.context.can_undo,
                            can_redo: ctx.params.context.can_redo,
                            is_editing: ctx.params.context.is_editing,
                        },
                        &mut menubar_bus,
                    );
                    if ctx.overlay.assets.dialog_host.is_active() {
                        iris_bridge::assets::panel::render_assets_dialog_host(
                            scope,
                            &mut ctx.overlay.assets.dialog_host,
                            &mut ctx.overlay.assets.action_bus,
                            (size.width, size.height),
                        );
                    }
                    iris_bridge::status_bar::build_bottom_status_bar_scope(
                        scope,
                        &iris_bridge::status_bar::StatusBarParams {
                            screen_width: size.width,
                            screen_height: size.height,
                            status_spans: ctx.params.context.status_spans,
                        },
                    );
                    scope.empty_box_passive_named(
                        "IrisBottomStatusBarDivider",
                        irisui::prelude::Style::new()
                            .position_absolute()
                            .left(0.0)
                            .top(size.height - iris_bridge::status_bar::STATUS_BAR_HEIGHT)
                            .width(size.width)
                            .height(1.0)
                            .background(iris_bridge::theme::BORDER_MICRON),
                    );
                },
                |scope, ctx| {
                    iris_bridge::viewport_hud::camera_hud::update_camera_hud(
                        scope,
                        ctx.params.viewport.camera,
                        ctx.params.viewport.viewport_rect,
                    );
                    iris_bridge::viewport_hud::compass::update_compass_canvas(
                        scope,
                        ctx.params.viewport.camera,
                        ctx.params.viewport.viewport_rect,
                    );
                    iris_bridge::stats::graph::refresh_live_oscilloscope(
                        scope,
                        ctx.params.telemetry.frame_pacing,
                        ctx.params.telemetry.frame_pacing_stats,
                    );
                    iris_bridge::stats::cpu_breakdown::update_cpu_breakdown_texts(
                        scope,
                        ctx.params.telemetry.cpu_timings,
                    );
                },
            )
        };
        self.iris_overlay.tree = tree;
        self.iris_overlay.layout_engine = layout_engine;
        self.iris_overlay.text_system = text_system;
        self.iris_overlay.notifier = notifier;
        for action in menubar_bus.drain() {
            self.workbench.action_bus.emit(action);
        }
        if let Ok(outcome) = frame_outcome {
            self.iris_overlay
                .apply_workbench_frame_outcome(&outcome, &self.frame_pacing);
        }

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
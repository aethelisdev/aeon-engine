// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::docking;
use crate::ui::iris_bridge::{self, IrisEditorOverlay};
use crate::ui::preferences;
use crate::ui::types::EngineUiAction;
use crate::ui::workbench::state::EngineUi;
use egui_wgpu::ScreenDescriptor;
use winit::window::Window;

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

impl EngineUi {
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

            // 1. Top Menubar Spacer (reserving top 26px for Iris UI MenuBar)
            egui::Panel::top("top_panel_spacer")
                .exact_size(IrisEditorOverlay::MENUBAR_HEIGHT)
                .resizable(false)
                .frame(egui::Frame::NONE)
                .show(ui, |_ui| {});

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

            // 2. Bottom Status Bar Spacer (reserving bottom 22px for Iris UI StatusBar)
            let bottom_resp = egui::Panel::bottom("utility_bar_spacer")
                .exact_size(IrisEditorOverlay::STATUS_BAR_HEIGHT)
                .resizable(false)
                .frame(egui::Frame::NONE)
                .show(ui, |_ui| {});
            ui_rects_collector
                .borrow_mut()
                .push(bottom_resp.response.rect);

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
                    ui_rects_collector.borrow_mut().push(rect);
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

        let mut is_hovering_interactive = false;
        let p = self.iris_overlay.cursor_pos;

        if let Some(ref targets) = self.iris_overlay.about_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.bottom_close_rect.contains_point(p)
                || targets.link_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }
        if let Some(ref targets) = self.iris_overlay.delete_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.confirm_btn_rect.contains_point(p)
                || targets.cancel_btn_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }
        if let Some(ref targets) = self.iris_overlay.new_folder_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.confirm_btn_rect.contains_point(p)
                || targets.cancel_btn_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }
        if let Some(ref targets) = self.iris_overlay.rename_targets
            && (targets.header_close_rect.contains_point(p)
                || targets.confirm_btn_rect.contains_point(p)
                || targets.cancel_btn_rect.contains_point(p))
        {
            is_hovering_interactive = true;
        }

        if is_hovering_interactive {
            window.set_cursor(winit::window::CursorIcon::Pointer);
        } else if self
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

        // 4. Iris UI Overlay Render Pass (Menubar, Modals, Splash, and Bottom Status Bar)
        let win_size = window.inner_size();
        if win_size.width > 0 && win_size.height > 0 {
            let iris_spans: Option<Vec<(String, irisui::prelude::Color)>> =
                status_message.as_ref().map(|(spans, _)| {
                    spans
                        .iter()
                        .map(|(txt, col)| {
                            (
                                txt.clone(),
                                irisui::prelude::Color::rgba(
                                    col.r() as f32 / 255.0,
                                    col.g() as f32 / 255.0,
                                    col.b() as f32 / 255.0,
                                    col.a() as f32 / 255.0,
                                ),
                            )
                        })
                        .collect()
                });

            let delete_target = self.asset_browser.delete_confirmation.as_deref();

            if self.asset_browser.new_folder_parent.is_some()
                && self.iris_overlay.new_folder_buffer.is_empty()
                && !self.asset_browser.new_folder_name.is_empty()
            {
                self.iris_overlay.new_folder_buffer = self.asset_browser.new_folder_name.clone();
            }

            let new_folder_parent = self.asset_browser.new_folder_parent.as_deref();

            if let Some(ref ren) = self.asset_browser.rename_state
                && self.iris_overlay.rename_buffer.is_empty()
                && !ren.current_name.is_empty()
            {
                self.iris_overlay.rename_buffer = ren.current_name.clone();
            }

            let rename_target = self
                .asset_browser
                .rename_state
                .as_ref()
                .map(|r| (r.target_path.as_path(), r.is_folder));

            self.iris_overlay
                .update_overlays(iris_bridge::OverlayUpdateParams {
                    dimensions: (win_size.width as f32, win_size.height as f32),
                    is_editing: *mode == ae_core::modules::EngineMode::Edit,
                    layout_state: &self.layout_state,
                    can_undo: !undo_stack.is_empty(),
                    can_redo: !redo_stack.is_empty(),
                    show_about: self.show_about,
                    delete_target,
                    new_folder_parent,
                    rename_target,
                    is_loading_assets,
                    status_spans: iris_spans.as_deref(),
                });
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
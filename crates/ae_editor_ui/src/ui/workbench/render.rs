// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ui::docking;
use crate::ui::iris_bridge::{self, IrisEditorOverlay};
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
        let _should_save_scene = &mut self.should_save_scene;
        let _should_load_scene = &mut self.should_load_scene;
        let selected_entity = &mut self.selected_entity;
        let last_selected_entity = &mut self.last_selected_entity;
        let inspector_euler = &mut self.inspector_euler;
        let inspector_color_hex = &mut self.inspector_color_hex;
        let saved_swatches = &mut self.saved_swatches;
        let wireframe_enabled = &mut self.wireframe_enabled;
        let grid_enabled = &mut self.grid_enabled;
        let is_loading_assets = self.is_loading_assets;
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
        let stats_rect_cell = std::cell::Cell::new(None);

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
                viewport_texture_id: self.viewport_texture_id,
                viewport_rect_out: &viewport_rect,
                stats_rect_out: &stats_rect_cell,
                enabled_modules,
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

            let mut cur_gs = (*graphics_settings).clone();
            let mut cur_snap = *snapping;
            let mut cur_cfg = editor_state.config.clone();
            let mut cur_live = editor_state.enable_live_editor_updates;
            let mut gs_changed = false;
            let mut snap_changed = false;
            let mut cfg_changed = false;
            let mut live_changed = false;

            while let Some(act) = self.pending_preferences_actions.pop() {
                match act {
                    iris_bridge::PreferencesAction::Close => {
                        self.show_preferences = false;
                    }
                    iris_bridge::PreferencesAction::SelectTab(t) => {
                        self.preferences_tab = t;
                        self.iris_overlay.preferences_tab = t;
                    }
                    iris_bridge::PreferencesAction::ToggleDropdown(dd) => {
                        self.iris_overlay.preferences_dropdown = dd;
                    }
                    iris_bridge::PreferencesAction::SetUiScale(s) => {
                        ui_actions.push(EngineUiAction::SetUiScale(s));
                    }
                    iris_bridge::PreferencesAction::Toggle(toggle_id) => match toggle_id {
                        iris_bridge::PreferencesToggleId::ShadowsEnabled => {
                            cur_gs.shadow_enabled = !cur_gs.shadow_enabled;
                            gs_changed = true;
                        }
                        iris_bridge::PreferencesToggleId::BloomEnabled => {
                            cur_gs.bloom_enabled = !cur_gs.bloom_enabled;
                            gs_changed = true;
                        }
                        iris_bridge::PreferencesToggleId::FogEnabled => {
                            cur_gs.fog_enabled = !cur_gs.fog_enabled;
                            gs_changed = true;
                        }
                        iris_bridge::PreferencesToggleId::LiveUpdatesEnabled => {
                            cur_live = !cur_live;
                            live_changed = true;
                        }
                        iris_bridge::PreferencesToggleId::Module(m) => {
                            ui_actions.push(EngineUiAction::ToggleModule(m));
                        }
                    },
                    iris_bridge::PreferencesAction::SetSliderValue(slider_id, val) => {
                        match slider_id {
                            iris_bridge::PreferencesSliderId::ShadowBias => {
                                cur_gs.shadow_bias = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::BloomIntensity => {
                                cur_gs.bloom_intensity = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::SunPitch => {
                                cur_gs.sun_pitch = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::SunYaw => {
                                cur_gs.sun_yaw = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::AtmosphereDensity => {
                                cur_gs.atmosphere_density = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::OzoneDensity => {
                                cur_gs.ozone_density = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::SunDiscSize => {
                                cur_gs.sun_disc_size = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::SunGlowStrength => {
                                cur_gs.sun_glow_strength = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::CloudCoverage => {
                                cur_gs.cloud_coverage = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::CloudDensity => {
                                cur_gs.cloud_density = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::CloudSpeed => {
                                cur_gs.cloud_speed = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::CloudEvolution => {
                                cur_gs.cloud_evolution = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::CloudAltitude => {
                                cur_gs.cloud_altitude = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::FogDistance => {
                                cur_gs.fog_distance = val;
                                gs_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::GridSize => {
                                cur_snap.grid_size = val;
                                snap_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::UndoHistoryLimit => {
                                cur_cfg.max_undo_history = val as usize;
                                cfg_changed = true;
                            }
                            iris_bridge::PreferencesSliderId::PhysicsFrequency => {
                                cur_cfg.physics_hz = val;
                                cfg_changed = true;
                            }
                        }
                    }
                    iris_bridge::PreferencesAction::SelectDropdownItem(dd_id, idx) => match dd_id {
                        iris_bridge::PreferencesDropdownId::UiScale => {
                            if let Some(&(scale_val, _)) =
                                iris_bridge::preferences::tabs::general::UI_SCALES.get(idx)
                            {
                                ui_actions.push(EngineUiAction::SetUiScale(scale_val));
                            }
                        }
                        iris_bridge::PreferencesDropdownId::ShadowResolution => {
                            if let Some(&res) =
                                iris_bridge::preferences::tabs::graphics::SHADOW_RES_OPTIONS
                                    .get(idx)
                            {
                                cur_gs.shadow_resolution = res;
                                gs_changed = true;
                            }
                        }
                        iris_bridge::PreferencesDropdownId::ShadowCascades => {
                            if let Some(&(cascades, _)) =
                                iris_bridge::preferences::tabs::graphics::CASCADE_OPTIONS.get(idx)
                            {
                                cur_gs.shadow_cascades = cascades;
                                gs_changed = true;
                            }
                        }
                        iris_bridge::PreferencesDropdownId::ShadowPcf => {
                            if let Some(&pcf) =
                                iris_bridge::preferences::tabs::graphics::PCF_OPTIONS.get(idx)
                            {
                                cur_gs.shadow_pcf = pcf;
                                gs_changed = true;
                            }
                        }
                        iris_bridge::PreferencesDropdownId::FpsLimit => {
                            if let Some(&fps) =
                                iris_bridge::preferences::tabs::graphics::FPS_OPTIONS.get(idx)
                            {
                                cur_gs.fps_limit = fps;
                                gs_changed = true;
                            }
                        }
                        iris_bridge::PreferencesDropdownId::MsaaSamples => {
                            if let Some(&(samples, _)) =
                                iris_bridge::preferences::tabs::graphics::MSAA_OPTIONS.get(idx)
                            {
                                cur_gs.msaa_samples = samples;
                                gs_changed = true;
                            }
                        }
                        iris_bridge::PreferencesDropdownId::SkyQuality => {
                            if let Some(&sky) =
                                iris_bridge::preferences::tabs::graphics::SKY_OPTIONS.get(idx)
                            {
                                cur_gs.sky_quality = sky;
                                gs_changed = true;
                            }
                        }
                        iris_bridge::PreferencesDropdownId::SnapMode => {
                            if let Some(&(mode, _)) =
                                iris_bridge::preferences::tabs::SNAP_MODE_OPTIONS.get(idx)
                            {
                                cur_snap.mode = mode;
                                snap_changed = true;
                            }
                        }
                    },
                    iris_bridge::PreferencesAction::Scroll(delta) => {
                        self.iris_overlay.preferences_scroll_y =
                            (self.iris_overlay.preferences_scroll_y + delta).max(0.0);
                    }
                    iris_bridge::PreferencesAction::ToggleSection(_) => {}
                }
            }

            for action in self.iris_overlay.take_viewport_hud_actions() {
                match action {
                    iris_bridge::ViewportHudAction::SetCameraMode(cmode) => {
                        ui_actions.push(EngineUiAction::SetCameraMode(cmode));
                    }
                    iris_bridge::ViewportHudAction::SetCameraTransform {
                        pitch,
                        yaw,
                        position,
                    } => {
                        ui_actions.push(EngineUiAction::SetCameraTransform {
                            pitch,
                            yaw,
                            position,
                        });
                    }
                    iris_bridge::ViewportHudAction::ToggleWireframe => {
                        *wireframe_enabled = !*wireframe_enabled;
                    }
                    iris_bridge::ViewportHudAction::SetGizmoMode(gmode) => {
                        self.gizmo_mode = gmode;
                    }
                    iris_bridge::ViewportHudAction::ToggleGizmoSpace => {
                        self.gizmo_space = self.gizmo_space.toggle();
                    }
                    iris_bridge::ViewportHudAction::ToggleSnapping => {
                        cur_snap.mode = match cur_snap.mode {
                            ae_editor::snapping::SnapMode::Off => {
                                ae_editor::snapping::SnapMode::Toggle
                            }
                            _ => ae_editor::snapping::SnapMode::Off,
                        };
                        snap_changed = true;
                    }
                    iris_bridge::ViewportHudAction::SelectEntity(ent) => {
                        ui_actions.push(EngineUiAction::SelectEntity(Some(ent)));
                    }
                    iris_bridge::ViewportHudAction::ToggleDropdown(dd) => {
                        self.iris_overlay.viewport_hud_dropdown = dd;
                    }
                }
            }

            for action in self.iris_overlay.take_stats_actions() {
                match action {
                    iris_bridge::StatsPanelAction::ToggleWireframe => {
                        *wireframe_enabled = !*wireframe_enabled;
                    }
                    iris_bridge::StatsPanelAction::ToggleGrid => {
                        *grid_enabled = !*grid_enabled;
                    }
                    iris_bridge::StatsPanelAction::Scroll(_) => {}
                }
            }

            if gs_changed {
                ui_actions.push(EngineUiAction::UpdateGraphicsSettings(cur_gs.clone()));
            }
            if snap_changed {
                ui_actions.push(EngineUiAction::UpdateSnapSettings(cur_snap));
            }
            if cfg_changed {
                ui_actions.push(EngineUiAction::UpdateEditorConfig(cur_cfg.clone()));
            }
            if live_changed {
                ui_actions.push(EngineUiAction::SetLiveEditorUpdates(cur_live));
            }

            let iris_vp_rect = irisui::prelude::Rect::new(
                self.last_viewport_rect.min.x,
                self.last_viewport_rect.min.y,
                self.last_viewport_rect.width(),
                self.last_viewport_rect.height(),
            );

            let iris_stats_rect = stats_rect_cell
                .get()
                .map(|r| irisui::prelude::Rect::new(r.min.x, r.min.y, r.width(), r.height()));

            self.iris_overlay
                .update_overlays(iris_bridge::OverlayUpdateParams {
                    dimensions: (win_size.width as f32, win_size.height as f32),
                    is_editing: *mode == ae_core::modules::EngineMode::Edit,
                    layout_state: &self.layout_state,
                    can_undo: !undo_stack.is_empty(),
                    can_redo: !redo_stack.is_empty(),
                    show_about: self.show_about,
                    show_preferences: self.show_preferences,
                    graphics_settings: &cur_gs,
                    snapping_settings: &cur_snap,
                    editor_config: &cur_cfg,
                    enable_live_updates: cur_live,
                    enabled_modules,
                    zoom_factor: self.ui_zoom_factor,
                    delete_target,
                    new_folder_parent,
                    rename_target,
                    is_loading_assets,
                    status_spans: iris_spans.as_deref(),
                    viewport_rect: iris_vp_rect,
                    camera,
                    wireframe_enabled: self.wireframe_enabled,
                    gizmo_mode: self.gizmo_mode,
                    gizmo_space: self.gizmo_space,
                    selected_entity: *selected_entity,
                    world,
                    stats_panel_rect: iris_stats_rect,
                    grid_enabled: *grid_enabled,
                    fps: smoothed_fps,
                    frame_pacing,
                    frame_pacing_stats: &frame_pacing_stats,
                    cpu_timings: &cpu_timings,
                    gpu_pass_timings: &gpu_pass_timings,
                    draw_call_stats: &draw_call_stats,
                    vram_stats: &vram_stats,
                    render_triangles,
                    render_vertices,
                    gpu_adapter_name,
                    gpu_backend,
                    active_entities_count: world.len() as usize,
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
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor studio render pass combining shared [`ae_app::RenderHost`] scene extraction with
//! editor-only gizmos, collider wireframes, icon billboards, telemetry snapshots, and Iris UI.

use crate::session::EditorSession;
use ae_core::modules::EngineMode;
use cgmath::InnerSpace;

impl EditorSession {
    /// Executes the full editor frame render pass: applies graphics/MSAA rebuilds, prepares
    /// shared 3D/2D scene extraction via [`RenderHost::prepare_frame`], prepares editor gizmo
    /// overlays, updates UI telemetry, and renders both viewport and Iris UI workbench.
    pub fn render(&mut self) -> Result<(), ae_renderer::render::RenderError> {
        self.window.profiler.begin_render();
        let live_size = self.window.window.inner_size();
        if live_size.width > 0
            && live_size.height > 0
            && (live_size.width != self.render.render_state.size.width
                || live_size.height != self.render.render_state.size.height)
        {
            self.render
                .resize_dimensions(live_size.width, live_size.height);
        }

        let effective_delta = if self.render.render_state.graphics_settings.fps_limit
            == ae_renderer::graphics_settings::FpsLimit::Uncapped
        {
            (self.core.time.delta_time - self.render.render_state.last_present_wait_secs)
                .max(0.0001)
        } else {
            self.core.time.delta_time.max(0.0001)
        };
        let fps = if self.core.time.delta_time > 0.0 {
            1.0 / effective_delta
        } else {
            0.0
        };

        self.sync_gizmo_orientation();
        if let Some(new_msaa) = self.render.render_state.apply_settings_changes() {
            self.gizmos.rebuild_for_msaa(
                &self.render.render_state.device,
                self.render.render_state.config.format,
                new_msaa,
                &self.editor,
            );
            self.render.rebuild_sprite_2d();
        }

        let empty_set = std::collections::HashSet::new();
        let (selected_set, active_ent) = if self.window.mode == EngineMode::Edit {
            (
                &self.editor.selected_entities_set,
                self.editor.primary_selected(),
            )
        } else {
            (&empty_set, None)
        };
        let prepared = self.render.prepare_scene(
            &mut self.core.world,
            &mut self.core.resources,
            &self.core.event_bus,
            selected_set,
            active_ent,
        );
        let is_2d = prepared.is_2d;
        let render_enabled = prepared.render_enabled;

        let has_gizmo_overlay = self.prepare_edit_gizmo_overlay(render_enabled, is_2d);
        if render_enabled && self.window.mode == EngineMode::Edit {
            self.gizmos.debug.collect_lines(
                &self.render.render_state.device,
                &self.render.render_state.queue,
                &self.core.world,
                &self.render.asset_manager,
                self.render.camera.build_view_projection_matrix(),
                &self.editor.selected_entities[..],
            );
            if !is_2d {
                let vp_w = self.ui.last_viewport_rect.width.max(1.0);
                let vp_h = self.ui.last_viewport_rect.height.max(1.0);
                self.gizmos
                    .billboards
                    .prepare_overlay(ae_gizmos_render::BillboardPrepareParams {
                        queue: &self.render.render_state.queue,
                        device: &self.render.render_state.device,
                        world: &self.core.world,
                        view_proj: self.render.camera.build_view_projection_matrix(),
                        viewport_size: [vp_w, vp_h],
                        selected_entities: &self.editor.selected_entities[..],
                        hovered_entity: None,
                    });
            }
        }

        self.sync_ui_telemetry();

        let sprite_overlay = ae_app::prepare_2d_overlay(
            self.render.sprite_2d.as_mut(),
            &self.render.render_state,
            &self.core.world,
            &self.render.camera,
            &self.render.asset_manager,
            render_enabled,
            is_2d,
        );

        let mut overlays: Vec<&dyn ae_renderer::render::OverlayRenderer> = Vec::new();
        if let Some(ref so) = sprite_overlay {
            overlays.push(so);
        }
        if render_enabled && self.window.mode == EngineMode::Edit {
            if has_gizmo_overlay {
                overlays.push(&self.gizmos.renderer);
            }
            if !is_2d {
                overlays.push(&self.gizmos.billboards);
            }
            overlays.push(&self.gizmos.debug);
        }

        let render_options = ae_renderer::render::RenderOptions {
            grid_enabled: self.editor.grid_enabled
                && self.window.mode == EngineMode::Edit
                && !is_2d,
            grid_scale: self.editor.snapping.grid_size,
            wireframe_enabled: self.editor.wireframe_enabled,
            scale_factor: self.ui.scale_factor(),
            is_2d_mode: is_2d,
        };

        let mut ui_actions = Vec::new();
        let ui = &mut self.ui;
        let window_ref = &self.window.window;
        let world = &self.core.world;
        let mode = &self.window.mode;
        let editor = &self.editor;
        let camera = &self.render.camera;
        let graphics_settings = self.render.render_state.graphics_settings;
        let snapping = &self.editor.snapping;
        let asset_manager = &self.render.asset_manager;
        let event_bus = &self.core.event_bus;
        #[cfg(feature = "mcp")]
        let mcp = &mut self.window.mcp;

        let mut ui_render_callback =
            |device: &wgpu::Device,
             queue: &wgpu::Queue,
             encoder: &mut wgpu::CommandEncoder,
             surface_view: &wgpu::TextureView,
             viewport_texture_view: Option<&wgpu::TextureView>,
             surface_size: (u32, u32)| {
                let prev_scale = ui.scale_factor();
                ui.on_window_resized(surface_size.0, surface_size.1);
                let viewport_rect = ui.render(ae_editor_ui::ui::EditorUiRenderParams {
                    device,
                    queue,
                    encoder,
                    window: window_ref,
                    window_surface_view: surface_view,
                    surface_size,
                    viewport_texture_view,
                    fps,
                    world,
                    mode,
                    undo_stack: &editor.undo_stack,
                    redo_stack: &editor.redo_stack,
                    graphics_settings: &graphics_settings,
                    snapping,
                    editor_state: editor,
                    camera,
                    models: &asset_manager.models,
                    textures: &asset_manager.textures,
                    shaders: &asset_manager.shaders,
                    enabled_modules: &event_bus.enabled_modules,
                    is_2d_mode: is_2d,
                    ui_actions: &mut ui_actions,
                });
                if (ui.scale_factor() - prev_scale).abs() > f32::EPSILON {
                    ui.on_window_resized(surface_size.0, surface_size.1);
                }
                #[cfg(feature = "mcp")]
                if let Some(session) = mcp.as_mut() {
                    session.render_overlay(
                        device,
                        queue,
                        encoder,
                        surface_view,
                        surface_size,
                        ui.scale_factor(),
                    );
                }
                viewport_rect
            };

        let params = ae_renderer::render::RenderFrameParams {
            scene: prepared.scene,
            camera,
            overlays: &overlays,
            editor_overlays: &[],
            asset_manager,
            enabled_modules: &event_bus.enabled_modules,
            options: &render_options,
            ui_renderer: Some(&mut ui_render_callback),
        };

        let res = self.render.render_state.render(params);
        drop(overlays);
        self.render.sync_camera_aspect_to_viewport();

        if let Err(e) = res {
            self.window.profiler.end_render();
            return Err(e);
        }

        self.process_ui_actions(ui_actions);
        self.sync_play_cursor_grab();
        self.window.profiler.end_render();
        Ok(())
    }

    fn sync_gizmo_orientation(&mut self) {
        self.gizmos.transform.mode = self.editor.gizmo_mode;
        self.gizmos.transform.space = self.editor.gizmo_space;
        self.gizmos.transform.entity_rotation = self
            .editor
            .primary_selected()
            .and_then(|ent| {
                self.core
                    .world
                    .inspect::<ae_plugin_api::Rotation, _>(ent, |r| {
                        cgmath::Quaternion::new(r.w, r.x, r.y, r.z)
                    })
                    .ok()
            })
            .unwrap_or_else(|| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
    }

    fn prepare_edit_gizmo_overlay(&mut self, render_enabled: bool, is_2d: bool) -> bool {
        if !render_enabled || self.window.mode != EngineMode::Edit || is_2d {
            return false;
        }
        let Some(ent) = self.editor.primary_selected() else {
            return false;
        };
        let Some(pos) = ae_transform::world_translation_for_entity(&self.core.world, ent) else {
            return false;
        };

        let cam_pos = self.render.camera.position_vec3();
        let cam_f = self.render.camera.get_forward();
        let right_dir = self.render.camera.get_right();
        let up_dir = self.render.camera.get_up();
        let dist = (pos - cam_pos).dot(cam_f).abs().max(1e-6);
        let screen = self.gizmo_screen_params();

        self.gizmos.renderer.prepare_overlay(
            &mut self.gizmos.transform,
            ae_gizmos_render::GizmoOverlayPrepareParams {
                queue: &self.render.render_state.queue,
                gizmo_pos: pos,
                camera_distance: dist,
                view_proj: self.render.camera.build_view_projection_matrix(),
                screen: &screen,
                cam_right: right_dir,
                cam_up: up_dir,
                cam_forward: cam_f,
                cam_pos,
            },
        );
        true
    }

    fn sync_ui_telemetry(&mut self) {
        let present_wait_ms = self.render.render_state.last_present_wait_secs * 1000.0;
        let ui_pass_ms = self
            .render
            .render_state
            .last_recording_timings
            .ui_pass_record_ms;
        self.window.profiler.ui_time = ui_pass_ms;
        let (models_bytes, textures_bytes) = self.render.asset_manager.get_memory_usage();

        self.ui
            .update_telemetry(ae_editor_ui::ui::EditorTelemetrySnapshot {
                profiler_ecs_ms: self.window.profiler.ecs_time,
                profiler_physics_ms: self.window.profiler.physics_time,
                profiler_render_ms: (self.window.profiler.render_time
                    - present_wait_ms
                    - ui_pass_ms)
                    .max(0.0),
                profiler_present_ms: present_wait_ms,
                profiler_ui_ms: ui_pass_ms,
                profiler_frame_ms: self.window.profiler.total_frame_time,
                cpu_timings: self.window.profiler.get_cpu_sync_timings(present_wait_ms),
                pass_recording_timings: self.render.render_state.last_recording_timings,
                frame_pacing: self.window.profiler.frame_pacing,
                frame_pacing_stats: self.window.profiler.calculate_pacing_stats(),
                draw_call_stats: self.render.render_state.last_render_stats.to_breakdown(),
                vram_stats: self
                    .render
                    .render_state
                    .get_vram_breakdown(&self.render.asset_manager),
                memory_models_mb: models_bytes as f32 / (1024.0 * 1024.0),
                memory_textures_mb: textures_bytes as f32 / (1024.0 * 1024.0),
                render_draw_calls: self.render.render_state.last_render_stats.draw_calls,
                render_triangles: self.render.render_state.last_render_stats.triangles,
                render_vertices: self.render.render_state.last_render_stats.vertices,
            });
        if !self.ui.has_gpu_adapter_info() {
            self.ui.set_gpu_adapter_info(
                &self.render.render_state.adapter_info.name,
                format!("{:?}", self.render.render_state.adapter_info.backend),
            );
        }
    }

    fn sync_play_cursor_grab(&mut self) {
        let should_grab = self.window.mode == EngineMode::Play
            && !self.sim.state_manager.is_paused()
            && !self.render.is_2d();
        if should_grab && !self.window.is_cursor_grabbed {
            self.set_cursor_grab(true);
        } else if !should_grab && self.window.is_cursor_grabbed {
            self.set_cursor_grab(false);
        }
    }
}
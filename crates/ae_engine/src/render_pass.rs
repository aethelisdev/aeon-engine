// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::engine::AeEngine;
use ae_core::modules::EngineMode;
use ae_light::{Light, LightUniform};
use ae_plugin_api::Position;
use cgmath::InnerSpace;

impl AeEngine {
    /// Extracts a snapshot of the current ECS state for the render pipeline.
    /// Passes the spatial grid reference to leverage high-performance culling.
    pub fn extract_render_scene(&self) -> ae_renderer::render::types::RenderScene {
        if self.dimension_mode == ae_2d::mode::ActiveDimensionMode::Mode2D {
            return ae_renderer::render::types::RenderScene {
                light_uniform: LightUniform::default(),
                triangle_instances: Vec::new(),
                cube_instances: Vec::new(),
                sphere_instances: Vec::new(),
                cylinder_instances: Vec::new(),
                capsule_instances: Vec::new(),
                torus_instances: Vec::new(),
                transparent_objs: Vec::new(),
                model_instance_data: std::collections::HashMap::new(),
                selected_primitive_instances: Vec::new(),
                selected_model_instances: Vec::new(),
                visible_entities: Vec::new(),
                culled_entities_count: 0,
            };
        }

        let empty_set = std::collections::HashSet::new();
        let (selected_set, active_ent) = if self.mode == EngineMode::Edit {
            (
                &self.editor.selected_entities_set,
                self.editor.primary_selected(),
            )
        } else {
            (&empty_set, None)
        };
        ae_renderer::render::types::RenderScene::extract(
            &self.world,
            &self.camera,
            &self.asset_manager,
            selected_set,
            active_ent,
            &self.spatial_grid,
        )
    }

    /// Performs the full render pass: lazy-syncs spatial grid, updates hierarchies,
    /// applies graphics settings, prepares overlays (gizmo + debug wireframe),
    /// extracts the visible scene utilizing the spatial grid, renders, processes UI actions,
    /// and handles Play→Edit scene restore.
    pub fn render(&mut self) -> Result<(), ae_renderer::render::RenderError> {
        self.profiler.begin_render();
        let is_2d = self.dimension_mode.is_2d();
        let render_enabled = if is_2d {
            self.event_bus
                .is_module_enabled(ae_core::modules::EngineModule::Render2D)
        } else {
            self.event_bus
                .is_module_enabled(ae_core::modules::EngineModule::Render)
        };
        if render_enabled && !is_2d {
            self.spatial_grid.sync(&self.world); // Lazy sync SpatialGrid if entity count changed
        }
        ae_transform::update_hierarchy_transforms(&mut self.world);
        // Compute FPS by subtracting VSync presentation wait from raw delta time.
        // This is only applied in Uncapped mode to reveal the true compute throughput of the engine,
        // without distorting the frame pacing of limited modes (which are capped at 60/120 FPS).
        let effective_delta = if self.render_state.graphics_settings.fps_limit
            == ae_renderer::graphics_settings::FpsLimit::Uncapped
        {
            (self.time.delta_time - self.render_state.last_present_wait_secs).max(0.0001)
        } else {
            self.time.delta_time.max(0.0001)
        };
        let fps = if self.time.delta_time > 0.0 {
            1.0 / effective_delta
        } else {
            0.0
        };
        if render_enabled && !is_2d {
            for (pos, light) in self.world.query_mut::<(&Position, &mut Light)>() {
                light.position = [pos.x, pos.y, pos.z];
            }
        }

        self.transform_gizmo.mode = self.editor.gizmo_mode;
        self.transform_gizmo.space = self.editor.gizmo_space;

        // Update entity rotation for Local space gizmo orientation
        if let Some(ent) = self.editor.primary_selected() {
            self.transform_gizmo.entity_rotation = self
                .world
                .inspect::<ae_plugin_api::Rotation, _>(ent, |r| {
                    cgmath::Quaternion::new(r.w, r.x, r.y, r.z)
                })
                .unwrap_or_else(|_| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
        } else {
            self.transform_gizmo.entity_rotation = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
        }

        // Apply graphics settings changes BEFORE rendering.
        // If MSAA changed, rebuild the gizmo renderer with the new sample count.
        if let Some(new_msaa) = self.render_state.apply_settings_changes() {
            self.gizmo_renderer = ae_gizmos_render::GizmoRenderer::new(
                &self.render_state.device,
                self.render_state.config.format,
                new_msaa,
            );
            self.transform_gizmo.mode = self.editor.gizmo_mode;
            self.transform_gizmo.space = self.editor.gizmo_space;
            self.debug_renderer.rebuild_pipeline(
                &self.render_state.device,
                self.render_state.config.format,
                new_msaa,
            );
            self.billboard_system.rebuild_pipeline(
                &self.render_state.device,
                self.render_state.config.format,
                new_msaa,
            );
            if let Some(ref mut sprite_sys) = self.sprite_2d_system {
                sprite_sys.rebuild(
                    &self.render_state.device,
                    &self.render_state.queue,
                    self.render_state.config.format,
                );
            }
        }

        // Prepare gizmo overlay: compute position, distance, write MVP uniform.
        // Then pass it as a generic OverlayRenderer trait object to render().
        // Gizmo is an editor tool — only active in 3D Edit mode.
        let overlay: Option<&dyn ae_renderer::render::OverlayRenderer> =
            if render_enabled && self.mode == EngineMode::Edit && !is_2d {
                if let Some(ent) = self.editor.primary_selected() {
                    let gizmo_pos = ae_transform::world_translation_for_entity(&self.world, ent);

                    if let Some(p) = gizmo_pos {
                        let cam_pos = cgmath::Vector3::new(
                            self.camera.position.x,
                            self.camera.position.y,
                            self.camera.position.z,
                        );
                        let cam_fwd = self.camera.get_forward();

                        let cam_f = cam_fwd.normalize();
                        let right_dir = cam_f.cross(cgmath::Vector3::unit_y());
                        let right_dir = if right_dir.magnitude2() < 0.001 {
                            cam_f.cross(cgmath::Vector3::unit_z()).normalize()
                        } else {
                            right_dir.normalize()
                        };
                        let up_dir = right_dir.cross(cam_f).normalize();

                        let dist = (p - cam_pos).dot(cam_fwd).abs().max(1e-6);
                        let screen = self.gizmo_screen_params();
                        self.gizmo_renderer.prepare_overlay(
                            &mut self.transform_gizmo,
                            ae_gizmos_render::GizmoOverlayPrepareParams {
                                queue: &self.render_state.queue,
                                gizmo_pos: p,
                                camera_distance: dist,
                                view_proj: self.camera.build_view_projection_matrix(),
                                screen: &screen,
                                cam_right: right_dir,
                                cam_up: up_dir,
                                cam_forward: cam_f,
                                cam_pos,
                            },
                        );
                        Some(&self.gizmo_renderer as &dyn ae_renderer::render::OverlayRenderer)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

        // Prepare debug wireframe overlay: collect collider lines when in Edit Mode.
        if render_enabled && self.mode == EngineMode::Edit {
            self.debug_renderer.collect_lines(
                &self.render_state.device,
                &self.render_state.queue,
                &self.world,
                &self.asset_manager,
                self.camera.build_view_projection_matrix(),
                &self.editor.selected_entities[..],
            );
        }

        let mut scene = if render_enabled {
            self.extract_render_scene()
        } else {
            ae_renderer::render::types::RenderScene {
                light_uniform: LightUniform::default(),
                triangle_instances: Vec::new(),
                cube_instances: Vec::new(),
                sphere_instances: Vec::new(),
                cylinder_instances: Vec::new(),
                capsule_instances: Vec::new(),
                torus_instances: Vec::new(),
                transparent_objs: Vec::new(),
                model_instance_data: std::collections::HashMap::new(),
                selected_primitive_instances: Vec::new(),
                selected_model_instances: Vec::new(),
                visible_entities: Vec::new(),
                culled_entities_count: 0,
            }
        };

        if render_enabled {
            self.resources.insert(ae_plugin_api::VisibleEntities {
                entities: std::mem::take(&mut scene.visible_entities),
            });
        }

        // Prepare 2D sprite batcher when in Mode2D
        let sprite_overlay = (render_enabled
            && self.dimension_mode == ae_2d::mode::ActiveDimensionMode::Mode2D)
            .then_some(())
            .and_then(|()| {
                self.sprite_2d_system.as_mut().map(|sprite_sys| {
                    sprite_sys.prepare_overlay(
                        &self.render_state.device,
                        &self.render_state.queue,
                        &self.world,
                        &self.camera,
                        &self.asset_manager.textures,
                    )
                })
            });

        // Build scene overlay list (Vec<&dyn OverlayRenderer>) for main forward pass (depth tested)
        let mut overlays: Vec<&dyn ae_renderer::render::OverlayRenderer> = Vec::new();
        // Build editor overlay list (Vec<&dyn OverlayRenderer>) for post-tonemap pass (unlit, pure sRGB)
        let editor_overlays: Vec<&dyn ae_renderer::render::OverlayRenderer> = Vec::new();

        if let Some(so) = &sprite_overlay {
            overlays.push(so);
        }
        if render_enabled && self.mode == EngineMode::Edit {
            if let Some(ov) = overlay {
                overlays.push(ov);
            }
            if !is_2d {
                let vp_w = self.ui.last_viewport_rect.width.max(1.0);
                let vp_h = self.ui.last_viewport_rect.height.max(1.0);
                self.billboard_system
                    .prepare_overlay(ae_gizmos_render::BillboardPrepareParams {
                        queue: &self.render_state.queue,
                        device: &self.render_state.device,
                        world: &self.world,
                        view_proj: self.camera.build_view_projection_matrix(),
                        viewport_size: [vp_w, vp_h],
                        selected_entities: &self.editor.selected_entities[..],
                        hovered_entity: None,
                    });
                overlays.push(&self.billboard_system);
            }
            overlays.push(&self.debug_renderer);
        }

        // Sync profiler snapshot to UI for display
        // Separate true GPU render work from VSync/swapchain present blocking.
        let present_wait_ms = self.render_state.last_present_wait_secs * 1000.0;
        let ui_pass_ms = self.render_state.last_recording_timings.ui_pass_record_ms;
        self.profiler.ui_time = ui_pass_ms;
        let (models_bytes, textures_bytes) = self.asset_manager.get_memory_usage();
        self.ui
            .update_telemetry(ae_editor_ui::ui::EditorTelemetrySnapshot {
                profiler_ecs_ms: self.profiler.ecs_time,
                profiler_physics_ms: self.profiler.physics_time,
                profiler_render_ms: (self.profiler.render_time - present_wait_ms - ui_pass_ms)
                    .max(0.0),
                profiler_present_ms: present_wait_ms,
                profiler_ui_ms: ui_pass_ms,
                profiler_frame_ms: self.profiler.total_frame_time,
                cpu_timings: self.profiler.get_cpu_sync_timings(present_wait_ms),
                pass_recording_timings: self.render_state.last_recording_timings,
                frame_pacing: self.profiler.frame_pacing,
                frame_pacing_stats: self.profiler.calculate_pacing_stats(),
                draw_call_stats: self.render_state.last_render_stats.to_breakdown(),
                vram_stats: self.render_state.get_vram_breakdown(&self.asset_manager),
                memory_models_mb: models_bytes as f32 / (1024.0 * 1024.0),
                memory_textures_mb: textures_bytes as f32 / (1024.0 * 1024.0),
                render_draw_calls: self.render_state.last_render_stats.draw_calls,
                render_triangles: self.render_state.last_render_stats.triangles,
                render_vertices: self.render_state.last_render_stats.vertices,
            });
        if !self.ui.has_gpu_adapter_info() {
            self.ui.set_gpu_adapter_info(
                &self.render_state.adapter_info.name,
                format!("{:?}", self.render_state.adapter_info.backend),
            );
        }

        let render_options = ae_renderer::render::RenderOptions {
            grid_enabled: self.editor.grid_enabled
                && self.mode == EngineMode::Edit
                && self.dimension_mode.is_3d(),
            grid_scale: self.editor.snapping.grid_size,
            wireframe_enabled: self.editor.wireframe_enabled,
            scale_factor: self.ui.scale_factor(),
            is_2d_mode: is_2d,
        };

        let mut ui_actions = Vec::new();

        // Split borrows to satisfy borrow checker in closure
        let ui = &mut self.ui;
        let world = &self.world;
        let mode = &self.mode;
        let editor = &self.editor;
        let camera = &self.camera;
        let graphics_settings = self.render_state.graphics_settings;
        let snapping = &self.editor.snapping;
        let asset_manager = &self.asset_manager;
        let event_bus = &self.event_bus;
        #[cfg(feature = "mcp")]
        let mcp = &mut self.mcp;

        let mut ui_render_callback =
            |device: &wgpu::Device,
             queue: &wgpu::Queue,
             encoder: &mut wgpu::CommandEncoder,
             window: &winit::window::Window,
             surface_view: &wgpu::TextureView,
             viewport_texture_view: Option<&wgpu::TextureView>,
             surface_size: (u32, u32)| {
                let prev_scale = ui.scale_factor();
                ui.on_window_resized(surface_size.0, surface_size.1);
                let viewport_rect = ui.render(ae_editor_ui::ui::EditorUiRenderParams {
                    device,
                    queue,
                    encoder,
                    window,
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
                    is_2d_mode: self.dimension_mode.is_2d(),
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
            scene,
            camera,
            overlays: &overlays,
            editor_overlays: &editor_overlays,
            asset_manager,
            enabled_modules: &event_bus.enabled_modules,
            options: &render_options,
            ui_renderer: Some(&mut ui_render_callback),
        };

        let res = self.render_state.render(params);

        let vp_rect = self.render_state.last_viewport_rect;
        let vp_w = vp_rect.max_x - vp_rect.min_x;
        let vp_h = vp_rect.max_y - vp_rect.min_y;
        if vp_w > 0.0 && vp_h > 0.0 {
            self.camera.aspect = vp_w / vp_h;
        }

        if let Err(e) = res {
            self.profiler.end_render();
            return Err(e);
        }

        self.process_ui_actions(ui_actions);

        // Declarative Cursor Sync: Ensure cursor grab state matches current EngineMode and Pause state
        let should_grab = self.mode == EngineMode::Play
            && !self.state_manager.is_paused()
            && !self.dimension_mode.is_2d();
        if should_grab && !self.is_cursor_grabbed {
            self.set_cursor_grab(true);
        } else if !should_grab && self.is_cursor_grabbed {
            self.set_cursor_grab(false);
        }

        self.profiler.end_render();
        Ok(())
    }
}
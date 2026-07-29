// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::engine::AeEngine;
use ae_core::ecs::{Light, Position, Velocity};
use ae_core::modules::EngineMode;
use ae_editor::undo_redo::EntitySnapshot;
use cgmath::InnerSpace;

impl AeEngine {
    /// Extracts a snapshot of the current ECS state for the render pipeline.
    /// Passes the spatial grid reference to leverage high-performance culling.
    pub fn extract_render_scene(&self) -> ae_renderer::render::types::RenderScene {
        ae_renderer::render::types::RenderScene::extract(
            &self.ecs.world,
            &self.camera,
            &self.asset_manager,
            &self.editor.selected_entities_set,
            &self.spatial_grid,
        )
    }

    /// Performs the full render pass: lazy-syncs spatial grid, updates hierarchies,
    /// applies graphics settings, prepares overlays (gizmo + debug wireframe),
    /// extracts the visible scene utilizing the spatial grid, renders, processes UI actions,
    /// and handles Play→Edit scene restore.
    pub fn render(&mut self) -> Result<(), ae_renderer::render::RenderError> {
        self.profiler.begin_render();
        let render_enabled = self
            .event_bus
            .is_module_enabled(ae_core::modules::EngineModule::Render);
        if render_enabled {
            self.spatial_grid.sync(&self.ecs.world); // Lazy sync SpatialGrid if entity count changed
        }
        ae_core::ecs::update_hierarchy_transforms(&mut self.ecs.world);
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
        if render_enabled {
            for (pos, light) in self.ecs.world.query_mut::<(&Position, &mut Light)>() {
                light.position = [pos.x, pos.y, pos.z];
            }
        }

        self.gizmo_system.mode = self.ui.gizmo_mode;
        self.gizmo_system.space = self.ui.gizmo_space;

        // Update entity rotation for Local space gizmo orientation
        if let Some(ent) = self.ui.selected_entity {
            if let Ok(r) = self.ecs.world.get::<&ae_core::ecs::Rotation>(ent) {
                self.gizmo_system.entity_rotation = cgmath::Quaternion::new(r.w, r.x, r.y, r.z);
            } else {
                self.gizmo_system.entity_rotation = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
            }
        } else {
            self.gizmo_system.entity_rotation = cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0);
        }

        // Apply graphics settings changes BEFORE rendering.
        // If MSAA changed, rebuild the gizmo system with the new sample count.
        if let Some(new_msaa) = self.render_state.apply_settings_changes() {
            self.gizmo_system = ae_editor::gizmo::GizmoSystem::new(
                &self.render_state.device,
                self.render_state.config.format,
                new_msaa,
            );
            self.gizmo_system.mode = self.ui.gizmo_mode;
            self.gizmo_system.space = self.ui.gizmo_space;
            self.debug_renderer.rebuild_pipeline(
                &self.render_state.device,
                self.render_state.config.format,
                new_msaa,
            );
        }

        // Prepare gizmo overlay: compute position, distance, write MVP uniform.
        // Then pass it as a generic OverlayRenderer trait object to render().
        // Gizmo is an editor tool — only active in Edit mode.
        let overlay: Option<&dyn ae_renderer::render::OverlayRenderer> =
            if render_enabled && self.mode == EngineMode::Edit {
                if let Some(ent) = self.ui.selected_entity {
                    let gizmo_pos =
                        if let Ok(gt) = self.ecs.world.get::<&ae_core::ecs::GlobalTransform>(ent) {
                            let mat = gt.0;
                            Some(cgmath::Vector3::new(mat.w.x, mat.w.y, mat.w.z))
                        } else if let Ok(pos) = self.ecs.world.get::<&ae_core::ecs::Position>(ent) {
                            Some(cgmath::Vector3::new(pos.x, pos.y, pos.z))
                        } else {
                            None
                        };

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
                        self.gizmo_system.prepare_overlay(
                            &self.render_state.queue,
                            p,
                            dist,
                            self.camera.build_view_projection_matrix(),
                            &screen,
                            right_dir,
                            up_dir,
                        );
                        Some(&self.gizmo_system as &dyn ae_renderer::render::OverlayRenderer)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

        // Prepare debug wireframe overlay: collect collider lines from ECS.
        if render_enabled {
            self.debug_renderer.collect_lines(
                &self.render_state.device,
                &self.render_state.queue,
                &self.ecs.world,
                &self.asset_manager,
                self.camera.build_view_projection_matrix(),
            );
        }

        // Build overlay list (Vec<&dyn OverlayRenderer>)
        let mut overlays: Vec<&dyn ae_renderer::render::OverlayRenderer> = Vec::new();
        if render_enabled {
            if overlay.is_some() {
                overlays.push(overlay.unwrap());
            }
            overlays.push(&self.debug_renderer);
        }

        let scene = if render_enabled {
            self.extract_render_scene()
        } else {
            ae_renderer::render::types::RenderScene {
                light_uniform: ae_renderer::render::types::LightUniform {
                    direction: [0.0, 1.0, 0.0],
                    _padding: 0,
                    color: [1.0, 1.0, 1.0],
                    _padding2: 0,
                    ambient_color: [0.1, 0.1, 0.15],
                    _padding3: 0,
                    fog_params: [0.0; 4],
                },
                triangle_instances: Vec::new(),
                cube_instances: Vec::new(),
                sphere_instances: Vec::new(),
                cylinder_instances: Vec::new(),
                capsule_instances: Vec::new(),
                torus_instances: Vec::new(),
                transparent_objs: Vec::new(),
                model_instance_data: std::collections::HashMap::new(),
                visible_entities: Vec::new(),
            }
        };

        if render_enabled {
            self.resources.insert(ae_plugin_api::VisibleEntities {
                entities: scene.visible_entities.clone(),
            });
        }

        // Sync profiler snapshot to UI for display
        self.ui.profiler_ecs_ms = self.profiler.ecs_time;
        // Separate true GPU render work from VSync/swapchain present blocking.
        let present_wait_ms = self.render_state.last_present_wait_secs * 1000.0;
        self.ui.profiler_render_ms = (self.profiler.render_time - present_wait_ms).max(0.0);
        self.ui.profiler_present_ms = present_wait_ms;
        self.ui.profiler_ui_ms = self.profiler.ui_time;
        self.ui.profiler_frame_ms = self.profiler.total_frame_time;
        let (models_bytes, textures_bytes) = self.asset_manager.get_memory_usage();
        self.ui.memory_models_mb = models_bytes as f32 / (1024.0 * 1024.0);
        self.ui.memory_textures_mb = textures_bytes as f32 / (1024.0 * 1024.0);

        let render_options = ae_renderer::render::RenderOptions {
            grid_enabled: self.ui.grid_enabled,
            wireframe_enabled: self.ui.wireframe_enabled,
        };

        let mut ui_actions = Vec::new();

        // Split borrows to satisfy borrow checker in closure
        let ui = &mut self.ui;
        let world = &self.ecs.world;
        let mode = &self.mode;
        let editor = &self.editor;
        let camera = &self.camera;
        let graphics_settings = self.render_state.graphics_settings.clone();
        let snapping = &self.editor.snapping;
        let asset_manager = &self.asset_manager;
        let event_bus = &self.event_bus;

        let res = self.render_state.render(
            world,
            scene,
            camera,
            &overlays,
            asset_manager,
            &event_bus.enabled_modules,
            &render_options,
            Some(
                &mut |device, queue, encoder, window, surface_view, viewport_texture_view| {
                    ui.sync_console();
                    let rect = ui.render(
                        device,
                        queue,
                        encoder,
                        window,
                        surface_view,
                        viewport_texture_view,
                        fps,
                        world,
                        mode,
                        &editor.undo_stack,
                        &editor.redo_stack,
                        camera.build_view_matrix(),
                        camera.build_projection_matrix(),
                        &graphics_settings,
                        snapping,
                        editor,
                        camera,
                        &asset_manager.models,
                        &asset_manager.textures,
                        &event_bus.enabled_modules,
                        &mut ui_actions,
                    );
                    ae_renderer::render::ViewportRect {
                        min_x: rect.min.x,
                        min_y: rect.min.y,
                        max_x: rect.max.x,
                        max_y: rect.max.y,
                    }
                },
            ),
        );

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

        // Declarative Cursor Sync: Ensure cursor grab state matches current EngineMode
        if self.mode == EngineMode::Edit && self.is_cursor_grabbed {
            self.set_cursor_grab(false);
        } else if self.mode == EngineMode::Play && !self.is_cursor_grabbed {
            self.set_cursor_grab(true);
        }

        if self.previous_mode != self.mode {
            self.previous_mode = self.mode;
            if self.mode == EngineMode::Play {
                self.set_cursor_grab(true);
                self.editor.camera_backup = Some(self.camera.clone());
                self.editor.scene_backup.clear();
                for ent_ref in self.ecs.world.iter() {
                    let ent = ent_ref.entity();
                    self.editor
                        .scene_backup
                        .insert(ent, EntitySnapshot::capture(&self.ecs.world, ent));
                }
                self.physics_world
                    .reset_simulation_poses(&mut self.ecs.world);
            } else {
                self.set_cursor_grab(false);
                if let Some(cam_backup) = self.editor.camera_backup.take() {
                    self.camera = cam_backup;
                }
                let entities: Vec<hecs::Entity> =
                    self.ecs.world.iter().map(|e| e.entity()).collect();
                for ent in entities {
                    if let Some(backup) = self.editor.scene_backup.get(&ent) {
                        backup.apply(&mut self.ecs.world, ent);
                        if let Ok(mut vel) = self.ecs.world.get::<&mut Velocity>(ent) {
                            vel.x = 0.0;
                            vel.y = 0.0;
                            vel.z = 0.0;
                        }
                        let _ = self.ecs.world.insert_one(ent, ae_core::ecs::TransformDirty);
                        let _ = self
                            .ecs
                            .world
                            .remove_one::<ae_core::ecs::GlobalTransform>(ent);
                    } else {
                        // Despawn newly created entities during play mode
                        let _ = self.ecs.world.despawn(ent);
                    }
                }

                // Re-spawn entities that were deleted during play mode
                for (old_ent, backup) in &self.editor.scene_backup {
                    if !self.ecs.world.contains(*old_ent) {
                        let new_ent = backup.spawn(&mut self.ecs.world);
                        let _ = self
                            .ecs
                            .world
                            .insert_one(new_ent, ae_core::ecs::TransformDirty);
                        let _ = self
                            .ecs
                            .world
                            .remove_one::<ae_core::ecs::GlobalTransform>(new_ent);
                    }
                }

                self.physics_world
                    .reset_simulation_poses(&mut self.ecs.world);
            }
        }
        self.profiler.end_render();
        Ok(())
    }
}
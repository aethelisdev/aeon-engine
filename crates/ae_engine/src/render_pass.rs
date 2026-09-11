// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::engine::AeEngine;
use ae_core::ecs::{Light, Position};
use ae_core::modules::EngineMode;
use cgmath::InnerSpace;

impl AeEngine {
    /// Extracts a snapshot of the current ECS state for the render pipeline.
    /// Passes the spatial grid reference to leverage high-performance culling.
    pub fn extract_render_scene(&self) -> ae_renderer::render::types::RenderScene {
        if self.dimension_mode == ae_2d::mode::ActiveDimensionMode::Mode2D {
            return ae_renderer::render::types::RenderScene {
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
                selected_primitive_instances: Vec::new(),
                selected_model_instances: Vec::new(),
                visible_entities: Vec::new(),
            };
        }

        let empty_set = std::collections::HashSet::new();
        let (selected_set, active_ent) = if self.mode == EngineMode::Edit {
            (&self.editor.selected_entities_set, self.ui.selected_entity)
        } else {
            (&empty_set, None)
        };
        ae_renderer::render::types::RenderScene::extract(
            &self.ecs.world,
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
        if render_enabled && !is_2d {
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
            if let Some((ref mut pipeline, ref mut batcher)) = self.sprite_2d_system {
                let new_pipeline = ae_2d::renderer::Sprite2DPipeline::new(
                    &self.render_state.device,
                    self.render_state.config.format,
                    None,
                    1,
                );
                *batcher = ae_2d::renderer::SpriteBatcher::new(
                    &self.render_state.device,
                    &self.render_state.queue,
                    &new_pipeline,
                );
                *pipeline = new_pipeline;
            }
        }

        // Prepare gizmo overlay: compute position, distance, write MVP uniform.
        // Then pass it as a generic OverlayRenderer trait object to render().
        // Gizmo is an editor tool — only active in 3D Edit mode.
        let overlay: Option<&dyn ae_renderer::render::OverlayRenderer> =
            if render_enabled && self.mode == EngineMode::Edit && !is_2d {
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
                            ae_editor::gizmo::render::GizmoOverlayPrepareParams {
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

        // Prepare debug wireframe overlay: collect collider lines when in Edit Mode (3D only).
        if render_enabled && self.mode == EngineMode::Edit && !is_2d {
            self.debug_renderer.collect_lines(
                &self.render_state.device,
                &self.render_state.queue,
                &self.ecs.world,
                &self.asset_manager,
                self.camera.build_view_projection_matrix(),
                &self.editor.selected_entities[..],
            );
        }

        let mut scene = if render_enabled {
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
                selected_primitive_instances: Vec::new(),
                selected_model_instances: Vec::new(),
                visible_entities: Vec::new(),
            }
        };

        if render_enabled {
            self.resources.insert(ae_plugin_api::VisibleEntities {
                entities: std::mem::take(&mut scene.visible_entities),
            });
        }

        // Prepare 2D sprite batcher when in Mode2D
        let mut sprite_overlay: Option<Sprite2DOverlay<'_>> = None;
        if render_enabled
            && self.dimension_mode == ae_2d::mode::ActiveDimensionMode::Mode2D
            && let Some((pipeline, batcher)) = &mut self.sprite_2d_system
        {
            batcher.begin_frame();
            let cam_view_proj = self.camera.build_view_projection_matrix();
            batcher.update_camera(&self.render_state.queue, &cam_view_proj);

            for (pos, scale, sprite, hidden_opt) in self
                .ecs
                .world
                .query::<(
                    &Position,
                    &ae_core::ecs::Scale,
                    &ae_2d::components::SpriteRenderer,
                    Option<&ae_core::ecs::Hidden>,
                )>()
                .iter()
            {
                if hidden_opt.is_some() {
                    continue;
                }
                let tex_id = sprite
                    .texture
                    .map(|t| {
                        use slotmap::Key;
                        t.data().as_ffi() as u32
                    })
                    .unwrap_or(0);
                let key = ae_2d::components::SpriteSortKey::from_layer_order_texture(
                    sprite.sorting_layer,
                    sprite.order_in_layer,
                    tex_id,
                );
                let model =
                    cgmath::Matrix4::from_translation(cgmath::Vector3::new(pos.x, pos.y, pos.z))
                        * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, 1.0);
                let instance = ae_2d::renderer::SpriteInstance {
                    model_matrix: model.into(),
                    uv_rect: sprite.uv_rect,
                    tint: sprite.tint,
                };
                batcher.push_instance(key, instance);
            }

            batcher.finish_and_upload(&self.render_state.device, &self.render_state.queue);
            sprite_overlay = Some(Sprite2DOverlay {
                pipeline,
                batcher,
                textures: &self.asset_manager.textures,
            });
        }

        // Build overlay list (Vec<&dyn OverlayRenderer>)
        let mut overlays: Vec<&dyn ae_renderer::render::OverlayRenderer> = Vec::new();
        if let Some(so) = &sprite_overlay {
            overlays.push(so);
        }
        if render_enabled && self.mode == EngineMode::Edit && !is_2d {
            if let Some(ov) = overlay {
                overlays.push(ov);
            }
            overlays.push(&self.debug_renderer);
        }

        // Sync profiler snapshot to UI for display
        self.ui.profiler_ecs_ms = self.profiler.ecs_time;
        self.ui.profiler_physics_ms = self.profiler.physics_time;
        // Separate true GPU render work from VSync/swapchain present blocking.
        let present_wait_ms = self.render_state.last_present_wait_secs * 1000.0;
        self.ui.profiler_render_ms = (self.profiler.render_time - present_wait_ms).max(0.0);
        self.ui.profiler_present_ms = present_wait_ms;
        self.ui.profiler_ui_ms = self.profiler.ui_time;
        self.ui.profiler_frame_ms = self.profiler.total_frame_time;
        self.ui.cpu_timings = self.profiler.get_cpu_sync_timings(present_wait_ms);
        self.ui.gpu_pass_timings = self.render_state.last_gpu_pass_timings;
        self.ui.frame_pacing = self.profiler.frame_pacing;
        self.ui.frame_pacing_stats = self.profiler.calculate_pacing_stats();
        self.ui.draw_call_stats = self.render_state.last_render_stats.to_breakdown();
        self.ui.vram_stats = self.render_state.get_vram_breakdown(&self.asset_manager);

        let (models_bytes, textures_bytes) = self.asset_manager.get_memory_usage();
        self.ui.memory_models_mb = models_bytes as f32 / (1024.0 * 1024.0);
        self.ui.memory_textures_mb = textures_bytes as f32 / (1024.0 * 1024.0);
        self.ui.render_draw_calls = self.render_state.last_render_stats.draw_calls;
        self.ui.render_triangles = self.render_state.last_render_stats.triangles;
        self.ui.render_vertices = self.render_state.last_render_stats.vertices;
        if self.ui.gpu_adapter_name.is_empty() {
            self.ui.gpu_adapter_name = self.render_state.adapter_info.name.clone();
            self.ui.gpu_backend = format!("{:?}", self.render_state.adapter_info.backend);
        }

        let render_options = ae_renderer::render::RenderOptions {
            grid_enabled: self.ui.grid_enabled
                && self.mode == EngineMode::Edit
                && self.dimension_mode.is_3d(),
            wireframe_enabled: self.ui.wireframe_enabled,
            scale_factor: self.ui.scale_factor(),
            is_2d_mode: is_2d,
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

        let mut ui_render_callback =
            |device: &wgpu::Device,
             queue: &wgpu::Queue,
             encoder: &mut wgpu::CommandEncoder,
             window: &winit::window::Window,
             surface_view: &wgpu::TextureView,
             viewport_texture_view: Option<&wgpu::TextureView>| {
                ui.sync_console();
                ui.render(ae_editor_ui::ui::EditorUiRenderParams {
                    device,
                    queue,
                    encoder,
                    window,
                    window_surface_view: surface_view,
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
                })
            };

        let params = ae_renderer::render::RenderFrameParams {
            scene,
            camera,
            overlays: &overlays,
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

/// Viewport overlay adapter that renders batched 2D sprites during the main viewport pass.
struct Sprite2DOverlay<'a> {
    pipeline: &'a ae_2d::renderer::Sprite2DPipeline,
    batcher: &'a ae_2d::renderer::SpriteBatcher,
    textures: &'a ae_renderer::asset::AssetStorage<ae_renderer::render::TextureAsset>,
}

impl<'a> ae_renderer::render::OverlayRenderer for Sprite2DOverlay<'a> {
    fn draw_overlay<'b>(&'b self, _queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'b>) {
        self.batcher.render(pass, self.pipeline, |id| {
            let handle =
                ae_renderer::asset::AssetHandle::from(slotmap::KeyData::from_ffi(id as u64));
            self.textures.get(handle).map(|t| &t.bind_group)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that 3D-specific overlays (debug line renderer and 3D gizmo)
    /// are strictly excluded when the active viewport dimension mode is 2D,
    /// preventing WGPU depth-stencil attachment validation panics.
    #[test]
    fn test_2d_mode_excludes_3d_debug_overlay() {
        let is_2d = true;
        let render_enabled = true;
        let mode = EngineMode::Edit;

        let should_collect_debug_lines = render_enabled && mode == EngineMode::Edit && !is_2d;
        let should_include_3d_overlays = render_enabled && mode == EngineMode::Edit && !is_2d;

        assert!(!should_collect_debug_lines);
        assert!(!should_include_3d_overlays);

        let is_3d_mode = false;
        let should_collect_in_3d = render_enabled && mode == EngineMode::Edit && !is_3d_mode;
        assert!(should_collect_in_3d);
    }
}
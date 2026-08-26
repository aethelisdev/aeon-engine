// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::engine::AeEngine;
use ae_core::modules::EngineMode;
use ae_editor::input::KeyCode;
use cgmath::Rotation3;

impl AeEngine {
    /// Core per-frame update: ticks time, processes async imports, purges stale entities,
    /// runs keyboard shortcuts, fixed-step physics, mode-specific logic, and plugin ticks.
    pub fn update(&mut self) {
        if self.render_state.size.width == 0 || self.render_state.size.height == 0 {
            return;
        }
        self.profiler.end_frame();
        self.profiler.begin_frame();
        self.time.tick();
        crate::importer::process_async_imports(self);

        // Track loaded texture files for live hot-reloading
        for path in self.asset_manager.texture_path_map.keys() {
            self.texture_watcher.track_file(path, None);
        }

        // Live Texture Hot-Reloading: Check disk modifications every 15 frames (~250ms)
        if self.time.frame_count > 0 && self.time.frame_count.is_multiple_of(15) {
            let modified_files = self.texture_watcher.check_modified_files();
            for path in modified_files {
                let path_str = path.to_string_lossy();
                if let Ok(cpu_data) =
                    ae_texture::parse_texture_file(&path_str, ae_texture::ColorSpace::Srgb)
                {
                    self.render_state.reload_cpu_texture_data(
                        &mut self.asset_manager,
                        &path,
                        cpu_data,
                    );
                }
            }
        }

        // Purge stale entity references (entities deleted by plugins, play mode, etc.)
        self.editor
            .selected_entities
            .retain(|e| self.ecs.world.contains(*e));

        // SINGLE SOURCE OF TRUTH: selected_entities → ui.selected_entity
        // ui.selected_entity is derived, never set directly elsewhere.
        self.ui.selected_entity = self.editor.selected_entities.first().copied();

        // Keyboard Shortcuts (Ctrl+C, Ctrl+V, Ctrl+O, F2, Ctrl+Z, Ctrl+Y, Ctrl+D, Delete, W/E/R/F)
        let shortcut_res = ae_editor::shortcuts::process_shortcuts(
            &self.input,
            &mut self.ecs.world,
            &mut self.editor,
            &mut self.ui.selected_entity,
            self.mode,
        );

        if shortcut_res.trigger_undo {
            self.undo();
        }
        if shortcut_res.trigger_redo {
            self.redo();
        }
        if shortcut_res.trigger_open_scene_dialog {
            let (tx, rx) = std::sync::mpsc::channel();
            self.ui.scene_dialog_receivers.push(rx);
            rayon::spawn(move || {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Aeon Scene", &["aee"])
                    .pick_file()
                {
                    let _ = tx.send(ae_editor_ui::ui::SceneDialogAction::LoadFrom(path));
                }
            });
        }
        if shortcut_res.trigger_save_scene_as
            || (shortcut_res.trigger_save_scene && self.editor.active_scene_path.is_none())
        {
            let (tx, rx) = std::sync::mpsc::channel();
            self.ui.scene_dialog_receivers.push(rx);
            rayon::spawn(move || {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Aeon Scene", &["aee"])
                    .set_file_name("scene.aee")
                    .save_file()
                {
                    let _ = tx.send(ae_editor_ui::ui::SceneDialogAction::SaveTo(path));
                }
            });
        } else if shortcut_res.trigger_save_scene
            && let Some(path) = self.editor.active_scene_path.as_ref()
        {
            self.process_ui_actions(vec![ae_editor_ui::ui::EngineUiAction::SaveSceneToPath(
                path.clone(),
            )]);
        }
        if shortcut_res.trigger_focus_selected {
            self.focus_selected();
        }
        if let Some(gm) = shortcut_res.new_gizmo_mode {
            self.ui.gizmo_mode = gm;
        }

        if self.mode == EngineMode::Play {
            // ESC key releases mouse lock and returns to Edit mode
            if self.input.is_key_just_pressed(KeyCode::Escape) {
                self.process_ui_actions(vec![ae_editor_ui::ui::EngineUiAction::ChangeMode(
                    EngineMode::Edit,
                )]);
                self.set_cursor_grab(false);
            }
        }

        // Sync gizmo space + entity rotation early so input handlers use correct orientation
        self.gizmo_system.mode = self.ui.gizmo_mode;
        self.gizmo_system.space = self.ui.gizmo_space;
        if let Some(ent) = self.ui.selected_entity
            && let Ok(r) = self.ecs.world.get::<&ae_core::ecs::Rotation>(ent)
        {
            self.gizmo_system.entity_rotation = cgmath::Quaternion::new(r.w, r.x, r.y, r.z);
        }

        self.profiler.begin_ecs();
        // Fixed Update Physics Loop
        let physics_enabled = self
            .event_bus
            .is_module_enabled(ae_core::modules::EngineModule::Physics);
        self.profiler.begin_physics();
        if physics_enabled {
            while self.time.consume_fixed_step() {
                if self.mode == EngineMode::Play {
                    let is_paused = self.state_manager.is_paused();
                    if !is_paused {
                        self.fixed_update_play_mode();
                        self.physics_world.step(
                            &mut self.ecs.world,
                            |handle| {
                                self.asset_manager
                                    .get_physics_mesh_data(handle)
                                    .map(|(v, i)| (v.as_slice(), i.as_slice()))
                            },
                            self.time.fixed_time_step,
                            &mut self.event_bus,
                        );
                    }
                }
            }
        } else {
            // Drain the accumulated time so it doesn't pile up when physics is disabled
            while self.time.consume_fixed_step() {}
        }

        // Ensure Rapier simulation is synced with editor spawns/moves/deletions.
        // Throttled by `physics_sync_dirty`: only runs when the scene has actually changed
        // (spawn, delete, inspector transform edit). Prevents O(N) ECS scans every frame
        // for scenes with 100K+ static entities.
        if self.mode == EngineMode::Edit && self.physics_sync_dirty {
            self.physics_sync_dirty = false;
            self.physics_world
                .sync_ecs_to_physics(&mut self.ecs.world, |handle| {
                    self.asset_manager
                        .get_physics_mesh_data(handle)
                        .map(|(v, i)| (v.as_slice(), i.as_slice()))
                });
        }
        self.profiler.end_physics();

        if self.mode == EngineMode::Play {
            self.update_play_mode();
        } else {
            self.update_edit_mode();
        }

        if self.editor.enable_live_editor_updates {
            // Live Update Path: hot-reload enabled, active file watches
            self.plugin_manager.hot_reload_enabled = true;
            self.plugin_manager.tick(
                &mut self.ecs.world,
                &mut self.resources,
                &mut self.event_bus,
                self.time.delta_time,
            );
        } else {
            // Fallback Path: Static runtime, zero hot-reload checks, maximum memory stability
            self.plugin_manager.hot_reload_enabled = false;
            self.plugin_manager.tick(
                &mut self.ecs.world,
                &mut self.resources,
                &mut self.event_bus,
                self.time.delta_time,
            );
        }

        // --- 3D SPATIAL AUDIO ENGINE UPDATE ---
        let is_audio_enabled = self
            .event_bus
            .is_module_enabled(ae_core::modules::EngineModule::Audio);
        let cam_pos = ae_audio::Vec3::new(
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
        );
        let right_vec = self.camera.get_right();
        let cam_right = ae_audio::Vec3::new(right_vec.x, right_vec.y, right_vec.z);
        self.audio_manager
            .update(&self.ecs.world, cam_pos, cam_right, is_audio_enabled);

        // --- SKELETAL ANIMATION PLAYER SYSTEM UPDATE ---
        let dt = self.time.delta_time;
        let mut candidate_entities = Vec::new();

        for (entity, player, model_id) in self
            .ecs
            .world
            .query_mut::<(
                hecs::Entity,
                &mut ae_animation::AnimationPlayer,
                &ae_core::ecs::ModelId,
            )>()
            .into_iter()
        {
            candidate_entities.push((entity, model_id.0, player.current_clip.is_none()));
        }

        for (entity, model_id, needs_clip) in candidate_entities {
            if self
                .ecs
                .world
                .get::<&ae_animation::Skeleton>(entity)
                .is_err()
                && let Some(asset) = self.asset_manager.models.get(model_id)
            {
                if let Some(ref skel) = asset.skeleton {
                    let _ = self.ecs.world.insert_one(entity, skel.clone());
                }
                if needs_clip
                    && !asset.animations.is_empty()
                    && let Ok(mut player) = self
                        .ecs
                        .world
                        .get::<&mut ae_animation::AnimationPlayer>(entity)
                {
                    player.play(asset.animations[0].clone());
                }
            }
        }

        let mut skinning_updates = Vec::new();

        for (_entity, player, skeleton, model_id) in self
            .ecs
            .world
            .query_mut::<(
                hecs::Entity,
                &mut ae_animation::AnimationPlayer,
                &ae_animation::Skeleton,
                &ae_core::ecs::ModelId,
            )>()
            .into_iter()
        {
            player.update(dt);
            if player.state == ae_animation::AnimationState::Playing {
                let local_poses = player.evaluate_pose(skeleton);
                let globals = skeleton.evaluate_global_transforms(&local_poses);
                let palette = ae_animation::compute_skinning_matrices(skeleton, &globals);
                skinning_updates.push((model_id.0, palette));
            }
        }

        for (model_handle, palette) in skinning_updates {
            self.render_state.update_model_skinning(
                &mut self.asset_manager,
                model_handle,
                &palette,
            );
        }

        self.profiler.end_ecs();
    }

    pub fn fixed_update_play_mode(&mut self) {
        if self.state_manager.is_paused() {
            return;
        }

        ae_editor::modes::fixed_update_play_mode(
            &self.input,
            &mut self.ecs,
            self.time.fixed_time_step,
        );

        // Fixed update for GameStateManager
        let mut state_cmd_buffer = ae_core::commands::EntityCommandBuffer::new();
        self.state_manager.fixed_update(
            &mut self.ecs.world,
            &mut self.event_bus,
            &mut state_cmd_buffer,
            self.time.fixed_time_step,
        );
        state_cmd_buffer.apply(&mut self.ecs.world);

        // Process Kinematic Character Controller (KCC) entities

        let dt = self.time.fixed_time_step;
        let mut kcc_entities = Vec::new();
        for (ent, _ctrl) in self
            .ecs
            .world
            .query::<(hecs::Entity, &ae_core::ecs::CharacterController)>()
            .iter()
        {
            kcc_entities.push(ent);
        }

        if !kcc_entities.is_empty() {
            use ae_physics::glam::Vec3;

            // Calculate camera-relative forward and right vectors on the XZ ground plane
            let fwd_vec = self.camera.get_forward();
            let right_vec = self.camera.get_right();
            let cam_fwd_xz = Vec3::new(fwd_vec.x, 0.0, fwd_vec.z).normalize();
            let cam_right_xz = Vec3::new(right_vec.x, 0.0, right_vec.z).normalize();

            let fwd_axis = self.input.get_axis("MoveForward");
            let right_axis = self.input.get_axis("MoveRight");

            let dir = cam_fwd_xz * fwd_axis + cam_right_xz * right_axis;
            let speed = if self.input.is_action_pressed("Run") {
                10.0f32
            } else {
                6.0f32
            };

            let move_dir = if dir.length_squared() > 0.001 {
                dir.normalize()
            } else {
                Vec3::ZERO
            };

            // Rotate character entity transform and physics body to face movement direction if moving
            if move_dir.length_squared() > 0.001 {
                let target_yaw = move_dir.x.atan2(move_dir.z);
                let rot_quat = cgmath::Quaternion::from_angle_y(cgmath::Rad(target_yaw));
                for &ent in &kcc_entities {
                    if let Ok(mut r) = self.ecs.world.get::<&mut ae_core::ecs::Rotation>(ent) {
                        r.x = rot_quat.v.x;
                        r.y = rot_quat.v.y;
                        r.z = rot_quat.v.z;
                        r.w = rot_quat.s;
                    }
                    if let Some(&handle) = self.physics_world.entity_to_body.get(&ent)
                        && let Some(body) = self.physics_world.rigid_body_set.get_mut(handle)
                    {
                        let rot_glam = ae_physics::glam::Quat::from_xyzw(
                            rot_quat.v.x,
                            rot_quat.v.y,
                            rot_quat.v.z,
                            rot_quat.s,
                        );
                        body.set_rotation(rot_glam, true);
                    }
                }
            }

            let jump_pressed = self.input.is_action_pressed("Jump");

            for ent in kcc_entities {
                let is_grounded = self
                    .ecs
                    .world
                    .get::<&ae_core::ecs::CharacterController>(ent)
                    .map(|c| c.is_grounded)
                    .unwrap_or(false);
                let mut vert_vel = self
                    .ecs
                    .world
                    .get::<&ae_core::ecs::Velocity>(ent)
                    .map(|v| v.y)
                    .unwrap_or(0.0);

                if is_grounded {
                    if jump_pressed {
                        vert_vel = 9.0; // Jump impulse velocity (1.8m high jump)
                        if let Ok(mut ctrl) = self
                            .ecs
                            .world
                            .get::<&mut ae_core::ecs::CharacterController>(ent)
                        {
                            ctrl.is_grounded = false;
                        }
                    } else {
                        vert_vel = 0.0; // Grounded: zero vertical velocity; snap_to_ground handles slopes/steps
                    }
                } else {
                    vert_vel -= 20.0 * dt; // Gravity
                }

                if let Ok(mut vel) = self.ecs.world.get::<&mut ae_core::ecs::Velocity>(ent) {
                    vel.x = 0.0;
                    vel.y = vert_vel;
                    vel.z = 0.0;
                }

                let translation = Vec3::new(
                    move_dir.x * speed * dt,
                    vert_vel * dt,
                    move_dir.z * speed * dt,
                );

                self.physics_world
                    .move_character(&mut self.ecs.world, ent, translation, dt);
            }
        }
    }

    pub fn update_play_mode(&mut self) {
        let was_paused = self.state_manager.is_paused();

        // 1. Tick stack-based GameStateManager
        let mut state_cmd_buffer = ae_core::commands::EntityCommandBuffer::new();
        self.state_manager.update(
            &mut self.ecs.world,
            &mut self.event_bus,
            &mut state_cmd_buffer,
            self.time.delta_time,
        );
        state_cmd_buffer.apply(&mut self.ecs.world);

        // 2. Toggle In-Game Pause overlay with 'P' key
        if self
            .input
            .is_key_just_pressed(ae_editor::input::KeyCode::KeyP)
        {
            if self.state_manager.is_paused() {
                self.state_manager.pop();
                let mut pop_cmd = ae_core::commands::EntityCommandBuffer::new();
                self.state_manager.update(
                    &mut self.ecs.world,
                    &mut self.event_bus,
                    &mut pop_cmd,
                    0.0,
                );
                pop_cmd.apply(&mut self.ecs.world);
            } else {
                self.state_manager.push(crate::hud::InGamePauseState::new());
                let mut push_cmd = ae_core::commands::EntityCommandBuffer::new();
                self.state_manager.update(
                    &mut self.ecs.world,
                    &mut self.event_bus,
                    &mut push_cmd,
                    0.0,
                );
                push_cmd.apply(&mut self.ecs.world);
            }
        }

        let is_paused = self.state_manager.is_paused();

        // Cursor grab synchronization with pause state
        if is_paused && !was_paused {
            self.set_cursor_grab(false);
        } else if !is_paused && was_paused {
            self.set_cursor_grab(true);
        }

        // 3. Tick In-Game HUD subsystem
        self.in_game_hud
            .update_from_events(&mut self.ecs.world, &mut self.event_bus);

        if is_paused {
            return;
        }

        ae_editor::modes::update_play_mode(&mut self.ecs, &mut self.camera, &mut self.editor);

        // Sync spatial grid in Play mode so moving player entities update their 3D cells and remain 100% visible
        self.spatial_grid.sync(&self.ecs.world);

        // Run gameplay behaviors (rotators, moving platforms, trigger zones, destructible targets, character actions)
        let cam_fwd = self.camera.get_forward();
        crate::behavior_runner::update_gameplay_behaviors(
            crate::behavior_runner::BehaviorRunnerParams {
                world: &mut self.ecs.world,
                physics_world: &mut self.physics_world,
                input: &self.input,
                event_bus: &mut self.event_bus,
                camera_forward: cam_fwd,
                delta_time: self.time.delta_time,
            },
        );
    }

    pub fn update_edit_mode(&mut self) {
        ae_editor::modes::update_edit_mode(
            &mut self.editor,
            &mut self.camera,
            &self.input,
            self.time.delta_time,
        );
    }

    /// Dispatches queued UI actions to the engine (within profiler UI timing).
    pub fn process_ui_actions(&mut self, actions: Vec<ae_editor_ui::ui::EngineUiAction>) {
        self.profiler.begin_ui();
        for action in &actions {
            if let ae_editor_ui::ui::EngineUiAction::ResumeGame = action
                && self.state_manager.is_paused()
            {
                self.state_manager.pop();
                let mut pop_cmd = ae_core::commands::EntityCommandBuffer::new();
                self.state_manager.update(
                    &mut self.ecs.world,
                    &mut self.event_bus,
                    &mut pop_cmd,
                    0.0,
                );
                pop_cmd.apply(&mut self.ecs.world);
                self.set_cursor_grab(true);
            }
        }

        let old_mode = self.previous_mode;
        let mut ctx = ae_editor_ui::ui_processor::UiContext {
            mode: &mut self.mode,
            world: &mut self.ecs.world,
            editor: &mut self.editor,
            ui: &mut self.ui,
            asset_manager: &mut self.asset_manager,
            camera: &mut self.camera,
            time: &mut self.time,
            event_bus: &mut self.event_bus,
            render_state: &mut self.render_state,
            dialog_receivers: &mut self.dialog_receivers,
        };
        ae_editor_ui::ui_processor::process_ui_actions(&mut ctx, actions);
        if self.mode != old_mode {
            self.in_game_hud.reset(&mut self.ecs.world);
            self.state_manager = ae_core::state::StateManager::new();
            self.previous_mode = self.mode;
            // Immediate physics world re-sync to reset dynamic Rapier simulation bodies & velocities
            self.physics_world
                .sync_ecs_to_physics(&mut self.ecs.world, |handle| {
                    self.asset_manager
                        .get_physics_mesh_data(handle)
                        .map(|(v, i)| (v.as_slice(), i.as_slice()))
                });
            self.physics_sync_dirty = false;
        }
        self.profiler.end_ui();
        // Any UI action (spawn, delete, transform edit) may have changed ECS state.
        // Mark physics dirty so sync_ecs_to_physics runs once on the next frame if needed.
        self.physics_sync_dirty = true;
    }

    pub fn undo(&mut self) {
        ae_editor::history::undo(&mut self.editor, &mut self.ecs.world);
        self.physics_sync_dirty = true;
    }

    pub fn redo(&mut self) {
        ae_editor::history::redo(&mut self.editor, &mut self.ecs.world);
        self.physics_sync_dirty = true;
    }

    pub fn focus_selected(&mut self) {
        ae_editor::interactions::focus_selected(&mut self.camera, &self.editor, &self.ecs.world);
    }
}
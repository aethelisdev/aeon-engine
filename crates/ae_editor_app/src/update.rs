// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Per-frame editor update loop delegating Play simulation to [`ae_app::SimulationHost`]
//! and handling Edit-mode shortcuts, texture hot reload, autosave, and project export polling.

use crate::session::EditorSession;
use ae_app::{PauseTransition, PlayFrameParams};
use ae_core::modules::EngineMode;
use ae_input::KeyCode;

impl EditorSession {
    /// Core per-frame update: advances time, polls async imports, checks hot-reloaded textures,
    /// runs keyboard shortcuts, steps fixed/variable simulation, and polls background exports.
    pub fn update(&mut self) {
        if self.render.render_state.size.width == 0 || self.render.render_state.size.height == 0 {
            return;
        }
        self.window.profiler.end_frame();
        self.window.profiler.begin_frame();
        self.core.time.tick();
        crate::actions::asset_import::poll_async_imports(self);

        self.poll_texture_hot_reload();

        let world_ref = &self.core.world;
        self.editor
            .selected_entities
            .retain(|e| world_ref.contains(*e));

        self.process_editor_shortcuts();

        self.window.profiler.begin_ecs();
        self.window.profiler.begin_physics();
        let is_play = self.window.mode == EngineMode::Play;
        self.sim
            .step_fixed_physics(&mut self.core, &self.render, is_play);
        if !is_play {
            self.sim
                .sync_dirty_physics(&mut self.core.world, &self.render.asset_manager);
        }
        self.window.profiler.end_physics();

        if is_play {
            self.step_editor_play_mode();
        } else {
            self.update_edit_mode();
            self.sim.plugin_manager.hot_reload_enabled = self.editor.enable_live_editor_updates;
            self.sim.plugin_manager.tick(
                &mut self.core.world,
                &mut self.core.resources,
                &mut self.core.event_bus,
                self.core.time.delta_time,
            );
            self.sim
                .update_audio(&self.core.world, &self.core.event_bus, &self.render.camera);
            ae_app::update_skeletal_animations(
                &mut self.core.world,
                &self.render.render_state,
                &mut self.render.asset_manager,
                self.core.time.delta_time,
            );
            self.poll_edit_autosave();
        }

        self.poll_async_export();
        self.window.profiler.end_ecs();
    }

    fn poll_texture_hot_reload(&mut self) {
        if self.core.time.frame_count == 0 || !self.core.time.frame_count.is_multiple_of(15) {
            return;
        }
        for path in self.render.asset_manager.texture_path_map.keys() {
            self.host_io.texture_watcher.track_file(path, None);
        }
        let modified_files = self.host_io.texture_watcher.check_modified_files();
        for path in modified_files {
            let path_str = path.to_string_lossy();
            if let Ok(cpu_data) =
                ae_texture::parse_texture_file(&path_str, ae_texture::ColorSpace::Srgb)
            {
                self.render.render_state.reload_cpu_texture_data(
                    &mut self.render.asset_manager,
                    &path,
                    cpu_data,
                );
            }
        }
    }

    fn step_editor_play_mode(&mut self) {
        let save_dir = std::path::PathBuf::from("Saved").join("SaveGames");
        let play_time_secs = self.core.time.total_time as u64;
        self.sim.plugin_manager.hot_reload_enabled = self.editor.enable_live_editor_updates;
        let pause_transition = self.sim.step_play_frame(
            &mut self.core,
            &mut self.render,
            &PlayFrameParams {
                mouse_sensitivity: self.editor.config.mouse_sensitivity,
                save_games_dir: &save_dir,
                cell_size: ae_scene::DEFAULT_WORLD_CELL_SIZE,
                play_time_secs,
                streaming_budget_per_frame: 4,
                tick_plugins: true,
            },
        );
        match pause_transition {
            PauseTransition::EnteredPause => self.set_cursor_grab(false),
            PauseTransition::ExitedPause if !self.render.is_2d() => self.set_cursor_grab(true),
            PauseTransition::ExitedPause | PauseTransition::Unchanged { .. } => {}
        }
    }

    fn poll_edit_autosave(&mut self) {
        if let Some(target_path) =
            self.host_io
                .autosave
                .poll_trigger(ae_scene::AutosavePollContext {
                    mode: self.window.mode,
                    enabled: self.editor.config.autosave_enabled,
                    interval_secs: self.editor.config.autosave_interval_secs,
                    dt: self.core.time.delta_time,
                    entity_count: self.core.world.len(),
                    autosaves_root: None,
                    saved_world_dir: self.host_io.saved_world_dir.as_deref(),
                    active_scene_path: self.editor.active_scene_path.as_deref(),
                })
            && let Ok(doc) = ae_scene::capture_world_document(
                &mut self.core.world,
                &self.render.asset_manager,
                &mut self.core.active_world_guid,
                self.render.is_2d(),
            )
        {
            self.host_io
                .autosave
                .dispatch_background_write(doc, target_path);
        }
    }

    fn process_editor_shortcuts(&mut self) {
        if self.ui.wants_keyboard_input() {
            return;
        }
        let shortcut_res = ae_editor::shortcuts::process_shortcuts(
            &self.core.input,
            &mut self.core.world,
            &mut self.editor,
            self.window.mode,
        );

        if shortcut_res.trigger_undo {
            self.undo();
        }
        if shortcut_res.trigger_redo {
            self.redo();
        }
        if shortcut_res.trigger_open_scene_dialog {
            crate::actions::scene_io::handle_open_load_scene_dialog(self);
        }
        if shortcut_res.trigger_save_scene_as
            || (shortcut_res.trigger_save_scene && self.editor.active_scene_path.is_none())
        {
            crate::actions::scene_io::handle_open_save_scene_dialog(self);
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
            self.editor.gizmo_mode = gm;
        }
    }

    /// Steps free-fly / orthographic Edit camera navigation.
    pub fn update_edit_mode(&mut self) {
        ae_camera_controller::step_free_camera(
            &mut self.render.camera,
            self.editor.config.camera_config(),
            ae_camera_controller::FreeCameraInput {
                forward: self.core.input.is_key_pressed(KeyCode::KeyW),
                backward: self.core.input.is_key_pressed(KeyCode::KeyS),
                left: self.core.input.is_key_pressed(KeyCode::KeyA),
                right: self.core.input.is_key_pressed(KeyCode::KeyD),
                up: self.core.input.is_key_pressed(KeyCode::KeyE),
                down: self.core.input.is_key_pressed(KeyCode::KeyQ),
                boost: self.core.input.is_key_pressed(KeyCode::ShiftLeft),
                right_mouse_pressed: self.editor.viewport_camera_look_active,
            },
            self.core.time.delta_time,
        );
        self.core.input.clear_mouse_delta();
    }

    /// Dispatches queued UI actions and applies physics/HUD resets across Edit ↔ Play transitions.
    pub fn process_ui_actions(&mut self, actions: Vec<ae_editor_ui::ui::EngineUiAction>) {
        if actions.is_empty() {
            return;
        }
        self.window.profiler.begin_ui();
        let old_mode = self.window.previous_mode;
        let physics_dirty = crate::actions::process_ui_actions(self, actions);
        if self.window.mode != old_mode {
            self.window.previous_mode = self.window.mode;
            self.sim
                .reset_for_mode_transition(&mut self.core.world, &self.render.asset_manager);
        } else if physics_dirty {
            self.sim.physics_sync_dirty = true;
        }
        self.window.profiler.end_ui();
    }

    /// Undoes the last editor command and marks physics for resync.
    pub fn undo(&mut self) {
        ae_editor::history::undo(&mut self.editor, &mut self.core.world);
        self.sim.physics_sync_dirty = true;
    }

    /// Redoes the last undone editor command and marks physics for resync.
    pub fn redo(&mut self) {
        ae_editor::history::redo(&mut self.editor, &mut self.core.world);
        self.sim.physics_sync_dirty = true;
    }

    /// Smoothly frames the primary selected entity inside the viewport camera.
    pub fn focus_selected(&mut self) {
        if let Some(&entity) = self.editor.selected_entities.first() {
            ae_camera_controller::focus_camera_on_entity(
                &mut self.render.camera,
                &self.core.world,
                entity,
                5.0,
            );
        }
    }

    fn poll_async_export(&mut self) {
        let Some(ref rx) = self.host_io.export_rx else {
            return;
        };
        let mut finished = false;
        while let Ok(evt) = rx.try_recv() {
            match evt {
                ae_export::ExportPipelineEvent::Progress(prog) => {
                    self.ui
                        .export_modal_mut()
                        .set_progress(prog.fraction, prog.message);
                }
                ae_export::ExportPipelineEvent::Finished(outcome) => {
                    self.ui.export_modal_mut().finish_success(
                        outcome.worlds_packed,
                        outcome.summary.total_assets,
                        &outcome.output_dir,
                    );
                    finished = true;
                }
                ae_export::ExportPipelineEvent::Error(err) => {
                    self.ui.export_modal_mut().finish_error(&err);
                    finished = true;
                }
            }
        }
        if finished {
            self.host_io.export_rx = None;
        }
    }
}
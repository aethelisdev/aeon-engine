// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Thin UI-action dispatcher wiring [`EngineUiAction`] onto [`ae_scene`], [`ae_editor`],
//! [`ae_camera`], [`ae_assets`], and OS dialog handlers.

use super::{asset_import, scene_io, spawning};
use crate::session::EditorSession;
use ae_core::modules::EngineMode;
use ae_editor_ui::ui::{EngineUiAction, PanelId};
use ae_scene::components as scene_comp;

fn push_property_undo<F>(
    editor: &mut ae_editor::editor_state::EditorState,
    ent: ae_ecs::Entity,
    changed: bool,
    build_prop: F,
) where
    F: FnOnce() -> ae_editor::undo_redo::Property,
{
    if changed {
        ae_editor::history::push_undo(
            editor,
            ae_editor::undo_redo::Command::Modify(ent, build_prop()),
        );
    }
}

fn change_mode(engine: &mut EditorSession, mode: EngineMode) {
    if engine.window.mode == mode {
        return;
    }
    if mode == EngineMode::Play {
        ae_editor::actions::begin_play_session(
            &mut engine.editor,
            &engine.core.world,
            &engine.render.camera,
        );
        engine.ui.prepare_for_play_mode();
    } else if engine.window.mode == EngineMode::Play && mode == EngineMode::Edit {
        ae_editor::actions::end_play_session(
            &mut engine.editor,
            &mut engine.core.world,
            &mut engine.render.camera,
        );
    }
    engine.window.mode = mode;
    log::info!("🎮 Engine mode changed to {:?}", mode);
}

fn dispatch_export_or_resume(engine: &mut EditorSession, action: &EngineUiAction) {
    if let EngineUiAction::StartProjectExport(req) = action
        && engine.host_io.export_rx.is_none()
    {
        let mut resolved_req = req.clone();
        if resolved_req.selected_worlds.is_empty() {
            if let Some(ref world_dir) = engine.host_io.saved_world_dir {
                let manifest = world_dir.join("world.aeworld");
                if manifest.is_file() {
                    resolved_req.selected_worlds.push(manifest);
                }
            } else if let Some(ref scene_path) = engine.editor.active_scene_path
                && scene_path.is_file()
            {
                resolved_req.selected_worlds.push(scene_path.clone());
            }
        }
        engine.ui.export_modal_mut().begin(&resolved_req.output_dir);
        engine.host_io.export_rx = Some(ae_export::spawn_export_pipeline(resolved_req));
    }
    if let EngineUiAction::ResumeGame = action
        && engine.sim.state_manager.is_paused()
    {
        engine.sim.state_manager.pop();
        let mut pop_cmd = ae_ecs::EntityCommandBuffer::new();
        engine.sim.state_manager.update(
            &mut engine.core.world,
            &mut engine.core.event_bus,
            &mut pop_cmd,
            0.0,
        );
        pop_cmd.apply(&mut engine.core.world);
        if !engine.render.dimension_mode.is_2d() {
            engine.set_cursor_grab(true);
        }
        engine.core.input.clear_mouse_delta();
    }
}

/// Executes a batch of [`EngineUiAction`] commands against `engine` and invalidates UI panels.
pub fn process_ui_actions(engine: &mut EditorSession, actions: Vec<EngineUiAction>) {
    let had_actions = !actions.is_empty();
    for action in actions {
        dispatch_export_or_resume(engine, &action);
        match action {
            // --- ENGINE MODE, EDITOR LAYOUT & HOST ACTIONS ---
            EngineUiAction::ChangeMode(m) => change_mode(engine, m),
            EngineUiAction::StartProjectExport(_) | EngineUiAction::ResumeGame => {}
            EngineUiAction::Undo => {
                ae_editor::history::undo(&mut engine.editor, &mut engine.core.world)
            }
            EngineUiAction::Redo => {
                ae_editor::history::redo(&mut engine.editor, &mut engine.core.world)
            }
            EngineUiAction::UndoBatch(b) => {
                ae_editor::actions::undo_batch(&mut engine.editor, &mut engine.core.world, b);
            }
            EngineUiAction::ToggleModule(module) => {
                let enabled = !engine.core.event_bus.is_module_enabled(module);
                engine.core.event_bus.set_module_enabled(module, enabled);
                log::info!("Module {:?} toggled to {}", module, enabled);
            }
            EngineUiAction::OpenPanel(panel) => engine.ui.open_or_focus_panel(panel),
            EngineUiAction::TogglePanel(panel) => engine.ui.toggle_panel(panel),
            EngineUiAction::ResetLayout => engine.ui.reset_dock_layout(),
            EngineUiAction::OpenPreferences => engine.ui.open_or_focus_panel(PanelId::Preferences),
            EngineUiAction::SetUiScale(scale) => engine.ui.ui_zoom_factor = scale.clamp(0.6, 2.0),
            EngineUiAction::SetThemePreset(preset) => engine.ui.apply_theme_preset(preset),
            EngineUiAction::SetWireframe(enabled) => {
                engine.editor.wireframe_enabled = enabled;
                engine.ui.tag_panel_redraw(PanelId::Viewport);
                engine.ui.tag_panel_redraw(PanelId::Stats);
            }
            EngineUiAction::SetGridEnabled(enabled) => {
                engine.editor.grid_enabled = enabled;
                engine.ui.tag_panel_redraw(PanelId::Viewport);
                engine.ui.tag_panel_redraw(PanelId::Stats);
            }
            EngineUiAction::SetGizmoMode(mode) => {
                engine.editor.gizmo_mode = mode;
                engine.ui.tag_panel_redraw(PanelId::Viewport);
            }
            EngineUiAction::ToggleGizmoSpace => {
                engine.editor.gizmo_space = engine.editor.gizmo_space.toggle();
                engine.ui.tag_panel_redraw(PanelId::Viewport);
            }
            EngineUiAction::PickAndAssignEntityTexture(ent) => {
                #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                    .pick_file()
                    .map(|p| p.to_string_lossy().into_owned())
                {
                    scene_comp::assign_texture_path(
                        &mut engine.core.world,
                        &engine.render.render_state,
                        &mut engine.render.asset_manager,
                        ent,
                        &path,
                    );
                }
                #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
                let _ = ent;
            }
            EngineUiAction::PickAndSetSubmeshTexture(model_id, idx) => {
                #[cfg(all(feature = "editor-dialogs", not(target_os = "android")))]
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Texture Image", &["png", "jpg", "jpeg", "tga", "bmp"])
                    .pick_file()
                    .map(|p| p.to_string_lossy().into_owned())
                {
                    scene_comp::set_model_submesh_texture_path(
                        &engine.render.render_state,
                        &mut engine.render.asset_manager,
                        model_id,
                        idx,
                        &path,
                    );
                }
                #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
                let _ = (model_id, idx);
            }
            EngineUiAction::GarbageCollect => {
                let (removed_models, removed_textures) = ae_scene::garbage_collect_unused_assets(
                    &engine.core.world,
                    &mut engine.render.asset_manager,
                );
                log::info!(
                    "🧹 Asset GC swept {removed_models} unused model(s) and {removed_textures} texture(s)."
                );
            }

            // --- ENTITY LIFECYCLE & SPAWNING ACTIONS ---
            EngineUiAction::SpawnModel(mid) => {
                spawning::handle_spawn_model_at(engine, mid, [0.0, 0.0, 0.0]);
            }
            EngineUiAction::SpawnModelAt(mid, pos) => {
                spawning::handle_spawn_model_at(engine, mid, pos);
            }
            EngineUiAction::SpawnModelPathAt(path, pos) => {
                if let Err(err) = spawning::handle_spawn_model_path_at(engine, &path, pos) {
                    engine.ui.set_status_message(
                        format!("Failed to load 3D model: {err}"),
                        irisui::prelude::Color::RED,
                    );
                }
            }
            EngineUiAction::SpawnSprite(tid) => {
                spawning::handle_spawn_sprite_at(engine, tid, [0.0, 0.0, 0.0]);
            }
            EngineUiAction::SpawnSpriteAt(tid, pos) => {
                spawning::handle_spawn_sprite_at(engine, tid, pos);
            }
            EngineUiAction::SpawnSpritePathAt(path, pos) => {
                spawning::handle_spawn_sprite_path_at(engine, &path, pos);
            }
            EngineUiAction::SpawnShape(shape) => spawning::handle_spawn_shape(engine, shape),
            EngineUiAction::SpawnDefaultSprite => spawning::handle_spawn_default_sprite(engine),
            EngineUiAction::SpawnPlayerSprite => spawning::handle_spawn_player_sprite(engine),
            EngineUiAction::SpawnEmpty2D => spawning::handle_spawn_empty_2d(engine),
            EngineUiAction::SpawnUiElement(ui_type) => {
                spawning::handle_spawn_ui_element(engine, ui_type);
            }
            EngineUiAction::SelectEntity(ent_opt) => engine.editor.select_single(ent_opt),
            EngineUiAction::DeleteSelected => {
                ae_editor::actions::delete_selected(&mut engine.core.world, &mut engine.editor);
            }
            EngineUiAction::StressTest(count) => spawning::handle_stress_test(engine, count),

            // --- PARENTING & HIERARCHY ACTIONS ---
            EngineUiAction::ParentEntity(child, parent) => {
                let _ = ae_transform::parent_entity_keep_world_transform(
                    &mut engine.core.world,
                    child,
                    parent,
                );
            }
            EngineUiAction::UnparentEntity(child) => {
                let _ = ae_transform::unparent_entity_keep_world_transform(
                    &mut engine.core.world,
                    child,
                );
            }
            EngineUiAction::ToggleVisibility(ent) => {
                scene_comp::toggle_visibility(&mut engine.core.world, ent);
            }

            // --- COMPONENT MUTATION ACTIONS ---
            EngineUiAction::LiveUpdatePosition(ent, pos) => {
                scene_comp::modify_position(&mut engine.core.world, ent, pos);
            }
            EngineUiAction::ModifyPosition(ent, old, pos) => {
                scene_comp::modify_position(&mut engine.core.world, ent, pos);
                push_property_undo(&mut engine.editor, ent, old != pos, || {
                    ae_editor::undo_redo::Property::Position(old, pos)
                });
            }
            EngineUiAction::LiveUpdateRotation(ent, rot) => {
                scene_comp::modify_rotation(&mut engine.core.world, ent, rot);
            }
            EngineUiAction::ModifyRotation(ent, old, rot) => {
                scene_comp::modify_rotation(&mut engine.core.world, ent, rot);
                push_property_undo(&mut engine.editor, ent, old != rot, || {
                    ae_editor::undo_redo::Property::Rotation(old, rot)
                });
            }
            EngineUiAction::LiveUpdateScale(ent, scale) => {
                scene_comp::modify_scale(&mut engine.core.world, ent, scale);
            }
            EngineUiAction::ModifyScale(ent, old, scale) => {
                scene_comp::modify_scale(&mut engine.core.world, ent, scale);
                push_property_undo(&mut engine.editor, ent, old != scale, || {
                    ae_editor::undo_redo::Property::Scale(old, scale)
                });
            }
            EngineUiAction::ModifyName(ent, old, new_name) => {
                scene_comp::modify_name(&mut engine.core.world, ent, new_name.clone());
                let changed = old != new_name;
                push_property_undo(&mut engine.editor, ent, changed, || {
                    ae_editor::undo_redo::Property::Name(old, new_name)
                });
            }
            EngineUiAction::ModifyColor(ent, old, color) => {
                scene_comp::modify_color(&mut engine.core.world, ent, color);
                push_property_undo(&mut engine.editor, ent, old != color, || {
                    ae_editor::undo_redo::Property::Color(old, color)
                });
            }
            EngineUiAction::ModifyLightColor(ent, old, color) => {
                scene_comp::modify_light_color(&mut engine.core.world, ent, color);
                if old != color {
                    let old_light = engine
                        .core
                        .world
                        .get_copied::<ae_light::Light>(ent)
                        .map(|l| ae_light::Light {
                            position: l.position,
                            color: old,
                        })
                        .unwrap_or(ae_light::Light {
                            position: [0.0, 0.0, 0.0],
                            color: old,
                        });
                    let new_light = engine
                        .core
                        .world
                        .get_copied::<ae_light::Light>(ent)
                        .unwrap_or(ae_light::Light {
                            position: [0.0, 0.0, 0.0],
                            color,
                        });
                    ae_editor::history::push_undo(
                        &mut engine.editor,
                        ae_editor::undo_redo::Command::Modify(
                            ent,
                            ae_editor::undo_redo::Property::Light(old_light, new_light),
                        ),
                    );
                }
            }
            EngineUiAction::AssignTextureToEntity(ent, path) => {
                scene_comp::assign_texture_path(
                    &mut engine.core.world,
                    &engine.render.render_state,
                    &mut engine.render.asset_manager,
                    ent,
                    &path,
                );
            }
            EngineUiAction::RemoveTextureFromEntity(ent) => {
                scene_comp::remove_texture(&mut engine.core.world, ent);
            }
            EngineUiAction::SetModelSubmeshAlphaMode(model_h, idx, mode) => {
                scene_comp::set_model_submesh_alpha_mode(
                    &mut engine.render.asset_manager,
                    model_h,
                    idx,
                    mode,
                );
            }
            EngineUiAction::SetModelSubmeshTexture(model_h, idx, path) => {
                scene_comp::set_model_submesh_texture_path(
                    &engine.render.render_state,
                    &mut engine.render.asset_manager,
                    model_h,
                    idx,
                    &path,
                );
            }
            EngineUiAction::AddComponent(ent, type_name) => {
                scene_comp::add_component(&mut engine.core.world, ent, type_name);
            }
            EngineUiAction::RemoveComponent(ent, type_name) => {
                scene_comp::remove_component(&mut engine.core.world, ent, type_name);
            }
            EngineUiAction::ModifyComponent(ent, type_name, data) => {
                scene_comp::modify_component(&mut engine.core.world, ent, type_name, &data);
            }
            EngineUiAction::CommitComponentModify(ent, type_name, old_data, new_data) => {
                scene_comp::modify_component(&mut engine.core.world, ent, type_name, &new_data);
                if old_data != new_data {
                    ae_editor::history::push_undo(
                        &mut engine.editor,
                        ae_editor::undo_redo::Command::Modify(
                            ent,
                            ae_editor::undo_redo::Property::Component {
                                type_name: type_name.to_string(),
                                old_data,
                                new_data,
                            },
                        ),
                    );
                    engine.editor.redo_stack.clear();
                }
            }
            EngineUiAction::ModifyLodThresholds(ent, t1, t2) => {
                scene_comp::modify_lod_thresholds(&mut engine.core.world, ent, t1, t2);
            }
            EngineUiAction::ModifyLodModel(ent, slot, handle_opt) => {
                scene_comp::modify_lod_model(&mut engine.core.world, ent, slot, handle_opt);
            }
            EngineUiAction::SpawnPhase1TestSandbox => {
                spawning::handle_spawn_phase1_test_sandbox(engine);
            }

            // --- CAMERA & SETTINGS ACTIONS ---
            EngineUiAction::SetCameraMode(mode) => engine.render.camera.mode = mode,
            EngineUiAction::ToggleCameraProjection => engine.render.camera.toggle_projection(),
            EngineUiAction::SetCameraTransform {
                pitch,
                yaw,
                position,
                mode,
            } => engine.render.camera.set_pose(pitch, yaw, position, mode),
            EngineUiAction::UpdateGraphicsSettings(settings) => {
                engine.render.render_state.graphics_settings = settings;
            }
            EngineUiAction::UpdateSnapSettings(snap) => engine.editor.snapping = snap,
            EngineUiAction::UpdateEditorConfig(cfg) => {
                engine.editor.config = cfg;
                engine
                    .core
                    .time
                    .set_fixed_hz(engine.editor.config.physics_hz);
            }
            EngineUiAction::SetLiveEditorUpdates(val) => {
                engine.editor.enable_live_editor_updates = val;
                let (msg, color) = if val {
                    (
                        "Live Editor Updates (Hot Reload) Enabled",
                        irisui::prelude::Color::rgb(70.0 / 255.0, 190.0 / 255.0, 120.0 / 255.0),
                    )
                } else {
                    (
                        "Live Editor Updates (Hot Reload) Disabled",
                        irisui::prelude::Color::rgb(220.0 / 255.0, 140.0 / 255.0, 60.0 / 255.0),
                    )
                };
                engine.ui.set_status_message(msg, color);
            }

            // --- SCENE, PREFAB & ASSET BROWSER I/O ACTIONS ---
            EngineUiAction::OpenModelDialog => asset_import::handle_open_model_dialog(engine),
            EngineUiAction::ImportDroppedFiles { paths, target } => {
                asset_import::handle_import_dropped_files(engine, paths, target);
            }
            EngineUiAction::OpenLoadPrefabDialog => {
                scene_io::handle_open_load_prefab_dialog(engine);
            }
            EngineUiAction::OpenSaveSceneDialog | EngineUiAction::SaveScene => {
                scene_io::handle_open_save_scene_dialog(engine);
            }
            EngineUiAction::OpenLoadSceneDialog | EngineUiAction::LoadScene => {
                scene_io::handle_open_load_scene_dialog(engine);
            }
            EngineUiAction::SaveSceneToPath(path) => {
                scene_io::handle_save_scene_to_path(engine, path);
            }
            EngineUiAction::LoadSceneFromPath(path) => {
                scene_io::handle_load_scene_from_path(engine, path);
            }
            EngineUiAction::SaveEntityAsPrefab(ent, path) => {
                scene_io::handle_save_entity_as_prefab(engine, ent, path);
            }
            EngineUiAction::InstantiatePrefab(path) => {
                scene_io::handle_instantiate_prefab(engine, path);
            }
            EngineUiAction::Exit => engine.editor.should_exit = true,
            EngineUiAction::DeleteAssetOrFolder(path) => {
                let _ = ae_assets::file_ops::delete_asset_or_folder(&path);
                engine.ui.refresh_asset_browser(Some(path.as_path()));
            }
            EngineUiAction::CreateSubfolder { parent, name } => {
                let _ = ae_assets::file_ops::create_subfolder(&parent, &name);
                engine.ui.refresh_asset_browser(None);
            }
            EngineUiAction::RenameAssetOrFolder { target, new_name } => {
                let _ = ae_assets::file_ops::rename_asset_or_folder(&target, &new_name);
                engine.ui.refresh_asset_browser(None);
            }

            _ => {}
        }
    }

    if had_actions {
        engine.ui.invalidate_all();
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Top-level [`EngineUiAction`] dispatcher routing UI intents directly onto [`AeEngine`].

use super::{components, scene_io, spawning, system};
use crate::engine::AeEngine;
use ae_editor_ui::ui::{EngineUiAction, PanelId};

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

fn dispatch_export_or_resume(engine: &mut AeEngine, action: &EngineUiAction) {
    if let EngineUiAction::StartProjectExport(req) = action
        && engine.export_rx.is_none()
    {
        let mut resolved_req = req.clone();
        if resolved_req.selected_worlds.is_empty() {
            if let Some(ref world_dir) = engine.saved_world_dir {
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
        engine.export_rx = Some(ae_export::spawn_export_pipeline(resolved_req));
    }
    if let EngineUiAction::ResumeGame = action
        && engine.state_manager.is_paused()
    {
        engine.state_manager.pop();
        let mut pop_cmd = ae_ecs::EntityCommandBuffer::new();
        engine
            .state_manager
            .update(&mut engine.world, &mut engine.event_bus, &mut pop_cmd, 0.0);
        pop_cmd.apply(&mut engine.world);
        if !engine.dimension_mode.is_2d() {
            engine.set_cursor_grab(true);
        }
        engine.input.clear_mouse_delta();
    }
}

/// Executes a batch of [`EngineUiAction`] commands against `engine` and marks the UI dirty when needed.
pub fn process_ui_actions(engine: &mut AeEngine, actions: Vec<EngineUiAction>) {
    let had_actions = !actions.is_empty();
    for action in actions {
        dispatch_export_or_resume(engine, &action);
        match action {
            // --- ENGINE MODE, EDITOR LAYOUT & HOST ACTIONS ---
            EngineUiAction::ChangeMode(m) => system::handle_change_mode(engine, m),
            EngineUiAction::StartProjectExport(_) | EngineUiAction::ResumeGame => {}
            EngineUiAction::Undo => system::handle_undo(engine),
            EngineUiAction::Redo => system::handle_redo(engine),
            EngineUiAction::UndoBatch(b) => system::handle_undo_batch(engine, b),
            EngineUiAction::ToggleModule(module) => system::handle_toggle_module(engine, module),
            EngineUiAction::OpenPanel(panel) => engine.ui.open_or_focus_panel(panel),
            EngineUiAction::TogglePanel(panel) => engine.ui.toggle_panel(panel),
            EngineUiAction::ResetLayout => engine.ui.reset_dock_layout(),
            EngineUiAction::OpenPreferences => engine.ui.open_or_focus_panel(PanelId::Preferences),
            EngineUiAction::SetUiScale(scale) => {
                engine.ui.ui_zoom_factor = scale.clamp(0.6, 2.0);
            }
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
                    components::handle_assign_texture(engine, ent, path);
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
                    components::handle_set_model_submesh_texture(engine, model_id, idx, path);
                }
                #[cfg(not(all(feature = "editor-dialogs", not(target_os = "android"))))]
                let _ = (model_id, idx);
            }
            EngineUiAction::GarbageCollect => system::handle_garbage_collect(engine),

            // --- ENTITY LIFECYCLE & SPAWNING ACTIONS ---
            EngineUiAction::SpawnModel(mid) => spawning::handle_spawn_model(engine, mid),
            EngineUiAction::SpawnModelAt(mid, pos) => {
                spawning::handle_spawn_model_at(engine, mid, pos);
            }
            EngineUiAction::SpawnModelPathAt(path, pos) => {
                if let Err(err) = spawning::handle_spawn_model_path_at(engine, path, pos) {
                    engine.ui.set_status_message(
                        format!("Failed to load 3D model: {err}"),
                        irisui::prelude::Color::RED,
                    );
                }
            }
            EngineUiAction::SpawnSprite(tid) => spawning::handle_spawn_sprite(engine, tid),
            EngineUiAction::SpawnSpriteAt(tid, pos) => {
                spawning::handle_spawn_sprite_at(engine, tid, pos);
            }
            EngineUiAction::SpawnSpritePathAt(path, pos) => {
                spawning::handle_spawn_sprite_path_at(engine, path, pos);
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
                ae_editor::actions::delete_selected(&mut engine.world, &mut engine.editor);
            }
            EngineUiAction::StressTest(count) => spawning::handle_stress_test(engine, count),

            // --- PARENTING & HIERARCHY ACTIONS ---
            EngineUiAction::ParentEntity(child, parent) => {
                let _ = ae_transform::parent_entity_keep_world_transform(
                    &mut engine.world,
                    child,
                    parent,
                );
            }
            EngineUiAction::UnparentEntity(child) => {
                let _ =
                    ae_transform::unparent_entity_keep_world_transform(&mut engine.world, child);
            }
            EngineUiAction::ToggleVisibility(ent) => {
                components::handle_toggle_visibility(&mut engine.world, ent);
            }

            // --- COMPONENT MUTATION ACTIONS ---
            EngineUiAction::LiveUpdatePosition(ent, pos) => {
                components::handle_modify_position(&mut engine.world, ent, pos);
            }
            EngineUiAction::ModifyPosition(ent, old, pos) => {
                components::handle_modify_position(&mut engine.world, ent, pos);
                push_property_undo(&mut engine.editor, ent, old != pos, || {
                    ae_editor::undo_redo::Property::Position(old, pos)
                });
            }
            EngineUiAction::LiveUpdateRotation(ent, rot) => {
                components::handle_modify_rotation(&mut engine.world, ent, rot);
            }
            EngineUiAction::ModifyRotation(ent, old, rot) => {
                components::handle_modify_rotation(&mut engine.world, ent, rot);
                push_property_undo(&mut engine.editor, ent, old != rot, || {
                    ae_editor::undo_redo::Property::Rotation(old, rot)
                });
            }
            EngineUiAction::LiveUpdateScale(ent, scale) => {
                components::handle_modify_scale(&mut engine.world, ent, scale);
            }
            EngineUiAction::ModifyScale(ent, old, scale) => {
                components::handle_modify_scale(&mut engine.world, ent, scale);
                push_property_undo(&mut engine.editor, ent, old != scale, || {
                    ae_editor::undo_redo::Property::Scale(old, scale)
                });
            }
            EngineUiAction::ModifyName(ent, old, new_name) => {
                components::handle_modify_name(&mut engine.world, ent, new_name.clone());
                let changed = old != new_name;
                push_property_undo(&mut engine.editor, ent, changed, || {
                    ae_editor::undo_redo::Property::Name(old, new_name)
                });
            }
            EngineUiAction::ModifyColor(ent, old, color) => {
                components::handle_modify_color(&mut engine.world, ent, color);
                push_property_undo(&mut engine.editor, ent, old != color, || {
                    ae_editor::undo_redo::Property::Color(old, color)
                });
            }
            EngineUiAction::ModifyLightColor(ent, old, color) => {
                components::handle_modify_light_color(&mut engine.world, ent, color);
                if old != color {
                    let old_light = engine
                        .world
                        .get_copied::<ae_plugin_api::Light>(ent)
                        .map(|l| ae_plugin_api::Light {
                            position: l.position,
                            color: old,
                        })
                        .unwrap_or(ae_plugin_api::Light {
                            position: [0.0, 0.0, 0.0],
                            color: old,
                        });
                    let new_light = engine
                        .world
                        .get_copied::<ae_plugin_api::Light>(ent)
                        .unwrap_or(ae_plugin_api::Light {
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
                components::handle_assign_texture(engine, ent, path);
            }
            EngineUiAction::RemoveTextureFromEntity(ent) => {
                components::handle_remove_texture(&mut engine.world, ent);
            }
            EngineUiAction::SetModelSubmeshAlphaMode(model_h, idx, mode) => {
                components::handle_set_model_submesh_alpha_mode(engine, model_h, idx, mode);
            }
            EngineUiAction::SetModelSubmeshTexture(model_h, idx, path) => {
                components::handle_set_model_submesh_texture(engine, model_h, idx, path);
            }
            EngineUiAction::AddComponent(ent, type_name) => {
                components::handle_add_component(&mut engine.world, ent, type_name);
            }
            EngineUiAction::RemoveComponent(ent, type_name) => {
                components::handle_remove_component(&mut engine.world, ent, type_name);
            }
            EngineUiAction::ModifyComponent(ent, type_name, data) => {
                components::handle_modify_component(&mut engine.world, ent, type_name, &data);
            }
            EngineUiAction::CommitComponentModify(ent, type_name, old_data, new_data) => {
                components::handle_modify_component(&mut engine.world, ent, type_name, &new_data);
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
                components::handle_modify_lod_thresholds(&mut engine.world, ent, t1, t2);
            }
            EngineUiAction::ModifyLodModel(ent, slot, handle_opt) => {
                components::handle_modify_lod_model(&mut engine.world, ent, slot, handle_opt);
            }
            EngineUiAction::SpawnPhase1TestSandbox => {
                spawning::handle_spawn_phase1_test_sandbox(engine);
            }

            // --- CAMERA & SETTINGS ACTIONS ---
            EngineUiAction::SetCameraMode(mode) => system::handle_set_camera_mode(engine, mode),
            EngineUiAction::ToggleCameraProjection => {
                system::handle_toggle_camera_projection(engine);
            }
            EngineUiAction::SetCameraTransform {
                pitch,
                yaw,
                position,
                mode,
            } => system::handle_set_camera_transform(engine, pitch, yaw, position, mode),
            EngineUiAction::UpdateGraphicsSettings(settings) => {
                system::handle_update_graphics_settings(engine, settings);
            }
            EngineUiAction::UpdateSnapSettings(snap) => {
                system::handle_update_snap_settings(engine, snap);
            }
            EngineUiAction::UpdateEditorConfig(cfg) => {
                system::handle_update_editor_config(engine, cfg);
            }
            EngineUiAction::SetLiveEditorUpdates(val) => {
                system::handle_set_live_editor_updates(engine, val);
            }

            // --- SCENE, PREFAB & ASSET BROWSER I/O ACTIONS ---
            EngineUiAction::OpenModelDialog => scene_io::handle_open_model_dialog(engine),
            EngineUiAction::ImportDroppedFiles { paths, target } => {
                scene_io::handle_import_dropped_files(engine, paths, target);
            }
            EngineUiAction::OpenLoadPrefabDialog => {
                scene_io::handle_open_load_prefab_dialog(engine);
            }
            EngineUiAction::OpenSaveSceneDialog => scene_io::handle_open_save_scene_dialog(engine),
            EngineUiAction::OpenLoadSceneDialog => scene_io::handle_open_load_scene_dialog(engine),
            EngineUiAction::SaveScene => scene_io::handle_open_save_scene_dialog(engine),
            EngineUiAction::LoadScene => scene_io::handle_open_load_scene_dialog(engine),
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
            EngineUiAction::Exit => {
                engine.editor.should_exit = true;
            }
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
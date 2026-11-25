// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! UI-action dispatcher wiring [`EngineUiAction`] onto [`ae_scene`], [`ae_editor`],
//! [`ae_camera`], [`ae_assets`], and OS dialog handlers.

use super::{asset_import, dialogs, scene_io, spawning};
use crate::session::EditorSession;
use ae_core::modules::EngineMode;
use ae_editor_ui::ui::{EngineUiAction, PanelId};
use ae_scene::components as scene_comp;
use std::path::Path;

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
        editor.redo_stack.clear();
    }
}

fn change_mode(engine: &mut EditorSession, mode: EngineMode) {
    if engine.window.mode == mode {
        return;
    }
    engine.ui.clear_interaction_history();
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

fn start_project_export(engine: &mut EditorSession, req: ae_export::ExportRequest) {
    if engine.host_io.export_rx.is_some() {
        return;
    }
    let mut resolved_req = req;
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

fn resume_paused_game(engine: &mut EditorSession) {
    if !engine.sim.state_manager.is_paused() {
        return;
    }
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

fn run_asset_file_op<T, E: std::fmt::Display>(
    engine: &mut EditorSession,
    label: &str,
    refresh_path: Option<&Path>,
    res: Result<T, E>,
) {
    if let Err(err) = res {
        log::warn!("{label} failed: {err}");
        engine
            .ui
            .set_status_message(format!("{label}: {err}"), irisui::prelude::Color::RED);
        return;
    }
    engine.ui.refresh_asset_browser(refresh_path);
}

/// Executes a batch of [`EngineUiAction`] commands against `engine`.
///
/// Returns `true` when at least one dispatched action modified ECS entities,
/// transforms, physics modules, or scene topology requiring Rapier resync.
pub fn process_ui_actions(engine: &mut EditorSession, actions: Vec<EngineUiAction>) -> bool {
    let mut physics_dirty = false;
    let mut full_ui_invalidate = false;
    for action in actions {
        let (p_dirty, ui_dirty) = dispatch_single_action(engine, action);
        physics_dirty |= p_dirty;
        full_ui_invalidate |= ui_dirty;
    }

    if full_ui_invalidate {
        engine.ui.invalidate_all();
    }
    physics_dirty
}

/// Dispatches a single [`EngineUiAction`] and returns `(physics_dirty, full_ui_invalidate)`.
fn dispatch_single_action(engine: &mut EditorSession, action: EngineUiAction) -> (bool, bool) {
    match action {
        // --- ENGINE MODE, EDITOR LAYOUT & HOST ACTIONS ---
        EngineUiAction::ChangeMode(mode) => {
            change_mode(engine, mode);
            (true, true)
        }
        EngineUiAction::StartProjectExport(req) => {
            start_project_export(engine, req);
            (false, false)
        }
        EngineUiAction::ResumeGame => {
            resume_paused_game(engine);
            (true, true)
        }
        EngineUiAction::Undo => {
            ae_editor::history::undo(&mut engine.editor, &mut engine.core.world);
            (true, true)
        }
        EngineUiAction::Redo => {
            ae_editor::history::redo(&mut engine.editor, &mut engine.core.world);
            (true, true)
        }
        EngineUiAction::UndoBatch(batch) => {
            ae_editor::actions::undo_batch(&mut engine.editor, &mut engine.core.world, batch);
            (true, true)
        }
        EngineUiAction::ToggleModule(module) => {
            let enabled = !engine.core.event_bus.is_module_enabled(module);
            engine.core.event_bus.set_module_enabled(module, enabled);
            log::info!("Module {:?} toggled to {}", module, enabled);
            (true, true)
        }
        EngineUiAction::OpenPanel(panel) => {
            engine.ui.open_or_focus_panel(panel);
            (false, true)
        }
        EngineUiAction::TogglePanel(panel) => {
            engine.ui.toggle_panel(panel);
            (false, true)
        }
        EngineUiAction::ResetLayout => {
            engine.ui.reset_dock_layout();
            (false, true)
        }
        EngineUiAction::OpenPreferences => {
            engine.ui.open_or_focus_panel(PanelId::Preferences);
            (false, true)
        }
        EngineUiAction::SetUiScale(scale) => {
            engine.ui.set_ui_scale(scale);
            let size = engine.render.render_state.size;
            engine.ui.on_window_resized(size.width, size.height);
            (false, true)
        }
        EngineUiAction::SetThemePreset(preset) => {
            engine.ui.apply_theme_preset(preset);
            (false, true)
        }
        EngineUiAction::SetWireframe(enabled) => {
            engine.editor.wireframe_enabled = enabled;
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            engine.ui.tag_panel_redraw(PanelId::Stats);
            (false, false)
        }
        EngineUiAction::SetShowAllColliders(enabled) => {
            engine.editor.show_all_colliders = enabled;
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            engine.ui.tag_panel_redraw(PanelId::Stats);
            (false, false)
        }
        EngineUiAction::SetGridEnabled(enabled) => {
            engine.editor.grid_enabled = enabled;
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            engine.ui.tag_panel_redraw(PanelId::Stats);
            (false, false)
        }
        EngineUiAction::SetGizmoMode(mode) => {
            engine.editor.gizmo_mode = mode;
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            (false, false)
        }
        EngineUiAction::ToggleGizmoSpace => {
            engine.editor.gizmo_space = engine.editor.gizmo_space.toggle();
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            (false, false)
        }
        EngineUiAction::PickAndAssignEntityTexture(ent) => {
            if let Some(path) = dialogs::pick_texture_file_sync() {
                scene_comp::assign_texture_path(
                    &mut engine.core.world,
                    &engine.render.render_state,
                    &mut engine.render.asset_manager,
                    ent,
                    &path,
                );
                return (false, true);
            }
            (false, false)
        }
        EngineUiAction::PickAndSetSubmeshTexture(model_id, idx) => {
            if let Some(path) = dialogs::pick_texture_file_sync() {
                scene_comp::set_model_submesh_texture_path(
                    &engine.render.render_state,
                    &mut engine.render.asset_manager,
                    model_id,
                    idx,
                    &path,
                );
                return (false, true);
            }
            (false, false)
        }
        EngineUiAction::GarbageCollect => {
            let (removed_models, removed_textures) = ae_scene::garbage_collect_unused_assets(
                &engine.core.world,
                &mut engine.render.asset_manager,
            );
            log::info!(
                "🧹 Asset GC swept {removed_models} unused model(s) and {removed_textures} texture(s)."
            );
            (false, true)
        }

        // --- ENTITY LIFECYCLE & SPAWNING ACTIONS ---
        EngineUiAction::SpawnModel(mid) => {
            spawning::handle_spawn_model_at(engine, mid, [0.0, 0.0, 0.0]);
            (true, true)
        }
        EngineUiAction::SpawnModelAt(mid, pos) => {
            spawning::handle_spawn_model_at(engine, mid, pos);
            (true, true)
        }
        EngineUiAction::SpawnModelPathAt(path, pos) => {
            if let Err(err) = spawning::handle_spawn_model_path_at(engine, &path, pos) {
                engine.ui.set_status_message(
                    format!("Failed to load 3D model: {err}"),
                    irisui::prelude::Color::RED,
                );
                return (false, false);
            }
            (true, true)
        }
        EngineUiAction::SpawnSprite(tid) => {
            spawning::handle_spawn_sprite_at(engine, tid, [0.0, 0.0, 0.0]);
            (true, true)
        }
        EngineUiAction::SpawnSpriteAt(tid, pos) => {
            spawning::handle_spawn_sprite_at(engine, tid, pos);
            (true, true)
        }
        EngineUiAction::SpawnSpritePathAt(path, pos) => {
            spawning::handle_spawn_sprite_path_at(engine, &path, pos);
            (true, true)
        }
        EngineUiAction::SpawnShape(shape) => {
            spawning::handle_spawn_shape(engine, shape);
            (true, true)
        }
        EngineUiAction::SpawnDefaultSprite => {
            spawning::handle_spawn_default_sprite(engine);
            (true, true)
        }
        EngineUiAction::SpawnPlayerSprite => {
            spawning::handle_spawn_player_sprite(engine);
            (true, true)
        }
        EngineUiAction::SpawnEmpty2D => {
            spawning::handle_spawn_empty_2d(engine);
            (true, true)
        }
        EngineUiAction::SpawnUiElement(ui_type) => {
            spawning::handle_spawn_ui_element(engine, ui_type);
            (false, true)
        }
        EngineUiAction::SelectEntity(ent_opt) => {
            engine.editor.select_single(ent_opt);
            engine.ui.refresh_selection_panels();
            (false, false)
        }
        EngineUiAction::DeleteSelected => {
            ae_editor::actions::delete_selected(&mut engine.core.world, &mut engine.editor);
            (true, true)
        }
        EngineUiAction::StressTest(count) => {
            spawning::handle_stress_test(engine, count);
            (true, true)
        }
        EngineUiAction::AaaOpenWorldTest => {
            log::warn!("Ignored unimplemented EngineUiAction::AaaOpenWorldTest");
            engine.ui.set_status_message(
                "Open world preset is not wired in this build",
                irisui::prelude::Color::rgb(220.0 / 255.0, 140.0 / 255.0, 60.0 / 255.0),
            );
            (false, false)
        }
        EngineUiAction::Explode => {
            log::warn!("Ignored unimplemented EngineUiAction::Explode");
            engine.ui.set_status_message(
                "Explode action is not implemented",
                irisui::prelude::Color::rgb(220.0 / 255.0, 140.0 / 255.0, 60.0 / 255.0),
            );
            (false, false)
        }

        // --- PARENTING & HIERARCHY ACTIONS ---
        EngineUiAction::ParentEntity(child, parent) => {
            let _ = ae_transform::parent_entity_keep_world_transform(
                &mut engine.core.world,
                child,
                parent,
            );
            (true, true)
        }
        EngineUiAction::UnparentEntity(child) => {
            let _ =
                ae_transform::unparent_entity_keep_world_transform(&mut engine.core.world, child);
            (true, true)
        }
        EngineUiAction::ToggleVisibility(ent) => {
            scene_comp::toggle_visibility(&mut engine.core.world, ent);
            (false, true)
        }

        // --- COMPONENT MUTATION ACTIONS ---
        EngineUiAction::LiveUpdatePosition(ent, pos) => {
            scene_comp::modify_position(&mut engine.core.world, ent, pos);
            (true, true)
        }
        EngineUiAction::ModifyPosition(ent, old, pos) => {
            scene_comp::modify_position(&mut engine.core.world, ent, pos);
            push_property_undo(&mut engine.editor, ent, old != pos, || {
                ae_editor::undo_redo::Property::Position(old, pos)
            });
            (true, true)
        }
        EngineUiAction::LiveUpdateRotation(ent, rot) => {
            scene_comp::modify_rotation(&mut engine.core.world, ent, rot);
            (true, true)
        }
        EngineUiAction::ModifyRotation(ent, old, rot) => {
            scene_comp::modify_rotation(&mut engine.core.world, ent, rot);
            push_property_undo(&mut engine.editor, ent, old != rot, || {
                ae_editor::undo_redo::Property::Rotation(old, rot)
            });
            (true, true)
        }
        EngineUiAction::LiveUpdateScale(ent, scale) => {
            scene_comp::modify_scale(&mut engine.core.world, ent, scale);
            (true, true)
        }
        EngineUiAction::ModifyScale(ent, old, scale) => {
            scene_comp::modify_scale(&mut engine.core.world, ent, scale);
            push_property_undo(&mut engine.editor, ent, old != scale, || {
                ae_editor::undo_redo::Property::Scale(old, scale)
            });
            (true, true)
        }
        EngineUiAction::ModifyName(ent, old, new_name) => {
            scene_comp::modify_name(&mut engine.core.world, ent, new_name.clone());
            let changed = old != new_name;
            push_property_undo(&mut engine.editor, ent, changed, || {
                ae_editor::undo_redo::Property::Name(old, new_name)
            });
            (false, true)
        }
        EngineUiAction::ModifyColor(ent, old, color) => {
            scene_comp::modify_color(&mut engine.core.world, ent, color);
            push_property_undo(&mut engine.editor, ent, old != color, || {
                ae_editor::undo_redo::Property::Color(old, color)
            });
            (false, true)
        }
        EngineUiAction::ModifyLightColor(ent, old, color) => {
            scene_comp::modify_light_color(&mut engine.core.world, ent, color);
            let light_pos = engine
                .core
                .world
                .get_copied::<ae_light::Light>(ent)
                .map_or([0.0, 0.0, 0.0], |light| light.position);
            push_property_undo(&mut engine.editor, ent, old != color, || {
                ae_editor::undo_redo::Property::Light(
                    ae_light::Light {
                        position: light_pos,
                        color: old,
                    },
                    ae_light::Light {
                        position: light_pos,
                        color,
                    },
                )
            });
            (false, true)
        }
        EngineUiAction::AssignTextureToEntity(ent, path) => {
            scene_comp::assign_texture_path(
                &mut engine.core.world,
                &engine.render.render_state,
                &mut engine.render.asset_manager,
                ent,
                &path,
            );
            (false, true)
        }
        EngineUiAction::RemoveTextureFromEntity(ent) => {
            scene_comp::remove_texture(&mut engine.core.world, ent);
            (false, true)
        }
        EngineUiAction::SetModelSubmeshAlphaMode(model_h, idx, mode) => {
            scene_comp::set_model_submesh_alpha_mode(
                &mut engine.render.asset_manager,
                model_h,
                idx,
                mode,
            );
            (false, true)
        }
        EngineUiAction::SetModelSubmeshTexture(model_h, idx, path) => {
            scene_comp::set_model_submesh_texture_path(
                &engine.render.render_state,
                &mut engine.render.asset_manager,
                model_h,
                idx,
                &path,
            );
            (false, true)
        }
        EngineUiAction::AddComponent(ent, type_name) => {
            scene_comp::add_component(&mut engine.core.world, ent, type_name);
            (true, true)
        }
        EngineUiAction::RemoveComponent(ent, type_name) => {
            scene_comp::remove_component(&mut engine.core.world, ent, type_name);
            (true, true)
        }
        EngineUiAction::ModifyComponent(ent, type_name, data) => {
            scene_comp::modify_component(&mut engine.core.world, ent, type_name, &data);
            (true, true)
        }
        EngineUiAction::CommitComponentModify(ent, type_name, old_data, new_data) => {
            scene_comp::modify_component(&mut engine.core.world, ent, type_name, &new_data);
            push_property_undo(&mut engine.editor, ent, old_data != new_data, || {
                ae_editor::undo_redo::Property::Component {
                    type_name: type_name.to_string(),
                    old_data,
                    new_data,
                }
            });
            (true, true)
        }
        EngineUiAction::ModifyLodThresholds(ent, t1, t2) => {
            scene_comp::modify_lod_thresholds(&mut engine.core.world, ent, t1, t2);
            (false, true)
        }
        EngineUiAction::ModifyLodModel(ent, slot, handle_opt) => {
            scene_comp::modify_lod_model(&mut engine.core.world, ent, slot, handle_opt);
            (false, true)
        }
        EngineUiAction::SpawnPhase1TestSandbox => {
            spawning::handle_spawn_phase1_test_sandbox(engine);
            (true, true)
        }

        // --- CAMERA & SETTINGS ACTIONS ---
        EngineUiAction::SetCameraMode(mode) => {
            engine.render.camera.mode = mode;
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            (false, false)
        }
        EngineUiAction::ToggleCameraProjection => {
            engine.render.camera.toggle_projection();
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            (false, false)
        }
        EngineUiAction::SetCameraTransform {
            pitch,
            yaw,
            position,
            mode,
        } => {
            engine.render.camera.set_pose(pitch, yaw, position, mode);
            engine.ui.tag_panel_redraw(PanelId::Viewport);
            (false, false)
        }
        EngineUiAction::UpdateGraphicsSettings(settings) => {
            engine.render.render_state.graphics_settings = settings;
            (false, true)
        }
        EngineUiAction::UpdateSnapSettings(snap) => {
            engine.editor.snapping = snap;
            (false, true)
        }
        EngineUiAction::UpdateEditorConfig(cfg) => {
            engine.editor.config = cfg;
            engine
                .core
                .time
                .set_fixed_hz(engine.editor.config.physics_hz);
            (false, true)
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
            (false, true)
        }

        // --- SCENE, PREFAB & ASSET BROWSER I/O ACTIONS ---
        EngineUiAction::OpenModelDialog => {
            asset_import::handle_open_model_dialog(engine);
            (false, false)
        }
        EngineUiAction::ImportDroppedFiles { paths, target } => {
            asset_import::handle_import_dropped_files(engine, paths, target);
            (true, true)
        }
        EngineUiAction::OpenLoadPrefabDialog => {
            scene_io::handle_open_load_prefab_dialog(engine);
            (false, false)
        }
        EngineUiAction::OpenSaveSceneDialog | EngineUiAction::SaveScene => {
            scene_io::handle_open_save_scene_dialog(engine);
            (false, false)
        }
        EngineUiAction::OpenLoadSceneDialog | EngineUiAction::LoadScene => {
            scene_io::handle_open_load_scene_dialog(engine);
            (false, false)
        }
        EngineUiAction::SaveSceneToPath(path) => {
            scene_io::handle_save_scene_to_path(engine, path);
            (false, true)
        }
        EngineUiAction::LoadSceneFromPath(path) => {
            scene_io::handle_load_scene_from_path(engine, path);
            (true, true)
        }
        EngineUiAction::SaveEntityAsPrefab(ent, path) => {
            scene_io::handle_save_entity_as_prefab(engine, ent, path);
            (false, false)
        }
        EngineUiAction::InstantiatePrefab(path) => {
            scene_io::handle_instantiate_prefab(engine, path);
            (true, true)
        }
        EngineUiAction::Exit => {
            engine.editor.should_exit = true;
            (false, false)
        }
        EngineUiAction::DeleteAssetOrFolder(path) => {
            let res = ae_assets::file_ops::delete_asset_or_folder(&path);
            run_asset_file_op(engine, "Delete asset", Some(path.as_path()), res);
            (false, true)
        }
        EngineUiAction::CreateSubfolder { parent, name } => {
            let res = ae_assets::file_ops::create_subfolder(&parent, &name);
            run_asset_file_op(engine, "Create subfolder", None, res);
            (false, true)
        }
        EngineUiAction::RenameAssetOrFolder { target, new_name } => {
            let res = ae_assets::file_ops::rename_asset_or_folder(&target, &new_name);
            run_asset_file_op(engine, "Rename asset", None, res);
            (false, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_editor::editor_state::EditorState;
    use ae_editor::undo_redo::{Command, Property};
    use ae_plugin_api::Position;

    #[test]
    fn test_push_property_undo_clears_redo_stack_only_on_actual_change() {
        let mut world = ae_ecs::World::new();
        let ent = world.spawn((Position::default(),));
        let mut editor = EditorState::default();

        let p0 = Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let p1 = Position {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        };
        editor
            .redo_stack
            .push(Command::Modify(ent, Property::Position(p0, p1)));

        // Negative case: unchanged edit must preserve redo history and avoid pushing undo.
        push_property_undo(&mut editor, ent, false, || Property::Position(p0, p0));
        assert!(editor.undo_stack.is_empty());
        assert_eq!(editor.redo_stack.len(), 1);

        // Positive case: changed edit records undo and invalidates stale redo history.
        push_property_undo(&mut editor, ent, true, || Property::Position(p0, p1));
        assert_eq!(editor.undo_stack.len(), 1);
        assert!(editor.redo_stack.is_empty());
    }
}
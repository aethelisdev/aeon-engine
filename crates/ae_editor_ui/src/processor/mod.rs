// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

pub mod components;
pub mod scene_io;
pub mod spawning;
pub mod system;
pub mod transform;

/// UI context to encapsulate all referenced fields from the engine shell.
/// This prevents circular dependencies between crates.
pub struct UiContext<'a> {
    pub mode: &'a mut ae_core::modules::EngineMode,
    pub world: &'a mut hecs::World,
    pub editor: &'a mut ae_editor::editor_state::EditorState,
    pub ui: &'a mut crate::ui::EngineUi,
    pub asset_manager: &'a mut ae_renderer::asset::AssetManager,
    pub camera: &'a mut ae_renderer::camera::Camera,
    pub time: &'a mut ae_core::time::Time,
    pub event_bus: &'a mut ae_core::events::DynamicEventBus,
    pub render_state: &'a mut ae_renderer::render::RenderState,
    pub dialog_receivers: &'a mut Vec<std::sync::mpsc::Receiver<std::path::PathBuf>>,
}

/// Dispatches UI action commands from the egui layer into engine state mutations.
/// Each `EngineUiAction` variant maps to a specific domain sub-handler in the `processor` module:
/// - `spawning`: Model, Sprite, Shape creation & entity deletion
/// - `transform`: Keep World Transform parenting & hierarchy updates
/// - `components`: Position, Rotation, Scale, Light, Camera, Physics, Audio & LOD modifications
/// - `scene_io`: Scene save/load, Prefab save/instantiate, Asset import dialogs
/// - `system`: Engine mode switches, module toggles, Undo/Redo, snapping parameters
pub fn process_ui_actions(ctx: &mut UiContext, actions: std::vec::Vec<crate::ui::EngineUiAction>) {
    for action in actions {
        match action {
            // --- ENGINE MODE & STATE ACTIONS ---
            crate::ui::EngineUiAction::ChangeMode(m) => system::handle_change_mode(ctx, m),
            crate::ui::EngineUiAction::Undo => system::handle_undo(ctx),
            crate::ui::EngineUiAction::Redo => system::handle_redo(ctx),
            crate::ui::EngineUiAction::UndoBatch(b) => system::handle_undo_batch(ctx, b),
            crate::ui::EngineUiAction::ToggleModule(module) => {
                system::handle_toggle_module(ctx, module)
            }
            crate::ui::EngineUiAction::GarbageCollect => system::handle_garbage_collect(ctx),

            // --- ENTITY LIFECYCLE & SPAWNING ACTIONS ---
            crate::ui::EngineUiAction::SpawnModel(mid) => spawning::handle_spawn_model(ctx, mid),
            crate::ui::EngineUiAction::SpawnSprite(tid) => spawning::handle_spawn_sprite(ctx, tid),
            crate::ui::EngineUiAction::SpawnShape(shape) => {
                spawning::handle_spawn_shape(ctx, shape)
            }
            crate::ui::EngineUiAction::SelectEntity(ent_opt) => {
                spawning::handle_select_entity(ctx, ent_opt)
            }
            crate::ui::EngineUiAction::DeleteSelected => spawning::handle_delete_selected(ctx),
            crate::ui::EngineUiAction::StressTest(count) => {
                spawning::handle_stress_test(ctx, count)
            }

            // --- PARENTING & HIERARCHY ACTIONS ---
            crate::ui::EngineUiAction::ParentEntity(child, parent) => {
                transform::handle_parent_entity(ctx, child, parent)
            }
            crate::ui::EngineUiAction::UnparentEntity(child) => {
                transform::handle_unparent_entity(ctx, child)
            }

            // --- COMPONENT MUTATION ACTIONS ---
            crate::ui::EngineUiAction::LiveUpdatePosition(ent, pos)
            | crate::ui::EngineUiAction::ModifyPosition(ent, _, pos) => {
                components::handle_modify_position(ctx, ent, pos)
            }
            crate::ui::EngineUiAction::LiveUpdateRotation(ent, rot)
            | crate::ui::EngineUiAction::ModifyRotation(ent, _, rot) => {
                components::handle_modify_rotation(ctx, ent, rot)
            }
            crate::ui::EngineUiAction::LiveUpdateScale(ent, scale)
            | crate::ui::EngineUiAction::ModifyScale(ent, _, scale) => {
                components::handle_modify_scale(ctx, ent, scale)
            }
            crate::ui::EngineUiAction::ModifyName(ent, _, new_name) => {
                components::handle_modify_name(ctx, ent, new_name)
            }
            crate::ui::EngineUiAction::ModifyColor(ent, _, color) => {
                components::handle_modify_color(ctx, ent, color)
            }
            crate::ui::EngineUiAction::ModifyLightColor(ent, _, color) => {
                components::handle_modify_light_color(ctx, ent, color)
            }
            crate::ui::EngineUiAction::AddRigidBody(ent, rb) => {
                components::handle_add_rigid_body(ctx, ent, rb)
            }
            crate::ui::EngineUiAction::RemoveRigidBody(ent) => {
                components::handle_remove_rigid_body(ctx, ent)
            }
            crate::ui::EngineUiAction::ModifyRigidBody(ent, rb) => {
                components::handle_modify_rigid_body(ctx, ent, rb)
            }
            crate::ui::EngineUiAction::AddCollider(ent, collider) => {
                components::handle_add_collider(ctx, ent, collider)
            }
            crate::ui::EngineUiAction::RemoveCollider(ent) => {
                components::handle_remove_collider(ctx, ent)
            }
            crate::ui::EngineUiAction::ModifyCollider(ent, collider) => {
                components::handle_modify_collider(ctx, ent, collider)
            }
            crate::ui::EngineUiAction::AddCharacterController(ent, cc) => {
                components::handle_add_character_controller(ctx, ent, cc)
            }
            crate::ui::EngineUiAction::RemoveCharacterController(ent) => {
                components::handle_remove_character_controller(ctx, ent)
            }
            crate::ui::EngineUiAction::ModifyCharacterController(ent, cc) => {
                components::handle_modify_character_controller(ctx, ent, cc)
            }
            crate::ui::EngineUiAction::AddLodGroup(ent) => {
                components::handle_add_lod_group(ctx, ent)
            }
            crate::ui::EngineUiAction::RemoveLodGroup(ent) => {
                components::handle_remove_lod_group(ctx, ent)
            }
            crate::ui::EngineUiAction::ModifyLodThresholds(ent, t1, t2) => {
                components::handle_modify_lod_thresholds(ctx, ent, t1, t2)
            }
            crate::ui::EngineUiAction::ModifyLodModel(ent, slot, handle_opt) => {
                components::handle_modify_lod_model(ctx, ent, slot, handle_opt)
            }
            crate::ui::EngineUiAction::AddAudioSource(ent) => {
                components::handle_add_audio_source(ctx, ent)
            }
            crate::ui::EngineUiAction::RemoveAudioSource(ent) => {
                components::handle_remove_audio_source(ctx, ent)
            }
            crate::ui::EngineUiAction::ModifyAudioSource(ent, source) => {
                components::handle_modify_audio_source(ctx, ent, source)
            }
            crate::ui::EngineUiAction::AddAudioListener(ent) => {
                components::handle_add_audio_listener(ctx, ent)
            }
            crate::ui::EngineUiAction::RemoveAudioListener(ent) => {
                components::handle_remove_audio_listener(ctx, ent)
            }

            // --- CAMERA & SETTINGS ACTIONS ---
            crate::ui::EngineUiAction::SetCameraMode(mode) => {
                system::handle_set_camera_mode(ctx, mode)
            }
            crate::ui::EngineUiAction::SetCameraTransform {
                pitch,
                yaw,
                position,
            } => system::handle_set_camera_transform(ctx, pitch, yaw, position),
            crate::ui::EngineUiAction::UpdateGraphicsSettings(settings) => {
                system::handle_update_graphics_settings(ctx, settings)
            }
            crate::ui::EngineUiAction::UpdateSnapSettings(snap) => {
                system::handle_update_snap_settings(ctx, snap)
            }
            crate::ui::EngineUiAction::UpdateEditorConfig(cfg) => {
                system::handle_update_editor_config(ctx, cfg)
            }
            crate::ui::EngineUiAction::SetLiveEditorUpdates(val) => {
                system::handle_set_live_editor_updates(ctx, val)
            }

            // --- SCENE & PREFAB I/O ACTIONS ---
            crate::ui::EngineUiAction::OpenModelDialog => scene_io::handle_open_model_dialog(ctx),
            crate::ui::EngineUiAction::OpenSaveSceneDialog => {
                scene_io::handle_open_save_scene_dialog(ctx)
            }
            crate::ui::EngineUiAction::OpenLoadSceneDialog => {
                scene_io::handle_open_load_scene_dialog(ctx)
            }
            crate::ui::EngineUiAction::SaveScene => scene_io::handle_save_scene(ctx),
            crate::ui::EngineUiAction::LoadScene => scene_io::handle_load_scene(ctx),
            crate::ui::EngineUiAction::SaveSceneToPath(path) => {
                scene_io::handle_save_scene_to_path(ctx, path)
            }
            crate::ui::EngineUiAction::LoadSceneFromPath(path) => {
                scene_io::handle_load_scene_from_path(ctx, path)
            }
            crate::ui::EngineUiAction::SaveEntityAsPrefab(ent, path) => {
                scene_io::handle_save_entity_as_prefab(ctx, ent, path)
            }
            crate::ui::EngineUiAction::InstantiatePrefab(path) => {
                scene_io::handle_instantiate_prefab(ctx, path)
            }

            _ => {}
        }
    }
}
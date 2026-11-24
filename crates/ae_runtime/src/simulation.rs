// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor-free shared simulation execution core: Play camera tracking, skeletal skinning,
//! fixed physics stepping, and `SaveGame` runtime delta slot execution.

use ae_core::events::{
    DynamicEventBus, LoadGameCompleted, LoadGameRequest, SaveGameCompleted, SaveGameRequest,
};
use ae_ecs::ComponentRegistry;
use ae_input::InputManager;
use ae_physics::world::PhysicsWorld;
use ae_plugin_api::{EntityGuid, PlayerTag, Position, TransformDirty};
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use ae_scene::RuntimeStreamingManager;
use ae_transform::update_hierarchy_transforms;
use ae_world::{
    ComponentRecord, EntityRecord, SaveSlotMetadata, SpatialCellCoord, WorldDeltaSnapshot,
    atomic_write_save, load_save_with_fallback,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Advances built-in Play-mode player movement (2D top-down/planar `PlayerTag` translation and
/// 3D `CharacterController` WASD + Sprint + Jump + Gravity kinematics) for one fixed timestep.
///
/// Shared between `ae_engine` Play mode and the standalone `ae_runtime` executable so exported
/// builds exhibit identical player controls without duplicating movement logic.
pub fn step_player_movement(
    input: &InputManager,
    world: &mut ae_ecs::World,
    physics_world: &mut PhysicsWorld,
    camera: &ae_camera::Camera,
    is_2d: bool,
    dt: f32,
) {
    let fwd_axis = input.get_axis("MoveForward");
    let right_axis = input.get_axis("MoveRight");
    let is_running = input.is_action_pressed("Run");

    if is_2d {
        let speed = if is_running { 10.0 } else { 5.0 };
        let move_x = right_axis * speed * dt;
        let move_y = fwd_axis * speed * dt;
        if move_x.abs() > 0.0001 || move_y.abs() > 0.0001 {
            for (_tag, pos) in world.query::<(&PlayerTag, &mut Position)>().iter() {
                pos.x += move_x;
                pos.y += move_y;
                pos.z = 0.0;
            }
        }
    }

    let kcc_entities: Vec<ae_ecs::Entity> = world
        .query::<(ae_ecs::Entity, &ae_plugin_api::CharacterController)>()
        .iter()
        .map(|(ent, _)| ent)
        .collect();
    if kcc_entities.is_empty() {
        return;
    }

    use ae_physics::glam::{Quat, Vec3};
    let fwd_vec = camera.get_forward();
    let right_vec = camera.get_right();
    let cam_fwd_xz = Vec3::new(fwd_vec.x, 0.0, fwd_vec.z).normalize_or_zero();
    let cam_right_xz = Vec3::new(right_vec.x, 0.0, right_vec.z).normalize_or_zero();

    let dir = cam_fwd_xz * fwd_axis + cam_right_xz * right_axis;
    let speed = if is_running { 10.0 } else { 6.0 };
    let move_dir = dir.normalize_or_zero();

    if move_dir.length_squared() > 0.001 {
        let rot_quat = Quat::from_rotation_y(move_dir.x.atan2(move_dir.z));
        for &ent in &kcc_entities {
            if let Ok(r) = world.get_mut::<ae_plugin_api::Rotation>(ent) {
                r.x = rot_quat.x;
                r.y = rot_quat.y;
                r.z = rot_quat.z;
                r.w = rot_quat.w;
            }
            if let Some(&handle) = physics_world.entity_to_body.get(&ent)
                && let Some(body) = physics_world.rigid_body_set.get_mut(handle)
            {
                body.set_rotation(rot_quat, true);
            }
        }
    }

    let jump_pressed = input.is_action_pressed("Jump");
    for ent in kcc_entities {
        let is_grounded = world
            .inspect::<ae_plugin_api::CharacterController, _>(ent, |c| c.is_grounded)
            .unwrap_or(false);
        let mut vert_vel = world
            .get_copied::<ae_plugin_api::Velocity>(ent)
            .map_or(0.0, |v| v.y);

        if is_grounded {
            if jump_pressed {
                vert_vel = 9.0;
                if let Ok(ctrl) = world.get_mut::<ae_plugin_api::CharacterController>(ent) {
                    ctrl.is_grounded = false;
                }
            } else {
                vert_vel = 0.0;
            }
        } else {
            vert_vel -= 20.0 * dt;
        }

        if let Ok(vel) = world.get_mut::<ae_plugin_api::Velocity>(ent) {
            vel.x = 0.0;
            vel.y = vert_vel;
            vel.z = 0.0;
        }

        let translation = Vec3::new(
            move_dir.x * speed * dt,
            vert_vel * dt,
            move_dir.z * speed * dt,
        );
        physics_world.move_character(world, ent, translation, dt);
    }
}

/// Evaluates active skeletal animation clips and pushes GPU skinning bone matrices.
pub fn update_skeletal_animations(
    world: &mut ae_ecs::World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    dt: f32,
) {
    let mut candidate_entities = Vec::new();
    for (entity, player, model_id) in world
        .query_mut::<(
            ae_ecs::Entity,
            &mut ae_animation::AnimationPlayer,
            &ae_plugin_api::ModelId,
        )>()
        .into_iter()
    {
        candidate_entities.push((entity, model_id.0, player.current_clip.is_none()));
    }

    for (entity, model_id, needs_clip) in candidate_entities {
        if !world.has::<ae_animation::Skeleton>(entity)
            && let Some(asset) = asset_manager.models.get(model_id)
        {
            if let Some(ref skel) = asset.skeleton {
                let _ = world.insert_one(entity, skel.clone());
            }
            if needs_clip
                && !asset.animations.is_empty()
                && let Ok(player) = world.get_mut::<ae_animation::AnimationPlayer>(entity)
            {
                player.play(asset.animations[0].clone());
            }
        }
    }

    let mut skinning_updates = Vec::new();
    for (_entity, player, skeleton, model_id) in world
        .query_mut::<(
            ae_ecs::Entity,
            &mut ae_animation::AnimationPlayer,
            &ae_animation::Skeleton,
            &ae_plugin_api::ModelId,
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
        render_state.update_model_skinning(asset_manager, model_handle, &palette);
    }
}

/// Parameters controlling runtime `SaveGame` slot capture and restoration from [`DynamicEventBus`].
pub struct SaveLoadDispatchContext<'a> {
    /// Directory containing savegame files (`Saved/SaveGames`).
    pub save_games_dir: &'a Path,
    /// Persistent GUID of the active world document.
    pub world_guid: EntityGuid,
    /// Spatial partition grid cell size used when indexing cell deltas.
    pub cell_size: f32,
    /// Total gameplay session duration in seconds written to save headers.
    pub play_time_secs: u64,
    /// Active ECS world whose stateful components are captured or restored.
    pub world: &'a mut ae_ecs::World,
    /// Event bus supplying [`SaveGameRequest`]/[`LoadGameRequest`] and receiving completions.
    pub event_bus: &'a mut DynamicEventBus,
    /// Optional runtime chunk streaming manager synchronized with loaded delta snapshots.
    pub streaming: Option<&'a mut RuntimeStreamingManager>,
}

/// Computes the canonical `.aesave` path for `slot_index` inside `save_games_dir`.
#[must_use]
pub fn save_slot_path(save_games_dir: &Path, slot_index: u16) -> PathBuf {
    save_games_dir.join(format!("slot_{slot_index}.aesave"))
}

/// Captures live ECS entity components partitioned by [`SpatialCellCoord`] into a [`WorldDeltaSnapshot`].
#[must_use]
pub fn capture_runtime_save_snapshot(
    world: &ae_ecs::World,
    metadata: SaveSlotMetadata,
    cell_size: f32,
) -> WorldDeltaSnapshot {
    let mut snapshot = WorldDeltaSnapshot::new(metadata);
    let registry = ae_plugin_api::engine_component_registry();

    let mut entity_to_guid = HashMap::new();
    for entity_ref in world.iter() {
        if let Some(guid) = entity_ref.get_copied::<EntityGuid>()
            && !guid.is_nil()
        {
            entity_to_guid.insert(entity_ref.entity(), guid);
        }
    }

    for entity_ref in world.iter() {
        let entity = entity_ref.entity();
        let Some(&guid) = entity_to_guid.get(&entity) else {
            continue;
        };
        let parent_guid = entity_ref
            .get_copied::<ae_plugin_api::Parent>()
            .and_then(|p| entity_to_guid.get(&p.0).copied());
        let mut rec = EntityRecord::new(guid, parent_guid);

        for handler in registry.handlers() {
            if matches!(handler.type_name(), "Parent" | "Children" | "EntityGuid") {
                continue;
            }
            if let Some(payload) = handler.capture(world, entity) {
                rec.components.push(ComponentRecord {
                    type_hash: handler.type_hash(),
                    schema_ver: handler.schema_version(),
                    flags: 0,
                    payload,
                });
            }
        }

        if let Some(pos) = entity_ref.get_copied::<Position>() {
            let cell = SpatialCellCoord::from_world_pos([pos.x, pos.y, pos.z], cell_size);
            snapshot.record_cell_modified(cell, rec);
        } else {
            snapshot.global_delta.upsert_modified(rec);
        }
    }

    snapshot
}

/// Applies loaded delta snapshot component overrides to live entities sharing matching [`EntityGuid`]s
/// and forwards the full snapshot to `streaming` when active.
pub fn apply_runtime_save_snapshot(
    world: &mut ae_ecs::World,
    snapshot: &WorldDeltaSnapshot,
    mut streaming: Option<&mut RuntimeStreamingManager>,
) {
    let registry = ae_plugin_api::engine_component_registry();
    let mut guid_to_entity = HashMap::new();
    for entity_ref in world.iter() {
        if let Some(guid) = entity_ref.get_copied::<EntityGuid>()
            && !guid.is_nil()
        {
            guid_to_entity.insert(guid, entity_ref.entity());
        }
    }

    let mut deltas_vec: Vec<_> = snapshot.cell_deltas.values().collect();
    apply_record_slice(
        world,
        registry,
        &guid_to_entity,
        &snapshot.global_delta.modified_entities,
    );
    for delta in deltas_vec.drain(..) {
        let mut despawn_set = HashSet::new();
        for &guid in &delta.destroyed_entities {
            if let Some(entity) = guid_to_entity.remove(&guid) {
                despawn_set.insert(entity);
            }
        }
        for entity in despawn_set {
            let _ = world.despawn(entity);
        }
        apply_record_slice(world, registry, &guid_to_entity, &delta.modified_entities);
        apply_record_slice(world, registry, &guid_to_entity, &delta.spawned_entities);
    }

    update_hierarchy_transforms(world);
    if let Some(ref mut mgr) = streaming {
        mgr.set_save_delta(Some(snapshot.clone()));
    }
}

fn apply_record_slice(
    world: &mut ae_ecs::World,
    registry: &ComponentRegistry,
    guid_to_entity: &HashMap<EntityGuid, ae_ecs::Entity>,
    records: &[EntityRecord],
) {
    for rec in records {
        let Some(&entity) = guid_to_entity.get(&rec.guid) else {
            continue;
        };
        for comp in &rec.components {
            if let Some(handler) = registry.get_by_type_hash(comp.type_hash) {
                let _ = handler.apply_versioned(world, entity, comp.schema_ver, &comp.payload);
            }
        }
        let _ = world.insert_one(entity, TransformDirty);
    }
}

/// Drains queued [`SaveGameRequest`] and [`LoadGameRequest`] commands from `ctx.event_bus`,
/// executing atomic `.aesave` persistence or `.bak` fallback loading and emitting completion events.
pub fn process_save_load_requests(ctx: &mut SaveLoadDispatchContext<'_>) -> bool {
    let mut world_changed = false;

    if let Some(save_reqs) = ctx.event_bus.receive::<SaveGameRequest>() {
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        for req in save_reqs {
            let meta = SaveSlotMetadata {
                slot_index: req.slot_index,
                world_guid: ctx.world_guid,
                timestamp_unix: now_unix,
                play_time_secs: ctx.play_time_secs,
                label: req.label,
            };
            let snapshot = capture_runtime_save_snapshot(ctx.world, meta, ctx.cell_size);
            let target_path = save_slot_path(ctx.save_games_dir, req.slot_index);
            match atomic_write_save(&target_path, &snapshot) {
                Ok(()) => ctx.event_bus.send(SaveGameCompleted {
                    slot_index: req.slot_index,
                    success: true,
                    error: None,
                }),
                Err(err) => ctx.event_bus.send(SaveGameCompleted {
                    slot_index: req.slot_index,
                    success: false,
                    error: Some(err.to_string()),
                }),
            }
        }
    }

    if let Some(load_reqs) = ctx.event_bus.receive::<LoadGameRequest>() {
        let known_hashes = ae_plugin_api::engine_component_registry().known_type_hashes();
        for req in load_reqs {
            let target_path = save_slot_path(ctx.save_games_dir, req.slot_index);
            match load_save_with_fallback(&target_path, Some(&known_hashes)) {
                Ok(snapshot) => {
                    let streaming_opt = ctx.streaming.as_deref_mut();
                    apply_runtime_save_snapshot(ctx.world, &snapshot, streaming_opt);
                    world_changed = true;
                    ctx.event_bus.send(LoadGameCompleted {
                        slot_index: req.slot_index,
                        success: true,
                        error: None,
                    });
                }
                Err(err) => ctx.event_bus.send(LoadGameCompleted {
                    slot_index: req.slot_index,
                    success: false,
                    error: Some(err.to_string()),
                }),
            }
        }
    }

    world_changed
}

/// Ticks the runtime [`GameStateManager`](ae_core::state::GameStateManager) and toggles
/// [`InGamePauseState`](ae_ui::InGamePauseState) when the `"Pause"` action is just pressed.
///
/// Returns `(was_paused, is_paused)` so callers can synchronize cursor grab and skip
/// gameplay camera/simulation updates while paused.
pub fn step_play_state_and_pause(
    state_manager: &mut ae_core::state::StateManager,
    world: &mut ae_ecs::World,
    event_bus: &mut DynamicEventBus,
    input: &InputManager,
    dt: f32,
) -> (bool, bool) {
    let was_paused = state_manager.is_paused();

    let mut state_cmd_buffer = ae_ecs::EntityCommandBuffer::new();
    state_manager.update(world, event_bus, &mut state_cmd_buffer, dt);
    state_cmd_buffer.apply(world);

    if input.is_action_just_pressed("Pause") {
        if state_manager.is_paused() {
            state_manager.pop();
        } else {
            state_manager.push(ae_ui::InGamePauseState::new());
        }
        let mut toggle_cmd = ae_ecs::EntityCommandBuffer::new();
        state_manager.update(world, event_bus, &mut toggle_cmd, 0.0);
        toggle_cmd.apply(world);
    }

    (was_paused, state_manager.is_paused())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::Name;
    use ae_world::GuidGenerator;

    #[test]
    fn test_ingame_save_slot_and_load_slot_event_roundtrip() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let save_dir = std::env::temp_dir().join(format!("ae_runtime_save_slot_{nanos}"));
        let _ = std::fs::remove_dir_all(&save_dir);

        let mut ids = GuidGenerator::from_seed(404);
        let world_guid = ids.next_entity_guid();
        let player_guid = ids.next_entity_guid();

        let mut world = ae_ecs::World::new();
        let player = world.spawn((
            player_guid,
            Name("Hero".to_string()),
            Position::new(42.0, 5.0, -18.0),
        ));

        let mut bus = DynamicEventBus::new();
        bus.save_slot(1, "Boss Door");

        process_save_load_requests(&mut SaveLoadDispatchContext {
            save_games_dir: &save_dir,
            world_guid,
            cell_size: 128.0,
            play_time_secs: 300,
            world: &mut world,
            event_bus: &mut bus,
            streaming: None,
        });

        let save_done = bus
            .receive::<SaveGameCompleted>()
            .expect("save completion emitted");
        assert!(save_done[0].success, "save err: {:?}", save_done[0].error);
        assert!(save_slot_path(&save_dir, 1).is_file());

        // Mutate player position and request restore from slot 1
        if let Ok(pos) = world.get_mut::<Position>(player) {
            pos.x = 999.0;
        }
        bus.load_slot(1);

        let changed = process_save_load_requests(&mut SaveLoadDispatchContext {
            save_games_dir: &save_dir,
            world_guid,
            cell_size: 128.0,
            play_time_secs: 350,
            world: &mut world,
            event_bus: &mut bus,
            streaming: None,
        });
        assert!(changed);

        let load_done = bus
            .receive::<LoadGameCompleted>()
            .expect("load completion emitted");
        assert!(load_done[0].success, "load err: {:?}", load_done[0].error);
        let restored_pos = world.get_copied::<Position>(player).expect("player pos");
        assert!((restored_pos.x - 42.0).abs() < 1e-5);
        assert!((restored_pos.z + 18.0).abs() < 1e-5);

        let _ = std::fs::remove_dir_all(&save_dir);
    }

    #[test]
    fn test_step_play_state_and_pause_toggles_pause_via_action_map() {
        let mut state_manager = ae_core::state::StateManager::new();
        let mut world = ae_ecs::World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut input = InputManager::new();

        let (was_paused, is_paused) = step_play_state_and_pause(
            &mut state_manager,
            &mut world,
            &mut event_bus,
            &input,
            0.016,
        );
        assert!(!was_paused);
        assert!(!is_paused);

        input.process_key_event(ae_input::KeyCode::KeyP, winit::event::ElementState::Pressed);
        let (was_paused, is_paused) = step_play_state_and_pause(
            &mut state_manager,
            &mut world,
            &mut event_bus,
            &input,
            0.016,
        );
        assert!(!was_paused);
        assert!(is_paused);
        assert!(
            world
                .query::<&ae_ui::PauseMenuUiTag>()
                .iter()
                .next()
                .is_some()
        );

        input.end_frame();
        input.process_key_event(
            ae_input::KeyCode::KeyP,
            winit::event::ElementState::Released,
        );
        input.end_frame();
        input.process_key_event(ae_input::KeyCode::KeyP, winit::event::ElementState::Pressed);
        let (was_paused, is_paused) = step_play_state_and_pause(
            &mut state_manager,
            &mut world,
            &mut event_bus,
            &input,
            0.016,
        );
        assert!(was_paused);
        assert!(!is_paused);
        assert!(
            world
                .query::<&ae_ui::PauseMenuUiTag>()
                .iter()
                .next()
                .is_none()
        );
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor-free shared simulation execution core: Play camera tracking, skeletal skinning,
//! fixed physics stepping, and `SaveGame` runtime delta slot execution.

use crate::streaming::RuntimeStreamingManager;
use ae_core::ecs::{
    ComponentRegistry, EcsManager, EntityGuid, PlayerTag, Position, TransformDirty,
    update_hierarchy_transforms,
};
use ae_core::events::{
    DynamicEventBus, LoadGameCompleted, LoadGameRequest, SaveGameCompleted, SaveGameRequest,
};
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use ae_world::{
    ComponentRecord, EntityRecord, SaveSlotMetadata, SpatialCellCoord, WorldDeltaSnapshot,
    atomic_write_save, load_save_with_fallback,
};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Updates Play-mode camera tracking (orthographic 2D follow or 3D orbit rig) without editor state.
///
/// Consumes `mouse_delta` and zeroes it out after applying rotation.
pub fn update_play_camera(
    ecs: &mut EcsManager,
    camera: &mut ae_core::camera::Camera,
    mouse_delta: &mut (f32, f32),
    mouse_sensitivity: f32,
) {
    if camera.mode == ae_core::camera::ProjectionMode::Orthographic {
        camera.yaw = cgmath::Rad(-std::f32::consts::FRAC_PI_2);
        camera.pitch = cgmath::Rad(0.0);
        *mouse_delta = (0.0, 0.0);

        if let Some(target_pos) = find_play_target_pos(&mut ecs.world, false) {
            camera.position.x = target_pos.x;
            camera.position.y = target_pos.y;
            camera.position.z = 10.0;
            camera.target.x = target_pos.x;
            camera.target.y = target_pos.y;
            camera.target.z = 0.0;
        } else {
            camera.target = camera.position + camera.get_forward();
        }
        return;
    }

    let (dx, dy) = *mouse_delta;
    if dx.abs() > 0.001 || dy.abs() > 0.001 {
        let sensitivity = mouse_sensitivity * 0.7;
        camera.yaw += cgmath::Rad(dx * sensitivity);
        camera.pitch -= cgmath::Rad(dy * sensitivity);

        let max_pitch = cgmath::Rad(std::f32::consts::FRAC_PI_2 * 0.83);
        if camera.pitch > max_pitch {
            camera.pitch = max_pitch;
        } else if camera.pitch < -max_pitch {
            camera.pitch = -max_pitch;
        }
    }
    *mouse_delta = (0.0, 0.0);

    if let Some(mut target_pos) = find_play_target_pos(&mut ecs.world, true) {
        target_pos.y += 1.5;
        let forward = camera.get_forward();
        let arm_distance = 6.0_f32;
        camera.position = target_pos - forward * arm_distance;
        camera.target = target_pos;
    } else {
        camera.target = camera.position + camera.get_forward();
    }
}

fn find_play_target_pos(
    world: &mut hecs::World,
    prefer_controller: bool,
) -> Option<cgmath::Point3<f32>> {
    if prefer_controller {
        if let Some(pos) = query_controller_pos(world) {
            return Some(pos);
        }
        return query_player_tag_pos(world);
    }
    if let Some(pos) = query_player_tag_pos(world) {
        return Some(pos);
    }
    query_controller_pos(world)
}

fn query_player_tag_pos(world: &mut hecs::World) -> Option<cgmath::Point3<f32>> {
    world
        .query_mut::<(&PlayerTag, &Position)>()
        .into_iter()
        .next()
        .map(|(_, pos)| cgmath::Point3::new(pos.x, pos.y, pos.z))
}

fn query_controller_pos(world: &mut hecs::World) -> Option<cgmath::Point3<f32>> {
    world
        .query_mut::<(&ae_core::ecs::CharacterController, &Position)>()
        .into_iter()
        .next()
        .map(|(_, pos)| cgmath::Point3::new(pos.x, pos.y, pos.z))
}

/// Evaluates active skeletal animation clips and pushes GPU skinning bone matrices.
pub fn update_skeletal_animations(
    world: &mut hecs::World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    dt: f32,
) {
    let mut candidate_entities = Vec::new();
    for (entity, player, model_id) in world
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
        if world.get::<&ae_animation::Skeleton>(entity).is_err()
            && let Some(asset) = asset_manager.models.get(model_id)
        {
            if let Some(ref skel) = asset.skeleton {
                let _ = world.insert_one(entity, skel.clone());
            }
            if needs_clip
                && !asset.animations.is_empty()
                && let Ok(mut player) = world.get::<&mut ae_animation::AnimationPlayer>(entity)
            {
                player.play(asset.animations[0].clone());
            }
        }
    }

    let mut skinning_updates = Vec::new();
    for (_entity, player, skeleton, model_id) in world
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
    pub world: &'a mut hecs::World,
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
    world: &hecs::World,
    metadata: SaveSlotMetadata,
    cell_size: f32,
) -> WorldDeltaSnapshot {
    let mut snapshot = WorldDeltaSnapshot::new(metadata);
    let registry = ComponentRegistry::global();

    let mut entity_to_guid = HashMap::new();
    for entity_ref in world.iter() {
        if let Some(guid) = entity_ref.get::<&EntityGuid>().map(|g| *g)
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
            .get::<&ae_core::ecs::Parent>()
            .and_then(|p| entity_to_guid.get(&p.0).copied());
        let mut rec = EntityRecord::new(guid, parent_guid);

        for handler in registry.handlers() {
            if matches!(handler.type_name(), "Parent" | "Children" | "EntityGuid") {
                continue;
            }
            if let Some(payload) = handler.capture(world, entity) {
                rec.components.push(ComponentRecord {
                    type_hash: handler.type_hash(),
                    schema_ver: 1,
                    flags: 0,
                    payload,
                });
            }
        }

        if let Some(pos) = entity_ref.get::<&Position>().map(|p| *p) {
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
    world: &mut hecs::World,
    snapshot: &WorldDeltaSnapshot,
    mut streaming: Option<&mut RuntimeStreamingManager>,
) {
    let registry = ComponentRegistry::global();
    let mut guid_to_entity = HashMap::new();
    for entity_ref in world.iter() {
        if let Some(guid) = entity_ref.get::<&EntityGuid>().map(|g| *g)
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
    world: &mut hecs::World,
    registry: &ComponentRegistry,
    guid_to_entity: &HashMap<EntityGuid, hecs::Entity>,
    records: &[EntityRecord],
) {
    for rec in records {
        let Some(&entity) = guid_to_entity.get(&rec.guid) else {
            continue;
        };
        for comp in &rec.components {
            if let Some(handler) = registry.get_by_type_hash(comp.type_hash) {
                let _ = handler.apply(world, entity, &comp.payload);
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
        let known_hashes = ComponentRegistry::global().known_type_hashes();
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

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::Name;
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

        let mut world = hecs::World::new();
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
        if let Ok(mut pos) = world.get::<&mut Position>(player) {
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
        let restored_pos = world.get::<&Position>(player).expect("player pos");
        assert!((restored_pos.x - 42.0).abs() < 1e-5);
        assert!((restored_pos.z + 18.0).abs() < 1e-5);

        let _ = std::fs::remove_dir_all(&save_dir);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified scene lifecycle crate owning binary world serialization, async & VFS world
//! loading, runtime chunk streaming, hierarchical prefabs, entity spawning, hierarchy
//! transform reparenting, and background Edit-mode autosave.

pub mod autosave;
pub mod loader;
pub mod prefab;
pub mod serialization;
pub mod spawning;
pub mod streaming;

pub use autosave::{
    AUTOSAVE_ROTATING_SLOTS, AutosavePollContext, DEFAULT_AUTOSAVE_INTERVAL_SECS,
    EditorAutosaveState, resolve_autosave_target, write_autosave_if_changed,
};
pub use loader::{
    AppliedWorldLoad, PrefixedWorldVfs, apply_loaded_scene_data, begin_async_scene_load,
    decode_world_document_from_vfs, prepare_world_load_request, spawn_world_records,
};
pub use prefab::{PREFAB_FORMAT_VERSION, PREFAB_MAGIC, Prefab};
pub use serialization::{
    DEFAULT_WORLD_CELL_SIZE, SavedWorldOutcome, WorldSaveRequest, capture_world_document,
    ensure_world_entity_guids, save_partitioned_world,
};
pub use spawning::{
    ModelSpawnDesc, SpriteSpawnDesc, collect_used_asset_handles, garbage_collect_unused_assets,
    resolve_asset_display_name, spawn_default_sprite_entity, spawn_empty_2d_entity,
    spawn_model_entity, spawn_phase1_sandbox, spawn_player_sprite_entity, spawn_shape_entity,
    spawn_sprite_entity, spawn_stress_test_cubes,
};
pub use streaming::{
    RuntimeStreamingManager, StreamingStepReport, bind_entity_assets_from_binding,
};
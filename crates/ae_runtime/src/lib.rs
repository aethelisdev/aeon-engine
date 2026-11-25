// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor-free standalone game player shell backed by [`ae_app`], static gameplay plugins,
//! and `.aepak` / partitioned VFS world loading.

pub mod app;
pub mod static_plugin;
pub mod world_loader;

pub use ae_app::{
    BehaviorRunnerParams, SaveLoadDispatchContext, apply_runtime_save_snapshot, behavior_runner,
    capture_runtime_save_snapshot, process_save_load_requests, save_slot_path, simulation,
    step_play_state_and_pause, step_player_movement, update_gameplay_behaviors,
    update_skeletal_animations,
};
pub use app::{RuntimeSession, run_desktop_player};
pub use static_plugin::{StaticGameLogicBackend, StaticUpdateFn, attach_static_game_logic};
pub use world_loader::{
    LoadedRuntimeWorld, RuntimeLaunchArgs, discover_vfs_and_entry, load_world_into_runtime,
    resolve_entry_world_path,
};
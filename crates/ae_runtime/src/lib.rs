// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor-free shared simulation execution core, runtime chunk streaming, static gameplay
//! backend bridge, `.aepak` world loading, and standalone player shell.

pub mod app;
pub mod behavior_runner;
pub mod simulation;
pub mod static_plugin;
pub mod world_loader;

pub use app::{RuntimeSession, run_desktop_player};
pub use behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
pub use simulation::{
    SaveLoadDispatchContext, apply_runtime_save_snapshot, capture_runtime_save_snapshot,
    process_save_load_requests, save_slot_path, step_play_state_and_pause, step_player_movement,
    update_skeletal_animations,
};
pub use static_plugin::{StaticGameLogicBackend, StaticUpdateFn, attach_static_game_logic};
pub use world_loader::{
    LoadedRuntimeWorld, RuntimeLaunchArgs, discover_vfs_and_entry, load_world_into_runtime,
    resolve_entry_world_path,
};
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular application lifecycle, phase schedules, plugin registry, and shared headless
//! render and simulation hosts for Aeon Engine applications.

pub mod app;
pub mod behavior_runner;
pub mod core_world;
pub mod plugin;
pub mod render_host;
pub mod schedule;
pub mod simulation;
pub mod simulation_host;

pub use app::AeApp;
pub use behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
pub use core_world::CoreWorld;
pub use plugin::{AppPlugin, PluginRegistry};
pub use render_host::{PreparedRenderScene, RenderHost, empty_render_scene, prepare_2d_overlay};
pub use schedule::{AppPhase, AppSchedule, AppSystemFn};
pub use simulation::{
    SaveLoadDispatchContext, apply_runtime_save_snapshot, capture_runtime_save_snapshot,
    process_save_load_requests, save_slot_path, step_play_state_and_pause, step_player_movement,
    update_skeletal_animations,
};
pub use simulation_host::{PauseTransition, PlayFrameParams, SimulationHost};
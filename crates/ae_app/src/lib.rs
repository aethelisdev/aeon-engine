// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular application lifecycle, phase schedules, plugin registry/groups, and feature-gated
//! render and simulation hosts for Aeon Engine applications.

pub mod app;
#[cfg(feature = "gameplay")]
pub mod behavior_runner;
pub mod core_world;
pub mod plugin;
pub mod plugin_group;
pub mod plugins;
#[cfg(feature = "rendering")]
pub mod render_host;
pub mod schedule;
#[cfg(feature = "gameplay")]
pub mod simulation;
#[cfg(feature = "gameplay")]
pub mod simulation_host;

pub use app::AeApp;
#[cfg(feature = "gameplay")]
pub use behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
pub use core_world::{CoreWorld, NonSendResources};
pub use plugin::{AppPlugin, PluginRegistry};
pub use plugin_group::{PluginGroup, PluginGroupBuilder};
pub use plugins::DefaultCorePlugins;
#[cfg(all(feature = "rendering", feature = "2d"))]
pub use render_host::prepare_2d_overlay;
#[cfg(feature = "rendering")]
pub use render_host::{PreparedRenderScene, RenderHost, empty_render_scene};
pub use schedule::{AppPhase, AppSchedule, AppSystemFn};
#[cfg(feature = "gameplay")]
pub use simulation::{
    SaveLoadDispatchContext, apply_runtime_save_snapshot, capture_runtime_save_snapshot,
    process_save_load_requests, save_slot_path, step_play_state_and_pause, step_player_movement,
    update_skeletal_animations,
};
#[cfg(feature = "gameplay")]
pub use simulation_host::{PauseTransition, PlayFrameParams, SimulationHost};
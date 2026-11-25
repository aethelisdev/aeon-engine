// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Shared simulation execution host for physics, audio, gameplay behaviors, HUD, streaming,
//! skeletal animations, plugins, and runtime save slots.
//!
//! Eliminates duplicated Play-mode update loops between standalone runtime players and the editor.

use crate::behavior_runner::{BehaviorRunnerParams, update_gameplay_behaviors};
use crate::core_world::CoreWorld;
use crate::render_host::RenderHost;
use crate::simulation::{
    SaveLoadDispatchContext, process_save_load_requests, step_play_state_and_pause,
    step_player_movement, update_skeletal_animations,
};
use ae_audio::AudioManager;
use ae_camera::Camera;
use ae_camera_controller::{FollowCameraConfig, update_follow_camera};
use ae_core::events::DynamicEventBus;
use ae_core::modules::EngineModule;
use ae_ecs::{EntityCommandBuffer, World};
use ae_physics::PhysicsWorld;
use ae_plugin_host::PluginManager;
use ae_renderer::asset::AssetManager;
use ae_scene::{RuntimeStreamingManager, bind_entity_assets_from_binding};
use ae_state::StateManager;
use ae_transform::{integrate_unconstrained_velocities, update_hierarchy_transforms};
use ae_ui::InGameHudState;
use std::path::Path;
use std::sync::Arc;

/// Configuration parameters passed to [`SimulationHost::step_play_frame`].
pub struct PlayFrameParams<'a> {
    /// Mouse look sensitivity applied to [`update_follow_camera`].
    pub mouse_sensitivity: f32,
    /// Directory storing `.aesave` runtime save slots.
    pub save_games_dir: &'a Path,
    /// Spatial cell size used when capturing savegame cell deltas.
    pub cell_size: f32,
    /// Elapsed gameplay time in seconds written into savegame metadata.
    pub play_time_secs: u64,
    /// Maximum streaming cells instantiated per frame.
    pub streaming_budget_per_frame: usize,
    /// Whether plugin backends should tick during this frame.
    pub tick_plugins: bool,
}

/// Pause-state transition observed during a Play frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseTransition {
    /// Pause state did not change this frame.
    Unchanged {
        /// Current pause state.
        is_paused: bool,
    },
    /// Game entered pause state this frame (cursor should release).
    EnteredPause,
    /// Game resumed from pause state this frame (cursor grab may re-engage in 3D).
    ExitedPause,
}

impl PauseTransition {
    /// Returns `true` when gameplay is currently paused.
    #[inline]
    #[must_use]
    pub const fn is_paused(self) -> bool {
        match self {
            Self::Unchanged { is_paused } => is_paused,
            Self::EnteredPause => true,
            Self::ExitedPause => false,
        }
    }
}

/// Cohesive runtime simulation container uniting physics, audio, gameplay FSM, HUD, plugins,
/// and runtime chunk streaming.
pub struct SimulationHost {
    /// Rapier3D physics world and collider caches.
    pub physics_world: PhysicsWorld,
    /// Spatial 3D audio playback manager.
    pub audio_manager: AudioManager,
    /// Stack-based pushdown gameplay/pause state machine.
    pub state_manager: StateManager,
    /// Gameplay event HUD state mapper.
    pub in_game_hud: InGameHudState,
    /// Native/static scripting backend host.
    pub plugin_manager: PluginManager,
    /// Runtime spatial chunk streamer (populated when a partitioned world is mounted).
    pub streaming: Option<RuntimeStreamingManager>,
    /// Dirty flag triggering lazy ECS->Rapier resync after structural world edits.
    pub physics_sync_dirty: bool,
}

impl SimulationHost {
    /// Creates a new [`SimulationHost`] with default subsystems.
    #[must_use]
    pub fn new(
        audio_manager: AudioManager,
        plugin_manager: PluginManager,
        streaming: Option<RuntimeStreamingManager>,
    ) -> Self {
        Self {
            physics_world: PhysicsWorld::new(),
            audio_manager,
            state_manager: StateManager::new(),
            in_game_hud: InGameHudState::new(),
            plugin_manager,
            streaming,
            physics_sync_dirty: false,
        }
    }

    /// Runs accumulated fixed-timestep physics and player/FSM stepping when `run_play_simulation`
    /// is active (draining accumulated steps otherwise to avoid spiral catch-up).
    pub fn step_fixed_physics(
        &mut self,
        core: &mut CoreWorld,
        render: &RenderHost,
        run_play_simulation: bool,
    ) {
        let physics_enabled = core.event_bus.is_module_enabled(EngineModule::Physics);
        if !physics_enabled {
            while core.time.consume_fixed_step() {}
            return;
        }

        while core.time.consume_fixed_step() {
            if !run_play_simulation || self.state_manager.is_paused() {
                continue;
            }
            integrate_unconstrained_velocities(&mut core.world, core.time.fixed_time_step);

            let mut state_cmd = EntityCommandBuffer::new();
            self.state_manager.fixed_update(
                &mut core.world,
                &mut core.event_bus,
                &mut state_cmd,
                core.time.fixed_time_step,
            );
            state_cmd.apply(&mut core.world);

            step_player_movement(
                &core.input,
                &mut core.world,
                &mut self.physics_world,
                &render.camera,
                render.is_2d(),
                core.time.fixed_time_step,
            );

            self.physics_world.step(
                &mut core.world,
                |handle| {
                    render
                        .asset_manager
                        .get_physics_mesh_data(handle)
                        .map(|(v, i)| (v.as_slice(), i.as_slice()))
                },
                core.time.fixed_time_step,
                &mut core.event_bus,
            );
        }
    }

    /// Performs lazy ECS-to-Rapier synchronization when [`Self::physics_sync_dirty`] is set.
    pub fn sync_dirty_physics(&mut self, world: &mut World, asset_manager: &AssetManager) {
        if !self.physics_sync_dirty {
            return;
        }
        self.physics_sync_dirty = false;
        self.physics_world.sync_ecs_to_physics(world, |handle| {
            asset_manager
                .get_physics_mesh_data(handle)
                .map(|(v, i)| (v.as_slice(), i.as_slice()))
        });
    }

    /// Executes one active Play-mode variable-timestep frame: pause toggle, follow camera,
    /// chunk streaming, gameplay behaviors, scripting plugins, runtime save slots, HUD, audio,
    /// and skeletal skinning.
    pub fn step_play_frame(
        &mut self,
        core: &mut CoreWorld,
        render: &mut RenderHost,
        params: &PlayFrameParams<'_>,
    ) -> PauseTransition {
        let (was_paused, is_paused) = step_play_state_and_pause(
            &mut self.state_manager,
            &mut core.world,
            &mut core.event_bus,
            &core.input,
            core.time.delta_time,
        );

        let transition = match (was_paused, is_paused) {
            (false, true) => {
                core.clear_inputs_for_pause();
                PauseTransition::EnteredPause
            }
            (true, false) => {
                core.input.clear_mouse_delta();
                PauseTransition::ExitedPause
            }
            _ => PauseTransition::Unchanged { is_paused },
        };

        if !is_paused {
            self.step_active_play_systems(core, render, params);
        } else {
            core.input.clear_mouse_delta();
        }

        if process_save_load_requests(&mut SaveLoadDispatchContext {
            save_games_dir: params.save_games_dir,
            world_guid: core.active_world_guid,
            cell_size: params.cell_size,
            play_time_secs: params.play_time_secs,
            world: &mut core.world,
            event_bus: &mut core.event_bus,
            streaming: self.streaming.as_mut(),
        }) {
            self.physics_sync_dirty = true;
        }

        self.in_game_hud
            .update_from_events(&mut core.world, &mut core.event_bus);
        self.update_audio(&core.world, &core.event_bus, &render.camera);
        update_skeletal_animations(
            &mut core.world,
            &render.render_state,
            &mut render.asset_manager,
            core.time.delta_time,
        );

        transition
    }

    fn step_active_play_systems(
        &mut self,
        core: &mut CoreWorld,
        render: &mut RenderHost,
        params: &PlayFrameParams<'_>,
    ) {
        update_follow_camera(
            &mut core.world,
            &mut render.camera,
            core.input.mouse_delta_mut(),
            FollowCameraConfig {
                mouse_sensitivity: params.mouse_sensitivity,
                ..Default::default()
            },
        );

        render.spatial_grid.sync(&core.world);

        if let Some(mut streaming) = self.streaming.take() {
            let active_vfs = Arc::clone(streaming.vfs());
            let render_state = &render.render_state;
            let report = streaming.step(
                &mut core.world,
                &mut render.asset_manager,
                [
                    render.camera.position.x,
                    render.camera.position.y,
                    render.camera.position.z,
                ],
                params.streaming_budget_per_frame,
                |world, asset_manager, entity, binding| {
                    bind_entity_assets_from_binding(
                        world,
                        render_state,
                        asset_manager,
                        Some(active_vfs.as_ref()),
                        entity,
                        binding,
                    );
                },
            );
            if report.spawned_entities > 0 || report.despawned_entities > 0 {
                update_hierarchy_transforms(&mut core.world);
                self.physics_sync_dirty = true;
            }
            self.streaming = Some(streaming);
        }

        let cam_fwd = render.camera.get_forward();
        update_gameplay_behaviors(BehaviorRunnerParams {
            world: &mut core.world,
            physics_world: &mut self.physics_world,
            input: &core.input,
            event_bus: &mut core.event_bus,
            camera_forward: cam_fwd,
            delta_time: core.time.delta_time,
        });

        if params.tick_plugins {
            self.plugin_manager.tick(
                &mut core.world,
                &mut core.resources,
                &mut core.event_bus,
                core.time.delta_time,
            );
        }
    }

    /// Updates spatial 3D audio listener and source positions for the current frame.
    pub fn update_audio(&mut self, world: &World, event_bus: &DynamicEventBus, camera: &Camera) {
        let is_audio_enabled = event_bus.is_module_enabled(EngineModule::Audio);
        let cam_pos = ae_audio::Vec3::new(camera.position.x, camera.position.y, camera.position.z);
        let right = camera.get_right();
        let cam_right = ae_audio::Vec3::new(right.x, right.y, right.z);
        self.audio_manager
            .update(world, cam_pos, cam_right, is_audio_enabled);
    }

    /// Resets transient HUD, state machine, and Rapier poses when switching Edit ↔ Play modes.
    pub fn reset_for_mode_transition(&mut self, world: &mut World, asset_manager: &AssetManager) {
        self.in_game_hud.reset(world);
        self.state_manager = StateManager::new();
        self.physics_world.sync_ecs_to_physics(world, |handle| {
            asset_manager
                .get_physics_mesh_data(handle)
                .map(|(v, i)| (v.as_slice(), i.as_slice()))
        });
        self.physics_world.reset_simulation_poses(world);
        self.physics_sync_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pause_transition_state_queries() {
        assert!(!PauseTransition::Unchanged { is_paused: false }.is_paused());
        assert!(PauseTransition::Unchanged { is_paused: true }.is_paused());
        assert!(PauseTransition::EnteredPause.is_paused());
        assert!(!PauseTransition::ExitedPause.is_paused());
    }
}
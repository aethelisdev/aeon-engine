// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Game state trait, execution context, and stack transition descriptors.

use ae_ecs::EntityCommandBuffer;
use ae_plugin_api::DynamicEventBus;

/// Execution context passed to [`GameState`] lifecycle hooks.
pub struct StateContext<'a> {
    /// Mutable reference to the ECS world for querying or reading entity components.
    pub world: &'a mut ae_ecs::World,
    /// Shared reference to dynamic event bus for dispatching and reading gameplay events.
    pub event_bus: &'a mut DynamicEventBus,
    /// Deferred entity command buffer for queueing safe mutations at frame boundaries.
    pub commands: &'a mut EntityCommandBuffer,
    /// Elapsed frame delta time in seconds.
    pub delta_time: f32,
    /// Flag indicating whether the state machine is currently running a paused overlay.
    pub is_paused: bool,
}

/// Transition requests returned by [`GameState::on_update`] or queued on [`crate::StateManager`].
pub enum StateTransition {
    /// Keep current state active without any transition.
    None,
    /// Pauses current state and pushes a new overlay state on top of the stack.
    Push(Box<dyn GameState>),
    /// Pops the active state from top of the stack, destroying it and resuming the underlying state.
    Pop,
    /// Destroys the active state and replaces it with a new state.
    Switch(Box<dyn GameState>),
    /// Clears all states from the stack and sets a new root state.
    ClearAndSet(Box<dyn GameState>),
}

/// Interface defining lifecycle hooks and execution callbacks for high-level game states.
pub trait GameState: Send + Sync + 'static {
    /// Returns the human-readable display name of this game state.
    fn name(&self) -> &'static str;

    /// Returns `true` when this state represents a simulation-pausing overlay.
    ///
    /// Used by [`crate::StateManager::is_paused`] in place of string comparisons.
    fn is_pause_state(&self) -> bool {
        false
    }

    /// Invoked when this state is first entered or pushed onto the state stack.
    fn on_enter(&mut self, _ctx: &mut StateContext<'_>) {}

    /// Invoked when this state is popped or destroyed from the state stack.
    fn on_exit(&mut self, _ctx: &mut StateContext<'_>) {}

    /// Invoked when another state is pushed on top of this state.
    fn on_pause(&mut self, _ctx: &mut StateContext<'_>) {}

    /// Invoked when an overlying state is popped and this state becomes active again.
    fn on_resume(&mut self, _ctx: &mut StateContext<'_>) {}

    /// Main per-frame update loop for the active state.
    fn on_update(&mut self, _ctx: &mut StateContext<'_>, _dt: f32) -> StateTransition {
        StateTransition::None
    }

    /// Fixed-step simulation update loop for physics synchronization.
    fn on_fixed_update(&mut self, _ctx: &mut StateContext<'_>, _fixed_dt: f32) {}
}
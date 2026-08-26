// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Stack-Based Game State Machine & High-Level Flow Management.
//!
//! Provides a flexible, pushdown automaton state architecture for managing game modes,
//! menus, in-game simulation, pause overlays, and level transitions in 100% Safe Rust.
//!

use crate::commands::EntityCommandBuffer;
use ae_plugin_api::DynamicEventBus;

/// Execution context passed to `GameState` lifecycle hooks.
/// Encapsulates access to ECS storage, command queue, event bus, and timing metrics.
pub struct StateContext<'a> {
    /// Mutable reference to the ECS world for querying or reading entity components.
    pub world: &'a mut hecs::World,
    /// Shared reference to dynamic event bus for dispatching and reading gameplay events.
    pub event_bus: &'a mut DynamicEventBus,
    /// Deferred entity command buffer for queueing safe mutations at frame boundaries.
    pub commands: &'a mut EntityCommandBuffer,
    /// Elapsed frame delta time in seconds.
    pub delta_time: f32,
    /// Flag indicating whether the state machine is currently running a paused overlay.
    pub is_paused: bool,
}

/// Transition requests returned by `GameState::on_update` or dispatched to `StateManager`.
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
    /// Returns the human-readable display name of this game state (e.g., "Playing", "Paused", "MainMenu").
    fn name(&self) -> &'static str;

    /// Invoked when this state is first entered or pushed onto the state stack.
    fn on_enter(&mut self, _ctx: &mut StateContext<'_>) {}

    /// Invoked when this state is popped or destroyed from the state stack.
    fn on_exit(&mut self, _ctx: &mut StateContext<'_>) {}

    /// Invoked when another state is pushed on top of this state (e.g. pause menu overlay).
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

/// Default in-game simulation state representing active gameplay.
#[derive(Default)]
pub struct DefaultPlayingState;

impl GameState for DefaultPlayingState {
    fn name(&self) -> &'static str {
        "Playing"
    }
}

/// Default pause state representing a paused overlay.
#[derive(Default)]
pub struct DefaultPausedState;

impl GameState for DefaultPausedState {
    fn name(&self) -> &'static str {
        "Paused"
    }
}

/// Pushdown automaton managing the stack of active `GameState` instances.
/// Handles state transitions, lifecycle dispatches (`on_enter`, `on_exit`, `on_pause`, `on_resume`),
/// and per-frame update propagation with zero `unsafe` blocks.
pub struct StateManager {
    stack: Vec<Box<dyn GameState>>,
    pending_transition: Option<StateTransition>,
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    /// Creates a new `StateManager` initialized with `DefaultPlayingState`.
    pub fn new() -> Self {
        Self {
            stack: vec![Box::new(DefaultPlayingState)],
            pending_transition: None,
        }
    }

    /// Creates a new `StateManager` with a customized initial root state.
    pub fn with_initial_state<S: GameState>(initial: S) -> Self {
        Self {
            stack: vec![Box::new(initial)],
            pending_transition: None,
        }
    }

    /// Returns the name of the currently active state on top of the stack, or `"Empty"` if stack is empty.
    pub fn active_state_name(&self) -> &'static str {
        self.stack.last().map(|s| s.name()).unwrap_or("Empty")
    }

    /// Returns the number of states currently stacked in the automaton.
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if there are no states in the stack.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns `true` if the active state is a pause state or if a pause overlay is stacked.
    pub fn is_paused(&self) -> bool {
        let name = self.active_state_name();
        name == "Paused" || name == "InGamePauseMenu" || self.stack.len() > 1
    }

    /// Schedules pushing a new state onto the top of the stack.
    pub fn push<S: GameState>(&mut self, state: S) {
        self.pending_transition = Some(StateTransition::Push(Box::new(state)));
    }

    /// Schedules popping the top state from the stack.
    pub fn pop(&mut self) {
        self.pending_transition = Some(StateTransition::Pop);
    }

    /// Schedules switching the active state with a new state.
    pub fn switch<S: GameState>(&mut self, state: S) {
        self.pending_transition = Some(StateTransition::Switch(Box::new(state)));
    }

    /// Executes the active state's `on_update` loop and applies any requested transitions.
    pub fn update(
        &mut self,
        world: &mut hecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        dt: f32,
    ) {
        // 1. Process any transition queued before update
        self.apply_pending_transition(world, event_bus, commands, dt);

        if self.stack.is_empty() {
            return;
        }

        // 2. Temporarily pop the active state to satisfy borrow rules
        let mut active_state = match self.stack.pop() {
            Some(s) => s,
            None => return,
        };

        let transition = {
            let mut ctx = StateContext {
                world,
                event_bus,
                commands,
                delta_time: dt,
                is_paused: !self.stack.is_empty(),
            };
            active_state.on_update(&mut ctx, dt)
        };

        // 3. Restore active state back to stack
        self.stack.push(active_state);

        // 4. Apply transition returned from on_update
        if !matches!(transition, StateTransition::None) {
            self.pending_transition = Some(transition);
            self.apply_pending_transition(world, event_bus, commands, dt);
        }
    }

    /// Executes the active state's `on_fixed_update` loop for physics integration.
    pub fn fixed_update(
        &mut self,
        world: &mut hecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        fixed_dt: f32,
    ) {
        if self.stack.is_empty() {
            return;
        }

        let mut active_state = match self.stack.pop() {
            Some(s) => s,
            None => return,
        };

        {
            let mut ctx = StateContext {
                world,
                event_bus,
                commands,
                delta_time: fixed_dt,
                is_paused: !self.stack.is_empty(),
            };
            active_state.on_fixed_update(&mut ctx, fixed_dt);
        }

        self.stack.push(active_state);
    }

    /// Applies pending transitions with move semantics and dispatches enter/exit/pause/resume hooks.
    fn apply_pending_transition(
        &mut self,
        world: &mut hecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        dt: f32,
    ) {
        let transition = match self.pending_transition.take() {
            Some(t) => t,
            None => return,
        };

        match transition {
            StateTransition::None => {}
            StateTransition::Push(mut new_state) => {
                // Pause current active state if present
                if let Some(mut current) = self.stack.pop() {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: true,
                    };
                    current.on_pause(&mut ctx);
                    self.stack.push(current);
                }

                // Enter new state
                {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    new_state.on_enter(&mut ctx);
                }
                self.stack.push(new_state);
            }
            StateTransition::Pop => {
                // Exit and destroy active state
                if let Some(mut exiting) = self.stack.pop() {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    exiting.on_exit(&mut ctx);
                }

                // Resume underlying state if present
                if let Some(mut resuming) = self.stack.pop() {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    resuming.on_resume(&mut ctx);
                    self.stack.push(resuming);
                }
            }
            StateTransition::Switch(mut new_state) => {
                // Exit and destroy current active state
                if let Some(mut exiting) = self.stack.pop() {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    exiting.on_exit(&mut ctx);
                }

                // Enter new state
                {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    new_state.on_enter(&mut ctx);
                }
                self.stack.push(new_state);
            }
            StateTransition::ClearAndSet(mut root_state) => {
                // Exit all states in reverse order
                while let Some(mut exiting) = self.stack.pop() {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    exiting.on_exit(&mut ctx);
                }

                // Enter root state
                {
                    let mut ctx = StateContext {
                        world,
                        event_bus,
                        commands,
                        delta_time: dt,
                        is_paused: false,
                    };
                    root_state.on_enter(&mut ctx);
                }
                self.stack.push(root_state);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockStateA {
        entered: bool,
        exited: bool,
        paused: bool,
        resumed: bool,
    }

    impl GameState for MockStateA {
        fn name(&self) -> &'static str {
            "StateA"
        }
        fn on_enter(&mut self, _ctx: &mut StateContext<'_>) {
            self.entered = true;
        }
        fn on_exit(&mut self, _ctx: &mut StateContext<'_>) {
            self.exited = true;
        }
        fn on_pause(&mut self, _ctx: &mut StateContext<'_>) {
            self.paused = true;
        }
        fn on_resume(&mut self, _ctx: &mut StateContext<'_>) {
            self.resumed = true;
        }
    }

    struct MockStateB;
    impl GameState for MockStateB {
        fn name(&self) -> &'static str {
            "StateB"
        }
    }

    #[test]
    fn test_state_manager_push_pop_lifecycle() {
        let mut world = hecs::World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut commands = EntityCommandBuffer::new();

        let mut sm = StateManager::with_initial_state(MockStateA::default());
        assert_eq!(sm.active_state_name(), "StateA");
        assert_eq!(sm.stack_depth(), 1);

        // Push StateB (StateA should receive on_pause)
        sm.push(MockStateB);
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);

        assert_eq!(sm.active_state_name(), "StateB");
        assert_eq!(sm.stack_depth(), 2);

        // Pop StateB (StateA should receive on_resume)
        sm.pop();
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);

        assert_eq!(sm.active_state_name(), "StateA");
        assert_eq!(sm.stack_depth(), 1);
    }

    #[test]
    fn test_state_manager_switch_and_clear() {
        let mut world = hecs::World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut commands = EntityCommandBuffer::new();

        let mut sm = StateManager::with_initial_state(MockStateA::default());
        sm.switch(MockStateB);
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);

        assert_eq!(sm.active_state_name(), "StateB");
        assert_eq!(sm.stack_depth(), 1);

        sm.push(MockStateA::default());
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.stack_depth(), 2);

        sm.pending_transition = Some(StateTransition::ClearAndSet(Box::new(DefaultPlayingState)));
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.active_state_name(), "Playing");
        assert_eq!(sm.stack_depth(), 1);
    }
}
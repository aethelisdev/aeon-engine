// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Pushdown automaton managing the stack of active [`GameState`] instances.

use crate::defaults::DefaultPlayingState;
use crate::types::{GameState, StateContext, StateTransition};
use ae_ecs::EntityCommandBuffer;
use ae_plugin_api::DynamicEventBus;

/// Pushdown automaton managing the stack of active [`GameState`] instances.
///
/// Handles state transitions, lifecycle dispatches (`on_enter`, `on_exit`, `on_pause`,
/// `on_resume`), and per-frame / fixed-step update propagation.
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
    /// Creates a new `StateManager` initialized with [`DefaultPlayingState`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            stack: vec![Box::new(DefaultPlayingState)],
            pending_transition: None,
        }
    }

    /// Creates a new `StateManager` with a customized initial root state.
    #[must_use]
    pub fn with_initial_state<S: GameState>(initial: S) -> Self {
        Self {
            stack: vec![Box::new(initial)],
            pending_transition: None,
        }
    }

    /// Returns the display name of the currently active state, or `"Empty"` if stack is empty.
    #[must_use]
    pub fn active_state_name(&self) -> &'static str {
        self.stack.last().map_or("Empty", |s| s.name())
    }

    /// Returns the number of states currently stacked in the automaton.
    #[must_use]
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if there are no states in the stack.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Returns `true` if the active state marks itself as a pause state or if an overlay state is
    /// stacked above the root state.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.stack.len() > 1 || self.stack.last().is_some_and(|s| s.is_pause_state())
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

    /// Schedules clearing the entire stack and replacing it with `root_state`.
    pub fn clear_and_set<S: GameState>(&mut self, root_state: S) {
        self.pending_transition = Some(StateTransition::ClearAndSet(Box::new(root_state)));
    }

    /// Executes the active state's `on_update` loop and applies any requested transitions.
    pub fn update(
        &mut self,
        world: &mut ae_ecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        dt: f32,
    ) {
        self.apply_pending_transition(world, event_bus, commands, dt);

        let Some(mut active_state) = self.stack.pop() else {
            return;
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

        self.stack.push(active_state);

        if !matches!(transition, StateTransition::None) {
            self.pending_transition = Some(transition);
            self.apply_pending_transition(world, event_bus, commands, dt);
        }
    }

    /// Executes the active state's `on_fixed_update` loop for physics integration.
    pub fn fixed_update(
        &mut self,
        world: &mut ae_ecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        fixed_dt: f32,
    ) {
        let Some(mut active_state) = self.stack.pop() else {
            return;
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

    fn apply_pending_transition(
        &mut self,
        world: &mut ae_ecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        dt: f32,
    ) {
        let Some(transition) = self.pending_transition.take() else {
            return;
        };

        match transition {
            StateTransition::None => {}
            StateTransition::Push(mut new_state) => {
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

                Self::enter_state(new_state.as_mut(), world, event_bus, commands, dt);
                self.stack.push(new_state);
            }
            StateTransition::Pop => {
                if let Some(mut exiting) = self.stack.pop() {
                    Self::exit_state(exiting.as_mut(), world, event_bus, commands, dt);
                }
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
                if let Some(mut exiting) = self.stack.pop() {
                    Self::exit_state(exiting.as_mut(), world, event_bus, commands, dt);
                }
                Self::enter_state(new_state.as_mut(), world, event_bus, commands, dt);
                self.stack.push(new_state);
            }
            StateTransition::ClearAndSet(mut root_state) => {
                while let Some(mut exiting) = self.stack.pop() {
                    Self::exit_state(exiting.as_mut(), world, event_bus, commands, dt);
                }
                Self::enter_state(root_state.as_mut(), world, event_bus, commands, dt);
                self.stack.push(root_state);
            }
        }
    }

    fn enter_state(
        state: &mut dyn GameState,
        world: &mut ae_ecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        dt: f32,
    ) {
        let mut ctx = StateContext {
            world,
            event_bus,
            commands,
            delta_time: dt,
            is_paused: false,
        };
        state.on_enter(&mut ctx);
    }

    fn exit_state(
        state: &mut dyn GameState,
        world: &mut ae_ecs::World,
        event_bus: &mut DynamicEventBus,
        commands: &mut EntityCommandBuffer,
        dt: f32,
    ) {
        let mut ctx = StateContext {
            world,
            event_bus,
            commands,
            delta_time: dt,
            is_paused: false,
        };
        state.on_exit(&mut ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defaults::DefaultPausedState;

    #[derive(Default)]
    struct MockStateA;

    impl GameState for MockStateA {
        fn name(&self) -> &'static str {
            "StateA"
        }
    }

    struct MockStateB;
    impl GameState for MockStateB {
        fn name(&self) -> &'static str {
            "StateB"
        }
    }

    #[test]
    fn test_state_manager_push_pop_and_is_paused() {
        let mut world = ae_ecs::World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut commands = EntityCommandBuffer::new();

        let mut sm = StateManager::with_initial_state(MockStateA);
        assert_eq!(sm.active_state_name(), "StateA");
        assert_eq!(sm.stack_depth(), 1);
        assert!(!sm.is_paused());

        sm.push(MockStateB);
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.active_state_name(), "StateB");
        assert_eq!(sm.stack_depth(), 2);
        assert!(sm.is_paused());

        sm.pop();
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.active_state_name(), "StateA");
        assert_eq!(sm.stack_depth(), 1);
        assert!(!sm.is_paused());
    }

    #[test]
    fn test_state_manager_switch_clear_and_empty_pop_guard() {
        let mut world = ae_ecs::World::new();
        let mut event_bus = DynamicEventBus::new();
        let mut commands = EntityCommandBuffer::new();

        let mut sm = StateManager::with_initial_state(DefaultPausedState);
        assert!(sm.is_paused());

        sm.switch(MockStateB);
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.active_state_name(), "StateB");
        assert!(!sm.is_paused());

        sm.push(MockStateA);
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.stack_depth(), 2);

        sm.clear_and_set(DefaultPlayingState);
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert_eq!(sm.active_state_name(), "Playing");
        assert_eq!(sm.stack_depth(), 1);

        // Popping down to empty stack and updating on empty stack must stay safe
        sm.pop();
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert!(sm.is_empty());
        assert_eq!(sm.active_state_name(), "Empty");
        assert!(!sm.is_paused());

        sm.pop();
        sm.update(&mut world, &mut event_bus, &mut commands, 0.016);
        sm.fixed_update(&mut world, &mut event_bus, &mut commands, 0.016);
        assert!(sm.is_empty());
    }
}
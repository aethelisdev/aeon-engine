// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for pushdown automaton GameState flow and lifecycle callbacks.
//!

use ae_core::events::DynamicEventBus;
use ae_core::state::{GameState, StateContext, StateManager, StateTransition};
use hecs::World;

struct TestLevelState {
    score: u32,
}

impl GameState for TestLevelState {
    fn name(&self) -> &'static str {
        "TestLevel"
    }
    fn on_update(&mut self, _ctx: &mut StateContext<'_>, _dt: f32) -> StateTransition {
        self.score += 10;
        if self.score == 30 {
            StateTransition::Push(Box::new(TestPauseState))
        } else {
            StateTransition::None
        }
    }
}

struct TestPauseState;

impl GameState for TestPauseState {
    fn name(&self) -> &'static str {
        "TestPause"
    }
    fn on_enter(&mut self, ctx: &mut StateContext<'_>) {
        ctx.commands.spawn_with(|w| {
            w.spawn((
                ae_core::ecs::Name("PauseMenuBanner".to_string()),
                ae_core::ecs::Position::new(0.0, 0.0, 0.0),
            ))
        });
    }
}

#[test]
fn test_game_state_machine_custom_stack_flow() {
    let mut world = World::new();
    let mut event_bus = DynamicEventBus::new();
    let mut sm = StateManager::with_initial_state(TestLevelState { score: 0 });

    assert_eq!(sm.active_state_name(), "TestLevel");
    assert_eq!(sm.stack_depth(), 1);

    // Frame 1: score = 10
    let mut cmd = ae_core::commands::EntityCommandBuffer::new();
    sm.update(&mut world, &mut event_bus, &mut cmd, 0.016);
    cmd.apply(&mut world);
    assert_eq!(sm.active_state_name(), "TestLevel");

    // Frame 2: score = 20
    sm.update(&mut world, &mut event_bus, &mut cmd, 0.016);
    cmd.apply(&mut world);
    assert_eq!(sm.active_state_name(), "TestLevel");

    // Frame 3: score = 30 -> Pushes TestPauseState!
    sm.update(&mut world, &mut event_bus, &mut cmd, 0.016);
    cmd.apply(&mut world);
    assert_eq!(sm.active_state_name(), "TestPause");
    assert_eq!(sm.stack_depth(), 2);

    // Verify deferred entity from on_enter was spawned into ECS world
    let mut pause_banner_found = false;
    for (name, _) in world
        .query::<(&ae_core::ecs::Name, &ae_core::ecs::Position)>()
        .iter()
    {
        if name.0 == "PauseMenuBanner" {
            pause_banner_found = true;
        }
    }
    assert!(
        pause_banner_found,
        "Pause menu banner entity must be spawned by state on_enter"
    );

    // Pop pause state -> returns to TestLevel
    sm.pop();
    sm.update(&mut world, &mut event_bus, &mut cmd, 0.016);
    cmd.apply(&mut world);
    assert_eq!(sm.active_state_name(), "TestLevel");
    assert_eq!(sm.stack_depth(), 1);
}
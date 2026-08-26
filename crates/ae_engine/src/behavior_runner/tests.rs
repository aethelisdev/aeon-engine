// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unit tests for gameplay behaviors and subsystem pipelines.
//!

use super::*;
use ae_core::ecs::{
    Color, DestructibleTarget, MovingPlatform, Position, Rotation, Rotator, Scale, TriggerZone,
    Velocity,
};

use ae_core::events::{DynamicEventBus, RaycastHitEvent, TargetDestroyedEvent};
use ae_editor::input::InputManager;
use ae_physics::world::PhysicsWorld;
use hecs::World;

#[test]
fn test_rotator_behavior_rotation_progression() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        Rotator::new(2.0, [0.0, 1.0, 0.0]),
    ));

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.5,
    });

    let rot = *world.get::<&Rotation>(ent).unwrap();
    assert!(
        rot.y.abs() > 0.01 || rot.w.abs() < 0.99,
        "Rotator must advance rotation quaternion"
    );
}

#[test]
fn test_moving_platform_waypoint_interpolation() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        MovingPlatform::new(5.0, [0.0, 0.0, 0.0], [10.0, 0.0, 0.0]),
    ));

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 1.0,
    });

    let pos = *world.get::<&Position>(ent).unwrap();
    assert!(
        (pos.x - 5.0).abs() < 0.1,
        "Platform should move 5.0 units in 1 second at 5.0 m/s"
    );
}

#[test]
fn test_trigger_zone_activation_and_elevation() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let trigger_ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Color::soft_blue(),
        TriggerZone {
            is_triggered: false,
            speed: 4.0,
            axis: [0.0, 1.0, 0.0],
            original_position: [0.0, 0.0, 0.0],
            target_position: [0.0, 4.0, 0.0],
            ping_pong_forward: true,
        },
    ));

    let other_ent = world.spawn((Position::new(0.0, 0.0, 0.0),));

    // Send TriggerEnter event
    event_bus.send(ae_core::events::TriggerEnter {
        entity_a: trigger_ent,
        entity_b: other_ent,
    });

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.5,
    });

    let zone = world.get::<&TriggerZone>(trigger_ent).unwrap();
    assert!(zone.is_triggered, "Trigger zone should be marked triggered");

    let pos = world.get::<&Position>(trigger_ent).unwrap();
    assert!(
        pos.y > 0.5,
        "Trigger zone should begin moving up towards target Y"
    );
}

#[test]
fn test_trigger_zone_step_on_and_step_off_elevation_cycle() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    // 1. Sensor Pad
    let sensor_pad = world.spawn((
        Position::new(10.0, 0.55, -4.0),
        Scale::new(4.0, 0.1, 4.0),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 10.0, 0.5],
            },
            friction: 0.0,
            restitution: 0.0,
            is_sensor: true,
        },
        TriggerZone::new(),
    ));

    // 2. Sliding Door
    let door = world.spawn((
        Position::new(10.0, 2.0, -7.0),
        ae_core::ecs::Collider {
            shape: ae_core::ecs::ColliderShape::Box {
                half_extents: [0.5, 0.5, 0.5],
            },
            friction: 0.5,
            restitution: 0.0,
            is_sensor: false,
        },
        TriggerZone {
            is_triggered: false,
            speed: 10.0,
            axis: [0.0, 1.0, 0.0],
            original_position: [10.0, 2.0, -7.0],
            target_position: [10.0, 6.0, -7.0],
            ping_pong_forward: true,
        },
    ));

    // 3. Player entity stepping on sensor pad (10.0, 0.55, -4.0)
    let player = world.spawn((Position::new(10.0, 1.0, -4.0), ae_core::ecs::PlayerTag));

    // Frame 1: Player is on the pad -> Door raises up
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.5,
    });

    let is_triggered = world.get::<&TriggerZone>(sensor_pad).unwrap().is_triggered;
    assert!(is_triggered, "Pad must be triggered when player is on it");
    let door_pos = *world.get::<&Position>(door).unwrap();
    assert!(
        door_pos.y > 2.0,
        "Door must move up towards 6.0 when player is on pad"
    );

    // Frame 2: Player steps OFF the pad to (50.0, 1.0, 50.0) -> Door lowers back down
    if let Ok(mut pos) = world.get::<&mut Position>(player) {
        pos.x = 50.0;
        pos.z = 50.0;
    }

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 1.0,
    });

    let is_triggered = world.get::<&TriggerZone>(sensor_pad).unwrap().is_triggered;
    assert!(
        !is_triggered,
        "Pad must NOT be triggered when player steps off"
    );
    let door_pos = *world.get::<&Position>(door).unwrap();
    assert!(
        door_pos.y <= 2.05,
        "Door must lower back down to original position (2.0) after player steps off"
    );
}

#[test]
fn test_destructible_target_damage_and_destruction_event() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let target_ent = world.spawn((
        Position::new(0.0, 0.0, 5.0),
        Color::red(),
        DestructibleTarget::new(50.0),
    ));

    // Send 30 damage hit
    event_bus.send(RaycastHitEvent {
        shooter: None,
        target: target_ent,
        hit_point: [0.0, 0.0, 4.5],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 30.0,
    });

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    {
        let target = world.get::<&DestructibleTarget>(target_ent).unwrap();
        assert_eq!(
            target.health, 20.0,
            "Health should drop to 20.0 after 30 damage"
        );
        assert!(
            target.hit_flash_timer > 0.0,
            "Hit flash timer should be active"
        );
    }

    // Send fatal 25 damage hit
    event_bus.send(RaycastHitEvent {
        shooter: None,
        target: target_ent,
        hit_point: [0.0, 0.0, 4.5],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 25.0,
    });

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    let target = world.get::<&DestructibleTarget>(target_ent).unwrap();
    assert_eq!(target.health, 0.0, "Health should clamp to 0.0");

    let destruction_events = event_bus.receive::<TargetDestroyedEvent>();
    assert!(
        destruction_events.is_some(),
        "TargetDestroyedEvent should be broadcast"
    );
    let events = destruction_events.unwrap();
    assert_eq!(events[0].target, target_ent);
}

#[test]
fn test_destructible_target_hit_flash_restores_original_color() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let target_ent = world.spawn((
        Position::new(0.0, 0.0, 5.0),
        Color::red(), // Original color is Red (1.0, 0.2, 0.2, 1.0)
        DestructibleTarget::new(100.0),
    ));

    // Send 10 damage hit
    event_bus.send(RaycastHitEvent {
        shooter: None,
        target: target_ent,
        hit_point: [0.0, 0.0, 4.5],
        hit_normal: [0.0, 0.0, -1.0],
        damage: 10.0,
    });

    // Step 1: Hit applied -> flash active
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    {
        let col = world.get::<&Color>(target_ent).unwrap();
        // During flash, color is bright impact color (not original red)
        assert_eq!(col.r, 1.0);
        assert_eq!(col.g, 0.9);
    }

    // Step 2: Simulate 300ms passing -> flash timer expires
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.30,
    });

    {
        let col = world.get::<&Color>(target_ent).unwrap();
        // After flash expires, original Red color must be 100% restored
        assert_eq!(col.r, 1.0, "Red channel must be 1.0");
        assert_eq!(col.g, 0.2, "Green channel must be 0.2 (Color::red)");
        assert_eq!(col.b, 0.2, "Blue channel must be 0.2 (Color::red)");
    }
}

#[test]
fn test_entities_without_character_action_do_not_shoot() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let mut input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    // Spawn player entity with ONLY PlayerTag (NO CharacterAction)
    let _player_ent = world.spawn((
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::PlayerTag,
    ));

    // Simulate Fire button press
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    // Verify NO EphemeralProjectile was spawned
    let projectile_count = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count, 0,
        "Entities without CharacterAction must NOT shoot projectiles"
    );

    // Now attach CharacterAction and verify it CAN shoot
    let shooter_ent = world.spawn((
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::CharacterAction::default(),
    ));

    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.016,
    });

    let projectile_count_after = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count_after, 1,
        "Entity with CharacterAction MUST spawn projectile on fire"
    );
    let _ = shooter_ent;
}

#[derive(Default)]
struct MockActorBehavior {
    start_invoked: bool,
    update_count: u32,
}

impl ae_core::behavior::Behavior for MockActorBehavior {
    fn on_start(
        &mut self,
        _entity: hecs::Entity,
        ctx: &mut ae_core::behavior::BehaviorContext<'_>,
    ) {
        self.start_invoked = true;
        // Schedule spawning a companion entity via deferred command buffer
        ctx.commands.spawn_with(|w| {
            w.spawn((
                Position::new(10.0, 20.0, 30.0),
                ae_core::ecs::Name("CompanionEntity".to_string()),
            ))
        });
    }

    fn on_update(
        &mut self,
        entity: hecs::Entity,
        ctx: &mut ae_core::behavior::BehaviorContext<'_>,
        dt: f32,
    ) {
        self.update_count += 1;
        if let Ok(mut pos) = ctx.world.get::<&mut Position>(entity) {
            pos.y += 10.0 * dt;
        }
    }
}

#[test]
fn test_native_behavior_lifecycle_and_deferred_commands() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let actor_ent = world.spawn((
        Position::new(0.0, 0.0, 0.0),
        Rotation::identity(),
        ae_core::behavior::NativeBehavior::new(MockActorBehavior::default()),
    ));

    // Execute first frame
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    // 1. Verify position was updated by on_update
    let actor_pos = *world.get::<&Position>(actor_ent).unwrap();
    assert!(
        (actor_pos.y - 1.0).abs() < 0.01,
        "Actor Y should increase by 1.0 unit (10.0 * 0.1)"
    );

    // 2. Verify companion entity was spawned by on_start via CommandBuffer
    let mut companion_found = false;
    for (pos, name) in world.query::<(&Position, &ae_core::ecs::Name)>().iter() {
        if name.0 == "CompanionEntity" {
            assert_eq!(pos.x, 10.0);
            assert_eq!(pos.y, 20.0);
            assert_eq!(pos.z, 30.0);
            companion_found = true;
        }
    }

    assert!(
        companion_found,
        "Companion entity spawned via command buffer must exist in World"
    );

    // Execute second frame to verify on_start is NOT called again
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let actor_pos_frame2 = *world.get::<&Position>(actor_ent).unwrap();
    assert!(
        (actor_pos_frame2.y - 2.0).abs() < 0.01,
        "Actor Y should now be 2.0"
    );
}

#[test]
fn test_entity_command_buffer_direct_operations() {
    let mut world = World::new();
    let mut cmd = ae_core::commands::EntityCommandBuffer::new();

    let ent1 = world.spawn((Position::new(0.0, 0.0, 0.0),));
    let ent2 = world.spawn((Position::new(5.0, 5.0, 5.0),));

    // Queue insert_one, remove_one, despawn, and spawn_with
    cmd.insert_one(ent1, ae_core::ecs::Color::red());
    cmd.remove_one::<Position>(ent2);
    cmd.despawn(ent1);
    cmd.spawn_with(|w| w.spawn((Position::new(100.0, 100.0, 100.0), ae_core::ecs::PlayerTag)));

    assert_eq!(cmd.len(), 4);
    assert!(!cmd.is_empty());

    // Apply commands to world
    cmd.apply(&mut world);
    assert!(cmd.is_empty());

    // ent1 must be despawned
    assert!(!world.contains(ent1));

    // ent2 must no longer have Position
    assert!(world.get::<&Position>(ent2).is_err());

    // New spawned entity with PlayerTag must exist at 100,100,100
    let mut player_found = false;
    for (pos, _) in world
        .query::<(&Position, &ae_core::ecs::PlayerTag)>()
        .iter()
    {
        if pos.x == 100.0 && pos.y == 100.0 && pos.z == 100.0 {
            player_found = true;
        }
    }
    assert!(player_found, "Deferred spawned player entity must exist");
}

#[test]
fn test_character_action_cooldown_and_speed() {
    let mut world = World::new();
    let mut physics = PhysicsWorld::new();
    let mut input = InputManager::new();
    let mut event_bus = DynamicEventBus::new();

    let _shooter = world.spawn((
        Position::new(0.0, 1.0, 0.0),
        Rotation::identity(),
        ae_core::ecs::CharacterAction {
            speed: 80.0,
            cooldown: 1.0,
            timer: 0.0,
            axis: [0.0, 0.0, -1.0],
        },
    ));

    // First shot (timer = 0.0, ready to fire)
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let mut bolt_velocity = None;
    for (vel, _) in world
        .query::<(&Velocity, &ae_core::ecs::EphemeralProjectile)>()
        .iter()
    {
        bolt_velocity = Some(*vel);
    }
    assert!(
        bolt_velocity.is_some(),
        "Projectile must be spawned on first shot"
    );
    let vel = bolt_velocity.unwrap();
    assert!(
        (vel.z - 80.0).abs() < 0.1,
        "Projectile speed must match configured speed 80.0 m/s"
    );

    // Second shot immediately (cooldown active: 1.0 - 0.1 = 0.9s remaining)
    input.end_frame();
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let projectile_count = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count, 1,
        "Second shot must be blocked by active cooldown"
    );

    // Advance time by 1.0s and release key so next press is registered
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Released,
    );
    input.end_frame();
    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 1.0,
    });

    // Third shot after cooldown has elapsed
    input.process_key_event(
        ae_editor::input::KeyCode::KeyF,
        winit::event::ElementState::Pressed,
    );

    update_gameplay_behaviors(BehaviorRunnerParams {
        world: &mut world,
        physics_world: &mut physics,
        input: &input,
        event_bus: &mut event_bus,
        camera_forward: cgmath::Vector3::unit_z(),
        delta_time: 0.1,
    });

    let projectile_count_after = world
        .query::<&ae_core::ecs::EphemeralProjectile>()
        .into_iter()
        .count();
    assert_eq!(
        projectile_count_after, 2,
        "Third shot must succeed after cooldown expiration"
    );
}

#[test]
fn test_game_state_machine_custom_stack_flow() {
    use ae_core::state::{GameState, StateContext, StateManager, StateTransition};

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
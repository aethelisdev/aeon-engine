// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Deterministic unit and fault-injection tests for the swappable `ae_ecs` layer.

use crate::{
    ComponentError, Entity, EntityBuilder, EntityCommandBuffer, NoSuchEntity, QueryOneError, World,
};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone, PartialEq)]
struct Label(String);

#[test]
fn test_world_spawn_query_builder_and_mutations() {
    let mut world = World::new();
    let e1 = world.spawn((Position(1.0, 2.0, 3.0), Velocity(4.0, 0.0, -2.0)));

    let mut builder = EntityBuilder::new();
    builder
        .add(Position(10.0, 0.0, 0.0))
        .add(Label("Runner".to_string()));
    assert!(builder.has::<Label>());
    let e2 = world.spawn(builder.build());

    assert_eq!(world.len(), 2);
    assert!(world.contains(e1));
    assert!(world.contains(e2));

    // Mutate via query_mut including ae_ecs::Entity + Option<&Velocity>
    let mut visited_ids = Vec::new();
    for (ent, pos, vel) in world.query_mut::<(Entity, &mut Position, Option<&Velocity>)>() {
        visited_ids.push(ent);
        if let Some(v) = vel {
            pos.0 += v.0;
            pos.1 += v.1;
            pos.2 += v.2;
        }
    }
    assert_eq!(visited_ids.len(), 2);
    assert_eq!(
        world.get_copied::<Position>(e1),
        Ok(Position(5.0, 2.0, 1.0))
    );
    assert_eq!(
        world.get_copied::<Position>(e2),
        Ok(Position(10.0, 0.0, 0.0))
    );

    // Direct exclusive &mut reference mutation without borrow guard wrapper
    world
        .get_mut::<Position>(e2)
        .expect("e2 position mutable ref")
        .1 = 7.0;
    assert_eq!(world.inspect::<Position, f32>(e2, |p| p.1), Ok(7.0));

    // Reconstruct entity bits roundtrip
    let bits = e1.to_bits().get();
    assert_eq!(Entity::from_bits(bits), Some(e1));

    // Command buffer deferred insert/remove/despawn
    let mut cmds = EntityCommandBuffer::new();
    cmds.insert_one(e2, Velocity(1.0, 2.0, 3.0));
    cmds.despawn(e1);
    assert_eq!(cmds.len(), 2);
    cmds.apply(&mut world);
    assert!(cmds.is_empty());

    assert!(!world.contains(e1));
    assert_eq!(
        world.get_copied::<Velocity>(e2),
        Ok(Velocity(1.0, 2.0, 3.0))
    );
}

#[test]
fn test_ecs_rejects_dangling_zero_bits_and_missing_components() {
    let mut world = World::new();
    let live = world.spawn((Position(0.0, 0.0, 0.0),));

    // Negative / fault-injection: zero bits never form a valid entity
    assert_eq!(Entity::from_bits(0), None);

    // Negative: operating on DANGLING entity safely returns NoSuchEntity / Unsatisfied
    assert!(!world.contains(Entity::DANGLING));
    assert_eq!(world.despawn(Entity::DANGLING), Err(NoSuchEntity));
    assert!(matches!(
        world.inspect::<Position, f32>(Entity::DANGLING, |p| p.0),
        Err(ComponentError::NoSuchEntity)
    ));
    assert_eq!(
        world.insert_one(Entity::DANGLING, Position(1.0, 1.0, 1.0)),
        Err(NoSuchEntity)
    );

    // Negative: removing or querying an absent component reports MissingComponent / Unsatisfied
    assert!(matches!(
        world.remove_one::<Velocity>(live),
        Err(ComponentError::MissingComponent(_))
    ));
    assert_eq!(
        world.query_one_mut::<&Velocity>(live),
        Err(QueryOneError::Unsatisfied)
    );
    assert_eq!(
        world.get_mut::<Velocity>(live),
        Err(QueryOneError::Unsatisfied)
    );
    assert_eq!(
        world.query_one_mut::<&Position>(Entity::DANGLING),
        Err(QueryOneError::NoSuchEntity)
    );
}
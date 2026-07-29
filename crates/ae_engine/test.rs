// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
fn main() {
    let mut world = hecs::World::new();
    world.spawn((1i32, 1.0f32));
    for (entity, i, f) in world.query_mut::<(hecs::Entity, &i32, &f32)>() {
        println!("{:?} {} {}", entity, i, f);
    }
}
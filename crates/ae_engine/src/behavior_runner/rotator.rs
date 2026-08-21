// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rotator subsystem for angular rotation of spinning objects and player entities.
//!

use ae_core::ecs::{BehaviorComponent, Rotation};
use ae_physics::world::PhysicsWorld;
use cgmath::{InnerSpace, Rotation3};
use hecs::{Entity, World};

/// Updates continuous angular rotation for entities with `BehaviorType::Rotator`.
pub fn update_rotators(
    world: &mut World,
    physics_world: &mut PhysicsWorld,
    rotator_entities: &[Entity],
    dt: f32,
    dirty_entities: &mut Vec<Entity>,
) {
    for &ent in rotator_entities {
        if let Ok(mut behavior) = world.get::<&mut BehaviorComponent>(ent) {
            let speed = behavior.speed;
            let axis = behavior.axis;
            behavior.timer += dt;

            if let Ok(mut rot) = world.get::<&mut Rotation>(ent) {
                let angle_rad = speed * dt;
                let axis_vec = cgmath::Vector3::new(axis[0], axis[1], axis[2]);
                let norm_axis = if axis_vec.magnitude2() > 0.001 {
                    axis_vec.normalize()
                } else {
                    cgmath::Vector3::unit_y()
                };

                let delta_q =
                    cgmath::Quaternion::from_axis_angle(norm_axis, cgmath::Rad(angle_rad));
                let cur_q = cgmath::Quaternion::new(rot.w, rot.x, rot.y, rot.z);
                let new_q = (cur_q * delta_q).normalize();

                rot.x = new_q.v.x;
                rot.y = new_q.v.y;
                rot.z = new_q.v.z;
                rot.w = new_q.s;

                if let Some(&handle) = physics_world.entity_to_body.get(&ent)
                    && let Some(body) = physics_world.rigid_body_set.get_mut(handle)
                {
                    let rot_glam = ae_physics::glam::Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w);
                    body.set_rotation(rot_glam, true);
                }
                dirty_entities.push(ent);
            }
        }
    }
}
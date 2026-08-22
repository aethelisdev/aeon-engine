// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Rotator subsystem for angular rotation of spinning objects.
//!

use ae_core::ecs::{Rotation, Rotator};
use ae_physics::world::PhysicsWorld;
use cgmath::{InnerSpace, Rotation3};
use hecs::{Entity, World};

/// Updates continuous angular rotation for entities holding a `Rotator` component.
pub fn update_rotators(
    world: &mut World,
    physics_world: &mut PhysicsWorld,
    dt: f32,
    dirty_entities: &mut Vec<Entity>,
) {
    for (ent, rot, rotator) in world.query_mut::<(Entity, &mut Rotation, &Rotator)>() {
        let speed = rotator.speed;
        let axis = rotator.axis;

        let angle_rad = speed * dt;
        let axis_vec = cgmath::Vector3::new(axis[0], axis[1], axis[2]);
        let norm_axis = if axis_vec.magnitude2() > 0.001 {
            axis_vec.normalize()
        } else {
            cgmath::Vector3::unit_y()
        };

        let delta_q = cgmath::Quaternion::from_axis_angle(norm_axis, cgmath::Rad(angle_rad));
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
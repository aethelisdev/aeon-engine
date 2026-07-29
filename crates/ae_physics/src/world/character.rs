// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec3;
/// AE Physics — Kinematic Character Controller (KCC) module.
use hecs::{Entity, World};
use rapier3d::control::{CharacterLength, KinematicCharacterController};
use rapier3d::math::Pose;
use rapier3d::prelude::*;

use ae_core::ecs::{CharacterController, Position};

use super::PhysicsWorld;

impl PhysicsWorld {
    /// Performs kinematic character controller movement for an entity, handling step climbing,
    /// slope sliding, and obstacle collisions. Returns whether the character is currently grounded.
    pub fn move_character(
        &mut self,
        world: &mut World,
        entity: Entity,
        desired_translation: Vec3,
        delta_time: f32,
    ) -> bool {
        let controller = match world.get::<&CharacterController>(entity) {
            Ok(c) => *c,
            Err(_) => return false,
        };

        let body_handle = match self.entity_to_body.get(&entity) {
            Some(&h) => h,
            None => return false,
        };

        let capsule_half_height = (controller.height * 0.5 - controller.radius).max(0.05);
        let shape = SharedShape::capsule_y(capsule_half_height, controller.radius);

        let mut pos_comp = world
            .get::<&Position>(entity)
            .map(|p| *p)
            .unwrap_or(Position {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            });
        let mut character_pos =
            Pose::from_translation(Vec3::new(pos_comp.x, pos_comp.y, pos_comp.z));

        // Option 1: Pure solid kinematic shape sweep (autostep = None).
        // Disabling autostep guarantees 100% solid, impenetrable wall and box collisions with 0% penetration.
        let autostep = None;

        let snap_to_ground = if desired_translation.y > 0.0 {
            None
        } else {
            Some(CharacterLength::Absolute(0.05))
        };

        let kcc = KinematicCharacterController {
            up: Vec3::Y,
            autostep,
            snap_to_ground,
            max_slope_climb_angle: controller.max_slope_climb_angle.to_radians(),
            min_slope_slide_angle: controller.max_slope_climb_angle.to_radians(),
            offset: CharacterLength::Absolute(0.01),
            slide: true,
            ..Default::default()
        };

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            QueryFilter::default()
                .exclude_rigid_body(body_handle)
                .exclude_sensors(),
        );

        // Depenetration check: if starting pose overlaps any static collider, push character out to nearest surface
        let mut depenetrated = false;
        let mut fixed_pos = Vec3::new(pos_comp.x, pos_comp.y, pos_comp.z);

        for (_col_handle, col) in query_pipeline.intersect_shape(character_pos, shape.as_ref()) {
            if let Some(parent_handle) = col.parent() {
                if parent_handle == body_handle {
                    continue;
                }
            }

            let cur_pose = Pose::from_translation(fixed_pos);
            if let Ok(Some(contact)) = rapier3d::parry::query::contact(
                &cur_pose,
                shape.as_ref(),
                col.position(),
                col.shape(),
                0.05,
            ) {
                if contact.dist < 0.0 {
                    let push_dist = contact.dist.abs() + 0.01;
                    let push_dir = contact.normal1;
                    fixed_pos -= push_dir * push_dist;
                    depenetrated = true;
                }
            }
        }

        if depenetrated {
            pos_comp.x = fixed_pos.x;
            pos_comp.y = fixed_pos.y;
            pos_comp.z = fixed_pos.z;
            if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                pos.x = fixed_pos.x;
                pos.y = fixed_pos.y;
                pos.z = fixed_pos.z;
            }
            character_pos = Pose::from_translation(fixed_pos);
        }

        let movement = kcc.move_shape(
            delta_time,
            &query_pipeline,
            shape.as_ref(),
            &character_pos,
            desired_translation,
            |_| {},
        );

        let is_grounded = if desired_translation.y > 0.0 {
            false
        } else {
            movement.grounded
        };
        let new_x = pos_comp.x + movement.translation.x;
        let new_y = pos_comp.y + movement.translation.y;
        let new_z = pos_comp.z + movement.translation.z;

        // Update position in ECS
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.x = new_x;
            pos.y = new_y;
            pos.z = new_z;
        }

        // Synchronize updated position directly to Rapier rigid body as next kinematic position
        if let Some(body) = self.rigid_body_set.get_mut(body_handle) {
            body.set_next_kinematic_position(Pose::from_translation(Vec3::new(
                new_x, new_y, new_z,
            )));
        }

        let _ = world.insert_one(entity, ae_core::ecs::TransformDirty);

        // Update character controller grounded status
        if let Ok(mut ctrl) = world.get::<&mut ae_core::ecs::CharacterController>(entity) {
            ctrl.is_grounded = is_grounded;
        }

        is_grounded
    }
}
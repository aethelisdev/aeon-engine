// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use ae_core::ecs::{CharacterController, Position, Rotation, Velocity};
/// AE Physics — Rapier3D to ECS synchronization module.
use hecs::World;

use super::PhysicsWorld;

impl PhysicsWorld {
    /// Synchronizes updated positions and velocities from the Rapier simulation back to ECS components.
    /// Ignores Kinematic Character Controller (KCC) entities as their movement is managed by `move_character`.
    pub fn sync_physics_to_ecs(&mut self, world: &mut World) {
        for (&entity, &handle) in &self.entity_to_body {
            if !world.contains(entity) {
                continue;
            }
            if let Some(body) = self.rigid_body_set.get(handle) {
                let is_kcc = world.get::<&CharacterController>(entity).is_ok();
                if !is_kcc && (body.is_dynamic() || body.is_kinematic()) {
                    let pose = body.position();
                    let translation = pose.translation;
                    let rotation = pose.rotation;

                    if let Ok(mut pos) = world.get::<&mut Position>(entity) {
                        pos.x = translation.x;
                        pos.y = translation.y;
                        pos.z = translation.z;
                    }
                    if let Ok(mut rot) = world.get::<&mut Rotation>(entity) {
                        rot.x = rotation.x;
                        rot.y = rotation.y;
                        rot.z = rotation.z;
                        rot.w = rotation.w;
                    }
                    if let Ok(mut gt) = world.get::<&mut ae_core::ecs::GlobalTransform>(entity) {
                        let sx = world
                            .get::<&ae_core::ecs::Scale>(entity)
                            .map(|s| s.x)
                            .unwrap_or(1.0);
                        let sy = world
                            .get::<&ae_core::ecs::Scale>(entity)
                            .map(|s| s.y)
                            .unwrap_or(1.0);
                        let sz = world
                            .get::<&ae_core::ecs::Scale>(entity)
                            .map(|s| s.z)
                            .unwrap_or(1.0);

                        let mat = cgmath::Matrix4::from_translation(cgmath::Vector3::new(
                            translation.x,
                            translation.y,
                            translation.z,
                        )) * cgmath::Matrix4::from(cgmath::Quaternion::new(
                            rotation.w, rotation.x, rotation.y, rotation.z,
                        )) * cgmath::Matrix4::from_nonuniform_scale(sx, sy, sz);
                        gt.0 = mat;
                    }
                    if body.is_dynamic() {
                        if let Ok(mut vel) = world.get::<&mut Velocity>(entity) {
                            let linvel = body.linvel();
                            vel.x = linvel.x;
                            vel.y = linvel.y;
                            vel.z = linvel.z;
                        }
                    }
                }
            }
        }
    }

    /// Force-resets all Rapier3D rigid body positions and velocities from current ECS transforms.
    /// Useful when toggling Play/Edit mode or restoring scene backups.
    pub fn reset_simulation_poses(&mut self, world: &mut World) {
        use glam::{Quat, Vec3};
        use rapier3d::math::Pose;

        // Recompute world-space transforms for all entities in scene before resetting physics poses
        ae_core::ecs::update_hierarchy_transforms(world);

        for (&entity, &handle) in &self.entity_to_body {
            let (world_pos, world_rot) =
                if let Ok(gt) = world.get::<&ae_core::ecs::GlobalTransform>(entity) {
                    let mat = gt.0;
                    let trans = Vec3::new(mat.w.x, mat.w.y, mat.w.z);
                    let sx = Vec3::new(mat.x.x, mat.x.y, mat.x.z).length().max(1e-4);
                    let sy = Vec3::new(mat.y.x, mat.y.y, mat.y.z).length().max(1e-4);
                    let sz = Vec3::new(mat.z.x, mat.z.y, mat.z.z).length().max(1e-4);
                    let rot_mat3 = glam::Mat3::from_cols(
                        Vec3::new(mat.x.x / sx, mat.x.y / sx, mat.x.z / sx),
                        Vec3::new(mat.y.x / sy, mat.y.y / sy, mat.y.z / sy),
                        Vec3::new(mat.z.x / sz, mat.z.y / sz, mat.z.z / sz),
                    );
                    let rot = Quat::from_mat3(&rot_mat3);
                    (trans, rot)
                } else {
                    let pos_comp = world
                        .get::<&Position>(entity)
                        .map(|p| *p)
                        .unwrap_or(Position {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        });
                    let rot_comp = world
                        .get::<&Rotation>(entity)
                        .map(|r| *r)
                        .unwrap_or_else(|_| Rotation::identity());
                    (
                        Vec3::new(pos_comp.x, pos_comp.y, pos_comp.z),
                        Quat::from_xyzw(rot_comp.x, rot_comp.y, rot_comp.z, rot_comp.w),
                    )
                };

            let vel_comp = world
                .get::<&Velocity>(entity)
                .map(|v| *v)
                .unwrap_or(Velocity {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                });
            let final_pose = Pose::from_parts(world_pos, world_rot);

            if let Some(body) = self.rigid_body_set.get_mut(handle) {
                body.set_position(final_pose, true);
                body.set_linvel(Vec3::new(vel_comp.x, vel_comp.y, vel_comp.z), true);
                body.set_angvel(Vec3::ZERO, true);
            }
        }
    }
}
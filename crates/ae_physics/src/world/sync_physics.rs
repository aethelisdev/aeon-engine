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
                        let (sx, sy, sz) = world
                            .get::<&ae_core::ecs::Scale>(entity)
                            .map(|s| (s.x, s.y, s.z))
                            .unwrap_or((1.0, 1.0, 1.0));

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
        use glam::Vec3;
        use rapier3d::math::Pose;

        // Recompute world-space transforms for all entities in scene before resetting physics poses
        ae_core::ecs::update_hierarchy_transforms(world);

        for (&entity, &handle) in &self.entity_to_body {
            let (world_pos, world_rot) = if let Ok(gt) =
                world.get::<&ae_core::ecs::GlobalTransform>(entity)
            {
                let (trans, rot, _scale) = ae_core::math::conversions::matrix4_to_glam_trs(gt.0);
                (trans, rot)
            } else {
                use ae_core::math::conversions::ToGlam;
                let pos_comp = world
                    .get::<&Position>(entity)
                    .map(|p| p.to_glam())
                    .unwrap_or(glam::Vec3::ZERO);
                let rot_comp = world
                    .get::<&Rotation>(entity)
                    .map(|r| r.to_glam())
                    .unwrap_or(glam::Quat::IDENTITY);
                (pos_comp, rot_comp)
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
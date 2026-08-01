// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use glam::Vec3;
use hecs::{Entity, World};
use rapier3d::prelude::*;
/// AE Physics — ECS to Rapier3D synchronization module.
use std::collections::HashMap;

use ae_core::ecs::{
    AssetHandle, CharacterController, Collider, ColliderShape, ModelId, Position, RigidBody,
    RigidBodyType, Rotation, Scale, Shape, TransformDirty, Velocity,
};

use super::PhysicsWorld;

impl PhysicsWorld {
    /// Synchronizes ECS entities having `RigidBody` or `Collider` components to the Rapier simulation.
    /// Automatically spawns, updates, or deletes physical bodies in the Rapier world.
    /// Resolves mesh data for `Trimesh` and `ConvexHull` colliders using `get_mesh_data`.
    /// Includes the Upward Depenetration Guard for initial dynamic spawns.
    pub fn sync_ecs_to_physics<'a, F>(&mut self, world: &mut World, get_mesh_data: F)
    where
        F: Fn(AssetHandle) -> Option<(&'a [[f32; 3]], &'a [u32])>,
    {
        // 1. Remove deleted entities from simulation
        let mut to_remove = Vec::new();
        for (&entity, _) in &self.entity_to_body {
            let is_active = world.get::<&RigidBody>(entity).is_ok()
                || world.get::<&Collider>(entity).is_ok()
                || world.get::<&CharacterController>(entity).is_ok()
                || world.get::<&Shape>(entity).is_ok()
                || world.get::<&ModelId>(entity).is_ok();
            if !world.contains(entity) || !is_active {
                to_remove.push(entity);
            }
        }
        for entity in to_remove {
            if let Some(handle) = self.entity_to_body.remove(&entity) {
                self.body_to_entity.remove(&handle);
                self.rigid_body_set.remove(
                    handle,
                    &mut self.island_manager,
                    &mut self.collider_set,
                    &mut self.impulse_joint_set,
                    &mut self.multibody_joint_set,
                    true,
                );
            }
        }

        // 2. Add or update active bodies
        // Pre-allocate map capacity to avoid re-allocations based on active physics bodies
        let mut active_entities = HashMap::with_capacity(self.entity_to_body.len().max(16));
        for (entity, rb) in world.query::<(Entity, &RigidBody)>().iter() {
            active_entities.insert(entity, (Some(*rb), None));
        }
        for (entity, col) in world.query::<(Entity, &Collider)>().iter() {
            active_entities
                .entry(entity)
                .and_modify(|(_, c)| *c = Some(*col))
                .or_insert((None, Some(*col)));
        }
        for (entity, _ctrl) in world.query::<(Entity, &CharacterController)>().iter() {
            active_entities.entry(entity).or_insert((None, None));
        }
        for (entity, _shape) in world.query::<(Entity, &Shape)>().iter() {
            active_entities.entry(entity).or_insert((None, None));
        }
        for (entity, _model) in world.query::<(Entity, &ModelId)>().iter() {
            active_entities.entry(entity).or_insert((None, None));
        }

        for (entity, (rb_comp, col_comp)) in active_entities {
            let (world_pos, world_rot, scale_comp) =
                if let Ok(gt) = world.get::<&ae_core::ecs::GlobalTransform>(entity) {
                    let (trans, rot, scale) = ae_core::math::conversions::matrix4_to_glam_trs(gt.0);
                    (trans, rot, [scale.x, scale.y, scale.z])
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
                    let scale_comp = world
                        .get::<&Scale>(entity)
                        .map(|s| [s.x, s.y, s.z])
                        .unwrap_or([1.0, 1.0, 1.0]);
                    (pos_comp, rot_comp, scale_comp)
                };

            let vel_comp = world
                .get::<&Velocity>(entity)
                .map(|v| *v)
                .unwrap_or(Velocity {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                });
            let is_dirty = world.get::<&TransformDirty>(entity).is_ok();

            let pose = Pose::from_parts(world_pos, world_rot);

            if let Some(&handle) = self.entity_to_body.get(&entity) {
                // Body already exists - update it if dirty or if settings changed
                let is_kcc = world.get::<&CharacterController>(entity).is_ok();
                if let Some(body) = self.rigid_body_set.get_mut(handle) {
                    if is_kcc {
                        if is_dirty {
                            body.set_next_kinematic_position(pose);
                        }
                    } else if body.is_fixed() {
                        let pos_diff = (body.position().translation - pose.translation).length();
                        let r1 = body.position().rotation;
                        let r2 = pose.rotation;
                        // Dot-product quaternion distance: handles double-cover (q and -q represent
                        // the same rotation), so .abs() prevents false positives on sign flips.
                        let rot_diff = 1.0
                            - (r1.w * r2.w + r1.x * r2.x + r1.y * r2.y + r1.z * r2.z).abs();
                        if pos_diff > 1e-4 || rot_diff > 1e-5 {
                            body.set_position(pose, false);
                        }
                    } else if is_dirty {
                        body.set_position(pose, true);
                        body.set_linvel(Vec3::new(vel_comp.x, vel_comp.y, vel_comp.z), true);
                    }

                    let expected_type = if is_kcc {
                        rapier3d::prelude::RigidBodyType::KinematicPositionBased
                    } else if let Some(rb) = rb_comp {
                        match rb.body_type {
                            RigidBodyType::Static => rapier3d::prelude::RigidBodyType::Fixed,
                            RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
                            RigidBodyType::Kinematic => {
                                rapier3d::prelude::RigidBodyType::KinematicPositionBased
                            }
                        }
                    } else {
                        rapier3d::prelude::RigidBodyType::Fixed
                    };

                    if body.body_type() != expected_type {
                        body.set_body_type(expected_type, true);
                    }

                    if let Some(rb) = rb_comp {
                        if (body.gravity_scale() - rb.gravity_scale).abs() > 1e-4 {
                            body.set_gravity_scale(rb.gravity_scale, true);
                        }
                    }

                    // Dynamically update existing colliders or rebuild if scale/shape changed
                    let col_builder = if is_kcc {
                        if let Ok(ctrl) = world.get::<&CharacterController>(entity) {
                            let capsule_half_height = ctrl.capsule_half_height();
                            let is_sensor = col_comp.map(|c| c.is_sensor).unwrap_or(false);
                            Some(
                                ColliderBuilder::capsule_y(capsule_half_height, ctrl.radius)
                                    .sensor(is_sensor),
                            )
                        } else {
                            None
                        }
                    } else {
                        Self::build_collider_for_entity(
                            col_comp,
                            rb_comp,
                            scale_comp,
                            entity,
                            world,
                            &get_mesh_data,
                        )
                    };

                    if let Some(cb) = col_builder {
                        let target_sensor = col_comp.map(|c| c.is_sensor).unwrap_or(false);
                        let target_friction = col_comp.map(|c| c.friction).unwrap_or(0.7);
                        let target_restitution = col_comp.map(|c| c.restitution).unwrap_or(0.0);

                        let mut needs_rebuild = body.colliders().is_empty();

                        let new_collider = cb.build();

                        if let Some(&c_h) = body.colliders().first() {
                            if let Some(c) = self.collider_set.get_mut(c_h) {
                                if c.is_sensor() != target_sensor {
                                    needs_rebuild = true;
                                } else {
                                    c.set_friction(target_friction);
                                    c.set_restitution(target_restitution);

                                    // Detect if scale or shape dimensions changed in ECS
                                    if c.shape().shape_type() != new_collider.shape().shape_type() {
                                        needs_rebuild = true;
                                    } else {
                                        if let (Some(old_b), Some(new_b)) = (
                                            c.shape().as_cuboid(),
                                            new_collider.shape().as_cuboid(),
                                        ) {
                                            let diff =
                                                (old_b.half_extents - new_b.half_extents).abs();
                                            if diff.x > 1e-3 || diff.y > 1e-3 || diff.z > 1e-3 {
                                                needs_rebuild = true;
                                            }
                                        }
                                        if let (Some(old_s), Some(new_s)) =
                                            (c.shape().as_ball(), new_collider.shape().as_ball())
                                        {
                                            if (old_s.radius - new_s.radius).abs() > 1e-3 {
                                                needs_rebuild = true;
                                            }
                                        }
                                        if let (Some(old_c), Some(new_c)) = (
                                            c.shape().as_capsule(),
                                            new_collider.shape().as_capsule(),
                                        ) {
                                            if (old_c.radius - new_c.radius).abs() > 1e-3
                                                || (old_c.half_height() - new_c.half_height()).abs()
                                                    > 1e-3
                                            {
                                                needs_rebuild = true;
                                            }
                                        }
                                    }
                                }
                            } else {
                                needs_rebuild = true;
                            }
                        } else {
                            needs_rebuild = true;
                        }

                        if needs_rebuild {
                            let old_colliders: Vec<_> = body.colliders().to_vec();
                            for c_h in old_colliders {
                                self.collider_set.remove(
                                    c_h,
                                    &mut self.island_manager,
                                    &mut self.rigid_body_set,
                                    true,
                                );
                            }
                            self.collider_set.insert_with_parent(
                                new_collider,
                                handle,
                                &mut self.rigid_body_set,
                            );
                        }
                    }
                }
            } else {
                let is_kcc = world.get::<&CharacterController>(entity).is_ok();
                let mut builder = if is_kcc {
                    RigidBodyBuilder::kinematic_position_based()
                } else {
                    match rb_comp {
                        Some(rb) => match rb.body_type {
                            RigidBodyType::Static => RigidBodyBuilder::fixed(),
                            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic()
                                .gravity_scale(rb.gravity_scale)
                                .additional_mass(rb.mass)
                                .linear_damping(0.5)
                                .angular_damping(0.5)
                                .ccd_enabled(true),
                            RigidBodyType::Kinematic => {
                                RigidBodyBuilder::kinematic_position_based()
                            }
                        },
                        None => RigidBodyBuilder::fixed(),
                    }
                };

                let pose = Pose::from_parts(world_pos, world_rot);
                builder = builder.pose(pose).user_data(entity.to_bits().get() as u128);

                if is_kcc {
                    builder = builder.lock_rotations();
                } else {
                    builder = builder.linvel(Vec3::new(vel_comp.x, vel_comp.y, vel_comp.z));
                }

                let handle = self.rigid_body_set.insert(builder);
                self.entity_to_body.insert(entity, handle);
                self.body_to_entity.insert(handle, entity);

                let col_builder = if is_kcc {
                    if let Ok(ctrl) = world.get::<&CharacterController>(entity) {
                        let capsule_half_height = ctrl.capsule_half_height();
                        let is_sensor = col_comp.map(|c| c.is_sensor).unwrap_or(false);
                        Some(
                            ColliderBuilder::capsule_y(capsule_half_height, ctrl.radius)
                                .sensor(is_sensor),
                        )
                    } else {
                        None
                    }
                } else {
                    Self::build_collider_for_entity(
                        col_comp,
                        rb_comp,
                        scale_comp,
                        entity,
                        world,
                        &get_mesh_data,
                    )
                };

                if let Some(cb) = col_builder {
                    self.collider_set
                        .insert_with_parent(cb, handle, &mut self.rigid_body_set);
                }
            }

            // Remove TransformDirty flag after successfully syncing
            let _ = world.remove_one::<TransformDirty>(entity);
        }
    }

    /// Helper method that constructs a Rapier `ColliderBuilder` for an entity based on its explicit `Collider` component
    /// or fallback inferred `Shape` / `ModelId` components.
    fn build_collider_for_entity<'a, F>(
        col_comp: Option<Collider>,
        rb_comp: Option<RigidBody>,
        scale_comp: [f32; 3],
        entity: Entity,
        world: &World,
        get_mesh_data: &F,
    ) -> Option<ColliderBuilder>
    where
        F: Fn(AssetHandle) -> Option<(&'a [[f32; 3]], &'a [u32])>,
    {
        if let Some(col) = col_comp {
            let effective_col =
                if matches!(rb_comp.map(|r| r.body_type), Some(RigidBodyType::Dynamic))
                    && matches!(col.shape, ColliderShape::Trimesh)
                {
                    Collider {
                        shape: ColliderShape::ConvexHull,
                        friction: col.friction,
                        restitution: col.restitution,
                        is_sensor: col.is_sensor,
                    }
                } else {
                    col
                };
            Some(Self::create_collider_builder(
                &effective_col,
                scale_comp,
                entity,
                world,
                get_mesh_data,
            ))
        } else if let Ok(shape) = world.get::<&Shape>(entity) {
            let fallback_col = match *shape {
                Shape::Cube => Collider {
                    shape: ColliderShape::Box {
                        half_extents: [0.5, 0.5, 0.5],
                    },
                    friction: 0.7,
                    restitution: 0.0,
                    is_sensor: false,
                },
                Shape::Sphere => Collider {
                    shape: ColliderShape::Sphere { radius: 0.5 },
                    friction: 0.7,
                    restitution: 0.0,
                    is_sensor: false,
                },
                Shape::Cylinder | Shape::Capsule => Collider {
                    shape: ColliderShape::Capsule {
                        half_height: 0.15,
                        radius: 0.35,
                    },
                    friction: 0.7,
                    restitution: 0.0,
                    is_sensor: false,
                },
                _ => Collider {
                    shape: ColliderShape::Box {
                        half_extents: [0.5, 0.5, 0.5],
                    },
                    friction: 0.7,
                    restitution: 0.0,
                    is_sensor: false,
                },
            };
            Some(Self::create_collider_builder(
                &fallback_col,
                scale_comp,
                entity,
                world,
                get_mesh_data,
            ))
        } else if world.get::<&ModelId>(entity).is_ok() {
            let fallback_shape =
                if matches!(rb_comp.map(|r| r.body_type), Some(RigidBodyType::Dynamic)) {
                    ColliderShape::ConvexHull
                } else {
                    ColliderShape::Trimesh
                };
            let fallback_col = Collider {
                shape: fallback_shape,
                friction: 0.7,
                restitution: 0.0,
                is_sensor: false,
            };
            Some(Self::create_collider_builder(
                &fallback_col,
                scale_comp,
                entity,
                world,
                get_mesh_data,
            ))
        } else {
            None
        }
    }

    /// Internal helper method to build scaled Rapier ColliderBuilder structures.
    pub(super) fn create_collider_builder<'a, F>(
        col: &Collider,
        scale: [f32; 3],
        entity: Entity,
        world: &World,
        get_mesh_data: &F,
    ) -> ColliderBuilder
    where
        F: Fn(AssetHandle) -> Option<(&'a [[f32; 3]], &'a [u32])>,
    {
        let sx = scale[0].abs().max(1e-4);
        let sy = scale[1].abs().max(1e-4);
        let sz = scale[2].abs().max(1e-4);

        let col_builder = match col.shape {
            ColliderShape::Box { half_extents } => ColliderBuilder::cuboid(
                half_extents[0] * sx,
                half_extents[1] * sy,
                half_extents[2] * sz,
            ),
            ColliderShape::Sphere { radius } => {
                let s = sx.max(sy).max(sz);
                ColliderBuilder::ball(radius * s)
            }
            ColliderShape::Capsule {
                half_height,
                radius,
            } => {
                let s_xz = sx.max(sz);
                ColliderBuilder::capsule_y(half_height * sy, radius * s_xz)
            }
            ColliderShape::Trimesh => {
                if let Ok(model_id) = world.get::<&ModelId>(entity) {
                    if let Some((vertices, indices)) = get_mesh_data(model_id.0) {
                        let triangles: Vec<[u32; 3]> = indices
                            .chunks_exact(3)
                            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                            .collect();
                        let rapier_vertices: Vec<Vec3> = vertices
                            .iter()
                            .map(|v| Vec3::new(v[0] * sx, v[1] * sy, v[2] * sz))
                            .collect();
                        match ColliderBuilder::trimesh(rapier_vertices, triangles) {
                            Ok(b) => b,
                            Err(err) => {
                                log::warn!(
                                    "Failed to build trimesh collider for entity {:?}: {:?}",
                                    entity,
                                    err
                                );
                                ColliderBuilder::cuboid(0.5 * sx, 0.5 * sy, 0.5 * sz)
                            }
                        }
                    } else {
                        log::warn!(
                            "Trimesh collider requested but model asset data is missing or not loaded yet for entity {:?}",
                            entity
                        );
                        ColliderBuilder::cuboid(0.5 * sx, 0.5 * sy, 0.5 * sz)
                    }
                } else {
                    log::warn!(
                        "Trimesh collider requested but entity {:?} lacks ModelId component",
                        entity
                    );
                    ColliderBuilder::cuboid(0.5 * sx, 0.5 * sy, 0.5 * sz)
                }
            }
            ColliderShape::ConvexHull => {
                if let Ok(model_id) = world.get::<&ModelId>(entity) {
                    if let Some((vertices, _)) = get_mesh_data(model_id.0) {
                        let rapier_vertices: Vec<Vec3> = vertices
                            .iter()
                            .map(|v| Vec3::new(v[0] * sx, v[1] * sy, v[2] * sz))
                            .collect();
                        if let Some(builder) = ColliderBuilder::convex_hull(&rapier_vertices) {
                            builder
                        } else {
                            log::warn!("Failed to generate convex hull for entity {:?}", entity);
                            ColliderBuilder::cuboid(0.5 * sx, 0.5 * sy, 0.5 * sz)
                        }
                    } else {
                        log::warn!(
                            "Convex hull requested but model asset data is missing or not loaded yet for entity {:?}",
                            entity
                        );
                        ColliderBuilder::cuboid(0.5 * sx, 0.5 * sy, 0.5 * sz)
                    }
                } else {
                    log::warn!(
                        "Convex hull requested but entity {:?} lacks ModelId component",
                        entity
                    );
                    ColliderBuilder::cuboid(0.5 * sx, 0.5 * sy, 0.5 * sz)
                }
            }
        };

        col_builder
            .friction(col.friction)
            .restitution(col.restitution)
            .sensor(col.is_sensor)
            .contact_skin(0.02)
            .active_events(ActiveEvents::COLLISION_EVENTS)
    }
}
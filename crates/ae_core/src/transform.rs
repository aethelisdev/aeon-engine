// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Transform hierarchy propagation and unconstrained velocity integration systems.
//!
//! Operates directly on [`ae_ecs::World`] using shared transform components from
//! [`ae_plugin_api`] without wrapping `World` or duplicating ECS types.

use ae_ecs::{Entity, World};
use ae_plugin_api::{
    Collider, GlobalTransform, Parent, Position, RigidBody, Rotation, Scale, TransformDirty,
    Velocity,
};
use cgmath::SquareMatrix;
use std::collections::{HashMap, HashSet};

/// Integrates linear velocity into `Position` for entities not controlled by the physics solver.
///
/// Entities holding neither `RigidBody` nor `Collider` advance by `vel * delta_time`
/// when velocity is non-zero.
pub fn integrate_unconstrained_velocities(world: &mut World, delta_time: f32) {
    for (pos, vel, rb, col) in world.query_mut::<(
        &mut Position,
        &Velocity,
        Option<&RigidBody>,
        Option<&Collider>,
    )>() {
        if rb.is_none() && col.is_none() && (vel.x != 0.0 || vel.y != 0.0 || vel.z != 0.0) {
            pos.x += vel.x * delta_time;
            pos.y += vel.y * delta_time;
            pos.z += vel.z * delta_time;
        }
    }
}

/// Builds a TRS matrix from optional local components, clamping near-zero scale axes to avoid
/// singular transforms.
fn compose_local_matrix(
    pos: Option<Position>,
    rot: Option<Rotation>,
    scale: Option<Scale>,
) -> cgmath::Matrix4<f32> {
    let p = pos
        .map(|p| cgmath::Vector3::new(p.x, p.y, p.z))
        .unwrap_or_else(|| cgmath::Vector3::new(0.0, 0.0, 0.0));
    let q = rot
        .map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
        .unwrap_or_else(|| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
    let min = 1e-4;
    let clamp_axis = |v: f32| {
        if v.abs() < min {
            f32::copysign(min, v)
        } else {
            v
        }
    };
    let s = scale
        .map(|s| cgmath::Vector3::new(clamp_axis(s.x), clamp_axis(s.y), clamp_axis(s.z)))
        .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

    cgmath::Matrix4::from_translation(p)
        * cgmath::Matrix4::from(q)
        * cgmath::Matrix4::from_nonuniform_scale(s.x, s.y, s.z)
}

/// Updates world-space [`GlobalTransform`] matrices for all parent-child hierarchies and
/// synchronizes standalone entities holding `GlobalTransform`.
///
/// Uses dirty propagation (`TransformDirty`): unchanged subtrees skip matrix multiplication.
pub fn update_hierarchy_transforms(world: &mut World) {
    let mut all_entities_with_parent = Vec::new();
    let mut parent_to_children = HashMap::<Entity, Vec<Entity>>::new();

    for (entity, parent_ref) in world.query::<(Entity, &Parent)>().iter() {
        if world.contains(parent_ref.0) {
            all_entities_with_parent.push((entity, parent_ref.0));
            parent_to_children
                .entry(parent_ref.0)
                .or_default()
                .push(entity);
        }
    }

    let mut visited = HashSet::new();

    if !all_entities_with_parent.is_empty() {
        let mut root_entities = HashSet::new();
        for &parent in parent_to_children.keys() {
            if !world.has::<Parent>(parent) {
                root_entities.insert(parent);
            }
        }

        fn propagate(
            world: &mut World,
            entity: Entity,
            parent_transform: &cgmath::Matrix4<f32>,
            parent_to_children: &HashMap<Entity, Vec<Entity>>,
            visited: &mut HashSet<Entity>,
            parent_is_dirty: bool,
        ) {
            if !visited.insert(entity) {
                return;
            }

            let self_is_dirty = world.has::<TransformDirty>(entity);
            let has_gt = world.has::<GlobalTransform>(entity);
            let is_dirty = parent_is_dirty || self_is_dirty || !has_gt;

            let global_matrix = if is_dirty {
                let local_matrix = compose_local_matrix(
                    world.get_copied::<Position>(entity).ok(),
                    world.get_copied::<Rotation>(entity).ok(),
                    world.get_copied::<Scale>(entity).ok(),
                );
                let g_mat = parent_transform * local_matrix;
                if let Ok(gt) = world.get_mut::<GlobalTransform>(entity) {
                    gt.0 = g_mat;
                } else {
                    let _ = world.insert_one(entity, GlobalTransform(g_mat));
                }
                g_mat
            } else {
                world
                    .get_copied::<GlobalTransform>(entity)
                    .map(|gt| gt.0)
                    .unwrap_or(*parent_transform)
            };

            if let Some(children) = parent_to_children.get(&entity) {
                for &child in children {
                    propagate(
                        world,
                        child,
                        &global_matrix,
                        parent_to_children,
                        visited,
                        is_dirty,
                    );
                }
            }
        }

        let identity = cgmath::Matrix4::identity();
        for root in root_entities {
            let root_is_dirty = world.has::<TransformDirty>(root);
            propagate(
                world,
                root,
                &identity,
                &parent_to_children,
                &mut visited,
                root_is_dirty,
            );
        }
    }

    for (entity, (gt, pos_opt, rot_opt, scale_opt)) in world
        .query_mut::<(
            Entity,
            (
                &mut GlobalTransform,
                Option<&Position>,
                Option<&Rotation>,
                Option<&Scale>,
            ),
        )>()
        .into_iter()
    {
        if !visited.contains(&entity) {
            gt.0 = compose_local_matrix(pos_opt.copied(), rot_opt.copied(), scale_opt.copied());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_hierarchy_transforms_standalone_sync() {
        let mut world = World::new();
        let entity = world.spawn((
            Position {
                x: 10.0,
                y: 5.0,
                z: -2.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            GlobalTransform(cgmath::Matrix4::identity()),
        ));

        update_hierarchy_transforms(&mut world);

        let gt = world
            .get_copied::<GlobalTransform>(entity)
            .expect("GlobalTransform query must succeed");
        assert_eq!(gt.0.w.x, 10.0);
        assert_eq!(gt.0.w.y, 5.0);
        assert_eq!(gt.0.w.z, -2.0);
    }

    #[test]
    fn test_update_hierarchy_transforms_dirty_propagation() {
        let mut world = World::new();
        let parent = world.spawn((
            Position {
                x: 5.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            TransformDirty,
        ));
        let child = world.spawn((
            Parent(parent),
            Position {
                x: 2.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation::identity(),
            Scale {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        ));

        update_hierarchy_transforms(&mut world);

        let child_gt = world
            .get_copied::<GlobalTransform>(child)
            .expect("Child GlobalTransform query must succeed");
        assert_eq!(child_gt.0.w.x, 7.0);
    }

    #[test]
    fn test_integrate_unconstrained_velocities_skips_physics_and_zero() {
        let mut world = World::new();
        let free_ent = world.spawn((
            Position::new(1.0, 2.0, 3.0),
            Velocity {
                x: 2.0,
                y: -4.0,
                z: 0.5,
            },
        ));
        let physics_ent = world.spawn((
            Position::new(1.0, 2.0, 3.0),
            Velocity {
                x: 10.0,
                y: 10.0,
                z: 10.0,
            },
            RigidBody::default(),
        ));

        integrate_unconstrained_velocities(&mut world, 0.5);

        let free_pos = world.get_copied::<Position>(free_ent).expect("free pos");
        assert_eq!([free_pos.x, free_pos.y, free_pos.z], [2.0, 0.0, 3.25]);

        let phys_pos = world.get_copied::<Position>(physics_ent).expect("phys pos");
        assert_eq!([phys_pos.x, phys_pos.y, phys_pos.z], [1.0, 2.0, 3.0]);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Parent-child hierarchy reparenting with world-space transform preservation.
//!
//! Converts world-space matrices back into local `Position`/`Rotation`/`Scale` components
//! when linking or unlinking entities so visual world placement never jumps.

use crate::hierarchy::update_hierarchy_transforms;
use crate::matrix::{decompose_matrix, get_world_matrix};
use ae_ecs::{Entity, World};
use ae_plugin_api::{Children, GlobalTransform, Parent, TransformDirty};
use cgmath::SquareMatrix;

/// Detaches `child` from its current [`Parent`] entity's [`Children`] list without clearing the
/// child's own [`Parent`] component.
fn detach_from_current_parent(world: &mut World, child: Entity) {
    let Ok(old_parent_ref) = world.get_copied::<Parent>(child) else {
        return;
    };
    let old_parent = old_parent_ref.0;
    let mut remove_parent_children = false;

    if let Ok(old_children) = world.get_mut::<Children>(old_parent) {
        old_children.0.retain(|&e| e != child);
        if old_children.0.is_empty() {
            remove_parent_children = true;
        }
    }

    let remove_parent_gt = remove_parent_children && !world.has::<Parent>(old_parent);
    if remove_parent_children {
        let _ = world.remove_one::<Children>(old_parent);
    }
    if remove_parent_gt {
        let _ = world.remove_one::<GlobalTransform>(old_parent);
    }
}

/// Reparents `child` under `parent` while recalculating local TRS components to preserve exact
/// world-space position, rotation, and scale.
///
/// Returns `false` if either entity does not exist, `child == parent`, or attaching `parent` would
/// create a cycle (`parent` is a descendant of `child`).
pub fn parent_entity_keep_world_transform(
    world: &mut World,
    child: Entity,
    parent: Entity,
) -> bool {
    if child == parent || !world.contains(child) || !world.contains(parent) {
        return false;
    }
    let mut cursor = Some(parent);
    while let Some(curr) = cursor {
        if curr == child {
            return false;
        }
        cursor = world.get_copied::<Parent>(curr).ok().map(|p| p.0);
    }

    update_hierarchy_transforms(world);

    let child_world = get_world_matrix(world, child);
    let parent_world = get_world_matrix(world, parent);
    if let Some(parent_inv) = parent_world.invert() {
        let new_local_matrix = parent_inv * child_world;
        let (new_pos, new_rot, new_scale) = decompose_matrix(new_local_matrix);
        let _ = world.insert_one(child, new_pos);
        let _ = world.insert_one(child, new_rot);
        let _ = world.insert_one(child, new_scale);
    }

    detach_from_current_parent(world, child);

    let _ = world.insert_one(child, Parent(parent));
    if let Ok(children) = world.get_mut::<Children>(parent) {
        if !children.0.contains(&child) {
            children.0.push(child);
        }
    } else {
        let _ = world.insert_one(parent, Children(vec![child]));
    }

    update_hierarchy_transforms(world);
    let _ = world.insert_one(child, TransformDirty);
    let _ = world.insert_one(parent, TransformDirty);
    true
}

/// Detaches `child` into a root entity while converting its global transform into local TRS values.
///
/// Returns `false` when `child` no longer exists in `world`.
pub fn unparent_entity_keep_world_transform(world: &mut World, child: Entity) -> bool {
    if !world.contains(child) {
        return false;
    }

    update_hierarchy_transforms(world);

    let child_world = get_world_matrix(world, child);
    let (new_pos, new_rot, new_scale) = decompose_matrix(child_world);
    let _ = world.insert_one(child, new_pos);
    let _ = world.insert_one(child, new_rot);
    let _ = world.insert_one(child, new_scale);

    detach_from_current_parent(world, child);
    let _ = world.remove_one::<Parent>(child);

    update_hierarchy_transforms(world);
    let _ = world.insert_one(child, TransformDirty);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::{Position, Rotation, Scale};

    #[test]
    fn test_parent_and_unparent_keep_world_transform_and_reject_cycles() {
        let mut world = World::new();
        let parent = world.spawn((
            Position::new(4.0, 0.0, 0.0),
            Rotation::identity(),
            Scale::new(2.0, 2.0, 2.0),
            GlobalTransform(cgmath::Matrix4::identity()),
        ));
        let child = world.spawn((
            Position::new(10.0, 6.0, -2.0),
            Rotation::identity(),
            Scale::new(1.0, 1.0, 1.0),
            GlobalTransform(cgmath::Matrix4::identity()),
        ));

        assert!(!parent_entity_keep_world_transform(
            &mut world, child, child
        ));
        assert!(parent_entity_keep_world_transform(
            &mut world, child, parent
        ));
        assert!(!parent_entity_keep_world_transform(
            &mut world, parent, child
        ));

        let local_pos = world.get_copied::<Position>(child).expect("local pos");
        let local_scale = world.get_copied::<Scale>(child).expect("local scale");
        assert!((local_pos.x - 3.0).abs() < 1e-4);
        assert!((local_pos.y - 3.0).abs() < 1e-4);
        assert!((local_pos.z + 1.0).abs() < 1e-4);
        assert!((local_scale.x - 0.5).abs() < 1e-4);

        let gt = world.get_copied::<GlobalTransform>(child).expect("gt");
        assert!((gt.0.w.x - 10.0).abs() < 1e-4);
        assert!((gt.0.w.y - 6.0).abs() < 1e-4);
        assert!((gt.0.w.z + 2.0).abs() < 1e-4);

        assert!(unparent_entity_keep_world_transform(&mut world, child));
        assert!(!world.has::<Parent>(child));
        assert!(!world.has::<Children>(parent));
        let restored_pos = world.get_copied::<Position>(child).expect("restored pos");
        assert!((restored_pos.x - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_reparent_rejects_dangling_entities() {
        let mut world = World::new();
        let valid = world.spawn((Position::zero(),));
        let ghost = world.spawn((Position::zero(),));
        let _ = world.despawn(ghost);

        assert!(!parent_entity_keep_world_transform(
            &mut world, ghost, valid
        ));
        assert!(!parent_entity_keep_world_transform(
            &mut world, valid, ghost
        ));
        assert!(!unparent_entity_keep_world_transform(&mut world, ghost));
    }
}
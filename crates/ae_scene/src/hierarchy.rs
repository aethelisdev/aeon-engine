// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Scene graph parent-child hierarchy management with world-transform preservation.
//!
//! Computes global transform matrices, orthonormal matrix decomposition, and safe
//! `Parent`/`Children` link updates when reparenting or detaching entities.

use ae_ecs::{Entity, World};
use ae_plugin_api::{Children, GlobalTransform, Parent, Position, Rotation, Scale, TransformDirty};
use cgmath::SquareMatrix;

/// Retrieves the world-space transformation matrix for `entity`, prioritizing [`GlobalTransform`]
/// and falling back to local TRS matrix composition when unavailable.
#[must_use]
pub fn get_world_matrix(world: &World, entity: Entity) -> cgmath::Matrix4<f32> {
    if let Ok(gt) = world.get_copied::<GlobalTransform>(entity) {
        return gt.0;
    }
    let pos = world
        .get_copied::<Position>(entity)
        .ok()
        .map_or(cgmath::Vector3::new(0.0, 0.0, 0.0), |p| {
            cgmath::Vector3::new(p.x, p.y, p.z)
        });
    let rot = world
        .get_copied::<Rotation>(entity)
        .ok()
        .map_or(cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0), |r| {
            cgmath::Quaternion::new(r.w, r.x, r.y, r.z)
        });
    let scale = world
        .get_copied::<Scale>(entity)
        .ok()
        .map_or(cgmath::Vector3::new(1.0, 1.0, 1.0), |s| {
            cgmath::Vector3::new(s.x, s.y, s.z)
        });

    cgmath::Matrix4::from_translation(pos)
        * cgmath::Matrix4::from(rot)
        * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
}

/// Decomposes a 4x4 affine transform matrix into local [`Position`], [`Rotation`], and [`Scale`].
///
/// Guards against degenerate zero axes and preserves mirrored negative X scale from
/// negative 3x3 sub-matrix determinants.
#[must_use]
pub fn decompose_matrix(mat: cgmath::Matrix4<f32>) -> (Position, Rotation, Scale) {
    use cgmath::{InnerSpace, Matrix3, Quaternion, Vector3};

    let pos = Position {
        x: mat.w.x,
        y: mat.w.y,
        z: mat.w.z,
    };

    let col0 = Vector3::new(mat.x.x, mat.x.y, mat.x.z);
    let col1 = Vector3::new(mat.y.x, mat.y.y, mat.y.z);
    let col2 = Vector3::new(mat.z.x, mat.z.y, mat.z.z);

    let mut scale_x = col0.magnitude().max(0.00001);
    let scale_y = col1.magnitude().max(0.00001);
    let scale_z = col2.magnitude().max(0.00001);

    if col0.dot(col1.cross(col2)) < 0.0 {
        scale_x = -scale_x;
    }

    let scale = Scale {
        x: scale_x,
        y: scale_y,
        z: scale_z,
    };

    let rot_mat3 = Matrix3::from_cols(col0 / scale_x, col1 / scale_y, col2 / scale_z);
    let q: Quaternion<f32> = Quaternion::from(rot_mat3);
    let q_norm = if q.magnitude() > 0.00001 {
        q.normalize()
    } else {
        Quaternion::new(1.0, 0.0, 0.0, 0.0)
    };

    let rot = Rotation {
        x: q_norm.v.x,
        y: q_norm.v.y,
        z: q_norm.v.z,
        w: q_norm.s,
    };

    (pos, rot, scale)
}

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
    // Reject cyclic hierarchy links where `parent` already descends from `child`.
    let mut cursor = Some(parent);
    while let Some(curr) = cursor {
        if curr == child {
            return false;
        }
        cursor = world.get_copied::<Parent>(curr).ok().map(|p| p.0);
    }

    ae_core::transform::update_hierarchy_transforms(world);

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

    ae_core::transform::update_hierarchy_transforms(world);
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

    ae_core::transform::update_hierarchy_transforms(world);

    let child_world = get_world_matrix(world, child);
    let (new_pos, new_rot, new_scale) = decompose_matrix(child_world);
    let _ = world.insert_one(child, new_pos);
    let _ = world.insert_one(child, new_rot);
    let _ = world.insert_one(child, new_scale);

    detach_from_current_parent(world, child);
    let _ = world.remove_one::<Parent>(child);

    ae_core::transform::update_hierarchy_transforms(world);
    let _ = world.insert_one(child, TransformDirty);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Parent cannot be reparented beneath its own child
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
        assert!((restored_pos.y - 6.0).abs() < 1e-4);
        assert!((restored_pos.z + 2.0).abs() < 1e-4);
    }
}
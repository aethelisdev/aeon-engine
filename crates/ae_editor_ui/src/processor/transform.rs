// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::UiContext;
use cgmath::SquareMatrix;

/// Helper function to retrieve the world-space transformation matrix for an entity.
/// Prioritizes `GlobalTransform` component, falling back to local transform decomposition.
pub fn get_world_matrix(world: &hecs::World, entity: hecs::Entity) -> cgmath::Matrix4<f32> {
    if let Ok(gt) = world.get::<&ae_core::ecs::GlobalTransform>(entity) {
        gt.0
    } else {
        let pos = world
            .get::<&ae_core::ecs::Position>(entity)
            .ok()
            .map(|p| cgmath::Vector3::new(p.x, p.y, p.z))
            .unwrap_or_else(|| cgmath::Vector3::new(0.0, 0.0, 0.0));
        let rot = world
            .get::<&ae_core::ecs::Rotation>(entity)
            .ok()
            .map(|r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z))
            .unwrap_or_else(|| cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0));
        let scale = world
            .get::<&ae_core::ecs::Scale>(entity)
            .ok()
            .map(|s| cgmath::Vector3::new(s.x, s.y, s.z))
            .unwrap_or_else(|| cgmath::Vector3::new(1.0, 1.0, 1.0));

        cgmath::Matrix4::from_translation(pos)
            * cgmath::Matrix4::from(rot)
            * cgmath::Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
    }
}

/// Helper function to decompose a 4x4 matrix into Position, Rotation, and Scale components.
/// Ensures orthonormal rotation extraction and safe magnitude scale calculations for Keep World Transform parenting.
pub fn decompose_matrix(
    mat: cgmath::Matrix4<f32>,
) -> (
    ae_core::ecs::Position,
    ae_core::ecs::Rotation,
    ae_core::ecs::Scale,
) {
    use cgmath::{InnerSpace, Matrix3, Quaternion, Vector3};

    let pos = ae_core::ecs::Position {
        x: mat.w.x,
        y: mat.w.y,
        z: mat.w.z,
    };

    let col0 = Vector3::new(mat.x.x, mat.x.y, mat.x.z);
    let col1 = Vector3::new(mat.y.x, mat.y.y, mat.y.z);
    let col2 = Vector3::new(mat.z.x, mat.z.y, mat.z.z);

    let scale_x = col0.magnitude();
    let scale_y = col1.magnitude();
    let scale_z = col2.magnitude();

    let scale = ae_core::ecs::Scale {
        x: scale_x,
        y: scale_y,
        z: scale_z,
    };

    let rot_col0 = if scale_x > 0.00001 {
        col0 / scale_x
    } else {
        Vector3::unit_x()
    };
    let rot_col1 = if scale_y > 0.00001 {
        col1 / scale_y
    } else {
        Vector3::unit_y()
    };
    let rot_col2 = if scale_z > 0.00001 {
        col2 / scale_z
    } else {
        Vector3::unit_z()
    };

    let rot_mat3 = Matrix3::from_cols(rot_col0, rot_col1, rot_col2);
    let q: Quaternion<f32> = Quaternion::from(rot_mat3);
    let q_norm = if q.magnitude() > 0.00001 {
        q.normalize()
    } else {
        Quaternion::new(1.0, 0.0, 0.0, 0.0)
    };

    let rot = ae_core::ecs::Rotation {
        x: q_norm.v.x,
        y: q_norm.v.y,
        z: q_norm.v.z,
        w: q_norm.s,
    };

    (pos, rot, scale)
}

/// Handles parenting an entity while preserving its exact world-space position and scale ("Keep World Transform").
pub fn handle_parent_entity(ctx: &mut UiContext, child: hecs::Entity, parent: hecs::Entity) {
    // 1. Synchronize hierarchy global transforms before calculating relative offsets
    ae_core::ecs::update_hierarchy_transforms(ctx.world);

    // 2. Read world matrices of child and target parent
    let child_world = get_world_matrix(ctx.world, child);
    let parent_world = get_world_matrix(ctx.world, parent);

    // 3. Recalculate local transform to preserve exact world-space position/rotation/scale ("Keep World Transform")
    if let Some(parent_inv) = parent_world.invert() {
        let new_local_matrix = parent_inv * child_world;
        let (new_pos, new_rot, new_scale) = decompose_matrix(new_local_matrix);

        let _ = ctx.world.insert_one(child, new_pos);
        let _ = ctx.world.insert_one(child, new_rot);
        let _ = ctx.world.insert_one(child, new_scale);
    }

    let old_parent_opt = if let Ok(old_parent_ref) = ctx.world.get::<&ae_core::ecs::Parent>(child) {
        Some(old_parent_ref.0)
    } else {
        None
    };

    if let Some(old_parent) = old_parent_opt {
        let mut remove_parent_children = false;
        let mut remove_parent_gt = false;

        if let Ok(mut old_children) = ctx.world.get::<&mut ae_core::ecs::Children>(old_parent) {
            old_children.0.retain(|&e| e != child);
            if old_children.0.is_empty() {
                remove_parent_children = true;
                if ctx.world.get::<&ae_core::ecs::Parent>(old_parent).is_err() {
                    remove_parent_gt = true;
                }
            }
        }

        if remove_parent_children {
            let _ = ctx.world.remove_one::<ae_core::ecs::Children>(old_parent);
        }
        if remove_parent_gt {
            let _ = ctx
                .world
                .remove_one::<ae_core::ecs::GlobalTransform>(old_parent);
        }
    }

    let _ = ctx.world.insert_one(child, ae_core::ecs::Parent(parent));
    if let Ok(mut children) = ctx.world.get::<&mut ae_core::ecs::Children>(parent) {
        if !children.0.contains(&child) {
            children.0.push(child);
        }
    } else {
        let _ = ctx
            .world
            .insert_one(parent, ae_core::ecs::Children(vec![child]));
    }
    ae_core::ecs::update_hierarchy_transforms(ctx.world);
    let _ = ctx.world.insert_one(child, ae_core::ecs::TransformDirty);
    let _ = ctx.world.insert_one(parent, ae_core::ecs::TransformDirty);
}

/// Handles unparenting an entity while preserving its exact world-space position and scale ("Keep World Transform").
pub fn handle_unparent_entity(ctx: &mut UiContext, child: hecs::Entity) {
    // 1. Synchronize hierarchy global transforms before unparenting
    ae_core::ecs::update_hierarchy_transforms(ctx.world);

    // 2. Read world matrix and assign as unparented root local transform ("Keep World Transform")
    let child_world = get_world_matrix(ctx.world, child);
    let (new_pos, new_rot, new_scale) = decompose_matrix(child_world);

    let _ = ctx.world.insert_one(child, new_pos);
    let _ = ctx.world.insert_one(child, new_rot);
    let _ = ctx.world.insert_one(child, new_scale);

    let old_parent_opt = if let Ok(old_parent_ref) = ctx.world.get::<&ae_core::ecs::Parent>(child) {
        Some(old_parent_ref.0)
    } else {
        None
    };

    if let Some(old_parent) = old_parent_opt {
        let mut remove_parent_children = false;
        let mut remove_parent_gt = false;

        if let Ok(mut old_children) = ctx.world.get::<&mut ae_core::ecs::Children>(old_parent) {
            old_children.0.retain(|&e| e != child);
            if old_children.0.is_empty() {
                remove_parent_children = true;
                if ctx.world.get::<&ae_core::ecs::Parent>(old_parent).is_err() {
                    remove_parent_gt = true;
                }
            }
        }

        if remove_parent_children {
            let _ = ctx.world.remove_one::<ae_core::ecs::Children>(old_parent);
        }
        if remove_parent_gt {
            let _ = ctx
                .world
                .remove_one::<ae_core::ecs::GlobalTransform>(old_parent);
        }
    }
    let _ = ctx.world.remove_one::<ae_core::ecs::Parent>(child);
    ae_core::ecs::update_hierarchy_transforms(ctx.world);
    let _ = ctx.world.insert_one(child, ae_core::ecs::TransformDirty);
}
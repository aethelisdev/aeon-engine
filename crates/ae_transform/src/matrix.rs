// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Unified TRS matrix composition, world-space matrix resolution, and orthonormal decomposition.
//!
//! Eliminates duplicated per-crate `Matrix4` builders so rendering, picking, physics sync,
//! debug gizmos, and hierarchy reparenting share identical zero-scale clamping and mirrored
//! determinant behavior.

use ae_ecs::{Entity, World};
use ae_plugin_api::{GlobalTransform, Position, Rotation, Scale};

const MIN_SCALE_AXIS: f32 = 1e-4;

/// Clamps near-zero or non-finite scale components to avoid singular transform matrices while
/// preserving mirrored negative sign.
#[must_use]
#[inline]
pub fn clamp_non_zero_scale_axis(value: f32) -> f32 {
    if !value.is_finite() {
        1.0
    } else if value.abs() < MIN_SCALE_AXIS {
        f32::copysign(MIN_SCALE_AXIS, value)
    } else {
        value
    }
}

/// Builds a 4x4 local TRS matrix from optional `Position`, `Rotation`, and `Scale` components,
/// guarding against degenerate zero or non-finite scale axes.
#[must_use]
pub fn compose_local_matrix(
    pos: Option<Position>,
    rot: Option<Rotation>,
    scale: Option<Scale>,
) -> cgmath::Matrix4<f32> {
    let p = pos
        .filter(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite())
        .map_or_else(
            || cgmath::Vector3::new(0.0, 0.0, 0.0),
            |p| cgmath::Vector3::new(p.x, p.y, p.z),
        );
    let q = rot
        .filter(|r| r.x.is_finite() && r.y.is_finite() && r.z.is_finite() && r.w.is_finite())
        .map_or_else(
            || cgmath::Quaternion::new(1.0, 0.0, 0.0, 0.0),
            |r| cgmath::Quaternion::new(r.w, r.x, r.y, r.z),
        );
    let s = scale.map_or_else(
        || cgmath::Vector3::new(1.0, 1.0, 1.0),
        |s| {
            cgmath::Vector3::new(
                clamp_non_zero_scale_axis(s.x),
                clamp_non_zero_scale_axis(s.y),
                clamp_non_zero_scale_axis(s.z),
            )
        },
    );

    cgmath::Matrix4::from_translation(p)
        * cgmath::Matrix4::from(q)
        * cgmath::Matrix4::from_nonuniform_scale(s.x, s.y, s.z)
}

/// Builds a 2D planar sprite model matrix from a `Position` and `(scale_x, scale_y)`.
#[must_use]
#[inline]
pub fn compose_2d_model_matrix(pos: Position, scale_x: f32, scale_y: f32) -> cgmath::Matrix4<f32> {
    let tx = if pos.x.is_finite() { pos.x } else { 0.0 };
    let ty = if pos.y.is_finite() { pos.y } else { 0.0 };
    let tz = if pos.z.is_finite() { pos.z } else { 0.0 };
    let sx = if scale_x.is_finite() { scale_x } else { 1.0 };
    let sy = if scale_y.is_finite() { scale_y } else { 1.0 };

    cgmath::Matrix4::from_translation(cgmath::Vector3::new(tx, ty, tz))
        * cgmath::Matrix4::from_nonuniform_scale(sx, sy, 1.0)
}

/// Resolves the world-space 4x4 matrix from optional ECS transform components, prioritizing
/// cached [`GlobalTransform`] over local TRS composition.
#[must_use]
#[inline]
pub fn resolve_world_matrix(
    gt: Option<&GlobalTransform>,
    pos: Option<&Position>,
    rot: Option<&Rotation>,
    scale: Option<&Scale>,
) -> cgmath::Matrix4<f32> {
    if let Some(global_tf) = gt {
        global_tf.0
    } else {
        compose_local_matrix(pos.copied(), rot.copied(), scale.copied())
    }
}

/// Retrieves the world-space transformation matrix for `entity`, prioritizing [`GlobalTransform`]
/// and falling back to local TRS composition when unavailable.
#[must_use]
pub fn get_world_matrix(world: &World, entity: Entity) -> cgmath::Matrix4<f32> {
    if let Ok(gt) = world.get_copied::<GlobalTransform>(entity) {
        return gt.0;
    }
    compose_local_matrix(
        world.get_copied::<Position>(entity).ok(),
        world.get_copied::<Rotation>(entity).ok(),
        world.get_copied::<Scale>(entity).ok(),
    )
}

/// Returns the world-space translation vector for `entity` (`GlobalTransform` or local `Position`).
#[must_use]
pub fn world_translation_for_entity(world: &World, entity: Entity) -> Option<cgmath::Vector3<f32>> {
    if let Ok(gt) = world.get_copied::<GlobalTransform>(entity) {
        return Some(cgmath::Vector3::new(gt.0.w.x, gt.0.w.y, gt.0.w.z));
    }
    world
        .get_copied::<Position>(entity)
        .ok()
        .map(|pos| cgmath::Vector3::new(pos.x, pos.y, pos.z))
}

/// Extracts world-space position and positive per-axis scale magnitudes from optional
/// `GlobalTransform`, `Position`, and `Scale` components for bounding/culling checks.
#[must_use]
pub fn extract_world_position_and_scale(
    gt: Option<&GlobalTransform>,
    pos: Option<&Position>,
    scale: Option<&Scale>,
) -> (cgmath::Vector3<f32>, [f32; 3]) {
    use cgmath::InnerSpace;

    let world_pos = if let Some(global_tf) = gt {
        cgmath::Vector3::new(global_tf.0.w.x, global_tf.0.w.y, global_tf.0.w.z)
    } else if let Some(p) = pos {
        cgmath::Vector3::new(p.x, p.y, p.z)
    } else {
        cgmath::Vector3::new(0.0, 0.0, 0.0)
    };

    let extents = if let Some(s) = scale {
        [s.x.abs(), s.y.abs(), s.z.abs()]
    } else if let Some(global_tf) = gt {
        [
            cgmath::Vector3::new(global_tf.0.x.x, global_tf.0.x.y, global_tf.0.x.z).magnitude(),
            cgmath::Vector3::new(global_tf.0.y.x, global_tf.0.y.y, global_tf.0.y.z).magnitude(),
            cgmath::Vector3::new(global_tf.0.z.x, global_tf.0.z.y, global_tf.0.z.z).magnitude(),
        ]
    } else {
        [1.0, 1.0, 1.0]
    };

    (world_pos, extents)
}

/// Shared orthonormal TRS extraction core used by both `glam` conversions and ECS `Position`/
/// `Rotation`/`Scale` decomposition.
///
/// Clamps near-zero basis column lengths and flips X scale when the 3x3 determinant is negative
/// so the recovered quaternion remains right-handed.
#[must_use]
pub(crate) fn extract_trs_columns(
    mat: cgmath::Matrix4<f32>,
) -> (glam::Vec3, glam::Quat, glam::Vec3) {
    let trans = glam::Vec3::new(mat.w.x, mat.w.y, mat.w.z);

    let col0 = glam::Vec3::new(mat.x.x, mat.x.y, mat.x.z);
    let col1 = glam::Vec3::new(mat.y.x, mat.y.y, mat.y.z);
    let col2 = glam::Vec3::new(mat.z.x, mat.z.y, mat.z.z);

    let mut sx = col0.length().max(1e-5);
    let sy = col1.length().max(1e-5);
    let sz = col2.length().max(1e-5);

    if col0.dot(col1.cross(col2)) < 0.0 {
        sx = -sx;
    }

    let rot_mat3 = glam::Mat3::from_cols(col0 / sx, col1 / sy, col2 / sz);
    let raw_rot = glam::Quat::from_mat3(&rot_mat3);
    let rot = if raw_rot.is_finite() && raw_rot.length_squared() > 1e-10 {
        raw_rot.normalize()
    } else {
        glam::Quat::IDENTITY
    };

    (trans, rot, glam::Vec3::new(sx, sy, sz))
}

/// Decomposes a 4x4 affine matrix into local [`Position`], [`Rotation`], and [`Scale`].
///
/// Guards against degenerate zero axes and preserves mirrored negative X scale from
/// negative 3x3 sub-matrix determinants.
#[must_use]
pub fn decompose_matrix(mat: cgmath::Matrix4<f32>) -> (Position, Rotation, Scale) {
    let (trans, rot, scale) = extract_trs_columns(mat);
    (
        Position {
            x: trans.x,
            y: trans.y,
            z: trans.z,
        },
        Rotation {
            x: rot.x,
            y: rot.y,
            z: rot.z,
            w: rot.w,
        },
        Scale {
            x: scale.x,
            y: scale.y,
            z: scale.z,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::SquareMatrix;

    #[test]
    fn test_resolve_world_matrix_prioritizes_global_transform() {
        let gt = GlobalTransform(cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            100.0, 50.0, -4.0,
        )));
        let pos = Position::new(1.0, 2.0, 3.0);
        let result = resolve_world_matrix(Some(&gt), Some(&pos), None, None);
        assert_eq!([result.w.x, result.w.y, result.w.z], [100.0, 50.0, -4.0]);
    }

    #[test]
    fn test_zero_and_non_finite_scale_clamping_stays_invertible() {
        let mat = compose_local_matrix(
            Some(Position::new(3.0, -2.0, 5.0)),
            Some(Rotation::identity()),
            Some(Scale::new(0.0, -0.0, f32::NAN)),
        );
        assert!(mat.invert().is_some());
        let (_pos, _rot, scale) = decompose_matrix(mat);
        assert!((scale.x.abs() - MIN_SCALE_AXIS).abs() < 1e-6);
        assert!((scale.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_decompose_matrix_handles_negative_mirrored_determinant() {
        let mat = compose_local_matrix(
            Some(Position::new(1.0, 2.0, 3.0)),
            Some(Rotation::identity()),
            Some(Scale::new(-2.0, 3.0, 4.0)),
        );
        let (pos, rot, scale) = decompose_matrix(mat);
        assert!((pos.x - 1.0).abs() < 1e-5);
        assert!((pos.y - 2.0).abs() < 1e-5);
        assert!((pos.z - 3.0).abs() < 1e-5);
        assert!((scale.x + 2.0).abs() < 1e-5);
        assert!((scale.y - 3.0).abs() < 1e-5);
        assert!((scale.z - 4.0).abs() < 1e-5);
        assert!(rot.w.abs() > 0.99);
    }

    #[test]
    fn test_compose_2d_model_matrix_sanitizes_nan() {
        let mat = compose_2d_model_matrix(Position::new(f32::NAN, 4.0, 1.0), 2.0, f32::INFINITY);
        assert_eq!([mat.w.x, mat.w.y, mat.w.z], [0.0, 4.0, 1.0]);
        assert_eq!(mat.x.x, 2.0);
        assert_eq!(mat.y.y, 1.0);
    }
}
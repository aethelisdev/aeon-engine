// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::ecs::{Position, Rotation, Scale};

/// Extension trait for converting Aeon Engine math types to `glam` representations.
pub trait ToGlam {
    type Target;
    fn to_glam(&self) -> Self::Target;
}

/// Extension trait for converting `glam` math types back to Aeon Engine `cgmath`/ECS types.
pub trait ToCgmath {
    type Target;
    fn to_cgmath(&self) -> Self::Target;
}

impl ToGlam for Position {
    type Target = glam::Vec3;
    #[inline]
    fn to_glam(&self) -> Self::Target {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

impl ToGlam for Rotation {
    type Target = glam::Quat;
    #[inline]
    fn to_glam(&self) -> Self::Target {
        glam::Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}

impl ToGlam for Scale {
    type Target = glam::Vec3;
    #[inline]
    fn to_glam(&self) -> Self::Target {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

impl ToGlam for cgmath::Vector3<f32> {
    type Target = glam::Vec3;
    #[inline]
    fn to_glam(&self) -> Self::Target {
        glam::Vec3::new(self.x, self.y, self.z)
    }
}

impl ToGlam for cgmath::Quaternion<f32> {
    type Target = glam::Quat;
    #[inline]
    fn to_glam(&self) -> Self::Target {
        glam::Quat::from_xyzw(self.v.x, self.v.y, self.v.z, self.s)
    }
}

impl ToCgmath for glam::Vec3 {
    type Target = cgmath::Vector3<f32>;
    #[inline]
    fn to_cgmath(&self) -> Self::Target {
        cgmath::Vector3::new(self.x, self.y, self.z)
    }
}

impl ToCgmath for glam::Quat {
    type Target = cgmath::Quaternion<f32>;
    #[inline]
    fn to_cgmath(&self) -> Self::Target {
        cgmath::Quaternion::new(self.w, self.x, self.y, self.z)
    }
}

/// Decomposes a 4×4 cgmath Matrix into `glam` translation, rotation, and scale tuple.
/// Handles non-uniform scaling magnitudes, orthonormal rotation extraction, and
/// negative (mirrored) scale detection via rotation sub-matrix determinant sign.
/// When the determinant is negative, the X-axis scale is negated to preserve a
/// proper (right-handed) rotation quaternion.
pub fn matrix4_to_glam_trs(mat: cgmath::Matrix4<f32>) -> (glam::Vec3, glam::Quat, glam::Vec3) {
    let trans = glam::Vec3::new(mat.w.x, mat.w.y, mat.w.z);

    let col0 = glam::Vec3::new(mat.x.x, mat.x.y, mat.x.z);
    let col1 = glam::Vec3::new(mat.y.x, mat.y.y, mat.y.z);
    let col2 = glam::Vec3::new(mat.z.x, mat.z.y, mat.z.z);

    let mut sx = col0.length().max(1e-5);
    let sy = col1.length().max(1e-5);
    let sz = col2.length().max(1e-5);

    // Detect mirrored (negative) scale via 3×3 sub-matrix determinant sign
    let det = col0.dot(col1.cross(col2));
    if det < 0.0 {
        sx = -sx;
    }

    let rot_mat3 = glam::Mat3::from_cols(col0 / sx, col1 / sy, col2 / sz);
    let rot = glam::Quat::from_mat3(&rot_mat3);
    let scale = glam::Vec3::new(sx, sy, sz);

    (trans, rot, scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_conversions() {
        let pos = Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let gpos = pos.to_glam();
        assert_eq!(gpos, glam::Vec3::new(1.0, 2.0, 3.0));

        let cg_vec = gpos.to_cgmath();
        assert_eq!(cg_vec, cgmath::Vector3::new(1.0, 2.0, 3.0));

        let mat = cgmath::Matrix4::from_translation(cg_vec);
        let (t, _r, _s) = matrix4_to_glam_trs(mat);
        assert_eq!(t, glam::Vec3::new(1.0, 2.0, 3.0));
    }
}
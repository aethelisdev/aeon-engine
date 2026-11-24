// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Bidirectional math conversions between Aeon `cgmath`/ECS transform types and `glam`.

use crate::matrix::extract_trs_columns;
use ae_plugin_api::{Position, Rotation, Scale};

/// Extension trait for converting Aeon Engine math and transform types to `glam`.
pub trait ToGlam {
    /// Target `glam` type produced by conversion.
    type Target;

    /// Converts `self` into the corresponding `glam` type.
    fn to_glam(&self) -> Self::Target;
}

/// Extension trait for converting `glam` math types back to `cgmath`.
pub trait ToCgmath {
    /// Target `cgmath` type produced by conversion.
    type Target;

    /// Converts `self` into the corresponding `cgmath` type.
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

/// Decomposes a 4x4 `cgmath` matrix into `glam` `(translation, rotation, scale)` components.
///
/// Preserves mirrored negative X scale when the upper-left 3x3 determinant is negative and
/// guards against zero basis columns.
#[must_use]
pub fn matrix4_to_glam_trs(mat: cgmath::Matrix4<f32>) -> (glam::Vec3, glam::Quat, glam::Vec3) {
    extract_trs_columns(mat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::Zero;

    #[test]
    fn test_math_conversions_roundtrip() {
        let pos = Position::new(1.0, 2.0, 3.0);
        let gpos = pos.to_glam();
        assert_eq!(gpos, glam::Vec3::new(1.0, 2.0, 3.0));

        let cg_vec = gpos.to_cgmath();
        assert_eq!(cg_vec, cgmath::Vector3::new(1.0, 2.0, 3.0));

        let mat = cgmath::Matrix4::from_translation(cg_vec);
        let (t, r, s) = matrix4_to_glam_trs(mat);
        assert_eq!(t, glam::Vec3::new(1.0, 2.0, 3.0));
        assert!(r.is_near_identity());
        assert_eq!(s, glam::Vec3::ONE);
    }

    #[test]
    fn test_degenerate_zero_matrix_decomposition_stays_finite() {
        let zero = cgmath::Matrix4::zero();
        let (_t, r, s) = matrix4_to_glam_trs(zero);
        assert!(r.is_finite());
        assert!(s.is_finite());
        assert!(s.x > 0.0 && s.y > 0.0 && s.z > 0.0);
    }
}
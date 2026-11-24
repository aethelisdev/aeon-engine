// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! WGPU `[0, 1]` clip-space frustum extraction and sphere/AABB visibility culling.

/// View frustum defined by 6 normalized clip planes.
///
/// Extracted from a combined view-projection matrix using the Gribb-Hartmann
/// method, adapted for WGPU's `[0, 1]` Z-buffer range.
#[derive(Debug, Clone)]
pub struct Frustum {
    planes: [cgmath::Vector4<f32>; 6],
}

impl Frustum {
    /// Extracts 6 normalized frustum planes (Left, Right, Bottom, Top, Near, Far).
    #[must_use]
    pub fn from_matrix(m: cgmath::Matrix4<f32>) -> Self {
        let row0 = cgmath::Vector4::new(m.x.x, m.y.x, m.z.x, m.w.x);
        let row1 = cgmath::Vector4::new(m.x.y, m.y.y, m.z.y, m.w.y);
        let row2 = cgmath::Vector4::new(m.x.z, m.y.z, m.z.z, m.w.z);
        let row3 = cgmath::Vector4::new(m.x.w, m.y.w, m.z.w, m.w.w);

        let mut planes = [
            row3 + row0, // Left
            row3 - row0, // Right
            row3 + row1, // Bottom
            row3 - row1, // Top
            row2,        // Near Plane (WGPU [0..1] Z convention)
            row3 - row2, // Far Plane
        ];

        for plane in &mut planes {
            let length = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
            if length > 0.0 {
                *plane /= length;
            }
        }

        Self { planes }
    }

    /// Tests whether a bounding sphere is at least partially inside the frustum.
    #[must_use]
    pub fn is_sphere_visible(&self, center: cgmath::Vector3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            if distance < -radius {
                return false;
            }
        }
        true
    }

    /// Tests whether an Axis-Aligned Bounding Box (AABB) intersects the frustum via the p-vertex test.
    #[must_use]
    pub fn is_aabb_visible(
        &self,
        aabb_min: cgmath::Vector3<f32>,
        aabb_max: cgmath::Vector3<f32>,
    ) -> bool {
        for plane in &self.planes {
            let px = if plane.x >= 0.0 {
                aabb_max.x
            } else {
                aabb_min.x
            };
            let py = if plane.y >= 0.0 {
                aabb_max.y
            } else {
                aabb_min.y
            };
            let pz = if plane.z >= 0.0 {
                aabb_max.z
            } else {
                aabb_min.z
            };

            let distance = plane.x * px + plane.y * py + plane.z * pz + plane.w;
            if distance < 0.0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Camera, ProjectionMode};
    use cgmath::SquareMatrix;

    #[test]
    fn test_frustum_visibility_and_behind_camera_culling() {
        let cam = Camera {
            position: cgmath::Point3::new(0.0, 0.0, 5.0),
            yaw: cgmath::Rad(-std::f32::consts::FRAC_PI_2),
            pitch: cgmath::Rad(0.0),
            aspect: 1.0,
            fovy: 60.0,
            znear: 0.5,
            zfar: 100.0,
            mode: ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };
        let frustum = Frustum::from_matrix(cam.build_view_projection_matrix());

        // Point directly ahead at z = -10 must be visible.
        assert!(frustum.is_sphere_visible(cgmath::Vector3::new(0.0, 0.0, -10.0), 1.0));
        assert!(frustum.is_aabb_visible(
            cgmath::Vector3::new(-1.0, -1.0, -11.0),
            cgmath::Vector3::new(1.0, 1.0, -9.0)
        ));

        // Point behind camera eye at z = 20 must be culled.
        assert!(!frustum.is_sphere_visible(cgmath::Vector3::new(0.0, 0.0, 20.0), 1.0));
        assert!(!frustum.is_aabb_visible(
            cgmath::Vector3::new(-1.0, -1.0, 19.0),
            cgmath::Vector3::new(1.0, 1.0, 21.0)
        ));

        // Zero matrix should not produce NaN / panic.
        let deg = Frustum::from_matrix(cgmath::Matrix4::from_value(0.0));
        assert!(deg.is_sphere_visible(cgmath::Vector3::new(0.0, 0.0, 0.0), 1.0));
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// View frustum defined by 6 normalized clip planes.
/// Extracted from a combined view-projection matrix using the Gribb-Hartmann
/// method, adapted for WGPU's `[0, 1]` Z-buffer range. Used for CPU-side
/// sphere-frustum and AABB-frustum visibility testing to cull entities and
/// spatial grid cells before GPU submission.
#[derive(Debug, Clone)]
pub struct Frustum {
    planes: [cgmath::Vector4<f32>; 6],
}

impl Frustum {
    /// Extracts 6 frustum planes from a combined view-projection matrix.
    /// Planes are: Left, Right, Bottom, Top, Near, Far.
    /// Near plane uses `row2` directly (WGPU `[0,1]` Z convention).
    /// All planes are normalized for correct distance calculations.
    pub fn from_matrix(m: cgmath::Matrix4<f32>) -> Self {
        // Matrix columns (Column-Major). Arranged according to WGPU [0..1] Z-Buffer range.
        let row0 = cgmath::Vector4::new(m.x.x, m.y.x, m.z.x, m.w.x);
        let row1 = cgmath::Vector4::new(m.x.y, m.y.y, m.z.y, m.w.y);
        let row2 = cgmath::Vector4::new(m.x.z, m.y.z, m.z.z, m.w.z);
        let row3 = cgmath::Vector4::new(m.x.w, m.y.w, m.z.w, m.w.w);

        let mut planes = [
            row3 + row0, // Left
            row3 - row0, // Right
            row3 + row1, // Bottom
            row3 - row1, // Top
            row2,        // Near Plane (in WGPU M.z is directly used)
            row3 - row2, // Far Plane
        ];

        // Plane Normalization
        for plane in &mut planes {
            let length = (plane.x * plane.x + plane.y * plane.y + plane.z * plane.z).sqrt();
            if length > 0.0 {
                *plane /= length;
            }
        }

        Self { planes }
    }

    /// Tests whether a bounding sphere is at least partially inside the frustum.
    /// Returns `false` (culled) if the sphere is completely behind any of the
    /// 6 planes. Uses signed distance: `distance < -radius` means fully outside.
    pub fn is_sphere_visible(&self, center: cgmath::Vector3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            // If it is behind any of the 6 planes, it is completely culled.
            if distance < -radius {
                return false;
            }
        }
        true
    }

    /// Tests whether an Axis-Aligned Bounding Box (AABB) is at least partially
    /// inside the frustum using the p-vertex technique.
    /// For each frustum plane, the "positive vertex" (p-vertex) is selected as
    /// the AABB corner closest to the plane's inward direction. If the p-vertex
    /// lies completely behind any plane, the entire box is outside the frustum.
    /// This is significantly more precise than sphere testing for cubic volumes:
    /// a bounding sphere inflates a 250-unit cube to a 216-unit radius sphere,
    /// wasting ~63% of the test volume as false positives. AABB testing eliminates
    /// this inflation entirely, reducing visible SpatialGrid cells by 60-70%.
    pub fn is_aabb_visible(
        &self,
        aabb_min: cgmath::Vector3<f32>,
        aabb_max: cgmath::Vector3<f32>,
    ) -> bool {
        for plane in &self.planes {
            // Select the p-vertex: the corner of the AABB most aligned with the plane normal.
            // For each axis, if the plane normal component is positive, use max; otherwise use min.
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

            // If the p-vertex is behind the plane, the entire AABB is outside
            let distance = plane.x * px + plane.y * py + plane.z * pz + plane.w;
            if distance < 0.0 {
                return false;
            }
        }
        true
    }
}
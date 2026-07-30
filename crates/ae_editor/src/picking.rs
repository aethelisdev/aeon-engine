// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4};

/// Aeon Engine Picking & Raycasting Module
/// This module provides utilities to handle mouse-to-world interaction,
/// including ray generation from screen coordinates and intersection testing.

/// Represents a simple Ray in 3D space.
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub max_dist: f32, // Maximum distance until far plane
}

/// Creates a 3D Ray from 2D screen coordinates.
/// # Arguments
/// * `mx`, `my` - Mouse coordinates in pixels.
/// * `width`, `height` - Current dimensions of the renderer's surface.
/// * `vp_matrix` - View-Projection matrix of the camera.
/// # Returns
/// * `Some(Ray)` - If the coordinates are valid and viewport has size.
pub fn create_ray(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    vp_matrix: &Matrix4<f32>,
) -> Option<Ray> {
    if width <= 1e-6 || height <= 1e-6 {
        return None;
    }

    // Convert screen coordinates to Normalized Device Coordinates (NDC) [-1, 1]
    let mouse_x = (mx / width) * 2.0 - 1.0;
    let mouse_y = 1.0 - (my / height) * 2.0;

    // Invert the View-Projection matrix to go from Clip Space back to World Space
    let inv_vp = vp_matrix.invert().unwrap_or(Matrix4::identity());

    // Calculate near and far points by unprojecting Z=0 (near) and Z=1 (far)
    let near_vec4: Vector4<f32> = inv_vp * Vector4::new(mouse_x, mouse_y, 0.0, 1.0);
    let far_vec4: Vector4<f32> = inv_vp * Vector4::new(mouse_x, mouse_y, 1.0, 1.0);

    // Perspective divide (guard against divide by zero)
    if near_vec4.w.abs() < 1e-6 || far_vec4.w.abs() < 1e-6 {
        return None;
    }

    let ray_origin = Point3::from_vec(near_vec4.truncate() / near_vec4.w);
    let ray_end = Point3::from_vec(far_vec4.truncate() / far_vec4.w);

    if !ray_origin.x.is_finite() || !ray_end.x.is_finite() {
        return None;
    }

    // Direction calculation
    let ray_dir_unnorm = ray_end - ray_origin;
    let max_dist = ray_dir_unnorm.magnitude();
    if max_dist < 1e-8 {
        return None;
    }

    let ray_direction = ray_dir_unnorm.normalize();
    Some(Ray {
        origin: ray_origin,
        direction: ray_direction,
        max_dist,
    })
}

/// Intersection test using the Slab Method (Ray vs Axis-Aligned Bounding Box).
/// * `ray` - The Picking Ray (origin, direction, and max_dist).
/// * `min`, `max` - Bounds of the box.
/// # Returns
/// * `Some(f32)` - The distance `t` along the ray to the intersection point.
/// * `None` - If no intersection occurred or it's beyond `max_dist`.
pub fn intersect_aabb(ray: &Ray, min: [f32; 3], max: [f32; 3]) -> Option<f32> {
    let mut t_min = -f32::MAX;
    let mut t_max = f32::MAX;

    let ro = ray.origin;
    let rd = ray.direction;

    for i in 0..3 {
        // Guard against divide-by-zero for axial rays
        if rd[i].abs() < 1e-8 {
            if ro[i] < min[i] || ro[i] > max[i] {
                return None;
            }
            continue;
        }

        let inv_d = 1.0 / rd[i];
        let mut t1 = (min[i] - ro[i]) * inv_d;
        let mut t2 = (max[i] - ro[i]) * inv_d;

        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }
        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
    }

    if t_max >= t_min && t_max > 0.0 && t_min < ray.max_dist {
        Some(t_min)
    } else {
        None
    }
}

/// Utility to generate a Model Matrix from ECS components.
/// Prioritizes `GlobalTransform` if present (ensuring correct world-space raycasting for parent-child hierarchies),
/// falling back to local `Position`, `Rotation`, and `Scale` components.
pub fn compute_model_matrix(
    gt: Option<&ae_core::ecs::GlobalTransform>,
    pos: Option<&ae_core::ecs::Position>,
    rot: Option<&ae_core::ecs::Rotation>,
    scale: Option<&ae_core::ecs::Scale>,
) -> Matrix4<f32> {
    if let Some(global_tf) = gt {
        global_tf.0
    } else {
        let mut model = Matrix4::identity();
        if let Some(p) = pos {
            model = model * Matrix4::from_translation(Vector3::new(p.x, p.y, p.z));
        }
        if let Some(r) = rot {
            model = model * Matrix4::from(cgmath::Quaternion::new(r.w, r.x, r.y, r.z));
        }
        if let Some(s) = scale {
            let sx = if s.x.abs() < 1e-4 { 0.001 } else { s.x };
            let sy = if s.y.abs() < 1e-4 { 0.001 } else { s.y };
            let sz = if s.z.abs() < 1e-4 { 0.001 } else { s.z };
            model = model * Matrix4::from_nonuniform_scale(sx, sy, sz);
        }
        model
    }
}

/// Ray vs Bounding Sphere intersection test for 3D Viewport Billboard Icons.
/// Returns closest distance `t` along the ray if ray intersects sphere of `radius` at `center`.
pub fn intersect_sphere(ray: &Ray, center: Point3<f32>, radius: f32) -> Option<f32> {
    let oc = ray.origin - center;
    let b = oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - c;

    if discriminant < 0.0 {
        return None;
    }

    let t = -b - discriminant.sqrt();
    if t > 0.0 && t < ray.max_dist {
        Some(t)
    } else {
        None
    }
}

/// Ray vs Triangle intersection test using Möller–Trumbore algorithm.
pub fn intersect_triangle(ray: &Ray, v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> Option<f32> {
    let p0 = Point3::new(v0[0], v0[1], v0[2]);
    let p1 = Point3::new(v1[0], v1[1], v1[2]);
    let p2 = Point3::new(v2[0], v2[1], v2[2]);

    let edge1 = p1 - p0;
    let edge2 = p2 - p0;
    let h = ray.direction.cross(edge2);
    let a = edge1.dot(h);

    if a.abs() < 1e-7 {
        return None;
    }

    let f = 1.0 / a;
    let s = ray.origin - p0;
    let u = f * s.dot(h);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray.direction.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = f * edge2.dot(q);
    if t > 1e-5 && t < ray.max_dist {
        Some(t)
    } else {
        None
    }
}

/// Ray vs Mesh Triangles intersection test for precise 3D Asset selection.
pub fn intersect_mesh(ray: &Ray, vertices: &[[f32; 3]], indices: &[u32]) -> Option<f32> {
    let mut min_t = None;

    for chunk in indices.chunks_exact(3) {
        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        if i0 < vertices.len() && i1 < vertices.len() && i2 < vertices.len() {
            if let Some(t) = intersect_triangle(ray, vertices[i0], vertices[i1], vertices[i2]) {
                match min_t {
                    Some(curr_t) => {
                        if t < curr_t {
                            min_t = Some(t);
                        }
                    }
                    None => min_t = Some(t),
                }
            }
        }
    }

    min_t
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that compute_model_matrix prioritizes GlobalTransform over local position/rotation/scale.
    #[test]
    fn test_compute_model_matrix_global_transform_priority() {
        let gt_matrix = cgmath::Matrix4::from_translation(cgmath::Vector3::new(100.0, 50.0, 0.0));
        let gt = ae_core::ecs::GlobalTransform(gt_matrix);
        let pos = ae_core::ecs::Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let result = compute_model_matrix(Some(&gt), Some(&pos), None, None);
        assert_eq!(result.w.x, 100.0);
        assert_eq!(result.w.y, 50.0);
    }
}
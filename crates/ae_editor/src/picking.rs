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
/// This is used for ray-casting and intersection tests to move from World Space to Local Space.
pub fn compute_model_matrix(
    pos: Option<&ae_core::ecs::Position>,
    rot: Option<&ae_core::ecs::Rotation>,
    scale: Option<&ae_core::ecs::Scale>,
) -> Matrix4<f32> {
    let mut model = Matrix4::identity();
    if let Some(p) = pos {
        model = model * Matrix4::from_translation(Vector3::new(p.x, p.y, p.z));
    }
    if let Some(r) = rot {
        model = model * Matrix4::from(cgmath::Quaternion::new(r.w, r.x, r.y, r.z));
    }
    if let Some(s) = scale {
        model = model * Matrix4::from_nonuniform_scale(s.x, s.y, s.z);
    }
    model
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
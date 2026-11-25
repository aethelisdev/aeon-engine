// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
//! Aeon Engine Picking & Raycasting Module
//!
//! This module provides utilities to handle mouse-to-world interaction,
//! including ray generation from screen coordinates and intersection testing.

use cgmath::{EuclideanSpace, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4};

/// Represents a simple Ray in 3D space.
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub max_dist: f32, // Maximum distance until far plane
}

/// Creates a 3D Ray from 2D screen coordinates.
///
/// # Arguments
/// * `mx`, `my` - Mouse coordinates in pixels.
/// * `width`, `height` - Current dimensions of the renderer's surface.
/// * `vp_matrix` - View-Projection matrix of the camera.
///
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
///
/// * `ray` - The Picking Ray (origin, direction, and max_dist).
/// * `min`, `max` - Bounds of the box.
///
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

    if t_max >= t_min && t_max > 0.0 {
        let hit_t = if t_min >= 0.0 { t_min } else { 0.0 };
        if hit_t < ray.max_dist {
            Some(hit_t)
        } else {
            None
        }
    } else {
        None
    }
}

/// Ray vs Bounding Sphere intersection test for 3D Viewport Billboard Icons.
///
/// Returns closest distance `t` along the ray if ray intersects sphere of `radius` at `center`.
pub fn intersect_sphere(ray: &Ray, center: Point3<f32>, radius: f32) -> Option<f32> {
    let oc = ray.origin - center;
    let b = oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - c;

    if discriminant < 0.0 {
        return None;
    }

    let t1 = -b - discriminant.sqrt();
    let t2 = -b + discriminant.sqrt();

    let hit_t = if t1 > 0.0 {
        Some(t1)
    } else if t2 > 0.0 {
        Some(t2)
    } else {
        None
    };

    hit_t.filter(|&t| t < ray.max_dist)
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

    for chunk in indices.as_chunks::<3>().0 {
        let i0 = chunk[0] as usize;
        let i1 = chunk[1] as usize;
        let i2 = chunk[2] as usize;

        if i0 < vertices.len()
            && i1 < vertices.len()
            && i2 < vertices.len()
            && let Some(t) = intersect_triangle(ray, vertices[i0], vertices[i1], vertices[i2])
        {
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

    min_t
}

/// Computes the 3D world intersection point on the horizontal ground plane (`Y = 0`)
/// from screen coordinates (`[mx, my]`), a logical viewport rect (`[x, y, width, height]`),
/// and a camera view-projection matrix.
#[must_use]
pub fn compute_ground_intersection(
    mouse_screen_pos: [f32; 2],
    viewport_rect: [f32; 4],
    vp_matrix: &cgmath::Matrix4<f32>,
) -> Option<[f32; 3]> {
    let rel_x = mouse_screen_pos[0] - viewport_rect[0];
    let rel_y = mouse_screen_pos[1] - viewport_rect[1];

    let ray = create_ray(rel_x, rel_y, viewport_rect[2], viewport_rect[3], vp_matrix)?;
    if ray.direction.y.abs() < 1e-5 {
        return None;
    }

    let t = -ray.origin.y / ray.direction.y;
    if t <= 0.0 || t > 1000.0 {
        return None;
    }

    Some([
        ray.origin.x + t * ray.direction.x,
        0.0,
        ray.origin.z + t * ray.direction.z,
    ])
}

/// Projects a 3D world point back into logical 2D screen coordinates inside `viewport_rect` (`[x, y, w, h]`).
///
/// Returns `None` if the point lies behind the camera near clip threshold or far outside normalized bounds.
#[must_use]
pub fn project_world_point_to_viewport(
    world_pos: [f32; 3],
    viewport_rect: [f32; 4],
    vp_matrix: &cgmath::Matrix4<f32>,
) -> Option<[f32; 2]> {
    let pos_v4 = cgmath::Vector4::new(world_pos[0], world_pos[1], world_pos[2], 1.0);
    let clip_v4 = vp_matrix * pos_v4;

    if clip_v4.w <= 0.001 {
        return None;
    }
    let ndc_x = clip_v4.x / clip_v4.w;
    let ndc_y = clip_v4.y / clip_v4.w;
    if (-1.2..=1.2).contains(&ndc_x) && (-1.2..=1.2).contains(&ndc_y) {
        Some([
            viewport_rect[0] + (ndc_x + 1.0) * 0.5 * viewport_rect[2],
            viewport_rect[1] + (1.0 - ndc_y) * 0.5 * viewport_rect[3],
        ])
    } else {
        None
    }
}

/// Computes a fallback XZ ground coordinate `distance` units ahead of `cam_pos` toward `cam_target`.
#[must_use]
pub fn compute_camera_ground_fallback(
    cam_pos: [f32; 3],
    cam_target: [f32; 3],
    distance: f32,
) -> [f32; 3] {
    let fx = cam_target[0] - cam_pos[0];
    let fz = cam_target[2] - cam_pos[2];
    let len = (fx * fx + fz * fz).sqrt().max(0.001);
    [
        cam_pos[0] + (fx / len) * distance,
        0.0,
        cam_pos[2] + (fz / len) * distance,
    ]
}

/// Resolves a world ground-plane spawn coordinate for a drop inside `viewport_rect` (`[x, y, w, h]`).
///
/// When `drop_screen_pos` is `Some([x, y])`, validates finite bounds inside `viewport_rect`; when `None`,
/// uses the viewport center. Rejects degenerate/non-finite rects (`width <= 20` or `height <= 20`) and
/// points outside the viewport by returning `None`.
#[must_use]
pub fn resolve_ground_drop_world_pos(
    drop_screen_pos: Option<[f32; 2]>,
    viewport_rect: [f32; 4],
    camera: &ae_camera::Camera,
) -> Option<[f32; 3]> {
    let [vx, vy, vw, vh] = viewport_rect;
    if !vx.is_finite() || !vy.is_finite() || !vw.is_finite() || !vh.is_finite() {
        return None;
    }
    if vw <= 20.0 || vh <= 20.0 {
        return None;
    }

    let screen_pos = match drop_screen_pos {
        Some([px, py]) => {
            if !px.is_finite()
                || !py.is_finite()
                || px < vx
                || px > vx + vw
                || py < vy
                || py > vy + vh
            {
                return None;
            }
            [px, py]
        }
        None => [vx + vw * 0.5, vy + vh * 0.5],
    };

    Some(
        compute_ground_intersection(
            screen_pos,
            viewport_rect,
            &camera.build_view_projection_matrix(),
        )
        .unwrap_or_else(|| {
            compute_camera_ground_fallback(
                [camera.position.x, camera.position.y, camera.position.z],
                [camera.target.x, camera.target.y, camera.target.z],
                3.0,
            )
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that intersect_aabb returns a non-negative distance (0.0) when the ray origin is inside the box.
    #[test]
    fn test_intersect_aabb_origin_inside_box_non_negative() {
        let ray = Ray {
            origin: cgmath::Point3::new(0.0, 0.0, 0.0),
            direction: cgmath::Vector3::new(0.0, 0.0, 1.0),
            max_dist: 100.0,
        };
        let min = [-1.0, -1.0, -1.0];
        let max = [1.0, 1.0, 1.0];
        let hit = intersect_aabb(&ray, min, max);
        assert_eq!(hit, Some(0.0));
    }

    #[test]
    fn test_compute_ground_intersection_center_and_horizon() {
        let eye = cgmath::Point3::new(0.0, 10.0, 10.0);
        let center = cgmath::Point3::new(0.0, 0.0, 0.0);
        let up = cgmath::Vector3::unit_y();
        let view = cgmath::Matrix4::look_at_rh(eye, center, up);
        let proj = cgmath::perspective(cgmath::Deg(45.0), 800.0 / 600.0, 0.1, 1000.0);
        let vp = proj * view;
        let rect = [0.0, 0.0, 800.0, 600.0];

        let hit = compute_ground_intersection([400.0, 300.0], rect, &vp)
            .expect("center screen ray hits ground");
        assert!(hit[0].abs() < 0.05);
        assert_eq!(hit[1], 0.0);
        assert!(hit[2].abs() < 0.05);

        let projected = project_world_point_to_viewport(hit, rect, &vp).expect("projects back");
        assert!((projected[0] - 400.0).abs() < 1.0);
        assert!((projected[1] - 300.0).abs() < 1.0);

        let horizon_view =
            cgmath::Matrix4::look_at_rh(eye, cgmath::Point3::new(0.0, 10.0, 0.0), up);
        let horizon_vp = proj * horizon_view;
        assert!(compute_ground_intersection([400.0, 300.0], rect, &horizon_vp).is_none());
        assert!(compute_ground_intersection([400.0, 50.0], rect, &horizon_vp).is_none());
    }

    #[test]
    fn test_compute_camera_ground_fallback() {
        let pt = compute_camera_ground_fallback([0.0, 5.0, 10.0], [0.0, 0.0, 0.0], 3.0);
        assert!((pt[0] - 0.0).abs() < 1e-4);
        assert_eq!(pt[1], 0.0);
        assert!((pt[2] - 7.0).abs() < 1e-4);
    }

    #[test]
    fn test_resolve_ground_drop_world_pos_accepts_center_and_rejects_invalid_bounds() {
        let camera = ae_camera::Camera {
            position: cgmath::Point3::new(0.0, 5.0, 5.0),
            yaw: cgmath::Rad::from(cgmath::Deg(-90.0)),
            pitch: cgmath::Rad::from(cgmath::Deg(-45.0)),
            aspect: 900.0 / 600.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 1000.0,
            mode: ae_camera::ProjectionMode::Perspective,
            ortho_scale: 10.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };
        let viewport = [300.0, 60.0, 900.0, 600.0];

        let center_pos = resolve_ground_drop_world_pos(Some([750.0, 360.0]), viewport, &camera)
            .expect("Inside viewport drop must resolve spawn point");
        assert!(center_pos[0].abs() < 1e-3);
        assert!(center_pos[1].abs() < 1e-3);
        assert!(center_pos[2].abs() < 1e-3);

        let fallback_pos = resolve_ground_drop_world_pos(None, viewport, &camera)
            .expect("Unpositioned drop resolves viewport center");
        assert!((fallback_pos[0] - center_pos[0]).abs() < 1e-3);

        assert!(resolve_ground_drop_world_pos(Some([10.0, 10.0]), viewport, &camera).is_none());
        assert!(resolve_ground_drop_world_pos(None, [0.0, 0.0, 0.0, 0.0], &camera).is_none());
        assert!(
            resolve_ground_drop_world_pos(Some([f32::NAN, 300.0]), viewport, &camera).is_none()
        );
    }
}
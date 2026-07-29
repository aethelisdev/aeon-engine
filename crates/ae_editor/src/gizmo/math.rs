// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Gizmo math helpers — pure functions for ray/segment/plane intersection.
use cgmath::{InnerSpace, Vector3};

/// Finds the closest point and distance between a ray and a line segment.
/// # Arguments
/// * `ro` — Ray origin
/// * `rd` — Ray direction (unnormalized is fine)
/// * `p0` — Segment start
/// * `p1` — Segment end
/// # Returns
/// `(distance, closest_point_on_segment)`
pub fn ray_segment_closest(
    ro: Vector3<f32>,
    rd: Vector3<f32>,
    p0: Vector3<f32>,
    p1: Vector3<f32>,
) -> (f32, Vector3<f32>) {
    let d1 = rd;
    let d2 = p1 - p0;
    let r = ro - p0;

    let a = d1.dot(d1);
    let e = d2.dot(d2);
    let f = d2.dot(r);

    if a < 1e-8 || e < 1e-8 {
        return (r.magnitude(), p0);
    }

    let c = d1.dot(r);
    let b = d1.dot(d2);
    let denom = a * e - b * b;

    let mut s = if denom.abs() > 1e-8 {
        ((b * f - c * e) / denom).max(0.0)
    } else {
        0.0
    };

    let tf = b * s + f;
    let t = if tf < 0.0 {
        s = (-c / a).max(0.0);
        0.0_f32
    } else if tf > e {
        s = ((b - c) / a).max(0.0);
        1.0_f32
    } else {
        tf / e
    };

    let ray_pt = ro + d1 * s;
    let seg_pt = p0 + d2 * t;
    ((ray_pt - seg_pt).magnitude(), seg_pt)
}

/// Computes the intersection point of a ray and a plane.
/// # Arguments
/// * `ro` — Ray origin
/// * `rd` — Ray direction
/// * `plane_origin` — A point on the plane
/// * `plane_normal` — The plane's normal vector
/// # Returns
/// `Some(hit_point)` if the ray intersects the plane in front of the origin, `None` otherwise.
pub fn ray_plane(
    ro: Vector3<f32>,
    rd: Vector3<f32>,
    plane_origin: Vector3<f32>,
    plane_normal: Vector3<f32>,
) -> Option<Vector3<f32>> {
    let denom = rd.dot(plane_normal);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (plane_origin - ro).dot(plane_normal) / denom;
    if t < 0.0 {
        return None;
    }
    Some(ro + rd * t)
}
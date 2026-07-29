// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::ActiveAxis;
/// Gizmo rotate drag — computes rotation angle delta from drag vectors.
use cgmath::{InnerSpace, Vector3};

/// Computes the rotation delta (as Euler angles) from drag start/current vectors.
/// # Arguments
/// * `active_axis` — The rotation axis being dragged
/// * `drag_start_vector` — Normalized direction at drag start (from gizmo center to hit)
/// * `current_vec` — Normalized current direction (from gizmo center to current hit)
/// * `drag_plane_normal` — Normal of the rotation plane
/// # Returns
/// Euler angle delta as `Vector3<f32>`, or `None` if computation fails.
pub fn calculate_rotate_drag(
    active_axis: ActiveAxis,
    drag_start_vector: Vector3<f32>,
    current_vec: Vector3<f32>,
    drag_plane_normal: Vector3<f32>,
) -> Option<Vector3<f32>> {
    if current_vec.magnitude2() < 0.001 || current_vec.x.is_nan() {
        return None;
    }

    let cross = drag_start_vector.cross(current_vec);
    let sign = cross.dot(drag_plane_normal).signum();
    let mut angle = drag_start_vector.dot(current_vec).clamp(-1.0, 1.0).acos();
    if angle.is_nan() {
        return None;
    }
    angle *= sign;

    match active_axis {
        ActiveAxis::X => Some(Vector3::new(angle, 0.0, 0.0)),
        ActiveAxis::Y => Some(Vector3::new(0.0, angle, 0.0)),
        ActiveAxis::Z => Some(Vector3::new(0.0, 0.0, angle)),
        _ => None,
    }
}
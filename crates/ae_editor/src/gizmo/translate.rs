// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::ActiveAxis;
/// Gizmo translate drag — computes axis-constrained translation delta.
use cgmath::{InnerSpace, Quaternion, Rotation, Vector3};

/// Computes the translation delta from a drag plane hit.
/// # Arguments
/// * `active_axis` — The axis being dragged
/// * `current_hit` — The current ray-plane intersection point
/// * `drag_start_world` — The world position where the drag started
/// * `space` — The gizmo coordinate space (World or Local)
/// * `rotation` — Optional entity rotation quaternion for local space transform
/// # Returns
/// Axis-constrained translation delta, or `None` if the axis is invalid.
pub fn calculate_translate_drag(
    active_axis: ActiveAxis,
    current_hit: Vector3<f32>,
    drag_start_world: Vector3<f32>,
    space: super::space::GizmoSpace,
    rotation: Option<Quaternion<f32>>,
) -> Option<Vector3<f32>> {
    let delta = current_hit - drag_start_world;

    if space == super::space::GizmoSpace::Local {
        let rot = rotation.unwrap_or(Quaternion::new(1.0, 0.0, 0.0, 0.0));
        let ax = rot.rotate_vector(Vector3::unit_x());
        let ay = rot.rotate_vector(Vector3::unit_y());
        let az = rot.rotate_vector(Vector3::unit_z());

        match active_axis {
            ActiveAxis::X => Some(ax * delta.dot(ax)),
            ActiveAxis::Y => Some(ay * delta.dot(ay)),
            ActiveAxis::Z => Some(az * delta.dot(az)),
            ActiveAxis::PlaneXY => Some(ax * delta.dot(ax) + ay * delta.dot(ay)),
            ActiveAxis::PlaneXZ => Some(ax * delta.dot(ax) + az * delta.dot(az)),
            ActiveAxis::PlaneYZ => Some(ay * delta.dot(ay) + az * delta.dot(az)),
            ActiveAxis::Free => Some(delta),
            _ => None,
        }
    } else {
        match active_axis {
            ActiveAxis::X => Some(Vector3::new(delta.x, 0.0, 0.0)),
            ActiveAxis::Y => Some(Vector3::new(0.0, delta.y, 0.0)),
            ActiveAxis::Z => Some(Vector3::new(0.0, 0.0, delta.z)),
            ActiveAxis::PlaneXY => Some(Vector3::new(delta.x, delta.y, 0.0)),
            ActiveAxis::PlaneXZ => Some(Vector3::new(delta.x, 0.0, delta.z)),
            ActiveAxis::PlaneYZ => Some(Vector3::new(0.0, delta.y, delta.z)),
            ActiveAxis::Free => Some(delta),
            _ => None,
        }
    }
}
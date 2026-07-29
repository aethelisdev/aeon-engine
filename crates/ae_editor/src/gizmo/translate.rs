// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::ActiveAxis;
/// Gizmo translate drag — computes axis-constrained translation delta.
use cgmath::Vector3;

/// Computes the translation delta from a drag plane hit.
/// # Arguments
/// * `active_axis` — The axis being dragged
/// * `current_hit` — The current ray-plane intersection point
/// * `drag_start_world` — The world position where the drag started
/// # Returns
/// Axis-constrained translation delta, or `None` if the axis is invalid.
pub fn calculate_translate_drag(
    active_axis: ActiveAxis,
    current_hit: Vector3<f32>,
    drag_start_world: Vector3<f32>,
) -> Option<Vector3<f32>> {
    let delta = current_hit - drag_start_world;

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
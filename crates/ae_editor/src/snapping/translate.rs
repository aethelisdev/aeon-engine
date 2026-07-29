// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use cgmath::Vector3;

/// Rounds each component of a translation vector to the nearest grid step.
/// Returns the input unchanged if `grid` is below epsilon (0.001).
pub fn snap_translation(v: Vector3<f32>, grid: f32) -> Vector3<f32> {
    if grid <= 0.001 {
        return v;
    }
    Vector3::new(
        (v.x / grid).round() * grid,
        (v.y / grid).round() * grid,
        (v.z / grid).round() * grid,
    )
}
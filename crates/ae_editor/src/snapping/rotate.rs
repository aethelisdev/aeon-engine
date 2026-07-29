// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Snaps a rotation angle (in radians) to nearest step (in degrees).
pub fn snap_rotation(angle: f32, step_degrees: f32) -> f32 {
    let step = step_degrees.to_radians();
    if step <= 0.0001 {
        return angle;
    }
    (angle / step).round() * step
}
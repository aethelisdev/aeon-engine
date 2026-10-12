// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Math utilities for Inspector rotations and transformations.

/// Converts a normalized quaternion rotation into Euler angles in degrees (Pitch X, Yaw Y, Roll Z).
pub fn quaternion_to_euler_deg(rot: &ae_core::ecs::Rotation) -> [f32; 3] {
    let qx = rot.x;
    let qy = rot.y;
    let qz = rot.z;
    let qw = rot.w;

    // Roll (X)
    let sinr_cosp = 2.0 * (qw * qx + qy * qz);
    let cosr_cosp = 1.0 - 2.0 * (qx * qx + qy * qy);
    let rx = sinr_cosp.atan2(cosr_cosp).to_degrees();

    // Pitch (Y)
    let sinp = 2.0 * (qw * qy - qz * qx);
    let ry = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp).to_degrees()
    } else {
        sinp.asin().to_degrees()
    };

    // Yaw (Z)
    let siny_cosp = 2.0 * (qw * qz + qx * qy);
    let cosy_cosp = 1.0 - 2.0 * (qy * qy + qz * qz);
    let rz = siny_cosp.atan2(cosy_cosp).to_degrees();

    let clean = |v: f32| if v.abs() < 1e-4 { 0.0 } else { v };
    [clean(rx), clean(ry), clean(rz)]
}

/// Converts Euler angles in degrees (X, Y, Z) to normalized quaternion rotation.
pub fn euler_deg_to_quaternion(rx_deg: f32, ry_deg: f32, rz_deg: f32) -> ae_core::ecs::Rotation {
    let rx = rx_deg.to_radians() * 0.5;
    let ry = ry_deg.to_radians() * 0.5;
    let rz = rz_deg.to_radians() * 0.5;

    let cr = rx.cos();
    let sr = rx.sin();
    let cp = ry.cos();
    let sp = ry.sin();
    let cy = rz.cos();
    let sy = rz.sin();

    ae_core::ecs::Rotation {
        w: cr * cp * cy + sr * sp * sy,
        x: sr * cp * cy - cr * sp * sy,
        y: cr * sp * cy + sr * cp * sy,
        z: cr * cp * sy - sr * sp * cy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euler_z_rotation_roundtrip() {
        let quat = euler_deg_to_quaternion(0.0, 0.0, 45.0);
        let euler = quaternion_to_euler_deg(&quat);
        assert!((euler[0] - 0.0).abs() < 1e-3, "rx: {}", euler[0]);
        assert!((euler[1] - 0.0).abs() < 1e-3, "ry: {}", euler[1]);
        assert!((euler[2] - 45.0).abs() < 1e-3, "rz: {}", euler[2]);
    }

    #[test]
    fn test_euler_negative_z_rotation_roundtrip() {
        let quat = euler_deg_to_quaternion(0.0, 0.0, -30.0);
        let euler = quaternion_to_euler_deg(&quat);
        assert!((euler[0] - 0.0).abs() < 1e-3, "rx: {}", euler[0]);
        assert!((euler[1] - 0.0).abs() < 1e-3, "ry: {}", euler[1]);
        assert!((euler[2] - (-30.0)).abs() < 1e-3, "rz: {}", euler[2]);
    }

    #[test]
    fn test_euler_xy_rotation_roundtrip() {
        let quat = euler_deg_to_quaternion(-34.0, 41.0, 0.0);
        let euler = quaternion_to_euler_deg(&quat);
        assert!((euler[0] - (-34.0)).abs() < 1e-3, "rx: {}", euler[0]);
        assert!((euler[1] - 41.0).abs() < 1e-3, "ry: {}", euler[1]);
        assert!((euler[2] - 0.0).abs() < 1e-3, "rz: {}", euler[2]);
    }

    #[test]
    fn test_euler_continuous_y_drag_preserves_x_and_z() {
        let mut quat = euler_deg_to_quaternion(-34.0, 0.0, 0.0);
        for step in 1..=45 {
            let y_val = step as f32;
            let mut euler = quaternion_to_euler_deg(&quat);
            euler[1] = y_val;
            quat = euler_deg_to_quaternion(euler[0], euler[1], euler[2]);

            let result = quaternion_to_euler_deg(&quat);
            assert!(
                (result[0] - (-34.0)).abs() < 1e-2,
                "Step {}: X drifted to {}",
                step,
                result[0]
            );
            assert!(
                (result[1] - y_val).abs() < 1e-2,
                "Step {}: Y drifted to {}",
                step,
                result[1]
            );
            assert!(
                (result[2] - 0.0).abs() < 1e-2,
                "Step {}: Z drifted to {}",
                step,
                result[2]
            );
        }
    }

    #[test]
    fn test_euler_continuous_y_drag_full_range() {
        for y_deg in [-85, -45, 0, 45, 85] {
            let y_val = y_deg as f32;
            let quat = euler_deg_to_quaternion(0.0, y_val, 0.0);
            let result = quaternion_to_euler_deg(&quat);
            assert!((result[0] - 0.0).abs() < 1e-2);
            assert!((result[1] - y_val).abs() < 1e-2);
            assert!((result[2] - 0.0).abs() < 1e-2);
        }
    }
}
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

    [rx, ry, rz]
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
        z: cr * sp * sy - sr * sp * cy,
    }
}
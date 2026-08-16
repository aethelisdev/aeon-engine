// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

/// Helper to draw a single 3-component float (X, Y, Z) input row in the Inspector grid
/// with full wgpu/egui drag, undo snapshot, and reset triggers.
pub(super) fn draw_vec3_row(
    ui: &mut egui::Ui,
    label: &str,
    values: &mut [f32; 3],
    speed: f32,
    decimals: usize,
    reset_val: f32,
) -> (bool, bool, bool, bool, bool) {
    ui.label(label);
    let r_x = ui.add(
        egui::DragValue::new(&mut values[0])
            .prefix("X: ")
            .speed(speed)
            .fixed_decimals(decimals),
    );
    let r_y = ui.add(
        egui::DragValue::new(&mut values[1])
            .prefix("Y: ")
            .speed(speed)
            .fixed_decimals(decimals),
    );
    let r_z = ui.add(
        egui::DragValue::new(&mut values[2])
            .prefix("Z: ")
            .speed(speed)
            .fixed_decimals(decimals),
    );

    let reset_clicked = ui.button("🔄").clicked();
    if reset_clicked {
        values[0] = reset_val;
        values[1] = reset_val;
        values[2] = reset_val;
    }
    ui.end_row();

    let drag_started = r_x.drag_started() || r_y.drag_started() || r_z.drag_started();
    let drag_stopped = r_x.drag_stopped() || r_y.drag_stopped() || r_z.drag_stopped();
    let changed = r_x.changed() || r_y.changed() || r_z.changed();
    let is_dragging = r_x.dragged() || r_y.dragged() || r_z.dragged();

    (
        changed,
        drag_started,
        drag_stopped,
        is_dragging,
        reset_clicked,
    )
}

/// Converts quaternion rotation to Euler angles in degrees (roll, pitch, yaw -> X, Y, Z).
pub(super) fn quaternion_to_euler_deg(q: ae_core::ecs::Rotation) -> [f32; 3] {
    let qx = q.x;
    let qy = q.y;
    let qz = q.z;
    let qw = q.w;

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
pub(super) fn euler_deg_to_quaternion(
    rx_deg: f32,
    ry_deg: f32,
    rz_deg: f32,
) -> ae_core::ecs::Rotation {
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
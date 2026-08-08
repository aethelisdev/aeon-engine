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
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # Declarative 3D Vector Component & Quaternion Euler Binders
//!
//! Provides two-way bound declarative [`UiScope`] property methods that directly mutate
//! split scalar coordinates `(&mut f32, &mut f32, &mut f32)` and normalized quaternion
//! components `(&mut f32, &mut f32, &mut f32, &mut f32)` in-place while retaining continuous
//! Euler angle representations inside [`iris_core::dispatch::types::EventState`] to prevent
//! gimbal-lock pitch wrapping past $\pm 90^\circ$.

use crate::controls::numeric_drag::DragCurve;
use crate::declarative::scope::core::UiScope;
use crate::declarative::scope::properties::vectors::{PropertyVec3Options, PropertyVec3Response};

/// Converts a normalized quaternion `(qx, qy, qz, qw)` into Euler angles in degrees `[rx, ry, rz]`.
#[inline]
#[must_use]
pub fn quat_xyzw_to_euler_deg(qx: f32, qy: f32, qz: f32, qw: f32) -> [f32; 3] {
    let sinr_cosp = 2.0 * (qw * qx + qy * qz);
    let cosr_cosp = 1.0 - 2.0 * (qx * qx + qy * qy);
    let rx = sinr_cosp.atan2(cosr_cosp).to_degrees();

    let sinp = 2.0 * (qw * qy - qz * qx);
    let ry = if sinp.abs() >= 1.0 {
        std::f32::consts::FRAC_PI_2.copysign(sinp).to_degrees()
    } else {
        sinp.asin().to_degrees()
    };

    let siny_cosp = 2.0 * (qw * qz + qx * qy);
    let cosy_cosp = 1.0 - 2.0 * (qy * qy + qz * qz);
    let rz = siny_cosp.atan2(cosy_cosp).to_degrees();

    let clean = |v: f32| if v.abs() < 1e-4 { 0.0 } else { v };
    [clean(rx), clean(ry), clean(rz)]
}

/// Converts Euler angles in degrees `(rx_deg, ry_deg, rz_deg)` into a normalized quaternion `[qx, qy, qz, qw]`.
#[inline]
#[must_use]
pub fn euler_deg_to_quat_xyzw(rx_deg: f32, ry_deg: f32, rz_deg: f32) -> [f32; 4] {
    let rx = rx_deg.to_radians() * 0.5;
    let ry = ry_deg.to_radians() * 0.5;
    let rz = rz_deg.to_radians() * 0.5;

    let cr = rx.cos();
    let sr = rx.sin();
    let cp = ry.cos();
    let sp = ry.sin();
    let cy = rz.cos();
    let sy = rz.sin();

    [
        sr * cp * cy - cr * sp * sy,
        cr * sp * cy + sr * cp * sy,
        cr * cp * sy - sr * sp * cy,
        cr * cp * cy + sr * sp * sy,
    ]
}

/// Resolves Euler angles in degrees from quaternion components `(qx, qy, qz, qw)`,
/// preserving `cached_euler` when its reconstructed quaternion matches within gimbal equivalence (`|dot| > 0.999`).
#[inline]
#[must_use]
pub fn resolve_euler_deg_from_quat_xyzw(
    qx: f32,
    qy: f32,
    qz: f32,
    qw: f32,
    cached_euler: Option<&[f32; 3]>,
) -> [f32; 3] {
    if let Some(euler) = cached_euler {
        let [cx, cy, cz, cw] = euler_deg_to_quat_xyzw(euler[0], euler[1], euler[2]);
        let dot = (cx * qx + cy * qy + cz * qz + cw * qw).abs();
        if dot > 0.999 {
            return *euler;
        }
    }
    quat_xyzw_to_euler_deg(qx, qy, qz, qw)
}

impl<'a> UiScope<'a> {
    /// Emits a declarative two-way bound 3D vector property row that directly mutates
    /// split scalar component references `(&mut x, &mut y, &mut z)` in-place.
    ///
    /// Uses a quadratic response curve (`decimals: 2`) suitable for 3D spatial coordinates and scale factors.
    pub fn prop_vec3_xyz(
        &mut self,
        label: &str,
        xyz: (&mut f32, &mut f32, &mut f32),
        speed: f32,
        reset_value: Option<[f32; 3]>,
    ) -> PropertyVec3Response {
        let mut vals = [*xyz.0, *xyz.1, *xyz.2];
        let mut opts = PropertyVec3Options::new(speed)
            .with_decimals(2)
            .with_curve(DragCurve::quadratic());
        opts.reset_value = reset_value;

        let resp = self.property_vec3_with_options(label, &mut vals, opts);
        if resp.changed() {
            *xyz.0 = vals[0];
            *xyz.1 = vals[1];
            *xyz.2 = vals[2];
        }
        resp
    }

    /// Emits a declarative two-way bound 3D Euler angle property row (in degrees) that directly
    /// mutates normalized quaternion components `(&mut qx, &mut qy, &mut qz, &mut qw)` in-place.
    ///
    /// Automatically retains the continuous Euler angle representation inside [`UiTree::event_state`](iris_core::UiTree)
    /// across frames so pitch rotations past $\pm 90^\circ$ do not wrap or jump due to gimbal equivalence.
    pub fn prop_vec3_euler_quat(
        &mut self,
        label: &str,
        quat_xyzw: (&mut f32, &mut f32, &mut f32, &mut f32),
        speed: f32,
        reset_value: Option<[f32; 3]>,
    ) -> PropertyVec3Response {
        let base_tag = self.tag_for(label);
        let cached = self
            .tree
            .event_state
            .cached_euler
            .and_then(|(tag, euler)| (tag == base_tag).then_some(euler));
        let mut euler = resolve_euler_deg_from_quat_xyzw(
            *quat_xyzw.0,
            *quat_xyzw.1,
            *quat_xyzw.2,
            *quat_xyzw.3,
            cached.as_ref(),
        );

        let mut opts = PropertyVec3Options::new(speed)
            .with_decimals(1)
            .with_curve(DragCurve::Linear);
        opts.reset_value = reset_value;

        let resp = self.property_vec3_with_options(label, &mut euler, opts);
        if resp.changed() {
            let [nx, ny, nz, nw] = euler_deg_to_quat_xyzw(euler[0], euler[1], euler[2]);
            *quat_xyzw.0 = nx;
            *quat_xyzw.1 = ny;
            *quat_xyzw.2 = nz;
            *quat_xyzw.3 = nw;
        }
        self.tree.event_state.set_cached_euler(base_tag, euler);
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::declarative::scope::properties::property_vec3_reset_tag;
    use iris_core::UiTree;

    #[test]
    fn test_prop_vec3_xyz_mutates_scalars_in_place_on_reset() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let base_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for("Position")
        };
        tree.event_state.frame_clicked_tag = Some(property_vec3_reset_tag(base_tag));

        let (mut x, mut y, mut z) = (3.0, 4.0, 5.0);
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 0.1, Some([0.0; 3]));
        assert!(resp.changed());
        assert!(resp.reset_clicked);
        assert_eq!([x, y, z], [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_prop_vec3_euler_quat_retains_euler_past_90_deg_and_resets() {
        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let rot_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for("Rotation")
        };

        let [mut qx, mut qy, mut qz, mut qw] = euler_deg_to_quat_xyzw(-34.0, 120.0, 0.0);
        tree.event_state
            .set_cached_euler(rot_tag, [-34.0, 120.0, 0.0]);

        {
            let mut scope = UiScope::new(&mut tree, root);
            let resp = scope.prop_vec3_euler_quat(
                "Rotation",
                (&mut qx, &mut qy, &mut qz, &mut qw),
                0.5,
                Some([0.0, 0.0, 0.0]),
            );
            assert!(!resp.changed());
            assert_eq!(resp.new_values, [-34.0, 120.0, 0.0]);
        }

        // Trigger reset button click
        tree.event_state.frame_clicked_tag = Some(property_vec3_reset_tag(rot_tag));
        let mut scope = UiScope::new(&mut tree, root);
        let resp = scope.prop_vec3_euler_quat(
            "Rotation",
            (&mut qx, &mut qy, &mut qz, &mut qw),
            0.5,
            Some([0.0, 0.0, 0.0]),
        );
        assert!(resp.changed());
        assert_eq!([qx, qy, qz, qw], [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_numeric_drag_and_enter_and_reset_undo_redo_via_ctrl_z_y() {
        use crate::declarative::scope::properties::property_vec3_axis_tag;
        use iris_core::{KeyCode, Point, UiEvent};

        let mut tree = UiTree::new();
        let root = tree.create_root().expect("root");
        let base_tag = {
            let scope = UiScope::new(&mut tree, root);
            scope.tag_for("Position")
        };
        let x_tag = property_vec3_axis_tag(base_tag, 0);

        let (mut x, mut y, mut z) = (10.0, 20.0, 30.0);

        // 1. Simulate dragging X axis from 10.0 across two frames, then releasing
        tree.event_state.pointer_capture_tag = Some(x_tag);
        tree.event_state.is_pointer_drag_active = true;
        tree.event_state.frame_drag_delta = Some(Point::new(15.0, 0.0));
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp =
                scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
            assert!(resp.changed());
        }
        assert!(x > 10.0);
        let dragged_x = x;

        // Release drag
        tree.event_state.frame_drag_delta = None;
        tree.event_state.is_pointer_drag_active = false;
        tree.event_state.pointer_capture_tag = None;
        tree.event_state.frame_drag_released_tag = Some(x_tag);
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp =
                scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
            assert!(resp.committed());
        }
        tree.event_state.frame_drag_released_tag = None;

        // Press Ctrl+Z -> should undo drag back to 10.0
        let undo_res = tree.dispatch_event(&UiEvent::KeyboardInput {
            key: KeyCode::Character('z'),
            text: None,
            is_pressed: true,
            ctrl_held: true,
            shift_held: false,
        });
        assert!(undo_res.consumed);
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp =
                scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
            assert!(resp.changed());
        }
        assert!((x - 10.0).abs() < 1e-4);

        // Press Ctrl+Y -> should redo drag back to dragged_x
        let redo_res = tree.dispatch_event(&UiEvent::KeyboardInput {
            key: KeyCode::Character('y'),
            text: None,
            is_pressed: true,
            ctrl_held: true,
            shift_held: false,
        });
        assert!(redo_res.consumed);
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp =
                scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
            assert!(resp.changed());
        }
        assert!((x - dragged_x).abs() < 1e-4);

        // 2. Click X axis to start inline edit, type "42", press Enter, then press Ctrl+Z
        tree.event_state.frame_clicked_tag = Some(x_tag);
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let _ = scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
        }
        tree.event_state.frame_clicked_tag = None;
        assert!(tree.event_state.active_text_edit.is_some());

        // Type "42"
        let _ = tree.dispatch_event(&UiEvent::KeyboardInput {
            key: KeyCode::Character('4'),
            text: Some("42".to_string()),
            is_pressed: true,
            ctrl_held: false,
            shift_held: false,
        });
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let _ = scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
        }

        // Press Enter to commit "42"
        let _ = tree.dispatch_event(&UiEvent::KeyboardInput {
            key: KeyCode::Enter,
            text: None,
            is_pressed: true,
            ctrl_held: false,
            shift_held: false,
        });
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp =
                scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
            assert!(resp.changed());
            assert!(resp.committed());
        }
        assert!((x - 42.0).abs() < 1e-4);
        assert!(tree.event_state.active_text_edit.is_none());

        // Press Ctrl+Z after Enter -> should revert 42.0 back to dragged_x!
        let undo_enter = tree.dispatch_event(&UiEvent::KeyboardInput {
            key: KeyCode::Character('z'),
            text: None,
            is_pressed: true,
            ctrl_held: true,
            shift_held: false,
        });
        assert!(undo_enter.consumed);
        {
            let _ = tree.clear_children(root);
            let mut scope = UiScope::new(&mut tree, root);
            let resp =
                scope.prop_vec3_xyz("Position", (&mut x, &mut y, &mut z), 1.0, Some([0.0; 3]));
            assert!(resp.changed());
        }
        assert!((x - dragged_x).abs() < 1e-4);
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::GizmoScreenParams;
use super::core::{ActiveAxis, GizmoMode, GizmoSystem};
use super::math::ray_plane;
/// Gizmo input — handles the drag lifecycle (start, calculate, end).
use cgmath::{InnerSpace, Vector3};

impl GizmoSystem {
    /// Top-level input handler called each frame.
    /// 1. Updates hover state (when not dragging).
    /// 2. Starts drag on `left_just_pressed`.
    /// 3. Computes delta during drag.
    /// 4. Ends drag on `left_released`.
    /// Returns `Some(delta)` when a drag produces movement.
    pub fn handle_input(
        &mut self,
        ray_origin: Vector3<f32>,
        ray_dir: Vector3<f32>,
        gizmo_pos: Vector3<f32>,
        camera_pos: Vector3<f32>,
        cam_forward: Vector3<f32>,
        screen: &GizmoScreenParams,
        left_just_pressed: bool,
        left_pressed: bool,
        left_released: bool,
    ) -> Option<Vector3<f32>> {
        let dist_cam = (camera_pos - gizmo_pos)
            .dot(cam_forward.normalize())
            .abs()
            .max(1e-6);
        self.drag_scale = screen.axis_length_world(dist_cam);

        let cam_f = cam_forward.normalize();
        let right_dir = cam_f.cross(Vector3::unit_y());
        let right_dir = if right_dir.magnitude2() < 0.001 {
            cam_f.cross(Vector3::unit_z()).normalize()
        } else {
            right_dir.normalize()
        };
        let up_dir = right_dir.cross(cam_f).normalize();
        self.cam_right.set(right_dir);
        self.cam_up.set(up_dir);

        // 1) Update hover only when not dragging (stabilizes state)
        if !self.is_dragging {
            self.check_intersection(
                ray_origin,
                ray_dir,
                gizmo_pos,
                camera_pos,
                cam_forward,
                screen,
            );
        }

        // 2) End drag on release (priority)
        if left_released {
            self.end_drag();
            return None;
        }

        // 3) Start drag
        if left_just_pressed && !self.is_dragging {
            let _ = self.start_drag(ray_origin, ray_dir, gizmo_pos, cam_forward);
        }

        // 4) Calculate drag delta
        if left_pressed && self.is_dragging {
            return self.calculate_drag(ray_origin, ray_dir, gizmo_pos);
        }

        None
    }

    /// Begins a drag operation if a valid axis is hovered.
    pub fn start_drag(
        &mut self,
        ray_origin: Vector3<f32>,
        ray_dir: Vector3<f32>,
        gizmo_pos: Vector3<f32>,
        cam_forward: Vector3<f32>,
    ) -> bool {
        if self.hovered_axis == ActiveAxis::None {
            return false;
        }

        if !self.is_handle_allowed(self.hovered_axis) {
            return false;
        }

        self.active_axis = self.hovered_axis;
        self.is_dragging = true;
        self.drag_gizmo_pos = gizmo_pos;
        self.drag_scale_factor = 1.0;

        if self.mode == GizmoMode::Rotate {
            let axis_normal = match self.active_axis {
                ActiveAxis::X => self.oriented_axis(Vector3::unit_x()),
                ActiveAxis::Y => self.oriented_axis(Vector3::unit_y()),
                ActiveAxis::Z => self.oriented_axis(Vector3::unit_z()),
                ActiveAxis::PlaneYZ => self.oriented_axis(Vector3::unit_x()),
                ActiveAxis::PlaneXZ => self.oriented_axis(Vector3::unit_y()),
                ActiveAxis::PlaneXY => self.oriented_axis(Vector3::unit_z()),
                _ => cam_forward.normalize(),
            };
            let cam_f = cam_forward.normalize();
            if cam_f.dot(axis_normal).abs() > 0.15 {
                self.drag_plane_normal = axis_normal;
            } else {
                self.drag_plane_normal = cam_f;
            }
        } else {
            let cam_f = cam_forward.normalize();
            self.drag_plane_normal = match self.active_axis {
                ActiveAxis::X => {
                    let axis_dir = self.oriented_axis(Vector3::unit_x());
                    let mut n = axis_dir.cross(cam_f).cross(axis_dir);
                    if n.magnitude2() < 0.001 {
                        n = axis_dir.cross(self.oriented_axis(Vector3::unit_y()));
                    }
                    n.normalize()
                }
                ActiveAxis::Y => {
                    let axis_dir = self.oriented_axis(Vector3::unit_y());
                    let mut n = axis_dir.cross(cam_f).cross(axis_dir);
                    if n.magnitude2() < 0.001 {
                        n = axis_dir.cross(self.oriented_axis(Vector3::unit_x()));
                    }
                    n.normalize()
                }
                ActiveAxis::Z => {
                    let axis_dir = self.oriented_axis(Vector3::unit_z());
                    let mut n = axis_dir.cross(cam_f).cross(axis_dir);
                    if n.magnitude2() < 0.001 {
                        n = axis_dir.cross(self.oriented_axis(Vector3::unit_x()));
                    }
                    n.normalize()
                }
                ActiveAxis::PlaneXY => {
                    let normal = self.oriented_axis(Vector3::unit_z());
                    if cam_f.dot(normal).abs() > 0.15 {
                        normal
                    } else {
                        cam_f
                    }
                }
                ActiveAxis::PlaneXZ => {
                    let normal = self.oriented_axis(Vector3::unit_y());
                    if cam_f.dot(normal).abs() > 0.15 {
                        normal
                    } else {
                        cam_f
                    }
                }
                ActiveAxis::PlaneYZ => {
                    let normal = self.oriented_axis(Vector3::unit_x());
                    if cam_f.dot(normal).abs() > 0.15 {
                        normal
                    } else {
                        cam_f
                    }
                }
                _ => cam_f,
            };
        }

        let hit =
            ray_plane(ray_origin, ray_dir, gizmo_pos, self.drag_plane_normal).unwrap_or(gizmo_pos);
        self.drag_start_world = hit;
        self.drag_current_hit = hit;
        let mut start_vec = (hit - gizmo_pos).normalize();
        if start_vec.magnitude2().is_nan() || start_vec.magnitude2() < 0.001 {
            start_vec = Vector3::new(1., 0., 0.);
        }
        self.drag_start_vector = start_vec;
        self.drag_current_vector = start_vec;

        true
    }

    /// Computes the drag delta by delegating to translate or rotate modules.
    pub fn calculate_drag(
        &mut self,
        ray_origin: Vector3<f32>,
        ray_dir: Vector3<f32>,
        _gizmo_pos: Vector3<f32>,
    ) -> Option<Vector3<f32>> {
        if !self.is_dragging || self.active_axis == ActiveAxis::None {
            return None;
        }

        let current_hit = ray_plane(
            ray_origin,
            ray_dir,
            self.drag_gizmo_pos,
            self.drag_plane_normal,
        )?;
        self.drag_current_hit = current_hit;

        if self.mode == GizmoMode::Rotate {
            let current_vec = (current_hit - self.drag_gizmo_pos).normalize();
            if current_vec.magnitude2() < 0.001 || current_vec.x.is_nan() {
                return None;
            }
            self.drag_current_vector = current_vec;
            super::rotate::calculate_rotate_drag(
                self.active_axis,
                self.drag_start_vector,
                current_vec,
                self.drag_plane_normal,
            )
        } else {
            // Both Translate and Scale use the same axis-constrained delta logic
            if self.mode == GizmoMode::Scale && self.active_axis == ActiveAxis::Free {
                // Ratio-based uniform scale drag logic with O-ring radius clamp to prevent division by zero / hypersensitivity!
                let d_start = (self.drag_start_world - self.drag_gizmo_pos)
                    .magnitude()
                    .max(self.drag_scale * 0.15)
                    .max(1e-6);
                let d_current = (current_hit - self.drag_gizmo_pos).magnitude();
                let scale_factor = d_current / d_start;
                self.drag_scale_factor = scale_factor.max(0.1);
                let d_uniform = scale_factor - 1.0;
                Some(Vector3::new(d_uniform, d_uniform, d_uniform))
            } else if self.space == super::space::GizmoSpace::Local {
                // Local space: project the world-space delta onto the oriented axis.
                // For Translate mode, return the world-space translation vector (oriented).
                // For Scale mode, return the scale factor delta along each local axis.
                let delta = current_hit - self.drag_start_world;
                if self.mode == GizmoMode::Translate {
                    match self.active_axis {
                        ActiveAxis::X => {
                            let axis = self.oriented_axis(Vector3::unit_x());
                            Some(axis * delta.dot(axis))
                        }
                        ActiveAxis::Y => {
                            let axis = self.oriented_axis(Vector3::unit_y());
                            Some(axis * delta.dot(axis))
                        }
                        ActiveAxis::Z => {
                            let axis = self.oriented_axis(Vector3::unit_z());
                            Some(axis * delta.dot(axis))
                        }
                        ActiveAxis::PlaneXY => {
                            let ax = self.oriented_axis(Vector3::unit_x());
                            let ay = self.oriented_axis(Vector3::unit_y());
                            Some(ax * delta.dot(ax) + ay * delta.dot(ay))
                        }
                        ActiveAxis::PlaneXZ => {
                            let ax = self.oriented_axis(Vector3::unit_x());
                            let az = self.oriented_axis(Vector3::unit_z());
                            Some(ax * delta.dot(ax) + az * delta.dot(az))
                        }
                        ActiveAxis::PlaneYZ => {
                            let ay = self.oriented_axis(Vector3::unit_y());
                            let az = self.oriented_axis(Vector3::unit_z());
                            Some(ay * delta.dot(ay) + az * delta.dot(az))
                        }
                        ActiveAxis::Free => Some(delta),
                        _ => None,
                    }
                } else {
                    // Scale mode
                    match self.active_axis {
                        ActiveAxis::X => {
                            let axis = self.oriented_axis(Vector3::unit_x());
                            let d = delta.dot(axis);
                            Some(Vector3::new(d, 0.0, 0.0))
                        }
                        ActiveAxis::Y => {
                            let axis = self.oriented_axis(Vector3::unit_y());
                            let d = delta.dot(axis);
                            Some(Vector3::new(0.0, d, 0.0))
                        }
                        ActiveAxis::Z => {
                            let axis = self.oriented_axis(Vector3::unit_z());
                            let d = delta.dot(axis);
                            Some(Vector3::new(0.0, 0.0, d))
                        }
                        ActiveAxis::PlaneXY => {
                            let ax = self.oriented_axis(Vector3::unit_x());
                            let ay = self.oriented_axis(Vector3::unit_y());
                            Some(Vector3::new(delta.dot(ax), delta.dot(ay), 0.0))
                        }
                        ActiveAxis::PlaneXZ => {
                            let ax = self.oriented_axis(Vector3::unit_x());
                            let az = self.oriented_axis(Vector3::unit_z());
                            Some(Vector3::new(delta.dot(ax), 0.0, delta.dot(az)))
                        }
                        ActiveAxis::PlaneYZ => {
                            let ay = self.oriented_axis(Vector3::unit_y());
                            let az = self.oriented_axis(Vector3::unit_z());
                            Some(Vector3::new(0.0, delta.dot(ay), delta.dot(az)))
                        }
                        ActiveAxis::Free => Some(delta),
                        _ => None,
                    }
                }
            } else {
                super::translate::calculate_translate_drag(
                    self.active_axis,
                    current_hit,
                    self.drag_start_world,
                )
            }
        }
    }

    /// Ends the current drag operation and resets state.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.active_axis = ActiveAxis::None;
        self.hovered_axis = ActiveAxis::None;
        self.drag_scale_factor = 1.0;
    }
}
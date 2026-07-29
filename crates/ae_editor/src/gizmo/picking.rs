// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::{ActiveAxis, GizmoMode, GizmoScreenParams, GizmoSystem};
use super::math::{ray_plane, ray_segment_closest};
/// Gizmo picking — raycast intersection against gizmo axes and plane handles.
use cgmath::{InnerSpace, Rotation as _, Vector3};

impl GizmoSystem {
    /// Determines which axis handle the mouse ray is hovering over.
    /// Updates `self.hovered_axis` and returns the detected axis.
    pub fn check_intersection(
        &mut self,
        ray_origin: Vector3<f32>,
        ray_dir: Vector3<f32>,
        gizmo_pos: Vector3<f32>,
        camera_pos: Vector3<f32>,
        cam_forward: Vector3<f32>,
        screen: &GizmoScreenParams,
    ) -> ActiveAxis {
        let forward = cam_forward.normalize();
        let dist_cam = (camera_pos - gizmo_pos).dot(forward).abs().max(1e-6);

        let axis_len_world = screen.axis_length_world(dist_cam);
        let pick_radius_world = screen.pick_radius_world(dist_cam);

        let mut best_axis = ActiveAxis::None;
        let mut min_dist = pick_radius_world;

        if self.mode == GizmoMode::Rotate {
            let radius = axis_len_world;
            let planes = [
                (ActiveAxis::X, self.oriented_axis(Vector3::unit_x())),
                (ActiveAxis::Y, self.oriented_axis(Vector3::unit_y())),
                (ActiveAxis::Z, self.oriented_axis(Vector3::unit_z())),
            ];

            let mut min_cam_dist = f32::MAX;
            for (axis, normal) in &planes {
                if let Some(hit) = ray_plane(ray_origin, ray_dir, gizmo_pos, *normal) {
                    let d_center = (hit - gizmo_pos).magnitude();
                    if (d_center - radius).abs() < pick_radius_world * 2.5 {
                        let cam_d = (hit - camera_pos).magnitude();
                        if cam_d < min_cam_dist {
                            min_cam_dist = cam_d;
                            best_axis = *axis;
                        }
                    }
                }
            }
        } else {
            let mut is_free_handle = false;
            if self.mode == GizmoMode::Scale || self.mode == GizmoMode::Translate {
                if let Some(hit) = ray_plane(ray_origin, ray_dir, gizmo_pos, forward) {
                    let dist_center_plane = (hit - gizmo_pos).magnitude();
                    let o_ring_radius = axis_len_world * 0.15;
                    if dist_center_plane < o_ring_radius + pick_radius_world * 3.0 {
                        is_free_handle = true;
                    }
                }
            }

            if is_free_handle {
                best_axis = ActiveAxis::Free;
            } else {
                let axes = [
                    (ActiveAxis::X, self.oriented_axis(Vector3::unit_x())),
                    (ActiveAxis::Y, self.oriented_axis(Vector3::unit_y())),
                    (ActiveAxis::Z, self.oriented_axis(Vector3::unit_z())),
                ];

                for (axis, dir) in &axes {
                    let p1 = gizmo_pos + *dir * axis_len_world;
                    let (dist, _) = ray_segment_closest(ray_origin, ray_dir, gizmo_pos, p1);

                    if dist < min_dist {
                        min_dist = dist;
                        best_axis = *axis;
                    }
                }

                if self.mode == GizmoMode::Translate {
                    let radius = 0.03;
                    let offset_world = (radius * 8.0) * axis_len_world;
                    let size_world = (radius * 6.0) * axis_len_world;

                    let planes = [
                        (ActiveAxis::PlaneXY, self.oriented_axis(Vector3::unit_z())),
                        (ActiveAxis::PlaneXZ, self.oriented_axis(Vector3::unit_y())),
                        (ActiveAxis::PlaneYZ, self.oriented_axis(Vector3::unit_x())),
                    ];

                    let mut closest_cam_dist = f32::MAX;

                    for (p_axis, normal) in &planes {
                        if let Some(hit) = ray_plane(ray_origin, ray_dir, gizmo_pos, *normal) {
                            let local = hit - gizmo_pos;
                            let mut in_quad = false;
                            match p_axis {
                                ActiveAxis::PlaneXY => {
                                    if local.x >= offset_world
                                        && local.x <= offset_world + size_world
                                        && local.y >= offset_world
                                        && local.y <= offset_world + size_world
                                    {
                                        in_quad = true;
                                    }
                                }
                                ActiveAxis::PlaneXZ => {
                                    if local.x >= offset_world
                                        && local.x <= offset_world + size_world
                                        && local.z >= offset_world
                                        && local.z <= offset_world + size_world
                                    {
                                        in_quad = true;
                                    }
                                }
                                ActiveAxis::PlaneYZ => {
                                    if local.y >= offset_world
                                        && local.y <= offset_world + size_world
                                        && local.z >= offset_world
                                        && local.z <= offset_world + size_world
                                    {
                                        in_quad = true;
                                    }
                                }
                                _ => {}
                            }

                            if in_quad {
                                let cam_dist = (hit - camera_pos).magnitude();
                                if cam_dist < closest_cam_dist {
                                    closest_cam_dist = cam_dist;
                                    best_axis = *p_axis;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.hovered_axis = best_axis;
        best_axis
    }

    /// Returns the given world-axis direction rotated by the entity's rotation
    /// when in Local space mode, or the original direction in World mode.
    pub(crate) fn oriented_axis(&self, world_dir: Vector3<f32>) -> Vector3<f32> {
        match self.space {
            super::space::GizmoSpace::Local => self.entity_rotation.rotate_vector(world_dir),
            super::space::GizmoSpace::World => world_dir,
        }
    }
}
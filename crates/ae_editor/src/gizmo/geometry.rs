// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::{ActiveAxis, GizmoMode, GizmoSystem};
use super::render::GizmoVertex;
/// Aeon Engine - Gizmo Geometry Generation
/// This module handles mathematical geometry generation for the 3D translation, rotation, and scale gizmo handles.
/// It constructs procedural vertex data for torus rings, axis arrows, planar quads, and uniform scaling O-rings.
use cgmath::{InnerSpace, Rotation as _, Vector3};

impl GizmoSystem {
    /// Generates a torus (donut) mesh as TriangleList vertices.
    /// Used for rotation ring handles.
    /// ### Arguments
    /// * `major_radius` - The radius from the center of the torus to the center of the tube.
    /// * `minor_radius` - The radius of the tube itself.
    /// * `segments` - Number of segments around the major circumference.
    /// * `cross_segments` - Number of segments around the minor tube circumference.
    /// * `color` - RGB color of the torus vertices.
    /// * `axis` - The active axis defining the rotation plane of the torus (X, Y, or Z).
    pub(crate) fn build_torus(
        major_radius: f32,
        minor_radius: f32,
        segments: usize,
        cross_segments: usize,
        color: [f32; 3],
        axis: ActiveAxis,
    ) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();

        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;

            let (sin1, cos1) = a1.sin_cos();
            let (sin2, cos2) = a2.sin_cos();

            for j in 0..cross_segments {
                let ca1 = (j as f32) / (cross_segments as f32) * std::f32::consts::TAU;
                let ca2 = ((j + 1) as f32) / (cross_segments as f32) * std::f32::consts::TAU;

                let (csin1, ccos1) = ca1.sin_cos();
                let (csin2, ccos2) = ca2.sin_cos();

                let cp = |a_cos: f32, a_sin: f32, ca_cos: f32, ca_sin: f32| -> [f32; 3] {
                    let rx = (major_radius + minor_radius * ca_cos) * a_cos;
                    let ry = (major_radius + minor_radius * ca_cos) * a_sin;
                    let rz = minor_radius * ca_sin;

                    match axis {
                        ActiveAxis::X => [rz, rx, ry],
                        ActiveAxis::Y => [rx, rz, ry],
                        ActiveAxis::Z => [rx, ry, rz],
                        _ => [0., 0., 0.],
                    }
                };

                let p1 = cp(cos1, sin1, ccos1, csin1);
                let p2 = cp(cos2, sin2, ccos1, csin1);
                let p3 = cp(cos2, sin2, ccos2, csin2);
                let p4 = cp(cos1, sin1, ccos2, csin2);

                let v = |p| GizmoVertex { position: p, color };
                vertices.extend_from_slice(&[v(p1), v(p2), v(p3), v(p1), v(p3), v(p4)]);
            }
        }
        vertices
    }

    /// Generates a colored quad on the specified plane for planar translation handles.
    /// ### Arguments
    /// * `offset` - Distance from the center where the plane quad starts.
    /// * `size` - Width and height of the square quad.
    /// * `color` - RGB color of the plane quad vertices.
    /// * `axis` - The planar translation axis (PlaneXY, PlaneXZ, or PlaneYZ).
    pub(crate) fn build_plane_quad(
        offset: f32,
        size: f32,
        color: [f32; 3],
        axis: ActiveAxis,
    ) -> Vec<GizmoVertex> {
        let v = |p: [f32; 3]| GizmoVertex { position: p, color };
        let mut b = Vec::new();
        let (p0, p1, p2, p3) = match axis {
            ActiveAxis::PlaneXY => (
                [offset, offset, 0.0],
                [offset + size, offset, 0.0],
                [offset + size, offset + size, 0.0],
                [offset, offset + size, 0.0],
            ),
            ActiveAxis::PlaneXZ => (
                [offset, 0.0, offset],
                [offset + size, 0.0, offset],
                [offset + size, 0.0, offset + size],
                [offset, 0.0, offset + size],
            ),
            ActiveAxis::PlaneYZ => (
                [0.0, offset, offset],
                [0.0, offset + size, offset],
                [0.0, offset + size, offset + size],
                [0.0, offset, offset + size],
            ),
            _ => return b,
        };
        // Backface culling is off, so one quad is enough. We add two triangles.
        b.extend_from_slice(&[v(p0), v(p1), v(p2), v(p0), v(p2), v(p3)]);
        b
    }

    /// Generates rotation ring geometry (3 torus rings: X=Red, Y=Green, Z=Blue).
    /// ### Arguments
    /// * `radius` - The major radius of the rotation handles.
    pub(crate) fn build_rotation_vertices(radius: f32) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let thickness = radius * 0.04;

        let red = [1.0, 0.2, 0.2];
        let green = [0.2, 1.0, 0.2];
        let blue = [0.2, 0.4, 1.0];

        vertices.extend(Self::build_torus(
            radius,
            thickness,
            64,
            12,
            red,
            ActiveAxis::X,
        ));
        vertices.extend(Self::build_torus(
            radius,
            thickness,
            64,
            12,
            green,
            ActiveAxis::Y,
        ));
        vertices.extend(Self::build_torus(
            radius,
            thickness,
            64,
            12,
            blue,
            ActiveAxis::Z,
        ));

        vertices
    }

    /// Generates a cylinder + cone/cube arrow for one axis.
    /// ### Arguments
    /// * `len` - Total length of the arrow.
    /// * `radius` - Radius of the cylinder shaft.
    /// * `segments` - Tessellation segments for the cylinder and cone base.
    /// * `color` - RGB color of the arrow vertices.
    /// * `axis` - The axis (X, Y, or Z) along which the arrow is pointing.
    /// * `is_scale` - True if rendering a box tip for scale mode, false for a cone tip in translate mode.
    pub(crate) fn build_arrow(
        len: f32,
        radius: f32,
        segments: usize,
        color: [f32; 3],
        axis: ActiveAxis,
        is_scale: bool,
    ) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let head_len = len * 0.25;
        let cyl_len = len - head_len;
        let head_radius = if is_scale { radius * 2.5 } else { radius * 3.0 };

        let tf = |x: f32, y: f32, z: f32| -> [f32; 3] {
            match axis {
                ActiveAxis::X => [x, y, z],
                ActiveAxis::Y => [y, x, z],
                ActiveAxis::Z => [y, z, x],
                _ => [0., 0., 0.],
            }
        };
        let v = |p: [f32; 3]| GizmoVertex { position: p, color };

        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;

            let (s1, c1) = a1.sin_cos();
            let (s2, c2) = a2.sin_cos();

            let p1 = tf(0.0, c1 * radius, s1 * radius);
            let p2 = tf(0.0, c2 * radius, s2 * radius);
            let p3 = tf(cyl_len, c2 * radius, s2 * radius);
            let p4 = tf(cyl_len, c1 * radius, s1 * radius);

            vertices.extend_from_slice(&[v(p1), v(p2), v(p3), v(p1), v(p3), v(p4)]);

            if !is_scale {
                let c1_base = tf(cyl_len, c1 * head_radius, s1 * head_radius);
                let c2_base = tf(cyl_len, c2 * head_radius, s2 * head_radius);
                let c_tip = tf(len, 0.0, 0.0);
                let c_center = tf(cyl_len, 0.0, 0.0);

                vertices.extend_from_slice(&[v(c1_base), v(c2_base), v(c_tip)]);
                vertices.extend_from_slice(&[v(c1_base), v(c_center), v(c2_base)]);
            }
        }

        if is_scale {
            let s = head_radius;
            let min_x = cyl_len;
            let max_x = len;

            let p = [
                tf(min_x, -s, -s),
                tf(max_x, -s, -s),
                tf(max_x, s, -s),
                tf(min_x, s, -s),
                tf(min_x, -s, s),
                tf(max_x, -s, s),
                tf(max_x, s, s),
                tf(min_x, s, s),
            ];

            let indices = [
                0, 1, 2, 0, 2, 3, 1, 5, 6, 1, 6, 2, 5, 4, 7, 5, 7, 6, 4, 0, 3, 4, 3, 7, 3, 2, 6, 3,
                6, 7, 4, 5, 1, 4, 1, 0,
            ];

            for &idx in &indices {
                vertices.push(v(p[idx]));
            }
        }

        vertices
    }

    /// Builds complete axis geometry (3 arrows + optional planar handles).
    /// ### Arguments
    /// * `len` - Length of each axis arrow.
    /// * `is_scale` - True if generating geometry for scale mode (cubes), false for translate mode (cones/planes).
    pub(crate) fn build_axis_vertices(len: f32, is_scale: bool) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let radius = len * 0.03;

        let red = [1.0, 0.2, 0.2];
        let green = [0.2, 1.0, 0.2];
        let blue = [0.2, 0.4, 1.0];

        vertices.extend(Self::build_arrow(
            len,
            radius,
            16,
            red,
            ActiveAxis::X,
            is_scale,
        ));
        vertices.extend(Self::build_arrow(
            len,
            radius,
            16,
            green,
            ActiveAxis::Y,
            is_scale,
        ));
        vertices.extend(Self::build_arrow(
            len,
            radius,
            16,
            blue,
            ActiveAxis::Z,
            is_scale,
        ));

        // Add Planar Handles for Free/Planar Translation
        if !is_scale {
            let offset = radius * 8.0; // Moved further out (was 4.0)
            let p_size = radius * 6.0; // Larger for easier grabbing (was 3.0)

            // Planar handles are colored by their normal's orthogonal axis.
            // XY Plane ignores Z, so it is blue.
            vertices.extend(Self::build_plane_quad(
                offset,
                p_size,
                blue,
                ActiveAxis::PlaneXY,
            ));
            vertices.extend(Self::build_plane_quad(
                offset,
                p_size,
                green,
                ActiveAxis::PlaneXZ,
            ));
            vertices.extend(Self::build_plane_quad(
                offset,
                p_size,
                red,
                ActiveAxis::PlaneYZ,
            ));
        }

        vertices
    }

    /// Generates dynamic dashed interaction/guidelines during dragging or scaling.
    /// This generates temporary UI overlay lines such as:
    /// - Red center point coordinates for Scale mode.
    /// - Dashed axis alignment lines when dragging an axis.
    /// - Dashed bounding lines when dragging planar translation handles.
    /// - A solid rubber-band line connecting the origin to the current mouse drag position.
    pub(crate) fn build_interaction_lines(&self) -> Vec<GizmoVertex> {
        let mut lines = Vec::new();

        // Red Center Point always drawn exactly in the middle of all three gizmo modes!
        let s = 0.015; // Slightly smaller and sleeker (was 0.02)
        let red = [1.0, 0.2, 0.2]; // Bright red center point
        lines.push(GizmoVertex {
            position: [-s, 0.0, 0.0],
            color: red,
        });
        lines.push(GizmoVertex {
            position: [s, 0.0, 0.0],
            color: red,
        });
        lines.push(GizmoVertex {
            position: [0.0, -s, 0.0],
            color: red,
        });
        lines.push(GizmoVertex {
            position: [0.0, s, 0.0],
            color: red,
        });
        lines.push(GizmoVertex {
            position: [0.0, 0.0, -s],
            color: red,
        });
        lines.push(GizmoVertex {
            position: [0.0, 0.0, s],
            color: red,
        });

        if !self.is_dragging || self.active_axis == ActiveAxis::None {
            return lines;
        }

        let scale = self.drag_scale.max(1e-6);
        let mut local_start = (self.drag_start_world - self.drag_gizmo_pos) / scale;
        let mut local_end = (self.drag_current_hit - self.drag_gizmo_pos) / scale;

        if self.space == super::space::GizmoSpace::Local {
            let inv_rot = self.entity_rotation.conjugate();
            local_start = inv_rot.rotate_vector(local_start);
            local_end = inv_rot.rotate_vector(local_end);
        }

        let push_dashed = |lines: &mut Vec<GizmoVertex>,
                           start: Vector3<f32>,
                           dir: Vector3<f32>,
                           length: f32,
                           color: [f32; 3]| {
            let num_dashes = 40;
            let step = length / num_dashes as f32;
            let dash_ratio = 0.5;
            for i in 0..num_dashes {
                let t1 = -length * 0.5 + i as f32 * step;
                let t2 = t1 + step * dash_ratio;
                let p1 = start + dir * t1;
                let p2 = start + dir * t2;
                lines.push(GizmoVertex {
                    position: p1.into(),
                    color,
                });
                lines.push(GizmoVertex {
                    position: p2.into(),
                    color,
                });
            }
        };

        match self.mode {
            GizmoMode::Translate => {
                let col = match self.active_axis {
                    ActiveAxis::X => [1.0, 0.2, 0.2],
                    ActiveAxis::Y => [0.2, 1.0, 0.2],
                    ActiveAxis::Z => [0.2, 0.4, 1.0],
                    ActiveAxis::PlaneXY => [0.2, 0.4, 1.0],
                    ActiveAxis::PlaneXZ => [0.2, 1.0, 0.2],
                    ActiveAxis::PlaneYZ => [1.0, 0.2, 0.2],
                    _ => [1.0, 1.0, 1.0],
                };

                // 1) Axis line (1D) or plane axis lines (2D)
                match self.active_axis {
                    ActiveAxis::X => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::unit_x(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::Y => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::unit_y(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::Z => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::unit_z(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::PlaneXY => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, local_end.y, 0.0),
                            Vector3::unit_x(),
                            20.0,
                            col,
                        );
                        push_dashed(
                            &mut lines,
                            Vector3::new(local_end.x, 0.0, 0.0),
                            Vector3::unit_y(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::PlaneXZ => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, local_end.z),
                            Vector3::unit_x(),
                            20.0,
                            col,
                        );
                        push_dashed(
                            &mut lines,
                            Vector3::new(local_end.x, 0.0, 0.0),
                            Vector3::unit_z(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::PlaneYZ => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, local_end.z),
                            Vector3::unit_y(),
                            20.0,
                            col,
                        );
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, local_end.y, 0.0),
                            Vector3::unit_z(),
                            20.0,
                            col,
                        );
                    }
                    _ => {}
                }

                // 2) White drag line stretching from the start position to the current mouse position
                let start_pos = if self.active_axis == ActiveAxis::Free {
                    if local_start.magnitude2() > 0.001 {
                        local_start.normalize() * 0.15
                    } else {
                        local_start
                    }
                } else {
                    local_start
                };
                lines.push(GizmoVertex {
                    position: start_pos.into(),
                    color: [1.0, 1.0, 1.0],
                });
                lines.push(GizmoVertex {
                    position: local_end.into(),
                    color: [1.0, 1.0, 1.0],
                });
            }
            GizmoMode::Scale => {
                let col = match self.active_axis {
                    ActiveAxis::X => [1.0, 0.2, 0.2],
                    ActiveAxis::Y => [0.2, 1.0, 0.2],
                    ActiveAxis::Z => [0.2, 0.4, 1.0],
                    _ => [0.9, 0.9, 0.9],
                };

                // 1) Scale axis line (1D)
                match self.active_axis {
                    ActiveAxis::X => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::unit_x(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::Y => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::unit_y(),
                            20.0,
                            col,
                        );
                    }
                    ActiveAxis::Z => {
                        push_dashed(
                            &mut lines,
                            Vector3::new(0.0, 0.0, 0.0),
                            Vector3::unit_z(),
                            20.0,
                            col,
                        );
                    }
                    _ => {}
                }

                // 2) Premium dashed Rubber-Band line stretching from the center to the mouse pointer!
                let start_pos = if self.active_axis == ActiveAxis::Free {
                    if local_end.magnitude2() > 0.001 {
                        local_end.normalize() * 0.15
                    } else {
                        Vector3::new(0.0, 0.0, 0.0)
                    }
                } else {
                    Vector3::new(0.0, 0.0, 0.0)
                };
                let mag = (local_end - start_pos).magnitude();
                if mag > 0.01 {
                    push_dashed(
                        &mut lines,
                        start_pos,
                        (local_end - start_pos).normalize(),
                        mag * 2.0,
                        [1.0, 1.0, 1.0],
                    );
                }
            }
            GizmoMode::Rotate => {
                // White drag line stretching from start to current mouse position ("string" UX)
                // We always start the string from the small camera-facing hollow ring (radius 0.15)!
                let local_start_on_ring = if local_start.magnitude2() > 0.001 {
                    local_start.normalize() * 0.15
                } else {
                    local_start
                };
                lines.push(GizmoVertex {
                    position: local_start_on_ring.into(),
                    color: [1.0, 1.0, 1.0],
                });
                lines.push(GizmoVertex {
                    position: local_end.into(),
                    color: [1.0, 1.0, 1.0],
                });
            }
        }
        lines
    }

    /// Generates the uniform scale O-ring as an anti-aliased anti-flicker TriangleList mesh (Hollow Disk/Ring).
    /// Generates a perfectly circular flat disk that aligns to the camera view plane.
    /// Uses dynamic scale factors and highlighting depending on mouse hovering or dragging states.
    pub(crate) fn build_o_ring_mesh(&self) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let radius = 0.15 * self.drag_scale_factor;
        let thickness = 0.003;
        let segments = 64; // Since triangles are drawn with MSAA, 64 segments provide perfect smoothness!

        let color = if self.is_dragging {
            [1.0, 1.0, 1.0] // White when dragging
        } else if self.hovered_axis == ActiveAxis::Free {
            [1.0, 1.0, 1.0] // White when hovered
        } else {
            [0.85, 0.85, 0.85] // Light gray otherwise
        };

        // Ring thickness bounds (inner and outer radius)
        let inner_radius = radius - thickness;
        let outer_radius = radius + thickness;

        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;

            let (sin1, cos1) = a1.sin_cos();
            let (sin2, cos2) = a2.sin_cos();

            let cam_right = self.cam_right.get();
            let cam_up = self.cam_up.get();
            let mut p_in1 = (cam_right * cos1 + cam_up * sin1) * inner_radius;
            let mut p_out1 = (cam_right * cos1 + cam_up * sin1) * outer_radius;
            let mut p_in2 = (cam_right * cos2 + cam_up * sin2) * inner_radius;
            let mut p_out2 = (cam_right * cos2 + cam_up * sin2) * outer_radius;

            if self.space == super::space::GizmoSpace::Local {
                let inv_rot = self.entity_rotation.conjugate();
                p_in1 = inv_rot.rotate_vector(p_in1);
                p_out1 = inv_rot.rotate_vector(p_out1);
                p_in2 = inv_rot.rotate_vector(p_in2);
                p_out2 = inv_rot.rotate_vector(p_out2);
            }

            let v = |p: cgmath::Vector3<f32>| GizmoVertex {
                position: p.into(),
                color,
            };

            // Triangle 1
            vertices.push(v(p_in1));
            vertices.push(v(p_out1));
            vertices.push(v(p_out2));

            // Triangle 2
            vertices.push(v(p_in1));
            vertices.push(v(p_out2));
            vertices.push(v(p_in2));
        }

        vertices
    }
}
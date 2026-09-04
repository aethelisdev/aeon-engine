// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::core::{ActiveAxis, GizmoMode, GizmoSystem};
use super::render::GizmoVertex;
/// Aeon Engine - Gizmo Geometry Generation
/// This module handles mathematical geometry generation for the 3D translation, rotation, and scale gizmo handles.
/// It constructs procedural vertex data for torus rings, axis arrows, planar quads, and uniform scaling O-rings.
use cgmath::{InnerSpace, Rotation as _, Vector3};

impl GizmoSystem {
    /// Generates a colored quad on the specified plane for planar translation handles.
    /// ### Arguments
    /// * `offset` - Distance from the center where the plane quad starts.
    /// * `size` - Width and height of the square quad.
    /// * `color` - RGB color of the plane quad vertices.
    /// * `axis` - The planar translation axis (PlaneXY, PlaneXZ, or PlaneYZ).
    pub(crate) fn build_plane_quad(
        offset: f32,
        size: f32,
        color: [f32; 4],
        axis: ActiveAxis,
    ) -> Vec<GizmoVertex> {
        let v = |p: [f32; 3]| GizmoVertex {
            position: p,
            color,
            uv: [0.0, 0.0],
        };
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
        color: [f32; 4],
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
        let v = |p: [f32; 3]| GizmoVertex {
            position: p,
            color,
            uv: [0.0, 0.0],
        };

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

    /// Builds dynamic axis geometry (arrows + planar handles) with real-time hover and active drag highlighting.
    /// Applies subtle luminance highlighting ("hafif parlatma") to the hovered axis handle while preserving
    /// its distinctive color identity (Red for X, Green for Y, Blue for Z), and vibrant saturation when dragging.
    /// ### Arguments
    /// * `len` - Length of each axis arrow in local units.
    /// * `is_scale` - True for scale mode (cube tips), false for translation mode (cone tips + planar handles).
    pub(crate) fn build_dynamic_axis_vertices(&self, len: f32, is_scale: bool) -> Vec<GizmoVertex> {
        Self::build_axis_vertices_styled(
            len,
            is_scale,
            self.hovered_axis,
            self.active_axis,
            self.is_dragging,
        )
    }

    /// Internal procedural constructor for styled axis geometry with explicit hover and drag states.
    pub(crate) fn build_axis_vertices_styled(
        len: f32,
        is_scale: bool,
        hovered_axis: ActiveAxis,
        active_axis: ActiveAxis,
        is_dragging: bool,
    ) -> Vec<GizmoVertex> {
        let mut vertices = Vec::new();
        let radius = len * 0.03;

        let red = if is_dragging && active_axis == ActiveAxis::X {
            [1.0, 0.25, 0.25, 1.0]
        } else if hovered_axis == ActiveAxis::X {
            [1.0, 0.32, 0.32, 1.0] // Subtle, rich red hover highlight (preserves saturation)
        } else {
            [0.9, 0.2, 0.2, 1.0]
        };

        let green = if is_dragging && active_axis == ActiveAxis::Y {
            [0.25, 1.0, 0.25, 1.0]
        } else if hovered_axis == ActiveAxis::Y {
            [0.28, 0.98, 0.28, 1.0] // Subtle, rich green hover highlight (preserves saturation)
        } else {
            [0.2, 0.85, 0.2, 1.0]
        };

        let blue = if is_dragging && active_axis == ActiveAxis::Z {
            [0.25, 0.5, 1.0, 1.0]
        } else if hovered_axis == ActiveAxis::Z {
            [0.28, 0.55, 1.0, 1.0] // Subtle, rich blue hover highlight (preserves saturation)
        } else {
            [0.2, 0.45, 1.0, 1.0]
        };

        let r_x = if hovered_axis == ActiveAxis::X || (is_dragging && active_axis == ActiveAxis::X)
        {
            radius * 1.08
        } else {
            radius
        };
        let r_y = if hovered_axis == ActiveAxis::Y || (is_dragging && active_axis == ActiveAxis::Y)
        {
            radius * 1.08
        } else {
            radius
        };
        let r_z = if hovered_axis == ActiveAxis::Z || (is_dragging && active_axis == ActiveAxis::Z)
        {
            radius * 1.08
        } else {
            radius
        };

        vertices.extend(Self::build_arrow(
            len,
            r_x,
            16,
            red,
            ActiveAxis::X,
            is_scale,
        ));
        vertices.extend(Self::build_arrow(
            len,
            r_y,
            16,
            green,
            ActiveAxis::Y,
            is_scale,
        ));
        vertices.extend(Self::build_arrow(
            len,
            r_z,
            16,
            blue,
            ActiveAxis::Z,
            is_scale,
        ));

        // Add planar handle quads only in translation mode
        if !is_scale {
            let offset = len * 0.3;
            let p_size = len * 0.22;

            let p_size_xy = if hovered_axis == ActiveAxis::PlaneXY
                || (is_dragging && active_axis == ActiveAxis::PlaneXY)
            {
                p_size * 1.05
            } else {
                p_size
            };
            let p_size_xz = if hovered_axis == ActiveAxis::PlaneXZ
                || (is_dragging && active_axis == ActiveAxis::PlaneXZ)
            {
                p_size * 1.05
            } else {
                p_size
            };
            let p_size_yz = if hovered_axis == ActiveAxis::PlaneYZ
                || (is_dragging && active_axis == ActiveAxis::PlaneYZ)
            {
                p_size * 1.05
            } else {
                p_size
            };

            let p_blue = if is_dragging && active_axis == ActiveAxis::PlaneXY {
                [0.25, 0.5, 1.0, 0.8]
            } else if hovered_axis == ActiveAxis::PlaneXY {
                [0.28, 0.55, 1.0, 0.55] // Subtle blue planar hover highlight
            } else {
                [0.2, 0.45, 1.0, 0.4]
            };

            let p_green = if is_dragging && active_axis == ActiveAxis::PlaneXZ {
                [0.2, 1.0, 0.2, 0.8]
            } else if hovered_axis == ActiveAxis::PlaneXZ {
                [0.28, 0.98, 0.28, 0.55] // Subtle green planar hover highlight
            } else {
                [0.2, 0.85, 0.2, 0.4]
            };

            let p_red = if is_dragging && active_axis == ActiveAxis::PlaneYZ {
                [1.0, 0.2, 0.2, 0.8]
            } else if hovered_axis == ActiveAxis::PlaneYZ {
                [1.0, 0.32, 0.32, 0.55] // Subtle red planar hover highlight
            } else {
                [0.9, 0.2, 0.2, 0.4]
            };

            vertices.extend(Self::build_plane_quad(
                offset,
                p_size_xy,
                p_blue,
                ActiveAxis::PlaneXY,
            ));
            vertices.extend(Self::build_plane_quad(
                offset,
                p_size_xz,
                p_green,
                ActiveAxis::PlaneXZ,
            ));
            vertices.extend(Self::build_plane_quad(
                offset,
                p_size_yz,
                p_red,
                ActiveAxis::PlaneYZ,
            ));
        }

        vertices
    }

    /// Builds complete axis geometry (3 arrows + optional planar handles).
    /// ### Arguments
    /// * `len` - Length of each axis arrow.
    /// * `is_scale` - True if generating geometry for scale mode (cubes), false for translate mode (cones/planes).
    pub(crate) fn build_axis_vertices(len: f32, is_scale: bool) -> Vec<GizmoVertex> {
        Self::build_axis_vertices_styled(len, is_scale, ActiveAxis::None, ActiveAxis::None, false)
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
        let red = [1.0, 0.2, 0.2, 1.0]; // Bright red center point
        lines.push(GizmoVertex {
            position: [-s, 0.0, 0.0],
            color: red,
            uv: [0.0, 0.0],
        });
        lines.push(GizmoVertex {
            position: [s, 0.0, 0.0],
            color: red,
            uv: [0.0, 0.0],
        });
        lines.push(GizmoVertex {
            position: [0.0, -s, 0.0],
            color: red,
            uv: [0.0, 0.0],
        });
        lines.push(GizmoVertex {
            position: [0.0, s, 0.0],
            color: red,
            uv: [0.0, 0.0],
        });
        lines.push(GizmoVertex {
            position: [0.0, 0.0, -s],
            color: red,
            uv: [0.0, 0.0],
        });
        lines.push(GizmoVertex {
            position: [0.0, 0.0, s],
            color: red,
            uv: [0.0, 0.0],
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
                           color: [f32; 4]| {
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
                    uv: [0.0, 0.0],
                });
                lines.push(GizmoVertex {
                    position: p2.into(),
                    color,
                    uv: [0.0, 0.0],
                });
            }
        };

        match self.mode {
            GizmoMode::Select => {}
            GizmoMode::Translate => {
                let col = match self.active_axis {
                    ActiveAxis::X => [1.0, 0.2, 0.2, 1.0],
                    ActiveAxis::Y => [0.2, 1.0, 0.2, 1.0],
                    ActiveAxis::Z => [0.2, 0.4, 1.0, 1.0],
                    ActiveAxis::PlaneXY => [0.2, 0.4, 1.0, 1.0],
                    ActiveAxis::PlaneXZ => [0.2, 1.0, 0.2, 1.0],
                    ActiveAxis::PlaneYZ => [1.0, 0.2, 0.2, 1.0],
                    _ => [1.0, 1.0, 1.0, 1.0],
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
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [0.0, 0.0],
                });
                lines.push(GizmoVertex {
                    position: local_end.into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [0.0, 0.0],
                });
            }
            GizmoMode::Scale => {
                let col = match self.active_axis {
                    ActiveAxis::X => [1.0, 0.2, 0.2, 1.0],
                    ActiveAxis::Y => [0.2, 1.0, 0.2, 1.0],
                    ActiveAxis::Z => [0.2, 0.4, 1.0, 1.0],
                    _ => [0.9, 0.9, 0.9, 1.0],
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
                        [1.0, 1.0, 1.0, 1.0],
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
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [0.0, 0.0],
                });
                lines.push(GizmoVertex {
                    position: local_end.into(),
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [0.0, 0.0],
                });
            }
        }
        lines
    }

    /// Generates the uniform scale / center O-ring as an anti-aliased, zero-aliasing Screen-Space SDF Quad.
    /// Generates a camera-aligned Quad billboard with normalized `[-1.0, 1.0]` UVs.
    /// The fragment shader renders a mathematically perfect, sub-pixel anti-aliased circular ring with `fwidth()`.
    pub(crate) fn build_o_ring_mesh(&self) -> Vec<GizmoVertex> {
        let radius = 0.16 * self.drag_scale_factor;

        let color = if self.is_dragging || self.hovered_axis == ActiveAxis::Free {
            [1.0, 1.0, 1.0, 1.0] // White when dragging or hovered
        } else {
            [0.85, 0.85, 0.85, 1.0] // Light gray otherwise
        };

        let cam_right = self.cam_right.get();
        let cam_up = self.cam_up.get();

        let mut p_tl = (-cam_right + cam_up) * radius;
        let mut p_tr = (cam_right + cam_up) * radius;
        let mut p_br = (cam_right - cam_up) * radius;
        let mut p_bl = (-cam_right - cam_up) * radius;

        if self.space == super::space::GizmoSpace::Local {
            let inv_rot = self.entity_rotation.conjugate();
            p_tl = inv_rot.rotate_vector(p_tl);
            p_tr = inv_rot.rotate_vector(p_tr);
            p_br = inv_rot.rotate_vector(p_br);
            p_bl = inv_rot.rotate_vector(p_bl);
        }

        let v = |p: cgmath::Vector3<f32>, uv: [f32; 2]| GizmoVertex {
            position: p.into(),
            color,
            uv,
        };

        // Quad consisting of 2 triangles with exact normalized [-1.0, 1.0] UV coordinates
        vec![
            v(p_tl, [-1.0, 1.0]),
            v(p_bl, [-1.0, -1.0]),
            v(p_br, [1.0, -1.0]),
            v(p_tl, [-1.0, 1.0]),
            v(p_br, [1.0, -1.0]),
            v(p_tr, [1.0, 1.0]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_axis_vertices_hover_highlight() {
        let idle_verts = GizmoSystem::build_axis_vertices_styled(
            1.0,
            false,
            ActiveAxis::None,
            ActiveAxis::None,
            false,
        );
        let hovered_x_verts = GizmoSystem::build_axis_vertices_styled(
            1.0,
            false,
            ActiveAxis::X,
            ActiveAxis::None,
            false,
        );

        assert_eq!(idle_verts.len(), hovered_x_verts.len());
        // First vertex of X arrow should have higher luminance in green and blue channels when hovered
        let idle_x_col = idle_verts[0].color;
        let hovered_x_col = hovered_x_verts[0].color;
        assert!(hovered_x_col[0] >= idle_x_col[0]);
        assert!(hovered_x_col[1] > idle_x_col[1]); // Green channel elevated for luminance highlight
        assert!(hovered_x_col[2] > idle_x_col[2]); // Blue channel elevated for luminance highlight
    }

    #[test]
    fn test_dynamic_axis_scale_vertices() {
        let scale_verts = GizmoSystem::build_axis_vertices_styled(
            1.0,
            true,
            ActiveAxis::Y,
            ActiveAxis::None,
            false,
        );
        assert!(!scale_verts.is_empty());
        // Verify vertices are finite
        for v in &scale_verts {
            assert!(v.position[0].is_finite());
            assert!(v.position[1].is_finite());
            assert!(v.position[2].is_finite());
        }
    }
}
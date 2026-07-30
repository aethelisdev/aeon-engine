// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use super::vertex::DebugLineVertex;

/// Self-contained wireframe geometry line generators for 3D debug overlays.
pub struct DebugShapes;

impl DebugShapes {
    /// Generates 12-edge wireframe for a box collider.
    pub fn generate_box_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        he: [f32; 3],
        color: [f32; 3],
    ) {
        let corners = [
            [-he[0], -he[1], -he[2]],
            [he[0], -he[1], -he[2]],
            [he[0], he[1], -he[2]],
            [-he[0], he[1], -he[2]],
            [-he[0], -he[1], he[2]],
            [he[0], -he[1], he[2]],
            [he[0], he[1], he[2]],
            [-he[0], he[1], he[2]],
        ];

        let edges = [
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0), // front
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4), // back
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7), // connections
        ];

        for (a, b) in edges {
            let pa = Self::transform_point(model, corners[a]);
            let pb = Self::transform_point(model, corners[b]);
            verts.push(DebugLineVertex {
                position: pa,
                color,
            });
            verts.push(DebugLineVertex {
                position: pb,
                color,
            });
        }
    }

    /// Generates 3-ring wireframe for a sphere collider (XY, XZ, YZ planes).
    pub fn generate_sphere_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        radius: f32,
        color: [f32; 3],
    ) {
        let segments = 32;
        // XY ring
        Self::generate_circle(verts, model, radius, 0, 1, 2, segments, color);
        // XZ ring
        Self::generate_circle(verts, model, radius, 0, 2, 1, segments, color);
        // YZ ring
        Self::generate_circle(verts, model, radius, 1, 2, 0, segments, color);
    }

    /// Generates capsule wireframe: 2 hemisphere rings + 4 vertical lines.
    pub fn generate_capsule_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        half_height: f32,
        radius: f32,
        color: [f32; 3],
    ) {
        let segments = 24;

        // Top and bottom center circles (XZ plane)
        for offset_y in [half_height, -half_height] {
            for i in 0..segments {
                let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
                let p1 = [radius * a1.cos(), offset_y, radius * a1.sin()];
                let p2 = [radius * a2.cos(), offset_y, radius * a2.sin()];
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });
            }
        }

        // 4 vertical connecting lines
        for angle in [
            0.0_f32,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
            std::f32::consts::FRAC_PI_2 * 3.0,
        ] {
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            let top = Self::transform_point(model, [x, half_height, z]);
            let bot = Self::transform_point(model, [x, -half_height, z]);
            verts.push(DebugLineVertex {
                position: top,
                color,
            });
            verts.push(DebugLineVertex {
                position: bot,
                color,
            });
        }

        // Top hemisphere arc (XY plane)
        Self::generate_half_circle(verts, model, radius, half_height, true, 0, segments, color);
        // Top hemisphere arc (ZY plane)
        Self::generate_half_circle(verts, model, radius, half_height, true, 2, segments, color);
        // Bottom hemisphere arc (XY plane)
        Self::generate_half_circle(
            verts,
            model,
            radius,
            -half_height,
            false,
            0,
            segments,
            color,
        );
        // Bottom hemisphere arc (ZY plane)
        Self::generate_half_circle(
            verts,
            model,
            radius,
            -half_height,
            false,
            2,
            segments,
            color,
        );
    }

    /// Generates cylinder wireframe: top/bottom circle rings + 4 vertical lines.
    pub fn generate_cylinder_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        half_height: f32,
        radius: f32,
        color: [f32; 3],
    ) {
        let segments = 24;
        for offset_y in [half_height, -half_height] {
            for i in 0..segments {
                let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
                let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
                let p1 = [radius * a1.cos(), offset_y, radius * a1.sin()];
                let p2 = [radius * a2.cos(), offset_y, radius * a2.sin()];
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });
            }
        }

        for angle in [
            0.0_f32,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
            std::f32::consts::FRAC_PI_2 * 3.0,
        ] {
            let x = radius * angle.cos();
            let z = radius * angle.sin();
            let top = Self::transform_point(model, [x, half_height, z]);
            let bot = Self::transform_point(model, [x, -half_height, z]);
            verts.push(DebugLineVertex {
                position: top,
                color,
            });
            verts.push(DebugLineVertex {
                position: bot,
                color,
            });
        }
    }

    /// Generates 3-edge triangle wireframe.
    pub fn generate_triangle_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        color: [f32; 3],
    ) {
        let p1 = [-0.5, -0.5, 0.0];
        let p2 = [0.5, -0.5, 0.0];
        let p3 = [0.0, 0.5, 0.0];
        for (a, b) in [(p1, p2), (p2, p3), (p3, p1)] {
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, a),
                color,
            });
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, b),
                color,
            });
        }
    }

    /// Generates torus wireframe (outer + inner circle rings).
    pub fn generate_torus_lines(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        r_major: f32,
        r_minor: f32,
        color: [f32; 3],
    ) {
        let segments = 24;
        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
            for r in [r_major + r_minor, r_major - r_minor] {
                let p1 = [r * a1.cos(), 0.0, r * a1.sin()];
                let p2 = [r * a2.cos(), 0.0, r * a2.sin()];
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });
                verts.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });
            }
        }
    }

    /// Helper: generates a circle ring on a specified plane.
    pub fn generate_circle(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        radius: f32,
        axis_a: usize,
        axis_b: usize,
        axis_n: usize,
        segments: usize,
        color: [f32; 3],
    ) {
        for i in 0..segments {
            let a1 = (i as f32) / (segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32) / (segments as f32) * std::f32::consts::TAU;
            let mut p1 = [0.0_f32; 3];
            let mut p2 = [0.0_f32; 3];
            p1[axis_a] = radius * a1.cos();
            p1[axis_b] = radius * a1.sin();
            p1[axis_n] = 0.0;
            p2[axis_a] = radius * a2.cos();
            p2[axis_b] = radius * a2.sin();
            p2[axis_n] = 0.0;
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p1),
                color,
            });
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p2),
                color,
            });
        }
    }

    /// Helper: generates a half-circle arc for capsule hemispheres.
    pub fn generate_half_circle(
        verts: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        radius: f32,
        y_offset: f32,
        top: bool,
        horizontal_axis: usize,
        segments: usize,
        color: [f32; 3],
    ) {
        let half_segments = segments / 2;
        let start_angle = if top { 0.0 } else { std::f32::consts::PI };
        let end_angle = if top {
            std::f32::consts::PI
        } else {
            std::f32::consts::TAU
        };

        for i in 0..half_segments {
            let t1 = start_angle + (i as f32) / (half_segments as f32) * (end_angle - start_angle);
            let t2 =
                start_angle + ((i + 1) as f32) / (half_segments as f32) * (end_angle - start_angle);

            let mut p1 = [0.0_f32; 3];
            let mut p2 = [0.0_f32; 3];
            p1[horizontal_axis] = radius * t1.cos();
            p1[1] = y_offset + radius * t1.sin();
            p2[horizontal_axis] = radius * t2.cos();
            p2[1] = y_offset + radius * t2.sin();

            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p1),
                color,
            });
            verts.push(DebugLineVertex {
                position: Self::transform_point(model, p2),
                color,
            });
        }
    }

    /// Transforms a local-space point by the model matrix and returns world-space `[f32; 3]`.
    pub fn transform_point(model: &cgmath::Matrix4<f32>, local: [f32; 3]) -> [f32; 3] {
        let v = model * cgmath::Vector4::new(local[0], local[1], local[2], 1.0);
        [v.x / v.w, v.y / v.w, v.z / v.w]
    }

    /// Generates wireframe lines representing a complex mesh collider.
    pub fn generate_mesh_lines(
        vertices: &mut Vec<DebugLineVertex>,
        model: &cgmath::Matrix4<f32>,
        raw_vertices: &[[f32; 3]],
        raw_indices: &[u32],
        color: [f32; 3],
    ) {
        for chunk in raw_indices.chunks_exact(3) {
            let i0 = chunk[0] as usize;
            let i1 = chunk[1] as usize;
            let i2 = chunk[2] as usize;

            if i0 < raw_vertices.len() && i1 < raw_vertices.len() && i2 < raw_vertices.len() {
                let p0 = raw_vertices[i0];
                let p1 = raw_vertices[i1];
                let p2 = raw_vertices[i2];

                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p0),
                    color,
                });
                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });

                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p1),
                    color,
                });
                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });

                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p2),
                    color,
                });
                vertices.push(DebugLineVertex {
                    position: Self::transform_point(model, p0),
                    color,
                });
            }
        }
    }
}
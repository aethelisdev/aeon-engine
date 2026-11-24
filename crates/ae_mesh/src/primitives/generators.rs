// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Procedural parametric 3D mesh generators producing unit-bounded triangle lists.

use crate::vertex::Vertex;

/// Generates a parametric UV sphere mesh with radius `0.5` (fitting a `1x1x1` unit box)
/// and outward smooth normals.
///
/// Returns an empty vector if `latitude_bands == 0` or `longitude_bands == 0`.
#[must_use]
pub fn generate_sphere(latitude_bands: u32, longitude_bands: u32) -> Vec<Vertex> {
    if latitude_bands == 0 || longitude_bands == 0 {
        return Vec::new();
    }

    let mut vertices = Vec::with_capacity((latitude_bands * longitude_bands * 6) as usize);
    let mut raw_vertices =
        Vec::with_capacity(((latitude_bands + 1) * (longitude_bands + 1)) as usize);

    for lat in 0..=latitude_bands {
        let theta = lat as f32 * std::f32::consts::PI / latitude_bands as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=longitude_bands {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / longitude_bands as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            let u = lon as f32 / longitude_bands as f32;
            let v = lat as f32 / latitude_bands as f32;

            raw_vertices.push(Vertex {
                position: [x * 0.5, y * 0.5, z * 0.5],
                color: [1.0, 1.0, 1.0],
                normal: [x, y, z],
                uv: [u, v],
            });
        }
    }

    push_grid_triangles(
        &raw_vertices,
        0,
        latitude_bands,
        longitude_bands,
        &mut vertices,
    );
    vertices
}

/// Generates a capped parametric cylinder mesh fitting a `1x1x1` unit box (`radius = 0.5`, `height = 1.0`).
///
/// Returns an empty vector if `segments == 0`.
#[must_use]
pub fn generate_cylinder(segments: u32) -> Vec<Vertex> {
    if segments == 0 {
        return Vec::new();
    }

    let mut vertices = Vec::with_capacity(segments as usize * 12);

    let top_center = [0.0, 0.5, 0.0];
    let bottom_center = [0.0, -0.5, 0.0];

    let mut circle_pts = Vec::with_capacity(segments as usize + 1);
    for i in 0..=segments {
        let angle = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        circle_pts.push((angle.cos() * 0.5, angle.sin() * 0.5));
    }

    for i in 0..segments as usize {
        let (x0, z0) = circle_pts[i];
        let (x1, z1) = circle_pts[i + 1];

        let n0 = [x0 * 2.0, 0.0, z0 * 2.0];
        let n1 = [x1 * 2.0, 0.0, z1 * 2.0];

        let u0 = i as f32 / segments as f32;
        let u1 = (i + 1) as f32 / segments as f32;

        // Side triangles (CCW outward)
        vertices.push(Vertex {
            position: [x0, 0.5, z0],
            color: [1.0; 3],
            normal: n0,
            uv: [u0, 0.0],
        });
        vertices.push(Vertex {
            position: [x1, 0.5, z1],
            color: [1.0; 3],
            normal: n1,
            uv: [u1, 0.0],
        });
        vertices.push(Vertex {
            position: [x0, -0.5, z0],
            color: [1.0; 3],
            normal: n0,
            uv: [u0, 1.0],
        });

        vertices.push(Vertex {
            position: [x1, 0.5, z1],
            color: [1.0; 3],
            normal: n1,
            uv: [u1, 0.0],
        });
        vertices.push(Vertex {
            position: [x1, -0.5, z1],
            color: [1.0; 3],
            normal: n1,
            uv: [u1, 1.0],
        });
        vertices.push(Vertex {
            position: [x0, -0.5, z0],
            color: [1.0; 3],
            normal: n0,
            uv: [u0, 1.0],
        });

        // Top cap (+Y)
        vertices.push(Vertex {
            position: top_center,
            color: [1.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
        });
        vertices.push(Vertex {
            position: [x1, 0.5, z1],
            color: [1.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [x1 + 0.5, z1 + 0.5],
        });
        vertices.push(Vertex {
            position: [x0, 0.5, z0],
            color: [1.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [x0 + 0.5, z0 + 0.5],
        });

        // Bottom cap (-Y)
        vertices.push(Vertex {
            position: bottom_center,
            color: [1.0; 3],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5, 0.5],
        });
        vertices.push(Vertex {
            position: [x0, -0.5, z0],
            color: [1.0; 3],
            normal: [0.0, -1.0, 0.0],
            uv: [x0 + 0.5, z0 + 0.5],
        });
        vertices.push(Vertex {
            position: [x1, -0.5, z1],
            color: [1.0; 3],
            normal: [0.0, -1.0, 0.0],
            uv: [x1 + 0.5, z1 + 0.5],
        });
    }

    vertices
}

/// Generates a parametric capsule mesh (`radius = 0.35`, total height `1.0`).
///
/// Returns an empty vector if `segments == 0` or `rings == 0`.
#[must_use]
pub fn generate_capsule(segments: u32, rings: u32) -> Vec<Vertex> {
    if segments == 0 || rings == 0 {
        return Vec::new();
    }

    let mut vertices = Vec::new();
    let mut raw_vertices = Vec::new();

    let radius = 0.35;
    let cylinder_half_height = 0.15;

    // Top hemisphere
    for ring in 0..=rings {
        let theta = (ring as f32 * 0.5 * std::f32::consts::PI) / rings as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for seg in 0..=segments {
            let phi = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * cos_phi * sin_theta;
            let y = cylinder_half_height + radius * cos_theta;
            let z = radius * sin_phi * sin_theta;

            let u = seg as f32 / segments as f32;
            let v = ring as f32 / (rings * 2) as f32;

            raw_vertices.push(Vertex {
                position: [x, y, z],
                color: [1.0; 3],
                normal: [x / radius, (y - cylinder_half_height) / radius, z / radius],
                uv: [u, v],
            });
        }
    }

    // Bottom hemisphere
    let base_idx = raw_vertices.len();
    for ring in 0..=rings {
        let theta =
            std::f32::consts::PI * 0.5 + (ring as f32 * 0.5 * std::f32::consts::PI) / rings as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for seg in 0..=segments {
            let phi = seg as f32 * 2.0 * std::f32::consts::PI / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * cos_phi * sin_theta;
            let y = -cylinder_half_height + radius * cos_theta;
            let z = radius * sin_phi * sin_theta;

            let u = seg as f32 / segments as f32;
            let v = 0.5 + (ring as f32 / (rings * 2) as f32);

            raw_vertices.push(Vertex {
                position: [x, y, z],
                color: [1.0; 3],
                normal: [x / radius, (y + cylinder_half_height) / radius, z / radius],
                uv: [u, v],
            });
        }
    }

    push_grid_triangles(&raw_vertices, 0, rings, segments, &mut vertices);
    push_grid_triangles(&raw_vertices, base_idx, rings, segments, &mut vertices);

    let top_rim_start = (rings * (segments + 1)) as usize;
    let bottom_rim_start = base_idx;
    for seg in 0..segments {
        let t0 = top_rim_start + seg as usize;
        let t1 = t0 + 1;
        let b0 = bottom_rim_start + seg as usize;
        let b1 = b0 + 1;

        vertices.push(raw_vertices[t0]);
        vertices.push(raw_vertices[t1]);
        vertices.push(raw_vertices[b0]);

        vertices.push(raw_vertices[t1]);
        vertices.push(raw_vertices[b1]);
        vertices.push(raw_vertices[b0]);
    }

    vertices
}

fn push_grid_triangles(
    raw_vertices: &[Vertex],
    base_idx: usize,
    rows: u32,
    cols: u32,
    out: &mut Vec<Vertex>,
) {
    for row in 0..rows {
        for col in 0..cols {
            let first = base_idx + (row * (cols + 1) + col) as usize;
            let second = first + (cols + 1) as usize;

            out.push(raw_vertices[first]);
            out.push(raw_vertices[first + 1]);
            out.push(raw_vertices[second]);

            out.push(raw_vertices[first + 1]);
            out.push(raw_vertices[second + 1]);
            out.push(raw_vertices[second]);
        }
    }
}

/// Generates a parametric torus mesh (`main_radius = 0.35`, `tube_radius = 0.15`).
///
/// Returns an empty vector if `radial_segments == 0` or `tubular_segments == 0`.
#[must_use]
pub fn generate_torus(radial_segments: u32, tubular_segments: u32) -> Vec<Vertex> {
    if radial_segments == 0 || tubular_segments == 0 {
        return Vec::new();
    }

    let mut vertices = Vec::with_capacity((radial_segments * tubular_segments * 6) as usize);
    let mut raw_vertices =
        Vec::with_capacity(((radial_segments + 1) * (tubular_segments + 1)) as usize);

    let main_radius = 0.35;
    let tube_radius = 0.15;

    for r_seg in 0..=radial_segments {
        let u = r_seg as f32 * 2.0 * std::f32::consts::PI / radial_segments as f32;
        let cos_u = u.cos();
        let sin_u = u.sin();

        for t_seg in 0..=tubular_segments {
            let v = t_seg as f32 * 2.0 * std::f32::consts::PI / tubular_segments as f32;
            let cos_v = v.cos();
            let sin_v = v.sin();

            let x = (main_radius + tube_radius * cos_v) * cos_u;
            let y = tube_radius * sin_v;
            let z = (main_radius + tube_radius * cos_v) * sin_u;

            let nx = cos_v * cos_u;
            let ny = sin_v;
            let nz = cos_v * sin_u;

            let tex_u = r_seg as f32 / radial_segments as f32;
            let tex_v = t_seg as f32 / tubular_segments as f32;

            raw_vertices.push(Vertex {
                position: [x, y, z],
                color: [1.0; 3],
                normal: [nx, ny, nz],
                uv: [tex_u, tex_v],
            });
        }
    }

    push_grid_triangles(
        &raw_vertices,
        0,
        radial_segments,
        tubular_segments,
        &mut vertices,
    );

    vertices
}
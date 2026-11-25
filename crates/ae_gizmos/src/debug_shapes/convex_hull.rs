// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Safe 3D convex hull boundary wireframe extraction for physics collider visualization.
//!
//! Computes an outward-oriented convex hull over arbitrary vertex clouds and extracts
//! non-coplanar silhouette edges once per mesh asset so debug rendering avoids drawing
//! concave interior mesh triangles.

use std::collections::HashMap;

/// Precomputed local-space boundary edge segments of a 3D convex hull.
#[derive(Debug, Clone, Default)]
pub struct ConvexHullWireframe {
    /// Unique boundary line segments `(start, end)` in local model coordinates.
    pub edges: Vec<([f32; 3], [f32; 3])>,
}

#[derive(Clone)]
struct HullFace {
    verts: [usize; 3],
    normal: [f32; 3],
    outside: Vec<(usize, f32)>,
    alive: bool,
}

#[inline]
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn norm_sq(a: [f32; 3]) -> f32 {
    dot(a, a)
}

fn normalize(a: [f32; 3]) -> Option<[f32; 3]> {
    let len_sq = norm_sq(a);
    if len_sq <= 1e-24 {
        return None;
    }
    let inv = len_sq.sqrt().recip();
    Some([a[0] * inv, a[1] * inv, a[2] * inv])
}

#[inline]
fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}

fn collect_finite_points(raw_points: &[[f32; 3]]) -> Vec<[f32; 3]> {
    raw_points
        .iter()
        .copied()
        .filter(|p| p[0].is_finite() && p[1].is_finite() && p[2].is_finite())
        .collect()
}

fn aabb_edges(pts: &[[f32; 3]]) -> Vec<([f32; 3], [f32; 3])> {
    let mut min = pts[0];
    let mut max = pts[0];
    for p in pts.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    let c = [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ];
    let pairs = [
        (0, 1),
        (1, 2),
        (2, 3),
        (3, 0),
        (4, 5),
        (5, 6),
        (6, 7),
        (7, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];
    pairs.iter().map(|&(i, j)| (c[i], c[j])).collect()
}

fn make_oriented_face(
    pts: &[[f32; 3]],
    mut verts: [usize; 3],
    centroid: [f32; 3],
) -> Option<HullFace> {
    let n_raw = cross(
        sub(pts[verts[1]], pts[verts[0]]),
        sub(pts[verts[2]], pts[verts[0]]),
    );
    let mut normal = normalize(n_raw)?;
    if dot(normal, sub(centroid, pts[verts[0]])) > 0.0 {
        verts.swap(1, 2);
        normal = [-normal[0], -normal[1], -normal[2]];
    }
    Some(HullFace {
        verts,
        normal,
        outside: Vec::new(),
        alive: true,
    })
}

fn initial_tetrahedron(pts: &[[f32; 3]], eps: f32) -> Option<[usize; 4]> {
    let mut extremes = [0usize; 6];
    for (idx, p) in pts.iter().enumerate().skip(1) {
        if p[0] < pts[extremes[0]][0] {
            extremes[0] = idx;
        }
        if p[0] > pts[extremes[1]][0] {
            extremes[1] = idx;
        }
        if p[1] < pts[extremes[2]][1] {
            extremes[2] = idx;
        }
        if p[1] > pts[extremes[3]][1] {
            extremes[3] = idx;
        }
        if p[2] < pts[extremes[4]][2] {
            extremes[4] = idx;
        }
        if p[2] > pts[extremes[5]][2] {
            extremes[5] = idx;
        }
    }

    let mut p0 = 0;
    let mut p1 = 0;
    let mut best_sq = 0.0_f32;
    for &i in &extremes {
        for &j in &extremes {
            let d_sq = norm_sq(sub(pts[i], pts[j]));
            if d_sq > best_sq {
                best_sq = d_sq;
                p0 = i;
                p1 = j;
            }
        }
    }
    if best_sq <= eps * eps {
        return None;
    }

    let line_dir = normalize(sub(pts[p1], pts[p0]))?;
    let mut p2 = p0;
    let mut best_line_dist_sq = eps * eps;
    for (idx, &p) in pts.iter().enumerate() {
        let rel = sub(p, pts[p0]);
        let dist_sq = norm_sq(cross(rel, line_dir));
        if dist_sq > best_line_dist_sq {
            best_line_dist_sq = dist_sq;
            p2 = idx;
        }
    }
    if p2 == p0 || p2 == p1 {
        return None;
    }

    let tri_normal = normalize(cross(sub(pts[p1], pts[p0]), sub(pts[p2], pts[p0])))?;
    let mut p3 = p0;
    let mut best_plane_dist = eps;
    for (idx, &p) in pts.iter().enumerate() {
        let dist = dot(sub(p, pts[p0]), tri_normal).abs();
        if dist > best_plane_dist {
            best_plane_dist = dist;
            p3 = idx;
        }
    }
    if p3 == p0 || p3 == p1 || p3 == p2 {
        return None;
    }
    Some([p0, p1, p2, p3])
}

fn assign_points_to_faces(
    pts: &[[f32; 3]],
    faces: &mut [HullFace],
    candidate_faces: &[usize],
    point_indices: &[usize],
    eps: f32,
) {
    for &p_idx in point_indices {
        let p = pts[p_idx];
        let mut best_face = None;
        let mut best_dist = eps;
        for &f_idx in candidate_faces {
            if !faces[f_idx].alive {
                continue;
            }
            let v0 = pts[faces[f_idx].verts[0]];
            let dist = dot(sub(p, v0), faces[f_idx].normal);
            if dist > best_dist {
                best_dist = dist;
                best_face = Some(f_idx);
            }
        }
        if let Some(f_idx) = best_face {
            faces[f_idx].outside.push((p_idx, best_dist));
        }
    }
}

/// Computes unique non-coplanar boundary edges of the 3D convex hull enclosing `raw_points`.
///
/// Degenerate, collinear, or coplanar vertex sets safely fall back to bounding-box wireframe
/// edges without panicking; inputs containing fewer than two finite points yield an empty hull.
#[must_use]
pub fn build_convex_hull_wireframe(raw_points: &[[f32; 3]]) -> ConvexHullWireframe {
    let pts = collect_finite_points(raw_points);
    if pts.len() < 2 {
        return ConvexHullWireframe::default();
    }

    let mut min = pts[0];
    let mut max = pts[0];
    for p in pts.iter().skip(1) {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    let diag = norm_sq(sub(max, min)).sqrt().max(1e-6);
    let eps = diag * 1e-5;

    let Some([i0, i1, i2, i3]) = initial_tetrahedron(&pts, eps) else {
        return ConvexHullWireframe {
            edges: aabb_edges(&pts),
        };
    };

    let centroid = scale(add(add(pts[i0], pts[i1]), add(pts[i2], pts[i3])), 0.25);
    let seed_tris = [[i0, i1, i2], [i0, i1, i3], [i1, i2, i3], [i2, i0, i3]];
    let mut faces: Vec<HullFace> = Vec::with_capacity(64);
    for tri in seed_tris {
        if let Some(face) = make_oriented_face(&pts, tri, centroid) {
            faces.push(face);
        }
    }
    if faces.len() < 4 {
        return ConvexHullWireframe {
            edges: aabb_edges(&pts),
        };
    }

    let remaining: Vec<usize> = (0..pts.len())
        .filter(|&idx| idx != i0 && idx != i1 && idx != i2 && idx != i3)
        .collect();
    assign_points_to_faces(&pts, &mut faces, &[0, 1, 2, 3], &remaining, eps);

    let max_iterations = pts.len().saturating_mul(4).max(64);
    for _ in 0..max_iterations {
        let Some((seed_face_idx, apex_idx)) = pick_furthest_conflict(&faces) else {
            break;
        };
        let apex = pts[apex_idx];

        let mut visible_indices: Vec<usize> = faces
            .iter()
            .enumerate()
            .filter_map(|(idx, f)| {
                if !f.alive {
                    return None;
                }
                let dist = dot(sub(apex, pts[f.verts[0]]), f.normal);
                (dist > eps).then_some(idx)
            })
            .collect();
        if visible_indices.is_empty() {
            visible_indices.push(seed_face_idx);
        }

        let mut edge_counts: HashMap<(usize, usize), usize> = HashMap::new();
        for &idx in &visible_indices {
            let v = faces[idx].verts;
            for &(a, b) in &[(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
                *edge_counts.entry(edge_key(a, b)).or_insert(0) += 1;
            }
        }

        let mut horizon_edges: Vec<(usize, usize)> = Vec::new();
        for &idx in &visible_indices {
            let v = faces[idx].verts;
            for &(a, b) in &[(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
                if edge_counts.get(&edge_key(a, b)).copied() == Some(1) {
                    horizon_edges.push((a, b));
                }
            }
        }
        if horizon_edges.len() < 3 {
            if let Some(last_pos) = faces[seed_face_idx]
                .outside
                .iter()
                .position(|&(p, _)| p == apex_idx)
            {
                faces[seed_face_idx].outside.swap_remove(last_pos);
            }
            continue;
        }

        let mut orphaned_points: Vec<usize> = Vec::new();
        for &idx in &visible_indices {
            faces[idx].alive = false;
            for &(p_idx, _) in &faces[idx].outside {
                if p_idx != apex_idx {
                    orphaned_points.push(p_idx);
                }
            }
            faces[idx].outside.clear();
        }

        let new_start = faces.len();
        for (a, b) in horizon_edges {
            if let Some(new_face) = make_oriented_face(&pts, [a, b, apex_idx], centroid) {
                faces.push(new_face);
            }
        }
        let new_indices: Vec<usize> = (new_start..faces.len()).collect();
        assign_points_to_faces(&pts, &mut faces, &new_indices, &orphaned_points, eps);
    }

    let mut edge_normals: HashMap<(usize, usize), Vec<[f32; 3]>> = HashMap::new();
    for face in faces.iter().filter(|f| f.alive) {
        let v = face.verts;
        for &(a, b) in &[(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
            edge_normals
                .entry(edge_key(a, b))
                .or_default()
                .push(face.normal);
        }
    }

    let mut edges = Vec::with_capacity(edge_normals.len());
    for ((a, b), normals) in edge_normals {
        let is_coplanar_diagonal =
            normals.len() >= 2 && normals.windows(2).all(|w| dot(w[0], w[1]) > 0.9995);
        if !is_coplanar_diagonal {
            edges.push((pts[a], pts[b]));
        }
    }

    ConvexHullWireframe { edges }
}

fn pick_furthest_conflict(faces: &[HullFace]) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize, f32)> = None;
    for (f_idx, face) in faces.iter().enumerate() {
        if !face.alive {
            continue;
        }
        for &(p_idx, dist) in &face.outside {
            if best.is_none_or(|(_, _, cur)| dist > cur) {
                best = Some((f_idx, p_idx, dist));
            }
        }
    }
    best.map(|(f_idx, p_idx, _)| (f_idx, p_idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convex_hull_cube_drops_interior_points_and_coplanar_diagonals() {
        let pts = vec![
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            // Strict interior points that must not appear in hull edges
            [0.0, 0.0, 0.0],
            [0.25, -0.4, 0.1],
            [-0.5, 0.3, -0.2],
        ];
        let hull = build_convex_hull_wireframe(&pts);
        assert_eq!(
            hull.edges.len(),
            12,
            "Cube convex hull must emit exactly 12 boundary edges without face diagonals"
        );
        for (a, b) in &hull.edges {
            assert!(a.iter().all(|v| v.abs() > 0.9));
            assert!(b.iter().all(|v| v.abs() > 0.9));
        }
    }

    #[test]
    fn test_convex_hull_degenerate_and_non_finite_inputs() {
        assert!(build_convex_hull_wireframe(&[]).edges.is_empty());
        assert!(
            build_convex_hull_wireframe(&[[0.0, 0.0, 0.0], [f32::NAN, 1.0, 2.0]])
                .edges
                .is_empty()
        );

        let identical = vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [1.0, 2.0, 3.0]];
        assert_eq!(build_convex_hull_wireframe(&identical).edges.len(), 12);

        let coplanar = vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
        ];
        assert_eq!(build_convex_hull_wireframe(&coplanar).edges.len(), 12);
    }
}
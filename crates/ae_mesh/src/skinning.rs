// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! CPU linear-blend skeletal skinning for mesh vertices.

use crate::vertex::{SkinVertex, Vertex};

/// Evaluates linear-blend skinning transforms for every [`SkinVertex`] and writes
/// updated object-space positions and normals into `out_vertices`.
///
/// Vertices whose bone weight sum is below `0.001` or whose index exceeds
/// `out_vertices.len()` are skipped cleanly. If the animation palette is empty,
/// no vertices are modified. Out-of-range joint indices fall back to identity.
pub fn apply_skinning_palette(
    raw_skin_vertices: &[SkinVertex],
    palette: &ae_animation::SkinningPalette,
    out_vertices: &mut [Vertex],
) {
    if palette.is_empty() || raw_skin_vertices.is_empty() {
        return;
    }

    let limit = raw_skin_vertices.len().min(out_vertices.len());
    for (skin_v, dst_v) in raw_skin_vertices[..limit]
        .iter()
        .zip(out_vertices[..limit].iter_mut())
    {
        let w = skin_v.joint_weights;
        let total_w = w[0] + w[1] + w[2] + w[3];
        if !total_w.is_finite() || total_w < 0.001 {
            continue;
        }

        let mut skin_mat = glam::Mat4::ZERO;
        for slot in 0..4 {
            let joint_idx = skin_v.joint_indices[slot] as usize;
            let mat = palette
                .matrices
                .get(joint_idx)
                .map_or(glam::Mat4::IDENTITY, |m| m.to_mat4());
            skin_mat += mat * w[slot];
        }

        let pos = skin_mat.transform_point3(glam::Vec3::from_array(skin_v.bind_position));
        let norm = skin_mat
            .transform_vector3(glam::Vec3::from_array(skin_v.bind_normal))
            .normalize_or_zero();

        dst_v.position = pos.to_array();
        dst_v.normal = norm.to_array();
    }
}
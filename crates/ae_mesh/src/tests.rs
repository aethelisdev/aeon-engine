// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

use crate::{
    CUBE_VERTICES, GRID_QUAD_VERTICES, PYRAMID_VERTICES, QUAD_VERTICES, SkinVertex,
    SubmeshAlphaMode, Vertex, apply_skinning_palette, compute_aabb_bounding_radius,
    generate_capsule, generate_cylinder, generate_sphere, generate_torus, is_safe_path,
    parse_gltf_bytes, parse_gltf_file, parse_gltf_raw_geometry,
};
use std::path::{Path, PathBuf};

#[test]
fn test_primitive_constants_and_generators_bounds() {
    assert_eq!(PYRAMID_VERTICES.len(), 18);
    assert_eq!(CUBE_VERTICES.len(), 36);
    assert_eq!(QUAD_VERTICES.len(), 6);
    assert_eq!(GRID_QUAD_VERTICES.len(), 6);

    let sphere = generate_sphere(12, 16);
    assert_eq!(sphere.len(), 12 * 16 * 6);
    for v in &sphere {
        let r = (v.position[0].powi(2) + v.position[1].powi(2) + v.position[2].powi(2)).sqrt();
        assert!((r - 0.5).abs() < 1e-4, "sphere vertex off radius: {r}");
        assert!(v.normal.iter().all(|c| c.is_finite()));
    }

    let cylinder = generate_cylinder(12);
    assert_eq!(cylinder.len(), 12 * 12);

    let capsule = generate_capsule(8, 6);
    assert!(!capsule.is_empty());

    let torus = generate_torus(8, 6);
    assert_eq!(torus.len(), 8 * 6 * 6);
}

#[test]
fn test_primitive_triangle_winding_matches_outward_normals() {
    let assert_ccw_mesh = |verts: &[Vertex], label: &str| {
        let (tris, rem) = verts.as_chunks::<3>();
        assert!(rem.is_empty(), "{label} must be a triangle list");
        for (tri_idx, tri) in tris.iter().enumerate() {
            let p0 = glam::Vec3::from_array(tri[0].position);
            let p1 = glam::Vec3::from_array(tri[1].position);
            let p2 = glam::Vec3::from_array(tri[2].position);
            let cross = (p1 - p0).cross(p2 - p0);
            let n = glam::Vec3::from_array(tri[0].normal);
            if cross.length_squared() > 1e-10 {
                assert!(
                    cross.dot(n) > 0.0,
                    "{label} triangle {tri_idx} has inverted winding (cross={cross:?}, normal={n:?})"
                );
            }
        }
    };

    assert_ccw_mesh(PYRAMID_VERTICES, "PYRAMID_VERTICES");
    assert_ccw_mesh(CUBE_VERTICES, "CUBE_VERTICES");
    assert_ccw_mesh(GRID_QUAD_VERTICES, "GRID_QUAD_VERTICES");
    assert_ccw_mesh(&generate_sphere(8, 12), "sphere");
    assert_ccw_mesh(&generate_cylinder(12), "cylinder");
    assert_ccw_mesh(&generate_capsule(8, 6), "capsule");
    assert_ccw_mesh(&generate_torus(8, 6), "torus");

    let (quad_tris, quad_rem) = QUAD_VERTICES.as_chunks::<3>();
    assert!(quad_rem.is_empty(), "QUAD_VERTICES must be a triangle list");
    for (tri_idx, tri) in quad_tris.iter().enumerate() {
        let p0 = glam::Vec3::from_array(tri[0].position);
        let p1 = glam::Vec3::from_array(tri[1].position);
        let p2 = glam::Vec3::from_array(tri[2].position);
        let cross = (p1 - p0).cross(p2 - p0);
        let n = glam::Vec3::from_array(tri[0].normal);
        assert!(
            cross.dot(n) > 0.0,
            "QUAD_VERTICES triangle {tri_idx} has inverted winding"
        );
    }
}

#[test]
fn test_degenerate_zero_segment_generators_return_empty_without_nan() {
    assert!(generate_sphere(0, 10).is_empty());
    assert!(generate_sphere(10, 0).is_empty());
    assert!(generate_cylinder(0).is_empty());
    assert!(generate_capsule(0, 6).is_empty());
    assert!(generate_capsule(6, 0).is_empty());
    assert!(generate_torus(0, 6).is_empty());
    assert!(generate_torus(6, 0).is_empty());
}

#[test]
fn test_compute_aabb_bounding_radius_golden_and_faults() {
    // Centered unit cube [-0.5, 0.5]^3: center=origin, half_diag=sqrt(0.75) ~= 0.866 -> clamped to 1.0
    assert_eq!(compute_aabb_bounding_radius([-0.5; 3], [0.5; 3]), 1.0);

    // Shifted box [3..5, 0..0, 0..0]: center=(4,0,0) dist=4.0, half_diag=1.0 -> 5.0
    let radius = compute_aabb_bounding_radius([3.0, 0.0, 0.0], [5.0, 0.0, 0.0]);
    assert!((radius - 5.0).abs() < 1e-5, "got {radius}");

    // Non-finite inputs safely clamp to 1.0
    assert_eq!(compute_aabb_bounding_radius([f32::NAN; 3], [1.0; 3]), 1.0);
    assert_eq!(
        compute_aabb_bounding_radius([0.0; 3], [f32::INFINITY; 3]),
        1.0
    );
}

#[test]
fn test_apply_skinning_palette_transforms_and_ignores_invalid_weights() {
    let skin_verts = [
        SkinVertex {
            bind_position: [1.0, 2.0, 3.0],
            bind_normal: [0.0, 1.0, 0.0],
            joint_indices: [0, 99, 0, 0],
            joint_weights: [1.0, 0.0, 0.0, 0.0],
        },
        SkinVertex {
            bind_position: [5.0, 5.0, 5.0],
            bind_normal: [0.0, 0.0, 1.0],
            joint_indices: [0, 0, 0, 0],
            joint_weights: [0.0, 0.0, 0.0, 0.0],
        },
    ];
    let mut out = [
        Vertex {
            position: [0.0; 3],
            color: [1.0; 3],
            normal: [0.0; 3],
            uv: [0.0; 2],
        },
        Vertex {
            position: [9.0, 9.0, 9.0],
            color: [1.0; 3],
            normal: [1.0, 0.0, 0.0],
            uv: [0.0; 2],
        },
    ];

    let translation = glam::Mat4::from_translation(glam::Vec3::new(10.0, -2.0, 0.5));
    let palette = ae_animation::SkinningPalette {
        matrices: vec![ae_animation::JointMatrix::from_mat4(translation)],
    };

    apply_skinning_palette(&skin_verts, &palette, &mut out);

    // Vertex 0 is translated by joint 0
    assert!((out[0].position[0] - 11.0).abs() < 1e-5);
    assert!((out[0].position[1] - 0.0).abs() < 1e-5);
    assert!((out[0].position[2] - 3.5).abs() < 1e-5);

    // Vertex 1 has zero weight -> left untouched
    assert_eq!(out[1].position, [9.0, 9.0, 9.0]);
}

#[test]
fn test_path_security_and_invalid_gltf_inputs() {
    assert!(is_safe_path("models/robot.glb"));
    assert!(is_safe_path("Content/Meshes/crate.gltf"));
    assert!(!is_safe_path(""));
    assert!(!is_safe_path("../secret.glb"));
    assert!(!is_safe_path("models/../../etc/passwd"));
    assert!(!is_safe_path("\\\\remote\\share\\mesh.glb"));
    assert!(!is_safe_path("http://example.com/mesh.glb"));
    assert!(!is_safe_path("models/\0bad.glb"));

    assert!(parse_gltf_file("../blocked.glb", "blocked".into()).is_err());
    assert!(
        parse_gltf_bytes(
            b"not-a-gltf",
            PathBuf::from("bad.glb"),
            "bad.glb",
            String::new()
        )
        .is_err()
    );
    assert!(parse_gltf_raw_geometry(Path::new("nonexistent.txt")).is_none());
}

#[test]
fn test_parse_gltf_bytes_embedded_triangle() {
    // 1-triangle embedded GLTF 2.0 JSON with base64 data URI buffer
    // Positions: (0,0,0), (1,0,0), (0,1,0) -> 36 bytes + 6 bytes u16 indices [0, 1, 2] -> 42 bytes
    let mut bin = Vec::new();
    for p in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
        for f in p {
            bin.extend_from_slice(&f.to_le_bytes());
        }
    }
    for i in [0u16, 1, 2] {
        bin.extend_from_slice(&i.to_le_bytes());
    }

    const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut b64 = String::new();
    for chunk in bin.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        b64.push(B64[((n >> 18) & 63) as usize] as char);
        b64.push(B64[((n >> 12) & 63) as usize] as char);
        b64.push(if chunk.len() > 1 {
            B64[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        b64.push(if chunk.len() > 2 {
            B64[(n & 63) as usize] as char
        } else {
            '='
        });
    }

    let gltf_json = format!(
        r#"{{
          "asset": {{ "version": "2.0" }},
          "buffers": [{{ "uri": "data:application/octet-stream;base64,{b64}", "byteLength": 42 }}],
          "bufferViews": [
            {{ "buffer": 0, "byteOffset": 0, "byteLength": 36 }},
            {{ "buffer": 0, "byteOffset": 36, "byteLength": 6 }}
          ],
          "accessors": [
            {{ "bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3", "min": [0,0,0], "max": [1,1,0] }},
            {{ "bufferView": 1, "componentType": 5123, "count": 3, "type": "SCALAR" }}
          ],
          "meshes": [{{
            "primitives": [{{
              "attributes": {{ "POSITION": 0 }},
              "indices": 1
            }}]
          }}],
          "nodes": [{{ "mesh": 0 }}],
          "scenes": [{{ "nodes": [0] }}],
          "scene": 0
        }}"#
    );

    let parsed = parse_gltf_bytes(
        gltf_json.as_bytes(),
        PathBuf::from("tri.gltf"),
        "tri.gltf",
        "Tri".to_string(),
    )
    .expect("valid glTF triangle should parse");

    assert_eq!(parsed.all_vertices.len(), 3);
    assert_eq!(parsed.all_indices, vec![0, 1, 2]);
    assert_eq!(parsed.submeshes.len(), 1);
    assert_eq!(parsed.submeshes[0].alpha_mode, SubmeshAlphaMode::Opaque);
    assert_eq!(parsed.min, [0.0, 0.0, 0.0]);
    assert_eq!(parsed.max, [1.0, 1.0, 0.0]);
}
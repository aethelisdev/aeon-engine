// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! GLTF scene graph and primitive mesh geometry parsing.

use crate::render::types::{SkinVertex, Vertex};

/// Traverses GLTF scene node hierarchy and extracts all primitive geometry with world node transforms applied.
pub fn parse_gltf_scene_geometry(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    has_skeleton: bool,
) -> (
    Vec<Vertex>,
    Vec<u32>,
    Vec<[f32; 3]>,
    Vec<SkinVertex>,
    Vec<crate::render::types::ModelSubmesh>,
    [f32; 3],
    [f32; 3],
) {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();
    let mut raw_positions = Vec::new();
    let mut raw_skin_vertices = Vec::new();
    let mut submeshes = Vec::new();

    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    let has_real_skin = document.skins().next().is_some();
    let is_node_hierarchy = !has_real_skin && document.animations().next().is_some();

    let root_nodes: Vec<gltf::Node> = if let Some(default_scene) = document.default_scene() {
        default_scene.nodes().collect()
    } else if let Some(first_scene) = document.scenes().next() {
        first_scene.nodes().collect()
    } else {
        document.nodes().collect()
    };

    if !root_nodes.is_empty() {
        for root in &root_nodes {
            traverse_gltf_node(
                root,
                glam::Mat4::IDENTITY,
                buffers,
                images,
                &mut all_vertices,
                &mut all_indices,
                &mut raw_positions,
                &mut raw_skin_vertices,
                &mut submeshes,
                &mut min,
                &mut max,
                has_skeleton,
                has_real_skin,
                is_node_hierarchy,
            );
        }
    } else {
        for mesh in document.meshes() {
            process_gltf_primitive_mesh(
                &mesh,
                mesh.index(),
                glam::Mat4::IDENTITY,
                buffers,
                images,
                &mut all_vertices,
                &mut all_indices,
                &mut raw_positions,
                &mut raw_skin_vertices,
                &mut submeshes,
                &mut min,
                &mut max,
                has_skeleton,
                has_real_skin,
                is_node_hierarchy,
            );
        }
    }

    if min[0] > max[0] {
        min = [-0.5; 3];
        max = [0.5; 3];
    }

    (
        all_vertices,
        all_indices,
        raw_positions,
        raw_skin_vertices,
        submeshes,
        min,
        max,
    )
}

#[allow(clippy::too_many_arguments)]
fn traverse_gltf_node(
    node: &gltf::Node,
    parent_transform: glam::Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u32>,
    raw_positions: &mut Vec<[f32; 3]>,
    raw_skin_vertices: &mut Vec<SkinVertex>,
    submeshes: &mut Vec<crate::render::types::ModelSubmesh>,
    min: &mut [f32; 3],
    max: &mut [f32; 3],
    has_skeleton: bool,
    has_real_skin: bool,
    is_node_hierarchy: bool,
) {
    let local_transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local_transform;

    if let Some(mesh) = node.mesh() {
        process_gltf_primitive_mesh(
            &mesh,
            node.index(),
            world_transform,
            buffers,
            images,
            all_vertices,
            all_indices,
            raw_positions,
            raw_skin_vertices,
            submeshes,
            min,
            max,
            has_skeleton,
            has_real_skin,
            is_node_hierarchy,
        );
    }

    for child in node.children() {
        traverse_gltf_node(
            &child,
            world_transform,
            buffers,
            images,
            all_vertices,
            all_indices,
            raw_positions,
            raw_skin_vertices,
            submeshes,
            min,
            max,
            has_skeleton,
            has_real_skin,
            is_node_hierarchy,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn process_gltf_primitive_mesh(
    mesh: &gltf::Mesh,
    node_index: usize,
    world_transform: glam::Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    all_vertices: &mut Vec<Vertex>,
    all_indices: &mut Vec<u32>,
    raw_positions: &mut Vec<[f32; 3]>,
    raw_skin_vertices: &mut Vec<SkinVertex>,
    submeshes: &mut Vec<crate::render::types::ModelSubmesh>,
    min: &mut [f32; 3],
    max: &mut [f32; 3],
    has_skeleton: bool,
    has_real_skin: bool,
    is_node_hierarchy: bool,
) {
    let normal_matrix = glam::Mat3::from_mat4(world_transform);

    for primitive in mesh.primitives() {
        let mat = primitive.material();
        let pbr = mat.pbr_metallic_roughness();
        let base_color = pbr.base_color_factor();
        let texture_index = pbr
            .base_color_texture()
            .map(|t| t.texture().source().index());

        let has_transparent_pixels =
            texture_index
                .and_then(|idx| images.get(idx))
                .is_some_and(|img| {
                    if img.format == gltf::image::Format::R8G8B8A8 {
                        img.pixels.chunks_exact(4).any(|c| c[3] < 245)
                    } else {
                        false
                    }
                });

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
        let pos_iter = match reader.read_positions() {
            Some(iter) => iter,
            None => continue,
        };
        let mut norm_iter = reader.read_normals();
        let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());
        let mut tex_coord_iter = reader.read_tex_coords(0).map(|tc| tc.into_f32());

        let mut joint_iter = reader.read_joints(0).map(|j| j.into_u16());
        let mut weight_iter = reader.read_weights(0).map(|w| w.into_f32());

        let has_explicit_skin = joint_iter.is_some() && weight_iter.is_some();
        let is_skinned = has_explicit_skin && has_real_skin;

        let alpha_mode = match mat.alpha_mode() {
            gltf::material::AlphaMode::Opaque => {
                if has_transparent_pixels && !is_skinned {
                    crate::render::types::SubmeshAlphaMode::Mask
                } else {
                    crate::render::types::SubmeshAlphaMode::Opaque
                }
            }
            gltf::material::AlphaMode::Mask => crate::render::types::SubmeshAlphaMode::Mask,
            gltf::material::AlphaMode::Blend => {
                if is_skinned && !has_transparent_pixels {
                    crate::render::types::SubmeshAlphaMode::Opaque
                } else if has_transparent_pixels {
                    crate::render::types::SubmeshAlphaMode::Mask
                } else {
                    crate::render::types::SubmeshAlphaMode::Blend
                }
            }
        };
        let alpha_cutoff = mat.alpha_cutoff().unwrap_or(0.5);

        let start_vertex = all_vertices.len() as u32;
        let start_index = all_indices.len() as u32;

        for raw_pos in pos_iter {
            let raw_norm = norm_iter
                .as_mut()
                .and_then(|n| n.next())
                .unwrap_or([0.0, 1.0, 0.0]);
            let raw_col = col_iter
                .as_mut()
                .and_then(|c| c.next())
                .unwrap_or([1.0, 1.0, 1.0]);
            let color = [
                raw_col[0] * base_color[0],
                raw_col[1] * base_color[1],
                raw_col[2] * base_color[2],
            ];
            let uv = tex_coord_iter
                .as_mut()
                .and_then(|tc| tc.next())
                .unwrap_or([0.0, 0.0]);

            let (final_pos, final_norm) = if is_skinned {
                (raw_pos, raw_norm)
            } else {
                let p4 = world_transform * glam::Vec4::new(raw_pos[0], raw_pos[1], raw_pos[2], 1.0);
                let n3 = (normal_matrix * glam::Vec3::new(raw_norm[0], raw_norm[1], raw_norm[2]))
                    .normalize_or_zero();
                ([p4.x, p4.y, p4.z], [n3.x, n3.y, n3.z])
            };

            let (j_indices, j_weights) = if is_skinned {
                if let (Some(j_it), Some(w_it)) = (joint_iter.as_mut(), weight_iter.as_mut()) {
                    let raw_j = j_it.next().unwrap_or([0, 0, 0, 0]);
                    let raw_w = w_it.next().unwrap_or([0.0, 0.0, 0.0, 0.0]);
                    (
                        [
                            raw_j[0] as u32,
                            raw_j[1] as u32,
                            raw_j[2] as u32,
                            raw_j[3] as u32,
                        ],
                        raw_w,
                    )
                } else {
                    ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0])
                }
            } else if is_node_hierarchy {
                ([node_index as u32, 0, 0, 0], [1.0, 0.0, 0.0, 0.0])
            } else {
                ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0])
            };

            for i in 0..3 {
                if final_pos[i] < min[i] {
                    min[i] = final_pos[i];
                }
                if final_pos[i] > max[i] {
                    max[i] = final_pos[i];
                }
            }

            all_vertices.push(Vertex {
                position: final_pos,
                color,
                normal: final_norm,
                uv,
            });
            raw_positions.push(final_pos);

            if has_skeleton {
                let (bind_pos, bind_norm) = if is_skinned {
                    (raw_pos, raw_norm)
                } else {
                    (final_pos, final_norm)
                };
                raw_skin_vertices.push(SkinVertex {
                    bind_position: bind_pos,
                    bind_normal: bind_norm,
                    joint_indices: j_indices,
                    joint_weights: j_weights,
                });
            }
        }

        if let Some(indices) = reader.read_indices() {
            for idx in indices.into_u32() {
                all_indices.push(start_vertex + idx);
            }
        }

        let index_count = (all_indices.len() as u32) - start_index;
        if index_count > 0 {
            submeshes.push(crate::render::types::ModelSubmesh {
                start_index,
                index_count,
                texture_index,
                base_color,
                alpha_mode,
                alpha_cutoff,
            });
        }
    }
}
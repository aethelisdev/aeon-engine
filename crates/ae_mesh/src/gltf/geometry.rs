// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! glTF 2.0 scene graph traversal and primitive mesh geometry extraction.

use crate::submesh::{ModelSubmesh, SubmeshAlphaMode};
use crate::vertex::{SkinVertex, Vertex};
use std::path::Path;

/// Extracted full scene geometry, submesh slices, skinning attributes, and AABB bounds.
#[derive(Debug, Clone)]
pub struct ParsedGltfGeometry {
    /// Flattened object-space vertices across all scene meshes and primitives.
    pub all_vertices: Vec<Vertex>,
    /// Triangle index stream referencing `all_vertices`.
    pub all_indices: Vec<u32>,
    /// Raw positions parallel to `all_vertices` for physics mesh generation.
    pub raw_positions: Vec<[f32; 3]>,
    /// Optional per-vertex skinning bind attributes when `has_skeleton` is set.
    pub raw_skin_vertices: Vec<SkinVertex>,
    /// Contiguous submesh draw ranges and PBR surface parameters.
    pub submeshes: Vec<ModelSubmesh>,
    /// Minimum corner `[x, y, z]` of the extracted geometry AABB.
    pub min: [f32; 3],
    /// Maximum corner `[x, y, z]` of the extracted geometry AABB.
    pub max: [f32; 3],
}

/// Lightweight raw mesh geometry (positions, normals, triangle indices) without
/// animations, materials, or textures, intended for thumbnail rasterizers.
#[derive(Debug, Clone, PartialEq)]
pub struct RawMeshGeometry {
    /// Vertex positions `[x, y, z]`.
    pub vertices: Vec<[f32; 3]>,
    /// Vertex normals `[nx, ny, nz]` matching `vertices.len()`.
    pub normals: Vec<[f32; 3]>,
    /// Triangle indices referencing `vertices`.
    pub indices: Vec<u32>,
}

/// Extracts raw positions, normals, and triangle indices from a `.gltf` or `.glb` file on disk.
///
/// Returns `None` if the extension is not `.gltf`/`.glb`, loading fails, or the model
/// contains zero vertex positions.
#[must_use]
pub fn parse_gltf_raw_geometry(path: &Path) -> Option<RawMeshGeometry> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    if ext != "gltf" && ext != "glb" {
        return None;
    }

    let (doc, buffers, _) = gltf::import(path).ok()?;
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for mesh in doc.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|b| buffers.get(b.index()).map(|d| &d.0[..]));
            let base_idx = vertices.len() as u32;

            if let Some(pos_iter) = reader.read_positions() {
                vertices.extend(pos_iter);
            }
            if let Some(norm_iter) = reader.read_normals() {
                normals.extend(norm_iter);
            }
            if let Some(idx_iter) = reader.read_indices() {
                indices.extend(idx_iter.into_u32().map(|idx| base_idx + idx));
            }
        }
    }

    if vertices.is_empty() {
        return None;
    }

    normals.resize(vertices.len(), [0.0, 1.0, 0.0]);
    if indices.is_empty() {
        indices.extend(0..vertices.len() as u32);
    }

    Some(RawMeshGeometry {
        vertices,
        normals,
        indices,
    })
}

/// Traverses a glTF scene node hierarchy and extracts all primitive geometry with
/// world node transforms baked into static primitives.
#[must_use]
pub fn parse_gltf_scene_geometry(
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    has_skeleton: bool,
) -> ParsedGltfGeometry {
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

    let mut ctx = GltfGeometryContext {
        buffers,
        images,
        all_vertices: &mut all_vertices,
        all_indices: &mut all_indices,
        raw_positions: &mut raw_positions,
        raw_skin_vertices: &mut raw_skin_vertices,
        submeshes: &mut submeshes,
        min: &mut min,
        max: &mut max,
        has_skeleton,
        has_real_skin,
        is_node_hierarchy,
    };

    if !root_nodes.is_empty() {
        for root in &root_nodes {
            traverse_gltf_node(root, glam::Mat4::IDENTITY, &mut ctx);
        }
    } else {
        for mesh in document.meshes() {
            process_gltf_primitive_mesh(&mesh, mesh.index(), glam::Mat4::IDENTITY, &mut ctx);
        }
    }

    if min[0] > max[0] {
        min = [-0.5; 3];
        max = [0.5; 3];
    }

    ParsedGltfGeometry {
        all_vertices,
        all_indices,
        raw_positions,
        raw_skin_vertices,
        submeshes,
        min,
        max,
    }
}

struct GltfGeometryContext<'a> {
    buffers: &'a [gltf::buffer::Data],
    images: &'a [gltf::image::Data],
    all_vertices: &'a mut Vec<Vertex>,
    all_indices: &'a mut Vec<u32>,
    raw_positions: &'a mut Vec<[f32; 3]>,
    raw_skin_vertices: &'a mut Vec<SkinVertex>,
    submeshes: &'a mut Vec<ModelSubmesh>,
    min: &'a mut [f32; 3],
    max: &'a mut [f32; 3],
    has_skeleton: bool,
    has_real_skin: bool,
    is_node_hierarchy: bool,
}

fn traverse_gltf_node(
    node: &gltf::Node,
    parent_transform: glam::Mat4,
    ctx: &mut GltfGeometryContext<'_>,
) {
    let local_transform = glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local_transform;

    if let Some(mesh) = node.mesh() {
        process_gltf_primitive_mesh(&mesh, node.index(), world_transform, ctx);
    }

    for child in node.children() {
        traverse_gltf_node(&child, world_transform, ctx);
    }
}

fn resolve_alpha_mode(
    gltf_mode: gltf::material::AlphaMode,
    has_transparent_pixels: bool,
    is_skinned: bool,
) -> SubmeshAlphaMode {
    match gltf_mode {
        gltf::material::AlphaMode::Opaque => {
            if has_transparent_pixels && !is_skinned {
                SubmeshAlphaMode::Mask
            } else {
                SubmeshAlphaMode::Opaque
            }
        }
        gltf::material::AlphaMode::Mask => SubmeshAlphaMode::Mask,
        gltf::material::AlphaMode::Blend => {
            if is_skinned && !has_transparent_pixels {
                SubmeshAlphaMode::Opaque
            } else if has_transparent_pixels {
                SubmeshAlphaMode::Mask
            } else {
                SubmeshAlphaMode::Blend
            }
        }
    }
}

fn process_gltf_primitive_mesh(
    mesh: &gltf::Mesh,
    node_index: usize,
    world_transform: glam::Mat4,
    ctx: &mut GltfGeometryContext<'_>,
) {
    let mat3 = glam::Mat3::from_mat4(world_transform);
    let normal_matrix = if mat3.determinant().abs() > 1e-6 {
        mat3.inverse().transpose()
    } else {
        glam::Mat3::IDENTITY
    };

    for primitive in mesh.primitives() {
        let mat = primitive.material();
        let pbr = mat.pbr_metallic_roughness();
        let base_color = pbr.base_color_factor();
        let metallic_factor = pbr.metallic_factor();
        let roughness_factor = pbr.roughness_factor();
        let texture_index = pbr
            .base_color_texture()
            .map(|t| t.texture().source().index());

        let has_transparent_pixels = texture_index
            .and_then(|idx| ctx.images.get(idx))
            .is_some_and(|img| {
                img.format == gltf::image::Format::R8G8B8A8
                    && img.pixels.as_chunks::<4>().0.iter().any(|c| c[3] < 245)
            });

        let reader = primitive.reader(|buffer| Some(&ctx.buffers[buffer.index()]));
        let Some(pos_iter) = reader.read_positions() else {
            continue;
        };
        let mut norm_iter = reader.read_normals();
        let mut col_iter = reader.read_colors(0).map(|c| c.into_rgb_f32());
        let mut tex_coord_iter = reader.read_tex_coords(0).map(|tc| tc.into_f32());
        let mut joint_iter = reader.read_joints(0).map(|j| j.into_u16());
        let mut weight_iter = reader.read_weights(0).map(|w| w.into_f32());

        let is_skinned = joint_iter.is_some() && weight_iter.is_some() && ctx.has_real_skin;
        let alpha_mode = resolve_alpha_mode(mat.alpha_mode(), has_transparent_pixels, is_skinned);
        let alpha_cutoff = mat.alpha_cutoff().unwrap_or(0.5);

        let start_vertex = ctx.all_vertices.len() as u32;
        let start_index = ctx.all_indices.len() as u32;

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
                            u32::from(raw_j[0]),
                            u32::from(raw_j[1]),
                            u32::from(raw_j[2]),
                            u32::from(raw_j[3]),
                        ],
                        raw_w,
                    )
                } else {
                    ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0])
                }
            } else if ctx.is_node_hierarchy {
                ([node_index as u32, 0, 0, 0], [1.0, 0.0, 0.0, 0.0])
            } else {
                ([0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0])
            };

            for ((mn, mx), &p) in ctx.min.iter_mut().zip(ctx.max.iter_mut()).zip(&final_pos) {
                if p < *mn {
                    *mn = p;
                }
                if p > *mx {
                    *mx = p;
                }
            }

            ctx.all_vertices.push(Vertex {
                position: final_pos,
                color,
                normal: final_norm,
                uv,
            });
            ctx.raw_positions.push(final_pos);

            if ctx.has_skeleton {
                let (bind_pos, bind_norm) = if is_skinned {
                    (raw_pos, raw_norm)
                } else {
                    (final_pos, final_norm)
                };
                ctx.raw_skin_vertices.push(SkinVertex {
                    bind_position: bind_pos,
                    bind_normal: bind_norm,
                    joint_indices: j_indices,
                    joint_weights: j_weights,
                });
            }
        }

        if let Some(indices) = reader.read_indices() {
            for idx in indices.into_u32() {
                ctx.all_indices.push(start_vertex + idx);
            }
        }

        let index_count = (ctx.all_indices.len() as u32) - start_index;
        if index_count > 0 {
            ctx.submeshes.push(ModelSubmesh {
                start_index,
                index_count,
                texture_index,
                base_color,
                alpha_mode,
                alpha_cutoff,
                metallic_factor,
                roughness_factor,
                uv_scale: [1.0, 1.0],
            });
        }
    }
}
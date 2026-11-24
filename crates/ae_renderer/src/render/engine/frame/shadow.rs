// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Adapter feeding scene primitive and model geometry into [`ae_pbr::ShadowSystem`].

use super::forward::resolve_submesh_texture;
use crate::render::engine::state::RenderState;
use ae_mesh::SubmeshAlphaMode;
use ae_pbr::{INSTANCE_SIZE, RenderInstanceItem};
use std::collections::HashMap;

/// Per-primitive buffer offsets for depth-only directional shadow passes.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ShadowBatchOffsets {
    pub all_instances_count: usize,
    pub triangle_instances_count: usize,
    pub tri_start: usize,
    pub cube_instances_count: usize,
    pub cube_start: usize,
    pub sphere_instances_count: usize,
    pub sphere_start: usize,
    pub cylinder_instances_count: usize,
    pub cylinder_start: usize,
    pub capsule_instances_count: usize,
    pub capsule_start: usize,
    pub torus_instances_count: usize,
    pub torus_start: usize,
}

impl RenderState {
    /// Executes all active directional shadow cascade passes against current scene geometry.
    pub(crate) fn execute_shadow_passes(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        offsets: ShadowBatchOffsets,
        asset_manager: &crate::asset::AssetManager,
        model_instance_data: &HashMap<crate::asset::AssetHandle, Vec<RenderInstanceItem>>,
    ) {
        let primitives = &self.geometry;
        let inst_buf = &primitives.instance_buffer;
        let prim_draws = [
            (
                offsets.triangle_instances_count,
                offsets.tri_start,
                &primitives.vertex_buffer,
                primitives.triangle_num_vertices,
            ),
            (
                offsets.cube_instances_count,
                offsets.cube_start,
                &primitives.cube_vertex_buffer,
                36,
            ),
            (
                offsets.sphere_instances_count,
                offsets.sphere_start,
                &primitives.sphere_vertex_buffer,
                primitives.sphere_num_vertices,
            ),
            (
                offsets.cylinder_instances_count,
                offsets.cylinder_start,
                &primitives.cylinder_vertex_buffer,
                primitives.cylinder_num_vertices,
            ),
            (
                offsets.capsule_instances_count,
                offsets.capsule_start,
                &primitives.capsule_vertex_buffer,
                primitives.capsule_num_vertices,
            ),
            (
                offsets.torus_instances_count,
                offsets.torus_start,
                &primitives.torus_vertex_buffer,
                primitives.torus_num_vertices,
            ),
        ];

        self.shadow.execute_pass(
            encoder,
            self.graphics_settings.shadow_enabled,
            self.graphics_settings.shadow_cascades,
            offsets.all_instances_count > 0,
            |cascade_pass| {
                for (count, start, vb, vert_count) in prim_draws {
                    if count > 0 {
                        cascade_pass.draw_vertices(
                            vb.slice(..),
                            inst_buf.slice((start * INSTANCE_SIZE) as u64..),
                            0..vert_count,
                            0..count as u32,
                        );
                    }
                }

                for (handle, model) in asset_manager.models.iter() {
                    let count = model_instance_data.get(&handle).map_or(0, Vec::len) as u32;
                    if count == 0 {
                        continue;
                    }
                    let start = *self.frame_scratch.model_starts.get(&handle).unwrap_or(&0);
                    let inst_slice = inst_buf.slice((start * INSTANCE_SIZE) as u64..);
                    if model.submeshes.is_empty() {
                        cascade_pass.draw_indexed(
                            model.vertex_buffer.slice(..),
                            model.index_buffer.slice(..),
                            inst_slice,
                            0..model.num_indices,
                            0..count,
                        );
                    } else {
                        for submesh in &model.submeshes {
                            if submesh.alpha_mode == SubmeshAlphaMode::Opaque {
                                cascade_pass.draw_indexed(
                                    model.vertex_buffer.slice(..),
                                    model.index_buffer.slice(..),
                                    inst_slice,
                                    submesh.start_index
                                        ..(submesh.start_index + submesh.index_count),
                                    0..count,
                                );
                            }
                        }
                    }
                }

                let has_any_cutout = asset_manager.models.iter().any(|(handle, model)| {
                    model_instance_data
                        .get(&handle)
                        .is_some_and(|v| !v.is_empty())
                        && model
                            .submeshes
                            .iter()
                            .any(|s| s.alpha_mode == SubmeshAlphaMode::Mask)
                });
                if !has_any_cutout {
                    return;
                }

                cascade_pass.begin_cutout_stage();
                for (handle, model) in asset_manager.models.iter() {
                    let Some(insts) = model_instance_data.get(&handle) else {
                        continue;
                    };
                    if insts.is_empty()
                        || !model
                            .submeshes
                            .iter()
                            .any(|s| s.alpha_mode == SubmeshAlphaMode::Mask)
                    {
                        continue;
                    }
                    let start_offset = *self.frame_scratch.model_starts.get(&handle).unwrap_or(&0);
                    let mut cur = 0;
                    while cur < insts.len() {
                        let start = cur;
                        let override_tex = insts[cur].texture;
                        while cur < insts.len() && insts[cur].texture == override_tex {
                            cur += 1;
                        }
                        let count = (cur - start) as u32;
                        let inst_slice =
                            inst_buf.slice(((start_offset + start) * INSTANCE_SIZE) as u64..);

                        for submesh in &model.submeshes {
                            if submesh.alpha_mode == SubmeshAlphaMode::Mask {
                                let (_, bg) = resolve_submesh_texture(
                                    model,
                                    submesh,
                                    override_tex,
                                    asset_manager,
                                    &self.default_white_texture.bind_group,
                                );
                                cascade_pass.set_cutout_texture(bg);
                                cascade_pass.draw_indexed(
                                    model.vertex_buffer.slice(..),
                                    model.index_buffer.slice(..),
                                    inst_slice,
                                    submesh.start_index
                                        ..(submesh.start_index + submesh.index_count),
                                    0..count,
                                );
                            }
                        }
                    }
                }
            },
        );
    }
}
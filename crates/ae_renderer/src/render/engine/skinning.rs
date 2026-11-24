// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::state::RenderState;

impl RenderState {
    /// Evaluates vertex skinning transforms and updates the WGPU vertex buffer in real-time.
    ///
    /// For each vertex, calculates `position = sum(skin_matrix_i * weight_i * bind_position)`.
    pub fn update_model_skinning(
        &self,
        assets: &mut crate::asset::AssetManager,
        handle: crate::asset::AssetHandle,
        palette: &ae_animation::SkinningPalette,
    ) {
        if palette.is_empty() {
            return;
        }
        if let Some(model) = assets.models.get_mut(handle) {
            if model.raw_skin_vertices.is_empty() {
                return;
            }

            ae_mesh::apply_skinning_palette(
                &model.raw_skin_vertices,
                palette,
                &mut model.gpu_vertices,
            );

            self.queue.write_buffer(
                &model.vertex_buffer,
                0,
                bytemuck::cast_slice(&model.gpu_vertices),
            );
        }
    }
}
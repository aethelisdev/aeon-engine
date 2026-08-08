// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::state::RenderState;

impl RenderState {
    /// Evaluates vertex skinning transforms and updates the WGPU vertex buffer in real-time.
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

            for (i, skin_v) in model.raw_skin_vertices.iter().enumerate() {
                let j0 = skin_v.joint_indices[0] as usize;
                let j1 = skin_v.joint_indices[1] as usize;
                let j2 = skin_v.joint_indices[2] as usize;
                let j3 = skin_v.joint_indices[3] as usize;

                let m0 = palette
                    .matrices
                    .get(j0)
                    .map_or(glam::Mat4::IDENTITY, |m| m.to_mat4());
                let m1 = palette
                    .matrices
                    .get(j1)
                    .map_or(glam::Mat4::IDENTITY, |m| m.to_mat4());
                let m2 = palette
                    .matrices
                    .get(j2)
                    .map_or(glam::Mat4::IDENTITY, |m| m.to_mat4());
                let m3 = palette
                    .matrices
                    .get(j3)
                    .map_or(glam::Mat4::IDENTITY, |m| m.to_mat4());

                let w0 = skin_v.joint_weights[0];
                let w1 = skin_v.joint_weights[1];
                let w2 = skin_v.joint_weights[2];
                let w3 = skin_v.joint_weights[3];

                let total_w = w0 + w1 + w2 + w3;
                if total_w < 0.001 {
                    continue;
                }

                let skin_mat = m0 * w0 + m1 * w1 + m2 * w2 + m3 * w3;

                let pos = skin_mat.transform_point3(glam::Vec3::from_array(skin_v.bind_position));
                let norm = skin_mat.transform_vector3(glam::Vec3::from_array(skin_v.bind_normal));

                if i < model.gpu_vertices.len() {
                    model.gpu_vertices[i].position = pos.to_array();
                    model.gpu_vertices[i].normal = norm.to_array();
                }
            }

            self.queue.write_buffer(
                &model.vertex_buffer,
                0,
                bytemuck::cast_slice(&model.gpu_vertices),
            );
        }
    }
}
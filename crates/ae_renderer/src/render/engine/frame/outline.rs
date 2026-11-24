// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Adapter bridging selected ECS primitives and 3D models into [`ae_post_process::SelectionOutlinePass`].

use crate::render::engine::state::RenderState;
use crate::render::types::Instance;

impl RenderState {
    /// Records selected primitives and 3D models into the selection outline mask and composite passes.
    pub fn execute_selection_outline_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
        selected_prims: &[(ae_plugin_api::Shape, Instance)],
        selected_models: &[(crate::asset::AssetHandle, Instance)],
        asset_manager: &crate::asset::AssetManager,
    ) {
        let has_selection = !selected_prims.is_empty() || !selected_models.is_empty();
        self.outline.execute_pass(
            &self.device,
            encoder,
            target_view,
            &self.uniforms.camera_bind_group,
            has_selection,
            |recorder| {
                for (shape, inst) in selected_prims {
                    let inst_bytes = bytemuck::bytes_of(inst);
                    let (vb, vcount) = match shape {
                        ae_plugin_api::Shape::Cube => (&self.geometry.cube_vertex_buffer, 36),
                        ae_plugin_api::Shape::Sphere => (
                            &self.geometry.sphere_vertex_buffer,
                            self.geometry.sphere_num_vertices,
                        ),
                        ae_plugin_api::Shape::Cylinder => (
                            &self.geometry.cylinder_vertex_buffer,
                            self.geometry.cylinder_num_vertices,
                        ),
                        ae_plugin_api::Shape::Capsule => (
                            &self.geometry.capsule_vertex_buffer,
                            self.geometry.capsule_num_vertices,
                        ),
                        ae_plugin_api::Shape::Torus => (
                            &self.geometry.torus_vertex_buffer,
                            self.geometry.torus_num_vertices,
                        ),
                        ae_plugin_api::Shape::Triangle => (
                            &self.geometry.vertex_buffer,
                            self.geometry.triangle_num_vertices,
                        ),
                    };
                    recorder.draw_vertices(vb, vcount, inst_bytes);
                }

                for (m_handle, inst) in selected_models {
                    if let Some(m) = asset_manager.models.get(*m_handle) {
                        recorder.draw_indexed(
                            &m.vertex_buffer,
                            &m.index_buffer,
                            m.num_indices,
                            bytemuck::bytes_of(inst),
                        );
                    }
                }
            },
        );
    }
}
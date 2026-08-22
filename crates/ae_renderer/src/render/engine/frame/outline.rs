// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Screen-space silhouette outline render pass for selected entities.

use crate::render::engine::state::RenderState;
use crate::render::types::Instance;
use wgpu::util::DeviceExt;

impl RenderState {
    /// Executes the screen-space selection mask and outline composite pass.
    pub fn execute_selection_outline_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        selected_prims: &[(ae_core::ecs::Shape, Instance)],
        selected_models: &[(crate::asset::AssetHandle, Instance)],
        asset_manager: &crate::asset::AssetManager,
    ) {
        if let (Some(mask_view), Some(depth_view), Some(composite_bg)) = (
            self.outline.mask_view.as_ref(),
            self.outline.mask_depth_view.as_ref(),
            self.outline.composite_bind_group.as_ref(),
        ) && (!selected_prims.is_empty() || !selected_models.is_empty())
        {
            {
                let mut mask_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Selection Mask Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: mask_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                mask_pass.set_pipeline(&self.outline.mask_pipeline);
                mask_pass.set_bind_group(0, &self.uniforms.camera_bind_group, &[]);

                for (shape, inst) in selected_prims {
                    let single_inst_buf =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Selected Single Instance Buffer"),
                                contents: bytemuck::cast_slice(&[*inst]),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                    mask_pass.set_vertex_buffer(1, single_inst_buf.slice(..));

                    match shape {
                        ae_core::ecs::Shape::Cube => {
                            mask_pass
                                .set_vertex_buffer(0, self.geometry.cube_vertex_buffer.slice(..));
                            mask_pass.draw(0..36, 0..1);
                        }
                        ae_core::ecs::Shape::Sphere => {
                            mask_pass
                                .set_vertex_buffer(0, self.geometry.sphere_vertex_buffer.slice(..));
                            mask_pass.draw(0..self.geometry.sphere_num_vertices, 0..1);
                        }
                        ae_core::ecs::Shape::Cylinder => {
                            mask_pass.set_vertex_buffer(
                                0,
                                self.geometry.cylinder_vertex_buffer.slice(..),
                            );
                            mask_pass.draw(0..self.geometry.cylinder_num_vertices, 0..1);
                        }
                        ae_core::ecs::Shape::Capsule => {
                            mask_pass.set_vertex_buffer(
                                0,
                                self.geometry.capsule_vertex_buffer.slice(..),
                            );
                            mask_pass.draw(0..self.geometry.capsule_num_vertices, 0..1);
                        }
                        ae_core::ecs::Shape::Torus => {
                            mask_pass
                                .set_vertex_buffer(0, self.geometry.torus_vertex_buffer.slice(..));
                            mask_pass.draw(0..self.geometry.torus_num_vertices, 0..1);
                        }
                        ae_core::ecs::Shape::Triangle => {
                            mask_pass.set_vertex_buffer(0, self.geometry.vertex_buffer.slice(..));
                            mask_pass.draw(0..self.geometry.triangle_num_vertices, 0..1);
                        }
                    }
                }

                for (m_handle, inst) in selected_models {
                    if let Some(m) = asset_manager.models.get(*m_handle) {
                        let single_inst_buf =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Selected Model Instance Buffer"),
                                    contents: bytemuck::cast_slice(&[*inst]),
                                    usage: wgpu::BufferUsages::VERTEX,
                                });
                        mask_pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        mask_pass.set_vertex_buffer(1, single_inst_buf.slice(..));
                        mask_pass
                            .set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        mask_pass.draw_indexed(0..m.num_indices, 0, 0..1);
                    }
                }
            }

            {
                let target_view = self
                    .viewport_texture
                    .as_ref()
                    .map(|vt| &vt.view)
                    .unwrap_or(surface_view);

                let mut composite_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Outline Composite Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: target_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                composite_pass.set_pipeline(&self.outline.composite_pipeline);
                composite_pass.set_bind_group(0, composite_bg, &[]);
                composite_pass.draw(0..3, 0..1);
            }
        }
    }
}
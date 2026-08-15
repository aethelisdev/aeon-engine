// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Forward scene rendering pass for sky, grid, primitive meshes, 3D models, and sprites.

use crate::render::engine::state::{RenderOptions, RenderState};
use crate::render::types::{INSTANCE_SIZE, Instance, OverlayRenderer};

/// Context parameters required for executing the forward geometry and lighting pass.
pub struct ForwardPassContext<'a> {
    pub options: &'a RenderOptions,
    pub all_instances_len: usize,
    pub triangle_instances: &'a [(Instance, Option<crate::asset::AssetHandle>)],
    pub tri_start: usize,
    pub cube_instances: &'a [(Instance, Option<crate::asset::AssetHandle>)],
    pub cube_start: usize,
    pub sphere_instances: &'a [(Instance, Option<crate::asset::AssetHandle>)],
    pub sphere_start: usize,
    pub cylinder_instances: &'a [(Instance, Option<crate::asset::AssetHandle>)],
    pub cylinder_start: usize,
    pub capsule_instances: &'a [(Instance, Option<crate::asset::AssetHandle>)],
    pub capsule_start: usize,
    pub torus_instances: &'a [(Instance, Option<crate::asset::AssetHandle>)],
    pub torus_start: usize,
    pub transparent_objs: &'a [(f32, crate::asset::AssetHandle, Instance)],
    pub sprite_start: usize,
    pub model_instance_data: &'a std::collections::HashMap<
        crate::asset::AssetHandle,
        Vec<(Instance, Option<crate::asset::AssetHandle>)>,
    >,
    pub model_starts: &'a std::collections::HashMap<crate::asset::AssetHandle, usize>,
    pub overlays: &'a [&'a dyn OverlayRenderer],
    pub asset_manager: &'a crate::asset::AssetManager,
}

impl RenderState {
    /// Executes the main forward render pass:
    /// Skybox → Grid → Opaque Primitives & Models → Cutout Models → Wireframe → Transparent Models → Sprites → Overlays.
    pub fn execute_main_forward_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: ForwardPassContext<'_>,
    ) {
        let (color_view, resolve_target) = if self.post_process.msaa_samples > 1 {
            (
                &self.post_process.multisampled_framebuffer,
                Some(&self.post_process.scene_texture_view),
            )
        } else {
            (&self.post_process.scene_texture_view, None)
        };

        let env_color = self.graphics_settings.environment_color;
        let clear_rgb = wgpu::Color {
            r: (env_color[0] as f64).powf(2.2),
            g: (env_color[1] as f64).powf(2.2),
            b: (env_color[2] as f64).powf(2.2),
            a: 1.0,
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_rgb),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.post_process.depth_texture_view,
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

        pass.set_bind_group(0, &self.uniforms.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.uniforms.light_bind_group, &[]);
        pass.set_bind_group(2, &self.shadow.shadow_bind_group, &[]);

        // --- SKY ---
        pass.set_pipeline(&self.pipelines.sky_pipeline);
        pass.set_bind_group(1, &self.uniforms.sky_bind_group, &[]);
        pass.draw(0..3, 0..1);

        // Restore light group for opaque geometry
        pass.set_bind_group(1, &self.uniforms.light_bind_group, &[]);

        // --- GRID ---
        if ctx.options.grid_enabled {
            pass.set_pipeline(&self.pipelines.grid_pipeline);
            pass.set_vertex_buffer(0, self.geometry.grid_vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }

        if ctx.all_instances_len > 0 {
            pass.set_pipeline(&self.pipelines.render_pipeline);

            let draw_primitive_batch =
                |pass: &mut wgpu::RenderPass,
                 instances: &[(Instance, Option<crate::asset::AssetHandle>)],
                 buf_start: usize,
                 v_buf: &wgpu::Buffer,
                 num_verts: u32| {
                    let mut cur = 0;
                    while cur < instances.len() {
                        let start = cur;
                        let tex_h = instances[cur].1;
                        while cur < instances.len() && instances[cur].1 == tex_h {
                            cur += 1;
                        }
                        let bg = tex_h
                            .and_then(|h| ctx.asset_manager.textures.get(h))
                            .map(|t| &t.bind_group)
                            .unwrap_or(&self.default_white_texture.bind_group);
                        pass.set_bind_group(3, bg, &[]);
                        pass.set_vertex_buffer(0, v_buf.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry
                                .instance_buffer
                                .slice(((buf_start + start) * INSTANCE_SIZE) as u64..),
                        );
                        pass.draw(0..num_verts, 0..(cur - start) as u32);
                    }
                };

            if !ctx.triangle_instances.is_empty() {
                draw_primitive_batch(
                    &mut pass,
                    ctx.triangle_instances,
                    ctx.tri_start,
                    &self.geometry.vertex_buffer,
                    3,
                );
            }
            if !ctx.cube_instances.is_empty() {
                draw_primitive_batch(
                    &mut pass,
                    ctx.cube_instances,
                    ctx.cube_start,
                    &self.geometry.cube_vertex_buffer,
                    36,
                );
            }
            if !ctx.sphere_instances.is_empty() {
                draw_primitive_batch(
                    &mut pass,
                    ctx.sphere_instances,
                    ctx.sphere_start,
                    &self.geometry.sphere_vertex_buffer,
                    self.geometry.sphere_num_vertices,
                );
            }
            if !ctx.cylinder_instances.is_empty() {
                draw_primitive_batch(
                    &mut pass,
                    ctx.cylinder_instances,
                    ctx.cylinder_start,
                    &self.geometry.cylinder_vertex_buffer,
                    self.geometry.cylinder_num_vertices,
                );
            }
            if !ctx.capsule_instances.is_empty() {
                draw_primitive_batch(
                    &mut pass,
                    ctx.capsule_instances,
                    ctx.capsule_start,
                    &self.geometry.capsule_vertex_buffer,
                    self.geometry.capsule_num_vertices,
                );
            }
            if !ctx.torus_instances.is_empty() {
                draw_primitive_batch(
                    &mut pass,
                    ctx.torus_instances,
                    ctx.torus_start,
                    &self.geometry.torus_vertex_buffer,
                    self.geometry.torus_num_vertices,
                );
            }

            // --- PASS 1a: OPAQUE SUBMESHES ---
            for (handle, m) in ctx.asset_manager.models.iter() {
                if let Some(insts) = ctx.model_instance_data.get(&handle) {
                    if !insts.is_empty() {
                        let start_offset = *ctx.model_starts.get(&handle).unwrap_or(&0);
                        let mut cur = 0;
                        while cur < insts.len() {
                            let start = cur;
                            let override_tex = insts[cur].1;
                            while cur < insts.len() && insts[cur].1 == override_tex {
                                cur += 1;
                            }
                            let count = (cur - start) as u32;

                            pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                            pass.set_vertex_buffer(
                                1,
                                self.geometry
                                    .instance_buffer
                                    .slice(((start_offset + start) * INSTANCE_SIZE) as u64..),
                            );
                            pass.set_index_buffer(
                                m.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );

                            if m.submeshes.is_empty() {
                                let bg = override_tex
                                    .and_then(|h| ctx.asset_manager.textures.get(h))
                                    .map(|t| &t.bind_group)
                                    .unwrap_or(&self.default_white_texture.bind_group);

                                pass.set_bind_group(3, bg, &[]);
                                pass.draw_indexed(0..m.num_indices, 0, 0..count);
                            } else {
                                for submesh in &m.submeshes {
                                    if submesh.alpha_mode
                                        != crate::render::types::SubmeshAlphaMode::Opaque
                                    {
                                        continue;
                                    }
                                    let bg = if let Some(custom_tex) = override_tex {
                                        if let Some(tex_idx) = submesh.texture_index {
                                            if tex_idx > 0 && m.embedded_textures.len() > 1 {
                                                m.embedded_textures
                                                    .get(tex_idx)
                                                    .and_then(|&h| {
                                                        ctx.asset_manager.textures.get(h)
                                                    })
                                                    .map(|t| &t.bind_group)
                                                    .or_else(|| {
                                                        ctx.asset_manager
                                                            .textures
                                                            .get(custom_tex)
                                                            .map(|t| &t.bind_group)
                                                    })
                                            } else if Some(custom_tex) == m.default_texture {
                                                m.embedded_textures
                                                    .get(tex_idx)
                                                    .and_then(|&h| {
                                                        ctx.asset_manager.textures.get(h)
                                                    })
                                                    .map(|t| &t.bind_group)
                                            } else {
                                                ctx.asset_manager
                                                    .textures
                                                    .get(custom_tex)
                                                    .map(|t| &t.bind_group)
                                            }
                                        } else {
                                            ctx.asset_manager
                                                .textures
                                                .get(custom_tex)
                                                .map(|t| &t.bind_group)
                                        }
                                    } else {
                                        None
                                    }
                                    .unwrap_or(&self.default_white_texture.bind_group);

                                    pass.set_bind_group(3, bg, &[]);
                                    pass.draw_indexed(
                                        submesh.start_index
                                            ..(submesh.start_index + submesh.index_count),
                                        0,
                                        0..count,
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // --- PASS 1b: CUTOUT / MASK SUBMESHES (ALPHA TESTING WITH DEPTH WRITE) ---
            let mut has_any_cutout_submesh = false;
            for (handle, m) in ctx.asset_manager.models.iter() {
                if let Some(insts) = ctx.model_instance_data.get(&handle) {
                    if !insts.is_empty()
                        && m.submeshes
                            .iter()
                            .any(|s| s.alpha_mode == crate::render::types::SubmeshAlphaMode::Mask)
                    {
                        has_any_cutout_submesh = true;
                        break;
                    }
                }
            }

            if has_any_cutout_submesh {
                pass.set_pipeline(&self.pipelines.cutout_pipeline);
                for (handle, m) in ctx.asset_manager.models.iter() {
                    if let Some(insts) = ctx.model_instance_data.get(&handle) {
                        if !insts.is_empty()
                            && m.submeshes.iter().any(|s| {
                                s.alpha_mode == crate::render::types::SubmeshAlphaMode::Mask
                            })
                        {
                            let start_offset = *ctx.model_starts.get(&handle).unwrap_or(&0);
                            let mut cur = 0;
                            while cur < insts.len() {
                                let start = cur;
                                let override_tex = insts[cur].1;
                                while cur < insts.len() && insts[cur].1 == override_tex {
                                    cur += 1;
                                }
                                let count = (cur - start) as u32;

                                pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                                pass.set_vertex_buffer(
                                    1,
                                    self.geometry
                                        .instance_buffer
                                        .slice(((start_offset + start) * INSTANCE_SIZE) as u64..),
                                );
                                pass.set_index_buffer(
                                    m.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );

                                for submesh in &m.submeshes {
                                    if submesh.alpha_mode
                                        != crate::render::types::SubmeshAlphaMode::Mask
                                    {
                                        continue;
                                    }
                                    let bg = if let Some(custom_tex) = override_tex {
                                        if let Some(tex_idx) = submesh.texture_index {
                                            if tex_idx > 0 && m.embedded_textures.len() > 1 {
                                                m.embedded_textures
                                                    .get(tex_idx)
                                                    .and_then(|&h| {
                                                        ctx.asset_manager.textures.get(h)
                                                    })
                                                    .map(|t| &t.bind_group)
                                                    .or_else(|| {
                                                        ctx.asset_manager
                                                            .textures
                                                            .get(custom_tex)
                                                            .map(|t| &t.bind_group)
                                                    })
                                            } else if Some(custom_tex) == m.default_texture {
                                                m.embedded_textures
                                                    .get(tex_idx)
                                                    .and_then(|&h| {
                                                        ctx.asset_manager.textures.get(h)
                                                    })
                                                    .map(|t| &t.bind_group)
                                            } else {
                                                ctx.asset_manager
                                                    .textures
                                                    .get(custom_tex)
                                                    .map(|t| &t.bind_group)
                                            }
                                        } else {
                                            ctx.asset_manager
                                                .textures
                                                .get(custom_tex)
                                                .map(|t| &t.bind_group)
                                        }
                                    } else {
                                        None
                                    }
                                    .unwrap_or(&self.default_white_texture.bind_group);

                                    pass.set_bind_group(3, bg, &[]);
                                    pass.draw_indexed(
                                        submesh.start_index
                                            ..(submesh.start_index + submesh.index_count),
                                        0,
                                        0..count,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if ctx.options.wireframe_enabled {
            pass.set_pipeline(&self.pipelines.wireframe_pipeline);
            pass.set_bind_group(3, &self.default_white_texture.bind_group, &[]);
            if !ctx.cube_instances.is_empty() {
                pass.set_vertex_buffer(0, self.geometry.cube_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    self.geometry
                        .instance_buffer
                        .slice((ctx.cube_start * INSTANCE_SIZE) as u64..),
                );
                pass.draw(0..36, 0..ctx.cube_instances.len() as u32);
            }
            if !ctx.sphere_instances.is_empty() {
                pass.set_vertex_buffer(0, self.geometry.sphere_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    self.geometry
                        .instance_buffer
                        .slice((ctx.sphere_start * INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..self.geometry.sphere_num_vertices,
                    0..ctx.sphere_instances.len() as u32,
                );
            }
            if !ctx.cylinder_instances.is_empty() {
                pass.set_vertex_buffer(0, self.geometry.cylinder_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    self.geometry
                        .instance_buffer
                        .slice((ctx.cylinder_start * INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..self.geometry.cylinder_num_vertices,
                    0..ctx.cylinder_instances.len() as u32,
                );
            }
            if !ctx.capsule_instances.is_empty() {
                pass.set_vertex_buffer(0, self.geometry.capsule_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    self.geometry
                        .instance_buffer
                        .slice((ctx.capsule_start * INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..self.geometry.capsule_num_vertices,
                    0..ctx.capsule_instances.len() as u32,
                );
            }
            if !ctx.torus_instances.is_empty() {
                pass.set_vertex_buffer(0, self.geometry.torus_vertex_buffer.slice(..));
                pass.set_vertex_buffer(
                    1,
                    self.geometry
                        .instance_buffer
                        .slice((ctx.torus_start * INSTANCE_SIZE) as u64..),
                );
                pass.draw(
                    0..self.geometry.torus_num_vertices,
                    0..ctx.torus_instances.len() as u32,
                );
            }
            for (handle, m) in ctx.asset_manager.models.iter() {
                let c = ctx
                    .model_instance_data
                    .get(&handle)
                    .map(|v| v.len())
                    .unwrap_or(0) as u32;
                if c > 0 {
                    pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry.instance_buffer.slice(
                            ((*ctx.model_starts.get(&handle).unwrap_or(&0)) * INSTANCE_SIZE)
                                as u64..,
                        ),
                    );
                    pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    pass.draw_indexed(0..m.num_indices, 0, 0..c);
                }
            }
        }

        // --- PASS 2: TRANSPARENT SUBMESHES (ALPHA BLENDING) ---
        let mut has_any_blend_submesh = false;
        for (handle, m) in ctx.asset_manager.models.iter() {
            if let Some(insts) = ctx.model_instance_data.get(&handle) {
                if !insts.is_empty()
                    && m.submeshes
                        .iter()
                        .any(|s| s.alpha_mode == crate::render::types::SubmeshAlphaMode::Blend)
                {
                    has_any_blend_submesh = true;
                    break;
                }
            }
        }

        if has_any_blend_submesh {
            pass.set_pipeline(&self.pipelines.transparent_pipeline);
            for (handle, m) in ctx.asset_manager.models.iter() {
                if let Some(insts) = ctx.model_instance_data.get(&handle) {
                    if !insts.is_empty()
                        && m.submeshes
                            .iter()
                            .any(|s| s.alpha_mode == crate::render::types::SubmeshAlphaMode::Blend)
                    {
                        let start_offset = *ctx.model_starts.get(&handle).unwrap_or(&0);
                        let mut cur = 0;
                        while cur < insts.len() {
                            let start = cur;
                            let override_tex = insts[cur].1;
                            while cur < insts.len() && insts[cur].1 == override_tex {
                                cur += 1;
                            }
                            let count = (cur - start) as u32;

                            pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                            pass.set_vertex_buffer(
                                1,
                                self.geometry
                                    .instance_buffer
                                    .slice(((start_offset + start) * INSTANCE_SIZE) as u64..),
                            );
                            pass.set_index_buffer(
                                m.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );

                            for submesh in &m.submeshes {
                                if submesh.alpha_mode
                                    != crate::render::types::SubmeshAlphaMode::Blend
                                {
                                    continue;
                                }
                                let bg = if let Some(custom_tex) = override_tex {
                                    if let Some(tex_idx) = submesh.texture_index {
                                        if tex_idx > 0 && m.embedded_textures.len() > 1 {
                                            m.embedded_textures
                                                .get(tex_idx)
                                                .and_then(|&h| ctx.asset_manager.textures.get(h))
                                                .map(|t| &t.bind_group)
                                                .or_else(|| {
                                                    ctx.asset_manager
                                                        .textures
                                                        .get(custom_tex)
                                                        .map(|t| &t.bind_group)
                                                })
                                        } else if Some(custom_tex) == m.default_texture {
                                            m.embedded_textures
                                                .get(tex_idx)
                                                .and_then(|&h| ctx.asset_manager.textures.get(h))
                                                .map(|t| &t.bind_group)
                                        } else {
                                            ctx.asset_manager
                                                .textures
                                                .get(custom_tex)
                                                .map(|t| &t.bind_group)
                                        }
                                    } else {
                                        ctx.asset_manager
                                            .textures
                                            .get(custom_tex)
                                            .map(|t| &t.bind_group)
                                    }
                                } else {
                                    None
                                }
                                .unwrap_or(&self.default_white_texture.bind_group);

                                pass.set_bind_group(3, bg, &[]);
                                pass.draw_indexed(
                                    submesh.start_index
                                        ..(submesh.start_index + submesh.index_count),
                                    0,
                                    0..count,
                                );
                            }
                        }
                    }
                }
            }
        }

        if !ctx.transparent_objs.is_empty() {
            pass.set_pipeline(&self.pipelines.sprite_pipeline);
            pass.set_bind_group(2, &self.uniforms.light_bind_group, &[]);
            pass.set_vertex_buffer(0, self.geometry.quad_vertex_buffer.slice(..));
            let mut cur = 0;
            while cur < ctx.transparent_objs.len() {
                let start = cur;
                let tid = ctx.transparent_objs[cur].1;
                while cur < ctx.transparent_objs.len() && ctx.transparent_objs[cur].1 == tid {
                    cur += 1;
                }
                if let Some(t) = ctx.asset_manager.textures.get(tid) {
                    pass.set_bind_group(1, &t.bind_group, &[]);
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice(((ctx.sprite_start + start) * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(0..6, 0..(cur - start) as u32);
                }
            }
        }

        // Editor Overlays (gizmo, debug lines, etc.)
        for ov in ctx.overlays {
            ov.draw_overlay(&self.queue, &mut pass);
        }
    }
}
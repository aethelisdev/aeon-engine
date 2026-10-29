// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Forward scene rendering pass for skybox, ground grid, primitives, 3D models, sprites, and editor overlays.
//!
//! Employs contiguous batch iteration, precomputed matrix reflection parity, and redundant state command elimination
//! to minimize CPU-side submission overhead and GPU state thrashing.

use crate::render::engine::state::{RenderOptions, RenderState};
use crate::render::types::{
    FrameRenderStats, INSTANCE_SIZE, Instance, ModelAsset, ModelSubmesh, OverlayRenderer,
    RenderInstanceItem, SubmeshAlphaMode,
};

/// Prepared batch of model instances referencing cached GPU buffers and precomputed offsets.
#[derive(Copy, Clone, Debug, Default)]
pub struct PreparedModelBatch {
    /// Unique identifier of the underlying 3D model asset.
    pub handle: crate::asset::AssetHandle,
    /// Starting index within the contiguous flattened model instance scratch slice.
    pub instance_start: usize,
    /// Total number of contiguous model instances belonging to this model batch.
    pub instance_count: usize,
    /// Global instance offset within the GPU dynamic instance buffer.
    pub buffer_offset: usize,
}

/// Pre-sorted transparent submesh draw call element for back-to-front depth ordering.
#[derive(Copy, Clone, Debug, Default)]
pub struct TransparentSubmeshDraw {
    /// Squared Euclidean distance from camera eye to instance world translation.
    pub distance_sq: f32,
    /// Handle of the resident 3D model asset.
    pub handle: crate::asset::AssetHandle,
    /// Index of the blended submesh within `model.submeshes`.
    pub submesh_index: usize,
    /// Base instance offset of the parent batch inside the GPU instance buffer.
    pub buffer_offset: usize,
    /// Local index of the instance item within the parent batch.
    pub instance_index: usize,
    /// Optional texture override specified on the entity instance.
    pub override_tex: Option<crate::asset::AssetHandle>,
}

/// Context parameters required for executing the forward geometry and lighting pass.
pub struct ForwardPassContext<'a> {
    /// Viewport visual options (grid, wireframe, scale factor).
    pub options: &'a RenderOptions,
    /// Total number of valid instances staged in the global GPU instance buffer.
    pub all_instances_len: usize,
    /// Contiguous primitive triangle instances.
    pub triangle_instances: &'a [RenderInstanceItem],
    /// Global instance buffer start offset for triangle primitives.
    pub tri_start: usize,
    /// Contiguous primitive cube instances.
    pub cube_instances: &'a [RenderInstanceItem],
    /// Global instance buffer start offset for cube primitives.
    pub cube_start: usize,
    /// Contiguous primitive sphere instances.
    pub sphere_instances: &'a [RenderInstanceItem],
    /// Global instance buffer start offset for sphere primitives.
    pub sphere_start: usize,
    /// Contiguous primitive cylinder instances.
    pub cylinder_instances: &'a [RenderInstanceItem],
    /// Global instance buffer start offset for cylinder primitives.
    pub cylinder_start: usize,
    /// Contiguous primitive capsule instances.
    pub capsule_instances: &'a [RenderInstanceItem],
    /// Global instance buffer start offset for capsule primitives.
    pub capsule_start: usize,
    /// Contiguous primitive torus instances.
    pub torus_instances: &'a [RenderInstanceItem],
    /// Global instance buffer start offset for torus primitives.
    pub torus_start: usize,
    /// Back-to-front sorted billboard sprite instances.
    pub transparent_objs: &'a [(f32, crate::asset::AssetHandle, Instance)],
    /// Global instance buffer start offset for billboard sprites.
    pub sprite_start: usize,
    /// Contiguous slice of visible model batches active in the current frame.
    pub model_batches: &'a [PreparedModelBatch],
    /// Flattened slice of model instance items for texture and reflection indexing.
    pub model_instances: &'a [RenderInstanceItem],
    /// Pre-sorted transparent model submesh draws ordered back-to-front by camera distance.
    pub transparent_model_draws: &'a [TransparentSubmeshDraw],
    /// Active viewport overlays (gizmos, bounds, debug visuals).
    pub overlays: &'a [&'a dyn OverlayRenderer],
    /// Reference to the central asset manager for texture and mesh resolution.
    pub asset_manager: &'a crate::asset::AssetManager,
}

/// Active pipeline state discriminator for preventing redundant pipeline bindings.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ActivePipeline {
    Sky,
    Grid,
    RenderCcw,
    RenderCw,
    CutoutCcw,
    CutoutCw,
    Wireframe,
    Transparent,
    Sprite,
}

/// Tracks bound texture keys to eliminate redundant `set_bind_group(3, ...)` invocations.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum BoundTextureKey {
    DefaultWhite,
    Asset(crate::asset::AssetHandle),
}

/// Tracks currently bound GPU render pass state to eliminate redundant driver commands.
#[derive(Default)]
struct RenderPassTracker {
    pipeline: Option<ActivePipeline>,
    bound_texture: Option<BoundTextureKey>,
    bound_model: Option<crate::asset::AssetHandle>,
}

impl RenderPassTracker {
    #[inline]
    fn set_pipeline<'a>(
        &mut self,
        pass: &mut wgpu::RenderPass<'a>,
        pipeline_kind: ActivePipeline,
        pipeline: &'a wgpu::RenderPipeline,
    ) {
        if self.pipeline != Some(pipeline_kind) {
            pass.set_pipeline(pipeline);
            self.pipeline = Some(pipeline_kind);
        }
    }

    #[inline]
    fn bind_texture<'a>(
        &mut self,
        pass: &mut wgpu::RenderPass<'a>,
        key: BoundTextureKey,
        bind_group: &'a wgpu::BindGroup,
    ) {
        if self.bound_texture != Some(key) {
            pass.set_bind_group(3, bind_group, &[]);
            self.bound_texture = Some(key);
        }
    }

    #[inline]
    fn bind_model_buffers<'a>(
        &mut self,
        pass: &mut wgpu::RenderPass<'a>,
        handle: crate::asset::AssetHandle,
        model: &'a ModelAsset,
    ) {
        if self.bound_model != Some(handle) {
            pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.bound_model = Some(handle);
        }
    }

    #[inline]
    fn reset_model_buffers(&mut self) {
        self.bound_model = None;
    }
}

/// Resolves the texture handle and associated GPU bind group for a model submesh.
#[inline]
fn resolve_submesh_texture<'a>(
    model: &'a ModelAsset,
    submesh: &ModelSubmesh,
    override_tex: Option<crate::asset::AssetHandle>,
    asset_manager: &'a crate::asset::AssetManager,
    default_white: &'a wgpu::BindGroup,
) -> (BoundTextureKey, &'a wgpu::BindGroup) {
    let resolved_handle = match override_tex {
        Some(custom_tex) => {
            if let Some(tex_idx) = submesh.texture_index {
                if tex_idx > 0 && model.embedded_textures.len() > 1 {
                    model
                        .embedded_textures
                        .get(tex_idx)
                        .copied()
                        .filter(|h| asset_manager.textures.get(*h).is_some())
                        .or(Some(custom_tex))
                } else if Some(custom_tex) == model.default_texture {
                    model
                        .embedded_textures
                        .get(tex_idx)
                        .copied()
                        .or(Some(custom_tex))
                } else {
                    Some(custom_tex)
                }
            } else {
                Some(custom_tex)
            }
        }
        None => {
            if let Some(tex_idx) = submesh.texture_index {
                model
                    .embedded_textures
                    .get(tex_idx)
                    .copied()
                    .filter(|h| asset_manager.textures.get(*h).is_some())
                    .or(model.default_texture)
            } else {
                model.default_texture
            }
        }
    };

    if let Some(handle) = resolved_handle
        && let Some(tex) = asset_manager.textures.get(handle)
    {
        return (BoundTextureKey::Asset(handle), &tex.bind_group);
    }

    (BoundTextureKey::DefaultWhite, default_white)
}

/// Configuration descriptor for batching model submeshes by alpha blending mode.
struct ModelBatchPassConfig<'a> {
    batches: &'a [PreparedModelBatch],
    target_mode: SubmeshAlphaMode,
    is_cutout: bool,
}

impl RenderState {
    /// Executes the main forward render pass:
    /// Skybox → Grid → Opaque Primitives & Models → Cutout Models → Wireframe → Transparent Models → Sprites → Overlays.
    pub fn execute_main_forward_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        ctx: ForwardPassContext<'_>,
    ) -> FrameRenderStats {
        let mut stats = FrameRenderStats::default();

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
            label: Some("Main Forward Pass"),
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

        let mut tracker = RenderPassTracker::default();

        // --- SKY ---
        tracker.set_pipeline(&mut pass, ActivePipeline::Sky, &self.pipelines.sky_pipeline);
        pass.set_bind_group(1, &self.uniforms.sky_bind_group, &[]);
        pass.draw(0..3, 0..1);

        // Restore light group for geometry passes
        pass.set_bind_group(1, &self.uniforms.light_bind_group, &[]);

        // --- GRID ---
        if ctx.options.grid_enabled {
            tracker.set_pipeline(
                &mut pass,
                ActivePipeline::Grid,
                &self.pipelines.grid_pipeline,
            );
            pass.set_vertex_buffer(0, self.geometry.grid_vertex_buffer.slice(..));
            pass.draw(0..6, 0..1);
        }

        if ctx.all_instances_len > 0 {
            // --- PRIMITIVES (OPAQUE) ---
            self.draw_all_primitives(&mut pass, &ctx, &mut tracker, &mut stats);

            // --- PASS 1a: OPAQUE SUBMESHES ---
            self.draw_model_batches(
                &mut pass,
                ModelBatchPassConfig {
                    batches: ctx.model_batches,
                    target_mode: SubmeshAlphaMode::Opaque,
                    is_cutout: false,
                },
                &mut tracker,
                &ctx,
                &mut stats,
            );

            // --- PASS 1b: CUTOUT / MASK SUBMESHES ---
            let has_cutout = ctx.model_batches.iter().any(|b| {
                ctx.asset_manager
                    .models
                    .get(b.handle)
                    .map(|m| {
                        m.submeshes
                            .iter()
                            .any(|s| s.alpha_mode == SubmeshAlphaMode::Mask)
                    })
                    .unwrap_or(false)
            });
            if has_cutout {
                self.draw_model_batches(
                    &mut pass,
                    ModelBatchPassConfig {
                        batches: ctx.model_batches,
                        target_mode: SubmeshAlphaMode::Mask,
                        is_cutout: true,
                    },
                    &mut tracker,
                    &ctx,
                    &mut stats,
                );
            }
        }

        // --- WIREFRAME OVERLAY ---
        if ctx.options.wireframe_enabled {
            self.draw_wireframe_pass(&mut pass, &ctx, &mut tracker);
        }

        // --- PASS 2: TRANSPARENT SUBMESHES (BACK-TO-FRONT SORTED) ---
        if !ctx.transparent_model_draws.is_empty() {
            self.draw_transparent_submeshes(
                &mut pass,
                ctx.transparent_model_draws,
                &mut tracker,
                &ctx,
                &mut stats,
            );
        }

        // --- SPRITES (BACK-TO-FRONT SORTED) ---
        if !ctx.transparent_objs.is_empty() {
            self.draw_sprites(&mut pass, &ctx, &mut tracker, &mut stats);
        }

        // --- EDITOR OVERLAYS ---
        for ov in ctx.overlays {
            ov.draw_overlay(&self.queue, &mut pass);
        }

        drop(pass);
        stats
    }

    /// Draws all primitive geometry batches (triangles, cubes, spheres, cylinders, capsules, toruses).
    fn draw_all_primitives<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        ctx: &ForwardPassContext<'a>,
        tracker: &mut RenderPassTracker,
        stats: &mut FrameRenderStats,
    ) {
        tracker.reset_model_buffers();
        let primitives = [
            (
                ctx.triangle_instances,
                ctx.tri_start,
                &self.geometry.vertex_buffer,
                self.geometry.triangle_num_vertices,
            ),
            (
                ctx.cube_instances,
                ctx.cube_start,
                &self.geometry.cube_vertex_buffer,
                36,
            ),
            (
                ctx.sphere_instances,
                ctx.sphere_start,
                &self.geometry.sphere_vertex_buffer,
                self.geometry.sphere_num_vertices,
            ),
            (
                ctx.cylinder_instances,
                ctx.cylinder_start,
                &self.geometry.cylinder_vertex_buffer,
                self.geometry.cylinder_num_vertices,
            ),
            (
                ctx.capsule_instances,
                ctx.capsule_start,
                &self.geometry.capsule_vertex_buffer,
                self.geometry.capsule_num_vertices,
            ),
            (
                ctx.torus_instances,
                ctx.torus_start,
                &self.geometry.torus_vertex_buffer,
                self.geometry.torus_num_vertices,
            ),
        ];

        for (instances, buf_start, v_buf, num_verts) in primitives {
            if instances.is_empty() {
                continue;
            }
            pass.set_vertex_buffer(0, v_buf.slice(..));
            let mut cur = 0;
            while cur < instances.len() {
                let start = cur;
                let tex_h = instances[cur].texture;
                let mirrored = instances[cur].is_mirrored;
                while cur < instances.len()
                    && instances[cur].texture == tex_h
                    && instances[cur].is_mirrored == mirrored
                {
                    cur += 1;
                }
                let count = (cur - start) as u32;

                if mirrored {
                    tracker.set_pipeline(
                        pass,
                        ActivePipeline::RenderCw,
                        &self.pipelines.render_pipeline_cw,
                    );
                } else {
                    tracker.set_pipeline(
                        pass,
                        ActivePipeline::RenderCcw,
                        &self.pipelines.render_pipeline,
                    );
                }

                let (key, bg) = match tex_h.and_then(|h| ctx.asset_manager.textures.get(h)) {
                    Some(t) => (BoundTextureKey::Asset(tex_h.unwrap()), &t.bind_group),
                    None => (
                        BoundTextureKey::DefaultWhite,
                        &self.default_white_texture.bind_group,
                    ),
                };
                tracker.bind_texture(pass, key, bg);

                let offset_bytes = ((buf_start + start) * INSTANCE_SIZE) as u64;
                pass.set_vertex_buffer(1, self.geometry.instance_buffer.slice(offset_bytes..));
                pass.draw(0..num_verts, 0..count);

                stats.draw_calls += 1;
                if count > 1 {
                    stats.instanced_draw_calls += 1;
                } else {
                    stats.batched_draw_calls += 1;
                }
                stats.triangles += (num_verts as u64 / 3) * count as u64;
                stats.vertices += num_verts as u64 * count as u64;
                stats.entities_rendered += count;
            }
        }
    }

    /// Renders 3D model submeshes for a specific alpha mode (Opaque or Mask) across all active batches.
    fn draw_model_batches<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        config: ModelBatchPassConfig<'a>,
        tracker: &mut RenderPassTracker,
        ctx: &ForwardPassContext<'a>,
        stats: &mut FrameRenderStats,
    ) {
        for batch in config.batches {
            if batch.instance_count == 0 {
                continue;
            }
            let Some(model) = ctx.asset_manager.models.get(batch.handle) else {
                continue;
            };
            let has_matching_submesh = if model.submeshes.is_empty() {
                !config.is_cutout
            } else {
                model
                    .submeshes
                    .iter()
                    .any(|s| s.alpha_mode == config.target_mode)
            };
            if !has_matching_submesh {
                continue;
            }

            tracker.bind_model_buffers(pass, batch.handle, model);

            let instances = &ctx.model_instances
                [batch.instance_start..batch.instance_start + batch.instance_count];
            let mut cur = 0;
            while cur < instances.len() {
                let start = cur;
                let override_tex = instances[cur].texture;
                let mirrored = instances[cur].is_mirrored;
                while cur < instances.len()
                    && instances[cur].texture == override_tex
                    && instances[cur].is_mirrored == mirrored
                {
                    cur += 1;
                }
                let count = (cur - start) as u32;

                if config.is_cutout {
                    if mirrored {
                        tracker.set_pipeline(
                            pass,
                            ActivePipeline::CutoutCw,
                            &self.pipelines.cutout_pipeline_cw,
                        );
                    } else {
                        tracker.set_pipeline(
                            pass,
                            ActivePipeline::CutoutCcw,
                            &self.pipelines.cutout_pipeline,
                        );
                    }
                } else if mirrored {
                    tracker.set_pipeline(
                        pass,
                        ActivePipeline::RenderCw,
                        &self.pipelines.render_pipeline_cw,
                    );
                } else {
                    tracker.set_pipeline(
                        pass,
                        ActivePipeline::RenderCcw,
                        &self.pipelines.render_pipeline,
                    );
                }

                let offset_bytes = ((batch.buffer_offset + start) * INSTANCE_SIZE) as u64;
                pass.set_vertex_buffer(1, self.geometry.instance_buffer.slice(offset_bytes..));

                if model.submeshes.is_empty() {
                    let (key, bg) = match override_tex
                        .and_then(|h| ctx.asset_manager.textures.get(h))
                    {
                        Some(t) => (BoundTextureKey::Asset(override_tex.unwrap()), &t.bind_group),
                        None => (
                            BoundTextureKey::DefaultWhite,
                            &self.default_white_texture.bind_group,
                        ),
                    };
                    tracker.bind_texture(pass, key, bg);
                    pass.draw_indexed(0..model.num_indices, 0, 0..count);

                    stats.draw_calls += 1;
                    if count > 1 {
                        stats.instanced_draw_calls += 1;
                    } else {
                        stats.batched_draw_calls += 1;
                    }
                    stats.triangles += (model.num_indices as u64 / 3) * count as u64;
                    stats.vertices += model.raw_vertices.len() as u64 * count as u64;
                    stats.entities_rendered += count;
                } else {
                    for submesh in &model.submeshes {
                        if submesh.alpha_mode != config.target_mode {
                            continue;
                        }
                        let (key, bg) = resolve_submesh_texture(
                            model,
                            submesh,
                            override_tex,
                            ctx.asset_manager,
                            &self.default_white_texture.bind_group,
                        );
                        tracker.bind_texture(pass, key, bg);
                        pass.draw_indexed(
                            submesh.start_index..(submesh.start_index + submesh.index_count),
                            0,
                            0..count,
                        );

                        stats.draw_calls += 1;
                        if count > 1 {
                            stats.instanced_draw_calls += 1;
                        } else {
                            stats.batched_draw_calls += 1;
                        }
                        stats.triangles += (submesh.index_count as u64 / 3) * count as u64;
                        stats.vertices += model.raw_vertices.len() as u64 * count as u64;
                        stats.entities_rendered += count;
                    }
                }
            }
        }
    }

    /// Renders transparent 3D model submeshes in back-to-front sorted depth order.
    fn draw_transparent_submeshes<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        draws: &'a [TransparentSubmeshDraw],
        tracker: &mut RenderPassTracker,
        ctx: &ForwardPassContext<'a>,
        stats: &mut FrameRenderStats,
    ) {
        tracker.set_pipeline(
            pass,
            ActivePipeline::Transparent,
            &self.pipelines.transparent_pipeline,
        );

        for draw in draws {
            let Some(model) = ctx.asset_manager.models.get(draw.handle) else {
                continue;
            };
            tracker.bind_model_buffers(pass, draw.handle, model);

            if draw.submesh_index >= model.submeshes.len() {
                continue;
            }
            let submesh = &model.submeshes[draw.submesh_index];
            let (key, bg) = resolve_submesh_texture(
                model,
                submesh,
                draw.override_tex,
                ctx.asset_manager,
                &self.default_white_texture.bind_group,
            );
            tracker.bind_texture(pass, key, bg);

            let offset_bytes = ((draw.buffer_offset + draw.instance_index) * INSTANCE_SIZE) as u64;
            pass.set_vertex_buffer(1, self.geometry.instance_buffer.slice(offset_bytes..));
            pass.draw_indexed(
                submesh.start_index..(submesh.start_index + submesh.index_count),
                0,
                0..1,
            );

            stats.draw_calls += 1;
            stats.batched_draw_calls += 1;
            stats.triangles += submesh.index_count as u64 / 3;
            stats.vertices += model.raw_vertices.len() as u64;
            stats.entities_rendered += 1;
        }
    }

    /// Renders billboard quad sprites sorted back-to-front.
    fn draw_sprites<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        ctx: &ForwardPassContext<'a>,
        tracker: &mut RenderPassTracker,
        stats: &mut FrameRenderStats,
    ) {
        tracker.reset_model_buffers();
        tracker.set_pipeline(
            pass,
            ActivePipeline::Sprite,
            &self.pipelines.sprite_pipeline,
        );
        pass.set_bind_group(2, &self.uniforms.light_bind_group, &[]);
        pass.set_vertex_buffer(0, self.geometry.quad_vertex_buffer.slice(..));

        let mut cur = 0;
        while cur < ctx.transparent_objs.len() {
            let start = cur;
            let tid = ctx.transparent_objs[cur].1;
            while cur < ctx.transparent_objs.len() && ctx.transparent_objs[cur].1 == tid {
                cur += 1;
            }
            let count = (cur - start) as u32;
            if let Some(t) = ctx.asset_manager.textures.get(tid) {
                pass.set_bind_group(1, &t.bind_group, &[]);
                let offset_bytes = ((ctx.sprite_start + start) * INSTANCE_SIZE) as u64;
                pass.set_vertex_buffer(1, self.geometry.instance_buffer.slice(offset_bytes..));
                pass.draw(0..6, 0..count);

                stats.draw_calls += 1;
                stats.batched_draw_calls += 1;
                stats.triangles += 2 * count as u64;
                stats.vertices += 4 * count as u64;
                stats.entities_rendered += count;
            }
        }
    }

    /// Renders wireframe outlines for primitive geometries and models.
    fn draw_wireframe_pass<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        ctx: &ForwardPassContext<'a>,
        tracker: &mut RenderPassTracker,
    ) {
        tracker.reset_model_buffers();
        tracker.set_pipeline(
            pass,
            ActivePipeline::Wireframe,
            &self.pipelines.wireframe_pipeline,
        );
        tracker.bind_texture(
            pass,
            BoundTextureKey::DefaultWhite,
            &self.default_white_texture.bind_group,
        );

        let primitives = [
            (
                ctx.triangle_instances.len(),
                ctx.tri_start,
                &self.geometry.vertex_buffer,
                self.geometry.triangle_num_vertices,
            ),
            (
                ctx.cube_instances.len(),
                ctx.cube_start,
                &self.geometry.cube_vertex_buffer,
                36,
            ),
            (
                ctx.sphere_instances.len(),
                ctx.sphere_start,
                &self.geometry.sphere_vertex_buffer,
                self.geometry.sphere_num_vertices,
            ),
            (
                ctx.cylinder_instances.len(),
                ctx.cylinder_start,
                &self.geometry.cylinder_vertex_buffer,
                self.geometry.cylinder_num_vertices,
            ),
            (
                ctx.capsule_instances.len(),
                ctx.capsule_start,
                &self.geometry.capsule_vertex_buffer,
                self.geometry.capsule_num_vertices,
            ),
            (
                ctx.torus_instances.len(),
                ctx.torus_start,
                &self.geometry.torus_vertex_buffer,
                self.geometry.torus_num_vertices,
            ),
        ];

        for (count, start, v_buf, num_verts) in primitives {
            if count > 0 {
                pass.set_vertex_buffer(0, v_buf.slice(..));
                let offset_bytes = (start * INSTANCE_SIZE) as u64;
                pass.set_vertex_buffer(1, self.geometry.instance_buffer.slice(offset_bytes..));
                pass.draw(0..num_verts, 0..count as u32);
            }
        }

        for batch in ctx.model_batches {
            let count = batch.instance_count as u32;
            if count > 0
                && let Some(model) = ctx.asset_manager.models.get(batch.handle)
            {
                tracker.bind_model_buffers(pass, batch.handle, model);
                let offset_bytes = (batch.buffer_offset * INSTANCE_SIZE) as u64;
                pass.set_vertex_buffer(1, self.geometry.instance_buffer.slice(offset_bytes..));
                pass.draw_indexed(0..model.num_indices, 0, 0..count);
            }
        }
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame rendering pipeline execution, passes orchestration, and GPU submission.
//!

use super::forward;
use crate::render::engine::state::{RenderOptions, RenderState};
use crate::render::types::{FrameRenderStats, OverlayRenderer, RenderError, ViewportRect};
use crate::render::viewport_texture::ViewportTexture;
use winit::window::Window;

/// Granularity in pixels for viewport texture bucket allocation to prevent GPU target thrashing during interactive resizing.
pub const VIEWPORT_BUCKET_SIZE: u32 = 64;

/// Computes the quantized bucket dimension rounded up to the nearest multiple of `bucket_size`.
///
/// Returns 0 if `dim` is 0 to avoid empty render target allocation.
#[inline]
pub fn quantize_dimension(dim: u32, bucket_size: u32) -> u32 {
    if dim == 0 || bucket_size == 0 {
        return 0;
    }
    dim.div_ceil(bucket_size) * bucket_size
}

/// Type alias for the UI render callback to reduce type complexity.
pub type UiRenderCallback<'a> = &'a mut dyn FnMut(
    &wgpu::Device,
    &wgpu::Queue,
    &mut wgpu::CommandEncoder,
    &Window,
    &wgpu::TextureView,
    Option<&wgpu::TextureView>,
) -> ViewportRect;

/// Parameters for executing a single frame rendering pipeline pass.
pub struct RenderFrameParams<'a> {
    pub scene: crate::render::types::RenderScene,
    pub camera: &'a ae_core::camera::Camera,
    pub overlays: &'a [&'a dyn OverlayRenderer],
    pub asset_manager: &'a crate::asset::AssetManager,
    pub enabled_modules: &'a std::collections::HashSet<ae_core::modules::EngineModule>,
    pub options: &'a RenderOptions,
    pub ui_renderer: Option<UiRenderCallback<'a>>,
}

/// Context holding the acquired swapchain surface texture, its view, and presentation latency.
pub(crate) struct AcquiredFrameSurface {
    /// Active swapchain output texture acquired from the surface.
    pub output: wgpu::SurfaceTexture,
    /// Default texture view created for presentation rendering.
    pub surface_view: wgpu::TextureView,
    /// Latency elapsed during swapchain texture acquisition.
    pub acquire_wait: std::time::Duration,
}

impl RenderState {
    /// Evaluates the active environment color converted from sRGB to linear gamma space.
    ///
    /// Avoids repeating redundant perceptual powf(2.2) conversions across render passes.
    #[inline]
    pub fn environment_clear_color(&self) -> wgpu::Color {
        let env = self.graphics_settings.environment_color;
        wgpu::Color {
            r: (env[0] as f64).powf(2.2),
            g: (env[1] as f64).powf(2.2),
            b: (env[2] as f64).powf(2.2),
            a: 1.0,
        }
    }

    /// Acquires the next swapchain surface texture, handling surface loss and reconfiguration.
    pub(crate) fn acquire_surface_frame(&self) -> Result<AcquiredFrameSurface, RenderError> {
        let acquire_start = std::time::Instant::now();
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(tex)
            | wgpu::CurrentSurfaceTexture::Suboptimal(tex) => tex,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                return Err(RenderError::SurfaceLost);
            }
            other => return Err(RenderError::Other(format!("{:?}", other))),
        };
        let acquire_wait = acquire_start.elapsed();
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok(AcquiredFrameSurface {
            output,
            surface_view,
            acquire_wait,
        })
    }

    /// Submits the finished command encoder and presents the frame to the display subsystem.
    ///
    /// Aggregates surface acquisition, queue submission, and presentation latencies into `last_present_wait_secs`.
    pub(crate) fn submit_and_present(
        &mut self,
        encoder: wgpu::CommandEncoder,
        output: wgpu::SurfaceTexture,
        acquire_wait: std::time::Duration,
    ) {
        let submit_start = std::time::Instant::now();
        self.queue.submit(std::iter::once(encoder.finish()));
        let submit_wait = submit_start.elapsed();

        let present_start = std::time::Instant::now();
        self.queue.present(output);
        let present_wait = present_start.elapsed();

        self.last_present_wait_secs = (acquire_wait + submit_wait + present_wait).as_secs_f32();
    }

    /// Encodes a single color clear render pass on the target texture view.
    pub(crate) fn encode_clear_pass(
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        clear_color: wgpu::Color,
        label: &'static str,
    ) {
        let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    /// Executes the full frame render pipeline: shadow pass → main pass (sky, grid,
    /// opaque geometry, wireframe, sprites, overlays) → bloom/post-process → Iris UI.
    pub fn render(&mut self, params: RenderFrameParams<'_>) -> Result<(), RenderError> {
        let RenderFrameParams {
            scene,
            camera,
            overlays,
            asset_manager,
            enabled_modules,
            options,
            ui_renderer,
        } = params;
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }

        // Resize viewport texture and post-process systems if the last known viewport rect size changed or is not yet initialized
        let scale = if options.scale_factor > 0.0 {
            options.scale_factor
        } else {
            self.window.scale_factor() as f32
        };
        let vp_w = ((self.last_viewport_rect.max_x - self.last_viewport_rect.min_x) * scale)
            .max(0.0) as u32;
        let vp_h = ((self.last_viewport_rect.max_y - self.last_viewport_rect.min_y) * scale)
            .max(0.0) as u32;
        if vp_w > 0 && vp_h > 0 {
            let bucket_w = quantize_dimension(vp_w, VIEWPORT_BUCKET_SIZE);
            let bucket_h = quantize_dimension(vp_h, VIEWPORT_BUCKET_SIZE);

            let needs_resize = match &self.viewport_texture {
                Some(vt) => vt.width != bucket_w || vt.height != bucket_h,
                None => true,
            };
            if needs_resize {
                self.viewport_texture = Some(ViewportTexture::new(
                    &self.device,
                    bucket_w,
                    bucket_h,
                    vp_w,
                    vp_h,
                    self.config.format,
                    "Viewport Texture",
                ));

                // Re-configure post-process pipeline intermediate targets to bucketed viewport size
                let vp_config = wgpu::SurfaceConfiguration {
                    usage: self.config.usage,
                    format: self.config.format,
                    width: bucket_w,
                    height: bucket_h,
                    present_mode: self.config.present_mode,
                    alpha_mode: self.config.alpha_mode,
                    view_formats: vec![],
                    desired_maximum_frame_latency: self.config.desired_maximum_frame_latency,
                    color_space: self.config.color_space,
                };
                self.post_process.resize(
                    &self.device,
                    &vp_config,
                    self.graphics_settings.msaa_samples,
                );
            } else if let Some(vt) = &mut self.viewport_texture {
                vt.logical_width = vp_w;
                vt.logical_height = vp_h;
            }
            if self.outline.width != bucket_w || self.outline.height != bucket_h {
                self.outline
                    .resize(&self.device, &self.queue, bucket_w, bucket_h);
            }
        }

        let is_2d_mode = options.is_2d_mode;
        let render_enabled = if is_2d_mode {
            enabled_modules.contains(&ae_core::modules::EngineModule::Render2D)
        } else {
            enabled_modules.contains(&ae_core::modules::EngineModule::Render)
        };

        if !render_enabled {
            let AcquiredFrameSurface {
                output,
                surface_view,
                acquire_wait,
            } = self.acquire_surface_frame()?;
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Disabled Render Encoder"),
                });

            let clear_rgb = self.environment_clear_color();
            // Pass 1: Clear OS Surface View
            Self::encode_clear_pass(
                &mut encoder,
                &surface_view,
                clear_rgb,
                "Clear OS Surface Pass (Render Disabled)",
            );

            // Pass 2: Clear Viewport Texture (if present)
            if let Some(vt) = &self.viewport_texture {
                Self::encode_clear_pass(
                    &mut encoder,
                    &vt.view,
                    clear_rgb,
                    "Clear Viewport Texture Pass (Render Disabled)",
                );
            }

            if let Some(render_ui) = ui_renderer {
                let vt_view = self.viewport_texture.as_ref().map(|vt| &vt.view);
                self.last_viewport_rect = render_ui(
                    &self.device,
                    &self.queue,
                    &mut encoder,
                    &self.window,
                    &surface_view,
                    vt_view,
                );
            }

            self.submit_and_present(encoder, output, acquire_wait);
            return Ok(());
        }

        if is_2d_mode {
            return self.render_2d_frame(overlays, ui_renderer);
        }

        let triangle_instances = scene.triangle_instances;
        let cube_instances = scene.cube_instances;
        let sphere_instances = scene.sphere_instances;
        let cylinder_instances = scene.cylinder_instances;
        let capsule_instances = scene.capsule_instances;
        let torus_instances = scene.torus_instances;
        let mut transparent_objs = scene.transparent_objs;
        let model_instance_data = scene.model_instance_data;

        if transparent_objs.len() > 1 {
            transparent_objs.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
        }

        self.frame_scratch.clear();

        let total_primitive_instances = triangle_instances.len()
            + cube_instances.len()
            + sphere_instances.len()
            + cylinder_instances.len()
            + capsule_instances.len()
            + torus_instances.len()
            + transparent_objs.len();

        let total_model_instances: usize = model_instance_data.values().map(Vec::len).sum();

        self.frame_scratch
            .all_instances
            .reserve(total_primitive_instances + total_model_instances);

        // Buffer Prep
        let tri_start = 0;
        self.frame_scratch
            .all_instances
            .extend(triangle_instances.iter().map(|item| item.instance));

        let cube_start = self.frame_scratch.all_instances.len();
        self.frame_scratch
            .all_instances
            .extend(cube_instances.iter().map(|item| item.instance));

        let sphere_start = self.frame_scratch.all_instances.len();
        self.frame_scratch
            .all_instances
            .extend(sphere_instances.iter().map(|item| item.instance));

        let cylinder_start = self.frame_scratch.all_instances.len();
        self.frame_scratch
            .all_instances
            .extend(cylinder_instances.iter().map(|item| item.instance));

        let capsule_start = self.frame_scratch.all_instances.len();
        self.frame_scratch
            .all_instances
            .extend(capsule_instances.iter().map(|item| item.instance));

        let torus_start = self.frame_scratch.all_instances.len();
        self.frame_scratch
            .all_instances
            .extend(torus_instances.iter().map(|item| item.instance));

        let sprite_start = self.frame_scratch.all_instances.len();
        self.frame_scratch
            .all_instances
            .extend(transparent_objs.iter().map(|(_, _, i)| *i));

        let cam_pos = camera.position_vec3();

        for (handle, insts) in &model_instance_data {
            if insts.is_empty() {
                continue;
            }
            if let Some(model) = asset_manager.models.get(*handle) {
                let buffer_offset = self.frame_scratch.all_instances.len();
                self.frame_scratch
                    .model_starts
                    .insert(*handle, buffer_offset);
                self.frame_scratch
                    .all_instances
                    .extend(insts.iter().map(|item| item.instance));
                let instance_start = self.frame_scratch.model_instances.len();
                self.frame_scratch.model_instances.extend_from_slice(insts);
                let instance_count = insts.len();

                self.frame_scratch
                    .model_batches
                    .push(forward::PreparedModelBatch {
                        handle: *handle,
                        instance_start,
                        instance_count,
                        buffer_offset,
                    });

                for (submesh_index, submesh) in model.submeshes.iter().enumerate() {
                    if submesh.alpha_mode == crate::render::types::SubmeshAlphaMode::Blend {
                        for (inst_idx, item) in insts.iter().enumerate() {
                            let dx = item.world_position[0] - cam_pos.x;
                            let dy = item.world_position[1] - cam_pos.y;
                            let dz = item.world_position[2] - cam_pos.z;
                            let dist_sq = dx * dx + dy * dy + dz * dz;
                            self.frame_scratch.transparent_model_draws.push(
                                forward::TransparentSubmeshDraw {
                                    distance_sq: dist_sq,
                                    handle: *handle,
                                    submesh_index,
                                    buffer_offset,
                                    instance_index: inst_idx,
                                    override_tex: item.texture,
                                },
                            );
                        }
                    }
                }
            }
        }

        // Sort transparent model submeshes Back-to-Front (descending distance)
        // Uses in-place pdqsort (sort_unstable_by) with branchless total_cmp.
        // Runs in O(N) linear time on already-sorted or nearly-sorted frames without heap allocations.
        if self.frame_scratch.transparent_model_draws.len() > 1 {
            self.frame_scratch
                .transparent_model_draws
                .sort_unstable_by(|a, b| b.distance_sq.total_cmp(&a.distance_sq));
        }

        self.geometry.update_instances(
            &self.device,
            &self.queue,
            &self.frame_scratch.all_instances,
        );

        self.uniforms.update(&self.queue, camera);
        self.uniforms
            .update_environment(&self.queue, &self.graphics_settings);

        // Shadow Cascades share the sun vector
        self.shadow.update_cascades(
            &self.queue,
            &self.graphics_settings,
            camera,
            cgmath::Vector3::new(
                self.uniforms.light_uniform.direction[0],
                self.uniforms.light_uniform.direction[1],
                self.uniforms.light_uniform.direction[2],
            ),
        );

        // Write bloom intensity uniform
        self.queue.write_buffer(
            &self.post_process.bloom_params_buffer,
            0,
            bytemuck::cast_slice(&[self.graphics_settings.bloom_intensity]),
        );

        let AcquiredFrameSurface {
            output,
            surface_view,
            acquire_wait,
        } = self.acquire_surface_frame()?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("3D Forward Command Encoder"),
            });

        let shadow_timer = std::time::Instant::now();
        self.shadow.execute_pass(
            &mut encoder,
            crate::render::shadow::ShadowPassParams {
                graphics_settings: &self.graphics_settings,
                primitives: &self.geometry,
                instance_buffer: &self.geometry.instance_buffer,
                offsets: crate::render::shadow::ShadowPassBatchOffsets {
                    all_instances_count: self.frame_scratch.all_instances.len(),
                    triangle_instances_count: triangle_instances.len(),
                    tri_start,
                    cube_instances_count: cube_instances.len(),
                    cube_start,
                    sphere_instances_count: sphere_instances.len(),
                    sphere_start,
                    cylinder_instances_count: cylinder_instances.len(),
                    cylinder_start,
                    capsule_instances_count: capsule_instances.len(),
                    capsule_start,
                    torus_instances_count: torus_instances.len(),
                    torus_start,
                },
                asset_manager,
                model_instance_data: &model_instance_data,
                model_starts: &self.frame_scratch.model_starts,
                default_white_texture: &self.default_white_texture,
            },
        );
        let shadow_pass_record_ms = shadow_timer.elapsed().as_secs_f32() * 1000.0;

        // PASS 1: MAIN FORWARD PASS
        let main_timer = std::time::Instant::now();
        self.last_render_stats = self.execute_main_forward_pass(
            &mut encoder,
            forward::ForwardPassContext {
                options,
                all_instances_len: self.frame_scratch.all_instances.len(),
                triangle_instances: &triangle_instances,
                tri_start,
                cube_instances: &cube_instances,
                cube_start,
                sphere_instances: &sphere_instances,
                sphere_start,
                cylinder_instances: &cylinder_instances,
                cylinder_start,
                capsule_instances: &capsule_instances,
                capsule_start,
                torus_instances: &torus_instances,
                torus_start,
                transparent_objs: &transparent_objs,
                sprite_start,
                model_batches: &self.frame_scratch.model_batches,
                model_instances: &self.frame_scratch.model_instances,
                transparent_model_draws: &self.frame_scratch.transparent_model_draws,
                overlays,
                asset_manager,
            },
        );
        let main_opaque_pass_record_ms = main_timer.elapsed().as_secs_f32() * 1000.0;
        self.last_render_stats.culled_meshes = scene.culled_entities_count;

        // BLOOM & POST PROCESS
        let post_timer = std::time::Instant::now();
        let target_view = self
            .viewport_texture
            .as_ref()
            .map(|vt| &vt.view)
            .unwrap_or(&surface_view);
        self.post_process.execute(
            &mut encoder,
            target_view,
            &self.queue,
            self.graphics_settings.bloom_enabled,
        );

        // PASS 2: SCREEN-SPACE SILHOUETTE OUTLINE PASS
        self.execute_selection_outline_pass(
            &mut encoder,
            &surface_view,
            &scene.selected_primitive_instances,
            &scene.selected_model_instances,
            asset_manager,
        );
        let post_process_pass_record_ms = post_timer.elapsed().as_secs_f32() * 1000.0;

        let ui_timer = std::time::Instant::now();
        if let Some(render_ui) = ui_renderer {
            let vt_view = self.viewport_texture.as_ref().map(|vt| &vt.view);
            self.last_viewport_rect = render_ui(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &surface_view,
                vt_view,
            );
        }
        let ui_pass_record_ms = ui_timer.elapsed().as_secs_f32() * 1000.0;

        let total_record_ms = shadow_pass_record_ms
            + main_opaque_pass_record_ms
            + post_process_pass_record_ms
            + ui_pass_record_ms;
        self.last_recording_timings = ae_core::telemetry::RenderPassRecordingTimings {
            shadow_pass_record_ms,
            main_opaque_pass_record_ms,
            post_process_pass_record_ms,
            ui_pass_record_ms,
            total_record_ms,
        };

        self.submit_and_present(encoder, output, acquire_wait);

        Ok(())
    }

    /// Executes the isolated 2D graphics rendering pipeline.
    ///
    /// Completely decouples 2D execution from 3D graphics machinery:
    /// - 0 shadow cascades updated or rendered ($0.0$ ms shadow pass)
    /// - 0 3D PBR forward passes, skybox passes, or 3D primitive geometry buffer updates
    /// - 0 3D silhouette outline passes
    ///
    /// Clears the 2D viewport texture using environment settings, renders 2D overlays (instanced sprite batcher),
    /// and runs the Iris UI render pass.
    fn render_2d_frame(
        &mut self,
        overlays: &[&dyn OverlayRenderer],
        ui_renderer: Option<UiRenderCallback<'_>>,
    ) -> Result<(), RenderError> {
        let AcquiredFrameSurface {
            output,
            surface_view,
            acquire_wait,
        } = self.acquire_surface_frame()?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("2D Command Encoder"),
            });

        let clear_rgb = self.environment_clear_color();

        // Pass 1: Clear OS Surface View to neutral dock background
        let surface_clear = wgpu::Color {
            r: 0.05,
            g: 0.05,
            b: 0.07,
            a: 1.0,
        };
        Self::encode_clear_pass(
            &mut encoder,
            &surface_view,
            surface_clear,
            "Clear OS Surface Pass (2D)",
        );

        // Pass 2: Render 2D Sprite Viewport Pass
        let main_timer = std::time::Instant::now();
        let target_view = self
            .viewport_texture
            .as_ref()
            .map(|vt| &vt.view)
            .unwrap_or(&surface_view);

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("2D Viewport Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_rgb),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Draw all 2D overlays (Sprite2DOverlay, DebugRenderer)
            for ov in overlays {
                ov.draw_overlay(&self.queue, &mut pass);
            }
        }
        let main_opaque_pass_record_ms = main_timer.elapsed().as_secs_f32() * 1000.0;

        // Reset 3D render stats for 2D mode
        self.last_render_stats = FrameRenderStats::default();

        // UI pass
        let ui_timer = std::time::Instant::now();
        if let Some(render_ui) = ui_renderer {
            let vt_view = self.viewport_texture.as_ref().map(|vt| &vt.view);
            self.last_viewport_rect = render_ui(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &surface_view,
                vt_view,
            );
        }
        let ui_pass_record_ms = ui_timer.elapsed().as_secs_f32() * 1000.0;

        let total_record_ms = main_opaque_pass_record_ms + ui_pass_record_ms;
        self.last_recording_timings = ae_core::telemetry::RenderPassRecordingTimings {
            shadow_pass_record_ms: 0.0,
            main_opaque_pass_record_ms,
            post_process_pass_record_ms: 0.0,
            ui_pass_record_ms,
            total_record_ms,
        };

        self.submit_and_present(encoder, output, acquire_wait);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantize_dimension_edge_cases() {
        assert_eq!(quantize_dimension(0, 64), 0);
        assert_eq!(quantize_dimension(1, 64), 64);
        assert_eq!(quantize_dimension(63, 64), 64);
        assert_eq!(quantize_dimension(64, 64), 64);
        assert_eq!(quantize_dimension(65, 64), 128);
        assert_eq!(quantize_dimension(128, 64), 128);
        assert_eq!(quantize_dimension(1000, 128), 1024);
        assert_eq!(quantize_dimension(50, 0), 0);
    }

    #[test]
    fn test_transparent_sort_invariants() {
        let handle = crate::asset::AssetHandle::default();
        let mut draws = [
            forward::TransparentSubmeshDraw {
                distance_sq: 10.0,
                handle,
                submesh_index: 0,
                buffer_offset: 0,
                instance_index: 0,
                override_tex: None,
            },
            forward::TransparentSubmeshDraw {
                distance_sq: 50.0,
                handle,
                submesh_index: 0,
                buffer_offset: 0,
                instance_index: 0,
                override_tex: None,
            },
            forward::TransparentSubmeshDraw {
                distance_sq: 25.0,
                handle,
                submesh_index: 0,
                buffer_offset: 0,
                instance_index: 0,
                override_tex: None,
            },
        ];

        draws.sort_unstable_by(|a, b| b.distance_sq.total_cmp(&a.distance_sq));

        // Descending order (farthest first)
        assert_eq!(draws[0].distance_sq, 50.0);
        assert_eq!(draws[1].distance_sq, 25.0);
        assert_eq!(draws[2].distance_sq, 10.0);
    }
}
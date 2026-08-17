// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Frame Rendering Pipeline Sub-module.
//!
//! Coordinates the frame rendering lifecycle: viewport resizing, shadow cascades,
//! main forward geometry passes, post-processing bloom, selection outlines, and egui UI.
//!

pub mod forward;
pub mod outline;

pub use forward::*;

use super::state::{RenderOptions, RenderState};
use crate::render::types::{OverlayRenderer, RenderError, ViewportRect};
use crate::render::viewport_texture::ViewportTexture;
use winit::window::Window;

impl RenderState {
    /// Executes the full frame render pipeline: shadow pass → main pass (sky, grid,
    /// opaque geometry, wireframe, sprites, overlays) → bloom/post-process → egui UI.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn render(
        &mut self,
        _world: &hecs::World,
        scene: crate::render::types::RenderScene,
        camera: &ae_core::camera::Camera,
        overlays: &[&dyn OverlayRenderer],
        asset_manager: &crate::asset::AssetManager,
        enabled_modules: &std::collections::HashSet<ae_core::modules::EngineModule>,
        options: &RenderOptions,
        ui_renderer: Option<
            &mut dyn FnMut(
                &wgpu::Device,
                &wgpu::Queue,
                &mut wgpu::CommandEncoder,
                &Window,
                &wgpu::TextureView,
                Option<&wgpu::TextureView>,
            ) -> ViewportRect,
        >,
    ) -> Result<(), RenderError> {
        if self.size.width == 0 || self.size.height == 0 {
            return Ok(());
        }

        // Resize viewport texture and post-process systems if the last known viewport rect size changed or is not yet initialized
        let scale = self.window.scale_factor() as f32;
        let rect_w = self.last_viewport_rect.max_x - self.last_viewport_rect.min_x;
        let rect_h = self.last_viewport_rect.max_y - self.last_viewport_rect.min_y;

        let vp_w = if rect_w.is_finite() && rect_w > 0.0 {
            ((rect_w * scale) as u32).clamp(1, 16384)
        } else {
            0
        };
        let vp_h = if rect_h.is_finite() && rect_h > 0.0 {
            ((rect_h * scale) as u32).clamp(1, 16384)
        } else {
            0
        };
        if vp_w > 0 && vp_h > 0 {
            let needs_resize = match &self.viewport_texture {
                Some(vt) => vt.width != vp_w || vt.height != vp_h,
                None => true,
            };
            if needs_resize {
                self.viewport_texture = Some(ViewportTexture::new(
                    &self.device,
                    vp_w,
                    vp_h,
                    self.config.format,
                    "Viewport Texture",
                ));

                // Re-configure post-process pipeline intermediate targets to viewport size
                let mut vp_config = self.config.clone();
                vp_config.width = vp_w;
                vp_config.height = vp_h;
                self.post_process.resize(
                    &self.device,
                    &vp_config,
                    self.graphics_settings.msaa_samples,
                );
            }
            if self.outline.width != vp_w || self.outline.height != vp_h {
                self.outline.resize(&self.device, &self.queue, vp_w, vp_h);
            }
        }

        let render_enabled = enabled_modules.contains(&ae_core::modules::EngineModule::Render);

        if !render_enabled {
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
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Encoder"),
                });

            let env_color = self.graphics_settings.environment_color;
            let clear_rgb = wgpu::Color {
                r: (env_color[0] as f64).powf(2.2),
                g: (env_color[1] as f64).powf(2.2),
                b: (env_color[2] as f64).powf(2.2),
                a: 1.0,
            };
            {
                // Pass 1: Clear OS Surface View
                let _clear_surface_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear OS Surface Pass (Render Disabled)"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &surface_view,
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
            }

            if let Some(vt) = &self.viewport_texture {
                // Pass 2: Clear 3D Viewport Texture (different dimensions from surface_view)
                let _clear_vp_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Viewport Texture Pass (Render Disabled)"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &vt.view,
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
            }

            if let Some(render_ui) = ui_renderer {
                let vt_view = self.viewport_texture.as_ref().map(|vt| &vt.egui_view);
                self.last_viewport_rect = render_ui(
                    &self.device,
                    &self.queue,
                    &mut encoder,
                    &self.window,
                    &surface_view,
                    vt_view,
                );
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            let present_start = std::time::Instant::now();
            self.queue.present(output);
            let present_wait = present_start.elapsed();

            self.last_present_wait_secs = (acquire_wait + present_wait).as_secs_f32();

            return Ok(());
        }

        let triangle_instances = scene.triangle_instances;
        let cube_instances = scene.cube_instances;
        let sphere_instances = scene.sphere_instances;
        let cylinder_instances = scene.cylinder_instances;
        let capsule_instances = scene.capsule_instances;
        let torus_instances = scene.torus_instances;
        let mut transparent_objs = scene.transparent_objs;
        let model_instance_data = scene.model_instance_data;

        transparent_objs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // Buffer Prep
        let mut all_instances = Vec::new();
        let tri_start = 0;
        for (inst, _) in &triangle_instances {
            all_instances.push(*inst);
        }
        let cube_start = all_instances.len();
        for (inst, _) in &cube_instances {
            all_instances.push(*inst);
        }
        let sphere_start = all_instances.len();
        for (inst, _) in &sphere_instances {
            all_instances.push(*inst);
        }
        let cylinder_start = all_instances.len();
        for (inst, _) in &cylinder_instances {
            all_instances.push(*inst);
        }
        let capsule_start = all_instances.len();
        for (inst, _) in &capsule_instances {
            all_instances.push(*inst);
        }
        let torus_start = all_instances.len();
        for (inst, _) in &torus_instances {
            all_instances.push(*inst);
        }
        let sprite_start = all_instances.len();
        for (_, _, i) in &transparent_objs {
            all_instances.push(*i);
        }
        let mut model_starts =
            std::collections::HashMap::<crate::asset::AssetHandle, usize>::with_capacity(
                asset_manager.models.capacity(),
            );
        for (handle, insts) in &model_instance_data {
            model_starts.insert(*handle, all_instances.len());
            for (inst, _) in insts {
                all_instances.push(*inst);
            }
        }

        self.geometry
            .update_instances(&self.device, &self.queue, &all_instances);

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
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        self.shadow.execute_pass(
            &mut encoder,
            &self.graphics_settings,
            &self.geometry,
            &self.geometry.instance_buffer,
            all_instances.len(),
            triangle_instances.len(),
            tri_start,
            cube_instances.len(),
            cube_start,
            sphere_instances.len(),
            sphere_start,
            cylinder_instances.len(),
            cylinder_start,
            capsule_instances.len(),
            capsule_start,
            torus_instances.len(),
            torus_start,
            asset_manager,
            &model_instance_data,
            &model_starts,
            &self.default_white_texture,
        );

        // PASS 1: MAIN FORWARD PASS
        self.execute_main_forward_pass(
            &mut encoder,
            forward::ForwardPassContext {
                options,
                all_instances_len: all_instances.len(),
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
                model_instance_data: &model_instance_data,
                model_starts: &model_starts,
                overlays,
                asset_manager,
            },
        );

        // BLOOM & POST PROCESS
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

        if let Some(render_ui) = ui_renderer {
            let vt_view = self.viewport_texture.as_ref().map(|vt| &vt.egui_view);
            self.last_viewport_rect = render_ui(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &surface_view,
                vt_view,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        let present_start = std::time::Instant::now();
        self.queue.present(output);
        let present_wait = present_start.elapsed();

        self.last_present_wait_secs = (acquire_wait + present_wait).as_secs_f32();

        Ok(())
    }
}
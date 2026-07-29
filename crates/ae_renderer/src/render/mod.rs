// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub mod pipelines;
pub mod post_process;
pub mod primitives;
pub mod resources;
pub mod setup;
pub mod shadow;
pub mod types;
pub mod uniforms;
pub mod viewport_texture;

use std::sync::Arc;
use winit::window::Window;

/// Options containing viewport visualization flags.
#[derive(Clone, Copy, Debug)]
pub struct RenderOptions {
    pub grid_enabled: bool,
    pub wireframe_enabled: bool,
}

pub use types::*;
pub use viewport_texture::ViewportTexture;

/// Central render state owning all WGPU resources: device, queue, surface,
/// pipeline manager, uniforms, geometry system, post-processing, and shadow cascades.
pub struct RenderState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub viewport_texture: Option<ViewportTexture>,

    pub pipelines: crate::render::pipelines::PipelineManager,
    pub uniforms: crate::render::uniforms::SceneUniforms,
    pub geometry: crate::render::primitives::GeometrySystem,

    pub post_process: crate::render::post_process::PostProcessSystem,
    pub shadow: crate::render::shadow::ShadowSystem,

    // Settings
    pub graphics_settings: crate::graphics_settings::GraphicsSettings,
    /// Cached 3D viewport rect from the last completed egui frame.
    /// Updated every frame after UI renders. Used for mouse-in-viewport detection.
    pub last_viewport_rect: ViewportRect,
    /// Cached list of present modes supported by the surface on this GPU adapter.
    pub supported_present_modes: Vec<wgpu::PresentMode>,
    /// Wall-clock seconds spent blocking inside `get_current_texture()` + `present()`.
    /// On Windows DX12 when DXGI ALLOW_TEARING is unavailable, these calls block at
    /// VSync rate (~16.7ms at 60Hz). This value is subtracted from `time.delta_time`
    /// when calculating the FPS counter so that Uncapped mode displays the engine's
    /// true compute throughput rather than the display refresh rate.
    pub last_present_wait_secs: f32,
}

impl RenderState {
    /// Executes the full frame render pipeline: shadow pass → main pass (sky, grid,
    /// opaque geometry, wireframe, sprites, overlays) → bloom/post-process → egui UI.
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
        let vp_w = ((self.last_viewport_rect.max_x - self.last_viewport_rect.min_x) * scale)
            .max(0.0) as u32;
        let vp_h = ((self.last_viewport_rect.max_y - self.last_viewport_rect.min_y) * scale)
            .max(0.0) as u32;
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
            output.present();
            let present_wait = present_start.elapsed();

            self.last_present_wait_secs = (acquire_wait + present_wait).as_secs_f32();

            return Ok(());
        }

        // Global lighting is handled via update_environment using GraphicsSettings.

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
        all_instances.extend_from_slice(&triangle_instances);
        let cube_start = all_instances.len();
        all_instances.extend_from_slice(&cube_instances);
        let sphere_start = all_instances.len();
        all_instances.extend_from_slice(&sphere_instances);
        let cylinder_start = all_instances.len();
        all_instances.extend_from_slice(&cylinder_instances);
        let capsule_start = all_instances.len();
        all_instances.extend_from_slice(&capsule_instances);
        let torus_start = all_instances.len();
        all_instances.extend_from_slice(&torus_instances);
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
            all_instances.extend_from_slice(insts);
        }

        self.geometry
            .update_instances(&self.device, &self.queue, &all_instances);

        // We no longer read scene.light_uniform, as environment is global from Settings
        self.uniforms.update(&self.queue, camera);
        self.uniforms
            .update_environment(&self.queue, &self.graphics_settings);

        // Shadow Cascades also share the exact same sun vector
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

        // Time the texture acquisition — this is where DX12/VSync blocking occurs
        // when ALLOW_TEARING is unavailable. The elapsed time is subtracted from
        // delta_time for accurate Uncapped FPS reporting.
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
        );

        // PASS 1: MAIN
        {
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
            // Replace slot 1 with sky_bind_group for the sky shader
            pass.set_bind_group(1, &self.uniforms.sky_bind_group, &[]);
            pass.draw(0..3, 0..1);

            // Restore light group for opaque geometry
            pass.set_bind_group(1, &self.uniforms.light_bind_group, &[]);

            // --- GRID ---
            if options.grid_enabled {
                pass.set_pipeline(&self.pipelines.grid_pipeline);
                pass.set_vertex_buffer(0, self.geometry.grid_vertex_buffer.slice(..));
                pass.draw(0..6, 0..1);
            }

            if !all_instances.is_empty() {
                pass.set_pipeline(&self.pipelines.render_pipeline);
                if !triangle_instances.is_empty() {
                    pass.set_vertex_buffer(0, self.geometry.vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice((tri_start * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(0..3, 0..triangle_instances.len() as u32);
                }
                if !cube_instances.is_empty() {
                    pass.set_vertex_buffer(0, self.geometry.cube_vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice((cube_start * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(0..36, 0..cube_instances.len() as u32);
                }
                if !sphere_instances.is_empty() {
                    pass.set_vertex_buffer(0, self.geometry.sphere_vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice((sphere_start * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(
                        0..self.geometry.sphere_num_vertices,
                        0..sphere_instances.len() as u32,
                    );
                }
                if !cylinder_instances.is_empty() {
                    pass.set_vertex_buffer(0, self.geometry.cylinder_vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice((cylinder_start * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(
                        0..self.geometry.cylinder_num_vertices,
                        0..cylinder_instances.len() as u32,
                    );
                }
                if !capsule_instances.is_empty() {
                    pass.set_vertex_buffer(0, self.geometry.capsule_vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice((capsule_start * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(
                        0..self.geometry.capsule_num_vertices,
                        0..capsule_instances.len() as u32,
                    );
                }
                if !torus_instances.is_empty() {
                    pass.set_vertex_buffer(0, self.geometry.torus_vertex_buffer.slice(..));
                    pass.set_vertex_buffer(
                        1,
                        self.geometry
                            .instance_buffer
                            .slice((torus_start * INSTANCE_SIZE) as u64..),
                    );
                    pass.draw(
                        0..self.geometry.torus_num_vertices,
                        0..torus_instances.len() as u32,
                    );
                }
                for (handle, m) in asset_manager.models.iter() {
                    let c = model_instance_data
                        .get(&handle)
                        .map(|v| v.len())
                        .unwrap_or(0) as u32;
                    if c > 0 {
                        pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry.instance_buffer.slice(
                                ((*model_starts.get(&handle).unwrap_or(&0)) * INSTANCE_SIZE)
                                    as u64..,
                            ),
                        );
                        pass.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..m.num_indices, 0, 0..c);
                    }
                }

                if options.wireframe_enabled {
                    pass.set_pipeline(&self.pipelines.wireframe_pipeline);
                    if !cube_instances.is_empty() {
                        pass.set_vertex_buffer(0, self.geometry.cube_vertex_buffer.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry
                                .instance_buffer
                                .slice((cube_start * INSTANCE_SIZE) as u64..),
                        );
                        pass.draw(0..36, 0..cube_instances.len() as u32);
                    }
                    if !sphere_instances.is_empty() {
                        pass.set_vertex_buffer(0, self.geometry.sphere_vertex_buffer.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry
                                .instance_buffer
                                .slice((sphere_start * INSTANCE_SIZE) as u64..),
                        );
                        pass.draw(
                            0..self.geometry.sphere_num_vertices,
                            0..sphere_instances.len() as u32,
                        );
                    }
                    if !cylinder_instances.is_empty() {
                        pass.set_vertex_buffer(0, self.geometry.cylinder_vertex_buffer.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry
                                .instance_buffer
                                .slice((cylinder_start * INSTANCE_SIZE) as u64..),
                        );
                        pass.draw(
                            0..self.geometry.cylinder_num_vertices,
                            0..cylinder_instances.len() as u32,
                        );
                    }
                    if !capsule_instances.is_empty() {
                        pass.set_vertex_buffer(0, self.geometry.capsule_vertex_buffer.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry
                                .instance_buffer
                                .slice((capsule_start * INSTANCE_SIZE) as u64..),
                        );
                        pass.draw(
                            0..self.geometry.capsule_num_vertices,
                            0..capsule_instances.len() as u32,
                        );
                    }
                    if !torus_instances.is_empty() {
                        pass.set_vertex_buffer(0, self.geometry.torus_vertex_buffer.slice(..));
                        pass.set_vertex_buffer(
                            1,
                            self.geometry
                                .instance_buffer
                                .slice((torus_start * INSTANCE_SIZE) as u64..),
                        );
                        pass.draw(
                            0..self.geometry.torus_num_vertices,
                            0..torus_instances.len() as u32,
                        );
                    }
                    for (handle, m) in asset_manager.models.iter() {
                        let c = model_instance_data
                            .get(&handle)
                            .map(|v| v.len())
                            .unwrap_or(0) as u32;
                        if c > 0 {
                            pass.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                            pass.set_vertex_buffer(
                                1,
                                self.geometry.instance_buffer.slice(
                                    ((*model_starts.get(&handle).unwrap_or(&0)) * INSTANCE_SIZE)
                                        as u64..,
                                ),
                            );
                            pass.set_index_buffer(
                                m.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );
                            pass.draw_indexed(0..m.num_indices, 0, 0..c);
                        }
                    }
                }

                if !transparent_objs.is_empty() {
                    pass.set_pipeline(&self.pipelines.sprite_pipeline);
                    pass.set_bind_group(2, &self.uniforms.light_bind_group, &[]);
                    pass.set_vertex_buffer(0, self.geometry.quad_vertex_buffer.slice(..));
                    let mut cur = 0;
                    while cur < transparent_objs.len() {
                        let start = cur;
                        let tid = transparent_objs[cur].1;
                        while cur < transparent_objs.len() && transparent_objs[cur].1 == tid {
                            cur += 1;
                        }
                        if let Some(t) = asset_manager.textures.get(tid) {
                            pass.set_bind_group(1, &t.bind_group, &[]);
                            pass.set_vertex_buffer(
                                1,
                                self.geometry
                                    .instance_buffer
                                    .slice(((sprite_start + start) * INSTANCE_SIZE) as u64..),
                            );
                            pass.draw(0..6, 0..(cur - start) as u32);
                        }
                    }
                }
            }

            // Editor Overlays (gizmo, debug lines, etc.)
            for ov in overlays {
                ov.draw_overlay(&self.queue, &mut pass);
            }
        }

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
        output.present();
        let present_wait = present_start.elapsed();

        // Store total VSync blocking time for FPS counter correction
        self.last_present_wait_secs = (acquire_wait + present_wait).as_secs_f32();

        Ok(())
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use super::RenderState;
use crate::camera::Camera;
use crate::graphics_settings::{FpsLimit, GraphicsSettings};
use cgmath::InnerSpace;
use std::sync::Arc;
use winit::window::Window;

/// Helper function to choose the optimal WGPU `PresentMode` based on the requested `FpsLimit` and GPU capabilities.
/// Prioritizes `Mailbox` as first priority because it provides the highest frame-queue swap throughput
/// without causing DX12 driver teardown stalls. Falls back to `AutoNoVsync`, `Immediate`, and `Fifo`.
pub(crate) fn choose_present_mode(
    limit: FpsLimit,
    supported: &[wgpu::PresentMode],
) -> wgpu::PresentMode {
    let selected = match limit {
        FpsLimit::Uncapped | FpsLimit::Limit120 => {
            if supported.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else if supported.contains(&wgpu::PresentMode::AutoNoVsync) {
                wgpu::PresentMode::AutoNoVsync
            } else if supported.contains(&wgpu::PresentMode::Immediate) {
                wgpu::PresentMode::Immediate
            } else {
                wgpu::PresentMode::Fifo
            }
        }
        FpsLimit::Limit60 => {
            if supported.contains(&wgpu::PresentMode::AutoVsync) {
                wgpu::PresentMode::AutoVsync
            } else {
                wgpu::PresentMode::Fifo
            }
        }
    };
    selected
}

/// Determines the swap-chain frame latency (number of frames buffered ahead of display).
/// Uses `2` for all modes to allow double-buffered CPU-GPU pipelining.
pub(crate) fn choose_frame_latency(_limit: FpsLimit) -> u32 {
    2
}

impl RenderState {
    /// Initializes the full WGPU backend: adapter, device, surface, pipelines,
    /// shadow system, post-processing, and camera.
    pub async fn new(window: Arc<Window>) -> Result<(Self, Camera), String> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::from_build_config(),
            backend_options: wgpu::BackendOptions::default(),
            display: Default::default(),
            memory_budget_thresholds: Default::default(),
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter_opt = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await;

        let adapter = match adapter_opt {
            Ok(a) => a,
            Err(e) => {
                core::hint::cold_path();
                return Err(format!(
                    "No compatible graphics card found! Aeon Engine requires Vulkan, DX12, or Metal support.\nError: {:?}",
                    e
                ));
            }
        };

        let device_result = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::POLYGON_MODE_LINE
                    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: adapter.limits(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: Default::default(),
            })
            .await;

        let (device, queue) = match device_result {
            Ok(res) => res,
            Err(e) => {
                core::hint::cold_path();
                return Err(format!(
                    "Failed to establish graphics card connection:\n\n{}",
                    e
                ));
            }
        };

        let initial_msaa = 4;
        let mut graphics_settings = GraphicsSettings::default();
        graphics_settings.msaa_samples = initial_msaa;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let supported_present_modes = surface_caps.present_modes.clone();
        let present_mode =
            choose_present_mode(graphics_settings.fps_limit, &supported_present_modes);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            // Frame latency: 2 for all modes — enables double-buffered CPU-GPU pipelining.
            // CPU prepares frame N while GPU renders frame N-1, maximizing throughput.
            desired_maximum_frame_latency: choose_frame_latency(graphics_settings.fps_limit),
            color_space: wgpu::SurfaceColorSpace::Srgb,
        };
        surface.configure(&device, &config);

        // --- CAMERA SETUP ---
        let pos = cgmath::Point3::<f32>::new(0.0, 5.0, -10.0);
        let target = cgmath::Point3::<f32>::new(0.0, 0.0, 0.0);
        let dir = (target - pos).normalize();
        let pitch = dir.y.asin();
        let yaw = dir.z.atan2(dir.x);

        let camera = Camera {
            position: pos,
            yaw: cgmath::Rad(yaw),
            pitch: cgmath::Rad(pitch),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 2000.0,
            mode: crate::camera::ProjectionMode::Perspective,
            ortho_scale: 15.0,
            target: cgmath::Point3::new(0.0, 0.0, 0.0),
        };

        let uniforms = crate::render::uniforms::SceneUniforms::new(&device, &camera);
        let geometry = crate::render::primitives::GeometrySystem::new(&device);

        let shadow = crate::render::shadow::ShadowSystem::new(&device, &graphics_settings);

        let post_process = crate::render::post_process::PostProcessSystem::new(
            &device,
            &config,
            initial_msaa,
            graphics_settings.bloom_intensity,
        );

        let pipelines = crate::render::pipelines::PipelineManager::new(
            &device,
            &uniforms.camera_bind_group_layout,
            &uniforms.light_bind_group_layout,
            &shadow.shadow_bind_group_layout,
            &uniforms.texture_bind_group_layout,
            &uniforms.sky_bind_group_layout,
            post_process.scene_format,
            initial_msaa,
        );

        let outline = crate::render::pipelines::outline::SelectionOutlinePass::new(
            &device,
            post_process.scene_format,
            &uniforms.camera_bind_group_layout,
        );

        let default_white_cpu = ae_texture::FallbackTextureGenerator::white_1x1();
        let default_white_texture = crate::render::resources::upload_raw_texture(
            &device,
            &queue,
            &uniforms.texture_bind_group_layout,
            &default_white_cpu,
        );

        let state = Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            viewport_texture: None,
            pipelines,
            uniforms,
            geometry,
            default_white_texture,
            post_process,
            shadow,
            outline,
            graphics_settings,
            last_viewport_rect: super::ViewportRect::default(),
            supported_present_modes,
            last_present_wait_secs: 0.0,
        };
        Ok((state, camera))
    }

    /// Reconfigures the surface and resizes all render targets (MSAA, depth, bloom).
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.post_process
                .resize(&self.device, &self.config, self.post_process.msaa_samples);
        }
    }

    /// Checks if graphics settings changed and rebuilds GPU resources as needed.
    /// Returns `Some(new_msaa_count)` when MSAA sample count changed, so the caller
    /// (engine) can rebuild external systems (e.g. overlay renderers) that depend on MSAA.
    pub fn apply_settings_changes(&mut self) -> Option<u32> {
        let gs = &self.graphics_settings;
        let mut msaa_changed: Option<u32> = None;

        // --- MSAA Change ---
        if gs.msaa_samples != self.post_process.msaa_samples {
            let new_msaa = gs.msaa_samples.max(1);

            // 1. Resize post-process targets to update MSAA samples and textures
            self.post_process
                .resize(&self.device, &self.config, new_msaa);

            self.pipelines.rebuild_for_msaa(
                &self.device,
                &self.uniforms.camera_bind_group_layout,
                &self.uniforms.light_bind_group_layout,
                &self.shadow.shadow_bind_group_layout,
                &self.uniforms.texture_bind_group_layout,
                &self.uniforms.sky_bind_group_layout,
                self.post_process.scene_format,
                new_msaa,
            );

            msaa_changed = Some(new_msaa);
        }

        // --- Shadow Resolution Change ---
        let new_shadow_res = gs.shadow_resolution.as_u32();
        if self.shadow.shadow_depth_texture.size().width != new_shadow_res {
            self.shadow.resize_targets(&self.device, gs);
        }

        // --- FPS Limit / Present Mode Change ---
        let new_present_mode = choose_present_mode(gs.fps_limit, &self.supported_present_modes);
        let new_latency = choose_frame_latency(gs.fps_limit);
        if self.config.present_mode != new_present_mode
            || self.config.desired_maximum_frame_latency != new_latency
        {
            self.config.present_mode = new_present_mode;
            self.config.desired_maximum_frame_latency = new_latency;
            self.surface.configure(&self.device, &self.config);
            log::info!(
                "Present mode changed to {:?} (available: {:?}), frame latency={}",
                new_present_mode,
                self.supported_present_modes,
                new_latency
            );
        }

        msaa_changed
    }
}
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
    match limit {
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
    }
}

/// Default swap-chain frame latency (number of frames queued ahead on GPU).
/// Uses `2` to enable double-buffered CPU-GPU pipelining. CPU prepares frame N
/// while GPU renders frame N-1, maximizing throughput without input lag.
pub(crate) const DEFAULT_FRAME_LATENCY: u32 = 2;

/// Determines the swap-chain frame latency (number of frames buffered ahead of display).
/// Returns `DEFAULT_FRAME_LATENCY` (`2`) to allow double-buffered CPU-GPU pipelining.
#[inline]
pub(crate) fn choose_frame_latency(_limit: FpsLimit) -> u32 {
    DEFAULT_FRAME_LATENCY
}

/// Probes and logs all available graphics adapters enumerated on the system.
/// Discovers all hardware adapters across active WGPU backends (Vulkan, DirectX, Metal, OpenGL, CPU)
/// and writes their diagnostic metadata (device name, vendor/device IDs, device type, driver, driver info)
/// to standard engine logs. Returns the enumerated list of discovered adapter information structures
/// for upstream diagnostic analysis and hardware error reporting.
/// # Arguments
/// * `instance` - Reference to the initialized [`wgpu::Instance`].
/// # Returns
/// A vector of [`wgpu::AdapterInfo`] describing every detected physical or software graphics adapter.
/// If no adapters are discovered on the host system, the returned vector is empty.
pub async fn probe_and_log_adapters(instance: &wgpu::Instance) -> Vec<wgpu::AdapterInfo> {
    let adapters = instance.enumerate_adapters(wgpu::Backends::all()).await;
    let mut infos = Vec::with_capacity(adapters.len());

    if adapters.is_empty() {
        log::warn!(
            "[GPU PROBE] No graphics adapters discovered by WGPU instance enumeration across any backend."
        );
        return infos;
    }

    log::info!(
        "[GPU PROBE] Enumerating system graphics adapters (count = {}):",
        adapters.len()
    );

    for (idx, adapter) in adapters.into_iter().enumerate() {
        let info = adapter.get_info();
        log::info!(
            "  - Adapter #{idx}: '{}' | Backend: {:?} | Type: {:?} | Driver: '{}' ({}) | PCI ID: {:04x}:{:04x}",
            info.name,
            info.backend,
            info.device_type,
            info.driver,
            info.driver_info,
            info.vendor,
            info.device
        );
        infos.push(info);
    }

    infos
}

/// Attempts to pick the most optimal compatible graphics adapter from a list of pre-enumerated hardware adapters.
/// Filters adapters whose presentation capabilities match the window surface (`formats` is non-empty)
/// and prioritizes them according to modern rendering pipeline tiers:
/// 1. Vulkan Discrete GPU (dedicated hardware).
/// 2. Vulkan Integrated or alternative GPU.
/// 3. OpenGL (GL) backend adapter.
/// 4. Any remaining discrete GPU (e.g. Metal or DirectX).
/// 5. CPU / Software rasterizer fallback.
fn select_compatible_enumerated_adapter(
    adapters: Vec<wgpu::Adapter>,
    surface: &wgpu::Surface<'_>,
) -> Option<wgpu::Adapter> {
    let mut candidates: Vec<wgpu::Adapter> = adapters
        .into_iter()
        .filter(|adapter| !surface.get_capabilities(adapter).formats.is_empty())
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Tier 1: Vulkan Discrete GPU
    if let Some(pos) = candidates.iter().position(|a| {
        let info = a.get_info();
        info.backend == wgpu::Backend::Vulkan && info.device_type == wgpu::DeviceType::DiscreteGpu
    }) {
        return Some(candidates.remove(pos));
    }

    // Tier 2: Vulkan any GPU
    if let Some(pos) = candidates.iter().position(|a| {
        let info = a.get_info();
        info.backend == wgpu::Backend::Vulkan
    }) {
        return Some(candidates.remove(pos));
    }

    // Tier 3: OpenGL Backend
    if let Some(pos) = candidates.iter().position(|a| {
        let info = a.get_info();
        info.backend == wgpu::Backend::Gl
    }) {
        return Some(candidates.remove(pos));
    }

    // Tier 4: Discrete GPU on any other backend (Metal, DX12)
    if let Some(pos) = candidates
        .iter()
        .position(|a| a.get_info().device_type == wgpu::DeviceType::DiscreteGpu)
    {
        return Some(candidates.remove(pos));
    }

    // Tier 5: Remaining candidate (Integrated / CPU)
    candidates.pop()
}

/// Generates an exhaustive engineering diagnostic report when graphics adapter initialization fails across all tiers.
/// Inspects system environment variables, display servers (Wayland/X11), Linux Vulkan ICD manifests, and fallback error logs
/// to provide actionable guidance for developers and end users.
fn generate_hardware_diagnostic_report(
    discovered_adapters: &[wgpu::AdapterInfo],
    tier_errors: &[(&str, String)],
    decoupled_info: Option<&str>,
) -> String {
    use std::fmt::Write;
    let mut report = String::with_capacity(2048);
    let _ = writeln!(
        report,
        "\n================================================================================\n\
         [AEON ENGINE] GRAPHICS ADAPTER INITIALIZATION FAILURE\n\
         ================================================================================\n\
         No compatible graphics adapter could be initialized after evaluating all tiers.\n"
    );

    if let Some(info) = decoupled_info {
        let _ = writeln!(report, "--- DECOUPLED SURFACE DIAGNOSTIC ---\n  {info}\n");
    }

    let _ = writeln!(report, "--- EVALUATED FALLBACK TIERS & ERROR LOGS ---");
    for (tier_name, err_msg) in tier_errors {
        let _ = writeln!(report, "  * {tier_name}: {err_msg}");
    }

    let _ = writeln!(report, "\n--- DISCOVERED ADAPTERS (WGPU ENUMERATION) ---");
    if discovered_adapters.is_empty() {
        let _ = writeln!(
            report,
            "  (0 graphics adapters enumerated across all WGPU backends)"
        );
    } else {
        for (idx, info) in discovered_adapters.iter().enumerate() {
            let _ = writeln!(
                report,
                "  [{idx}] '{}' | Backend: {:?} | Type: {:?} | Driver: '{}' ({}) | PCI ID: {:04x}:{:04x}",
                info.name,
                info.backend,
                info.device_type,
                info.driver,
                info.driver_info,
                info.vendor,
                info.device
            );
        }
    }

    let _ = writeln!(report, "\n--- SYSTEM ENVIRONMENT & DRIVER DIAGNOSTICS ---");
    #[cfg(target_os = "linux")]
    {
        let wayland_disp = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let x11_disp = std::env::var("DISPLAY").unwrap_or_default();
        if !wayland_disp.is_empty() {
            let _ = writeln!(
                report,
                "  Display Protocol: Wayland (WAYLAND_DISPLAY=\"{wayland_disp}\")"
            );
        } else if !x11_disp.is_empty() {
            let _ = writeln!(report, "  Display Protocol: X11 (DISPLAY=\"{x11_disp}\")");
        } else {
            let _ = writeln!(
                report,
                "  Display Protocol: Unknown / Headless (neither WAYLAND_DISPLAY nor DISPLAY set)"
            );
        }

        let custom_icd = std::env::var("VK_ICD_FILENAMES").unwrap_or_default();
        if !custom_icd.is_empty() {
            let _ = writeln!(report, "  VK_ICD_FILENAMES: \"{custom_icd}\"");
        }

        let mut found_icds = Vec::new();
        for dir in &["/usr/share/vulkan/icd.d", "/etc/vulkan/icd.d"] {
            if let Ok(entries) = std::fs::read_dir(std::path::Path::new(dir)) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.ends_with(".json") {
                        found_icds.push(format!("{dir}/{name}"));
                    }
                }
            }
        }

        if found_icds.is_empty() {
            let _ = writeln!(
                report,
                "  Vulkan ICD Manifests: None found in standard directories (/usr/share/vulkan/icd.d, /etc/vulkan/icd.d)"
            );
        } else {
            let _ = writeln!(report, "  Detected Vulkan ICD Manifests:");
            for icd in &found_icds {
                let _ = writeln!(report, "    - {icd}");
            }
        }

        let _ = writeln!(
            report,
            "\n--- ACTIONABLE ROOT-CAUSE RESOLUTION GUIDANCE ---"
        );
        if found_icds.iter().any(|p| p.contains("nvidia")) && discovered_adapters.is_empty() {
            let _ = writeln!(
                report,
                "  * NVIDIA Driver / Kernel Module Mismatch:\n\
                   An NVIDIA Vulkan ICD manifest is present on disk, but physical device enumeration returned 0 adapters.\n\
                   This usually occurs when NVIDIA drivers are upgraded while an older kernel module remains active in RAM.\n\
                   -> Resolution: Reboot the system or reload NVIDIA kernel modules via modprobe."
            );
        }
        if found_icds.is_empty() {
            let _ = writeln!(
                report,
                "  * Missing Vulkan Drivers:\n\
                   No Vulkan ICD manifests were detected on the system.\n\
                   -> Resolution: Install hardware drivers (`nvidia-utils`, `vulkan-radeon`, or `vulkan-intel`)."
            );
        }
        let _ = writeln!(
            report,
            "  * Software Fallback Rasterizer:\n\
               To run using CPU rasterization without a functional GPU driver, install `vulkan-swrast` (Lavapipe) or `mesa-vulkan-drivers`.\n\
             * Hardware Verification Command:\n\
               Execute `vulkaninfo --summary` from your terminal to inspect system Vulkan driver health."
        );
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = writeln!(
            report,
            "--- ACTIONABLE ROOT-CAUSE RESOLUTION GUIDANCE ---\n\
             * Verify that GPU drivers are properly installed and up to date.\n\
             * Ensure hardware graphics acceleration is enabled in OS settings."
        );
    }

    let _ = writeln!(
        report,
        "================================================================================"
    );
    report
}

/// Evaluates a single adapter acquisition tier against window presentation capabilities.
/// If an adapter is returned but its surface presentation capabilities are empty, the candidate
/// is rejected and a descriptive diagnostic entry is appended to `tier_errors`.
async fn try_request_tier(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
    options: wgpu::RequestAdapterOptions<'_, '_>,
    tier_name: &'static str,
    tier_errors: &mut Vec<(&'static str, String)>,
) -> Option<wgpu::Adapter> {
    match instance.request_adapter(&options).await {
        Ok(adapter) if !surface.get_capabilities(&adapter).formats.is_empty() => Some(adapter),
        Ok(_) => {
            tier_errors.push((
                tier_name,
                "Acquired adapter cannot present to the surface (0 formats)".to_string(),
            ));
            None
        }
        Err(e) => {
            tier_errors.push((tier_name, format!("{e:?}")));
            None
        }
    }
}

/// Requests a graphics adapter using a resilient multi-tier fallback strategy that traverses
/// Vulkan discrete, Vulkan integrated, OpenGL, and software rasterizers before producing a rich diagnostic.
/// # Arguments
/// * `instance` - Active WGPU instance.
/// * `surface` - Window presentation surface to validate presentation capability against.
/// * `discovered_adapters` - Cached adapter descriptions from prior hardware enumeration.
/// # Returns
/// Returns the acquired [`wgpu::Adapter`] on success, or an actionable diagnostic error string.
pub async fn request_adapter_resilient(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
    discovered_adapters: &[wgpu::AdapterInfo],
) -> Result<wgpu::Adapter, String> {
    // Stage A: Candidate selection among pre-enumerated adapters
    let enumerated = instance.enumerate_adapters(wgpu::Backends::all()).await;
    if let Some(adapter) = select_compatible_enumerated_adapter(enumerated, surface) {
        let info = adapter.get_info();
        log::info!(
            "[GPU SETUP] Acquired compatible adapter via enumeration: '{}' (Backend: {:?}, Type: {:?})",
            info.name,
            info.backend,
            info.device_type
        );
        return Ok(adapter);
    }

    let mut tier_errors: Vec<(&'static str, String)> = Vec::with_capacity(5);

    let tiers = [
        (
            wgpu::PowerPreference::HighPerformance,
            false,
            "Tier 1 (High Performance Discrete)",
        ),
        (
            wgpu::PowerPreference::LowPower,
            false,
            "Tier 2 (Integrated / Low Power)",
        ),
        (
            wgpu::PowerPreference::None,
            false,
            "Tier 3 (Default / OpenGL)",
        ),
        (
            wgpu::PowerPreference::None,
            true,
            "Tier 4 (Software Fallback)",
        ),
    ];

    for (power_preference, force_fallback_adapter, tier_name) in tiers {
        if let Some(adapter) = try_request_tier(
            instance,
            surface,
            wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(surface),
                force_fallback_adapter,
                ..Default::default()
            },
            tier_name,
            &mut tier_errors,
        )
        .await
        {
            let info = adapter.get_info();
            log::info!(
                "[GPU SETUP] {tier_name} acquired: '{}' (Backend: {:?}, Type: {:?})",
                info.name,
                info.backend,
                info.device_type
            );
            return Ok(adapter);
        }
    }

    // Stage C: Decoupled probe (without surface) to detect if GPU exists physically
    let decoupled_check = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
            ..Default::default()
        })
        .await;

    let decoupled_info = match decoupled_check {
        Ok(headless) => {
            let info = headless.get_info();
            Some(format!(
                "Hardware GPU detected in headless mode: '{}' | Backend: {:?} | Driver: '{}'. However, this adapter cannot present to the window surface.",
                info.name, info.backend, info.driver
            ))
        }
        Err(e) => {
            tier_errors.push(("Decoupled Headless Probe", format!("{e:?}")));
            None
        }
    };

    core::hint::cold_path();
    Err(generate_hardware_diagnostic_report(
        discovered_adapters,
        &tier_errors,
        decoupled_info.as_deref(),
    ))
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
            .map_err(|e| format!("Failed to create window presentation surface: {e}"))?;

        let adapter_infos = probe_and_log_adapters(&instance).await;
        let adapter = request_adapter_resilient(&instance, &surface, &adapter_infos).await?;

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
        let graphics_settings = GraphicsSettings {
            msaa_samples: initial_msaa,
            ..Default::default()
        };

        let surface_caps = surface.get_capabilities(&adapter);
        if surface_caps.formats.is_empty() {
            core::hint::cold_path();
            return Err(format!(
                "The selected graphics adapter '{}' ({:?}) cannot present to the window surface (0 compatible surface formats).",
                adapter.get_info().name,
                adapter.get_info().backend
            ));
        }

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let supported_present_modes = surface_caps.present_modes;
        let present_mode =
            choose_present_mode(graphics_settings.fps_limit, &supported_present_modes);
        let alpha_mode = surface_caps
            .alpha_modes
            .first()
            .copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
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

        let shadow = crate::render::shadow::ShadowSystem::new(
            &device,
            &graphics_settings,
            &uniforms.texture_bind_group_layout,
        );

        let post_process = crate::render::post_process::PostProcessSystem::new(
            &device,
            &config,
            initial_msaa,
            graphics_settings.bloom_intensity,
        );

        let pipelines = crate::render::pipelines::PipelineManager::new(
            &device,
            &crate::render::pipelines::PipelineConfigParams {
                camera_bgl: &uniforms.camera_bind_group_layout,
                light_bgl: &uniforms.light_bind_group_layout,
                shadow_bgl: &shadow.shadow_bind_group_layout,
                texture_bgl: &uniforms.texture_bind_group_layout,
                sky_bgl: &uniforms.sky_bind_group_layout,
                scene_format: post_process.scene_format,
                msaa_samples: initial_msaa,
            },
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

        let adapter_info = adapter.get_info();

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
            last_render_stats: super::types::FrameRenderStats::default(),
            last_gpu_pass_timings: ae_core::telemetry::GpuPassTimings::default(),
            adapter_info,
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
                &crate::render::pipelines::PipelineConfigParams {
                    camera_bgl: &self.uniforms.camera_bind_group_layout,
                    light_bgl: &self.uniforms.light_bind_group_layout,
                    shadow_bgl: &self.shadow.shadow_bind_group_layout,
                    texture_bgl: &self.uniforms.texture_bind_group_layout,
                    sky_bgl: &self.uniforms.sky_bind_group_layout,
                    scene_format: self.post_process.scene_format,
                    msaa_samples: new_msaa,
                },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_and_log_adapters_execution() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::from_build_config(),
            backend_options: wgpu::BackendOptions::default(),
            display: Default::default(),
            memory_budget_thresholds: Default::default(),
        });
        let infos = pollster::block_on(probe_and_log_adapters(&instance));
        // Ensure probing returns a valid list without panicking
        assert!(infos.len() <= 64);
    }

    #[test]
    fn test_choose_present_mode_priorities() {
        let empty_modes: Vec<wgpu::PresentMode> = vec![];
        let chosen_empty = choose_present_mode(FpsLimit::Limit60, &empty_modes);
        assert_eq!(chosen_empty, wgpu::PresentMode::Fifo);

        let mailbox_available = vec![
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::Immediate,
            wgpu::PresentMode::Mailbox,
        ];
        let chosen_uncapped = choose_present_mode(FpsLimit::Uncapped, &mailbox_available);
        assert_eq!(chosen_uncapped, wgpu::PresentMode::Mailbox);

        let auto_vsync_available = vec![wgpu::PresentMode::Fifo, wgpu::PresentMode::AutoVsync];
        let chosen_60 = choose_present_mode(FpsLimit::Limit60, &auto_vsync_available);
        assert_eq!(chosen_60, wgpu::PresentMode::AutoVsync);
    }

    #[test]
    fn test_frame_latency_invariant() {
        assert_eq!(
            choose_frame_latency(FpsLimit::Limit60),
            DEFAULT_FRAME_LATENCY
        );
        assert_eq!(
            choose_frame_latency(FpsLimit::Uncapped),
            DEFAULT_FRAME_LATENCY
        );
    }

    #[test]
    fn test_hardware_diagnostic_report_generation() {
        let errs = [("Tier 1", "Surface format error".to_string())];
        let report = generate_hardware_diagnostic_report(&[], &errs, Some("Headless GPU detected"));
        assert!(report.contains("GRAPHICS ADAPTER INITIALIZATION FAILURE"));
        assert!(report.contains("Headless GPU detected"));
        assert!(report.contains("Tier 1"));
    }
}
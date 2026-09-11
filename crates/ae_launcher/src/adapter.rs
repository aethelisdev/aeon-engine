// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Resilient graphics adapter discovery, multi-tier fallback, and hardware diagnostic reporting for the launcher.
//!
//! Provides automated traversal across Vulkan Discrete GPU, Vulkan Integrated GPU, OpenGL, and software
//! rasterizers to guarantee launcher execution across varied Linux/Unix GPU driver configurations without panicking.
//!

use std::fmt::Write;

/// Multi-tier fallback strategy for launcher graphics adapter acquisition.
/// Enumerates physical hardware adapters first, prioritizing high-performance Vulkan adapters,
/// followed by OpenGL backends and software rasterizer fallbacks.
pub async fn request_launcher_adapter_and_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), String> {
    // 1. Enumerate and log all available adapters
    let enumerated = instance.enumerate_adapters(wgpu::Backends::all()).await;
    let mut discovered_infos = Vec::with_capacity(enumerated.len());

    for (idx, adapter) in enumerated.iter().enumerate() {
        let info = adapter.get_info();
        log::info!(
            "[LAUNCHER GPU] Discovered Adapter #{idx}: '{}' (Backend: {:?}, DeviceType: {:?}, Driver: '{}')",
            info.name,
            info.backend,
            info.device_type,
            info.driver
        );
        discovered_infos.push(info);
    }

    // 2. Filter candidates compatible with launcher surface presentation
    let mut candidates: Vec<wgpu::Adapter> = enumerated
        .into_iter()
        .filter(|adapter| !surface.get_capabilities(adapter).formats.is_empty())
        .collect();

    let mut selected_adapter: Option<wgpu::Adapter> = None;

    // Tier 1: Vulkan Discrete GPU
    if let Some(pos) = candidates.iter().position(|a| {
        let info = a.get_info();
        info.backend == wgpu::Backend::Vulkan && info.device_type == wgpu::DeviceType::DiscreteGpu
    }) {
        selected_adapter = Some(candidates.remove(pos));
    }

    // Tier 2: Vulkan Integrated or alternative GPU
    if selected_adapter.is_none()
        && let Some(pos) = candidates
            .iter()
            .position(|a| a.get_info().backend == wgpu::Backend::Vulkan)
    {
        selected_adapter = Some(candidates.remove(pos));
    }

    // Tier 3: OpenGL Backend
    if selected_adapter.is_none()
        && let Some(pos) = candidates
            .iter()
            .position(|a| a.get_info().backend == wgpu::Backend::Gl)
    {
        selected_adapter = Some(candidates.remove(pos));
    }

    // Tier 4: Any discrete GPU on other backends
    if selected_adapter.is_none()
        && let Some(pos) = candidates
            .iter()
            .position(|a| a.get_info().device_type == wgpu::DeviceType::DiscreteGpu)
    {
        selected_adapter = Some(candidates.remove(pos));
    }

    // Tier 5: Any remaining compatible candidate
    if selected_adapter.is_none() && !candidates.is_empty() {
        selected_adapter = candidates.pop();
    }

    // Tier 6: Direct instance request fallbacks if enumeration yielded no compatible adapter
    let mut tier_errors = Vec::new();
    if selected_adapter.is_none() {
        let fallback_tiers = [
            (
                "Vulkan/HighPerformance",
                wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                },
            ),
            (
                "Vulkan/LowPower",
                wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(surface),
                    force_fallback_adapter: false,
                    apply_limit_buckets: false,
                },
            ),
            (
                "Software Rasterizer Fallback",
                wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::None,
                    compatible_surface: Some(surface),
                    force_fallback_adapter: true,
                    apply_limit_buckets: false,
                },
            ),
        ];

        for (tier_name, options) in fallback_tiers {
            match instance.request_adapter(&options).await {
                Ok(adapter) if !surface.get_capabilities(&adapter).formats.is_empty() => {
                    selected_adapter = Some(adapter);
                    break;
                }
                Ok(_) => {
                    tier_errors.push((
                        tier_name,
                        "Acquired adapter cannot present to launcher surface (0 formats)"
                            .to_string(),
                    ));
                }
                Err(err) => {
                    tier_errors.push((tier_name, format!("{err:?}")));
                }
            }
        }
    }

    let adapter = match selected_adapter {
        Some(a) => a,
        None => {
            let report = generate_launcher_diagnostic_report(&discovered_infos, &tier_errors);
            return Err(report);
        }
    };

    let adapter_info = adapter.get_info();
    log::info!(
        "[LAUNCHER GPU] Successfully selected adapter: '{}' (Backend: {:?}, Type: {:?})",
        adapter_info.name,
        adapter_info.backend,
        adapter_info.device_type
    );

    let device_descriptor = wgpu::DeviceDescriptor {
        label: Some("Aeon Launcher Device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
        experimental_features: Default::default(),
    };

    match adapter.request_device(&device_descriptor).await {
        Ok((device, queue)) => Ok((adapter, device, queue)),
        Err(e) => {
            let mut report = String::with_capacity(1024);
            let _ = writeln!(
                report,
                "\n================================================================================"
            );
            let _ = writeln!(
                report,
                "            AEON LAUNCHER WGPU DEVICE REQUEST ERROR"
            );
            let _ = writeln!(
                report,
                "================================================================================"
            );
            let _ = writeln!(
                report,
                "Selected Adapter: '{}' (Backend: {:?})",
                adapter_info.name, adapter_info.backend
            );
            let _ = writeln!(report, "Error Details: {e}");
            let _ = writeln!(
                report,
                "================================================================================"
            );
            Err(report)
        }
    }
}

/// Generates an informative diagnostic report when graphics adapter initialization fails.
fn generate_launcher_diagnostic_report(
    discovered: &[wgpu::AdapterInfo],
    tier_errors: &[(&str, String)],
) -> String {
    let mut report = String::with_capacity(2048);
    let _ = writeln!(
        report,
        "\n================================================================================"
    );
    let _ = writeln!(
        report,
        "            AEON LAUNCHER GRAPHICS INITIALIZATION FAILURE"
    );
    let _ = writeln!(
        report,
        "================================================================================"
    );
    let _ = writeln!(
        report,
        "The Aeon Engine Launcher failed to obtain a compatible graphics adapter."
    );
    let _ = writeln!(report, "\n--- HARDWARE PROBE RESULTS ---");
    if discovered.is_empty() {
        let _ = writeln!(
            report,
            "No physical graphics adapters were enumerated by WGPU."
        );
    } else {
        for (i, info) in discovered.iter().enumerate() {
            let _ = writeln!(
                report,
                "  Adapter #{i}: '{}' [Backend: {:?}, Type: {:?}, Driver: '{}']",
                info.name, info.backend, info.device_type, info.driver
            );
        }
    }

    if !tier_errors.is_empty() {
        let _ = writeln!(report, "\n--- TIER FALLBACK ATTEMPTS ---");
        for (tier, err) in tier_errors {
            let _ = writeln!(report, "  * Tier '{tier}': {err}");
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = writeln!(report, "\n--- SYSTEM ENVIRONMENT ---");
        let wayland_disp = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
        let x11_disp = std::env::var("DISPLAY").unwrap_or_default();
        if !wayland_disp.is_empty() {
            let _ = writeln!(
                report,
                "  Display Server: Wayland (WAYLAND_DISPLAY=\"{wayland_disp}\")"
            );
        } else if !x11_disp.is_empty() {
            let _ = writeln!(report, "  Display Server: X11 (DISPLAY=\"{x11_disp}\")");
        } else {
            let _ = writeln!(report, "  Display Server: Unknown / Headless");
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

        if !found_icds.is_empty() {
            let _ = writeln!(report, "  Vulkan ICD Manifests:");
            for icd in &found_icds {
                let _ = writeln!(report, "    - {icd}");
            }
        }

        let _ = writeln!(report, "\n--- ACTIONABLE RESOLUTION GUIDANCE ---");
        if found_icds.iter().any(|p| p.contains("nvidia")) && discovered.is_empty() {
            let _ = writeln!(
                report,
                "  * NVIDIA Driver / Kernel Module Mismatch:\n\
                   An NVIDIA Vulkan ICD manifest exists, but physical enumeration returned 0 adapters.\n\
                   -> Resolution: Reboot the system to align the NVIDIA kernel module with updated user-space drivers."
            );
        } else {
            let _ = writeln!(
                report,
                "  * Ensure compatible graphics drivers are installed (`nvidia-utils`, `vulkan-radeon`, or `vulkan-intel`).\n\
                 * For CPU rasterization fallback, install `vulkan-swrast` (Lavapipe) or `mesa-vulkan-drivers`.\n\
                 * Verify Vulkan driver functionality using `vulkaninfo --summary`."
            );
        }
    }

    let _ = writeln!(
        report,
        "================================================================================"
    );
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the launcher diagnostic report contains key structural headings.
    #[test]
    fn test_launcher_diagnostic_report_formatting() {
        let discovered = Vec::new();
        let tier_errors = vec![("Vulkan/LowPower", "No adapter found".to_string())];
        let report = generate_launcher_diagnostic_report(&discovered, &tier_errors);

        assert!(report.contains("AEON LAUNCHER GRAPHICS INITIALIZATION FAILURE"));
        assert!(report.contains("HARDWARE PROBE RESULTS"));
        assert!(report.contains("TIER FALLBACK ATTEMPTS"));
        assert!(report.contains("Vulkan/LowPower"));
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
use ae_light::{
    CascadeShadowConfig, EnvironmentLightConfig, PcfQuality, ShadowResolution, SkyQuality,
};

/// Frame rate limit options for the render loop.
///
/// Applied via winit present mode / frame pacing. `Uncapped` allows
/// the GPU to render as fast as possible (may cause tearing without VSync).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FpsLimit {
    Limit60,
    Limit120,
    Uncapped,
}

impl FpsLimit {
    pub fn label(self) -> &'static str {
        match self {
            FpsLimit::Limit60 => "60 FPS",
            FpsLimit::Limit120 => "120 FPS",
            FpsLimit::Uncapped => "Uncapped",
        }
    }

    /// Returns the target frame-rate cap in Hertz, or `None` when uncapped.
    #[must_use]
    pub const fn target_hz(self) -> Option<f32> {
        match self {
            FpsLimit::Limit60 => Some(60.0),
            FpsLimit::Limit120 => Some(120.0),
            FpsLimit::Uncapped => None,
        }
    }
}

/// Aggregate runtime graphics configuration exposed in the Settings panel.
///
/// Controls shadow mapping, MSAA, bloom post-processing, FPS limiting,
/// sky atmosphere, sun parameters, and fog. Modified at runtime via
/// `EngineUiAction::UpdateGraphicsSettings`. All changes take effect
/// on the next frame without requiring pipeline recreation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GraphicsSettings {
    pub shadow_enabled: bool,
    pub shadow_resolution: ShadowResolution,
    pub shadow_pcf: PcfQuality,
    pub shadow_bias: f32,
    pub shadow_cascades: u32,
    pub shadow_cascade_splits: [f32; 4],

    pub msaa_samples: u32, // 1, 2, or 4

    pub bloom_enabled: bool,
    pub bloom_intensity: f32,

    pub fps_limit: FpsLimit,
    pub environment_color: [f32; 3],

    pub sky_quality: SkyQuality,
    pub sun_pitch: f32,
    pub sun_yaw: f32,
    pub atmosphere_density: f32,
    pub ozone_density: f32,
    pub sun_disc_size: f32,
    pub sun_glow_strength: f32,

    pub cloud_coverage: f32,
    pub cloud_density: f32,
    pub cloud_speed: f32,
    pub cloud_evolution: f32,
    pub cloud_altitude: f32,
    pub cloud_thickness: f32,

    pub fog_enabled: bool,
    pub fog_distance: f32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            shadow_enabled: true,
            shadow_resolution: ShadowResolution::High,
            shadow_pcf: PcfQuality::Soft,
            shadow_bias: 0.005,
            shadow_cascades: 4,
            shadow_cascade_splits: [4.0, 15.0, 50.0, 150.0],

            msaa_samples: 4,

            bloom_enabled: true,
            bloom_intensity: 1.0,

            fps_limit: FpsLimit::Limit120,
            environment_color: [0.12, 0.35, 0.65], // Natural Earth sky tone, less "neon/sterile" blue

            sky_quality: SkyQuality::High,
            sun_pitch: 0.5,
            sun_yaw: 0.5,
            atmosphere_density: 1.0, // Standard Earth atmosphere multiplier (1.0 = Default)
            ozone_density: 1.0,      // Physical Chappuis ozone absorption multiplier
            sun_disc_size: 1.0,      // Standard natural sun size multiplier (1.0 = Default)
            sun_glow_strength: 1.0,  // Standard sun glow halo multiplier (1.0 = Default)

            cloud_coverage: 0.45,
            cloud_density: 1.0,
            cloud_speed: 1.0,
            cloud_evolution: 1.0,
            cloud_altitude: 1500.0,
            cloud_thickness: 3000.0,

            fog_enabled: true,
            fog_distance: 800.0,
        }
    }
}

impl GraphicsSettings {
    /// Extracts the CPU shadow cascade configuration for [`ae_light::compute_cascade_light_space`].
    #[must_use]
    pub const fn to_cascade_config(&self) -> CascadeShadowConfig {
        CascadeShadowConfig {
            enabled: self.shadow_enabled,
            resolution: self.shadow_resolution,
            pcf: self.shadow_pcf,
            bias: self.shadow_bias,
            cascade_count: self.shadow_cascades,
            cascade_splits: self.shadow_cascade_splits,
        }
    }

    /// Extracts environment, sun, cloud, and fog parameters for [`ae_light::evaluate_environment`].
    #[must_use]
    pub const fn to_environment_light_config(&self) -> EnvironmentLightConfig {
        EnvironmentLightConfig {
            environment_color: self.environment_color,
            sky_quality: self.sky_quality,
            sun_pitch: self.sun_pitch,
            sun_yaw: self.sun_yaw,
            atmosphere_density: self.atmosphere_density,
            ozone_density: self.ozone_density,
            sun_disc_size: self.sun_disc_size,
            sun_glow_strength: self.sun_glow_strength,
            cloud_coverage: self.cloud_coverage,
            cloud_density: self.cloud_density,
            cloud_speed: self.cloud_speed,
            cloud_evolution: self.cloud_evolution,
            cloud_altitude: self.cloud_altitude,
            cloud_thickness: self.cloud_thickness,
            fog_enabled: self.fog_enabled,
            fog_distance: self.fog_distance,
        }
    }
}
// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
/// Shadow map resolution tiers controlling texture size and quality.
/// Higher resolutions produce sharper shadows but consume more GPU memory.
/// The `as_u32()` method returns the texture dimension directly.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShadowResolution {
    Low = 512,
    Medium = 1024,
    High = 2048,
    Ultra = 4096,
}

impl ShadowResolution {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn label(self) -> &'static str {
        match self {
            ShadowResolution::Low => "Low (512)",
            ShadowResolution::Medium => "Medium (1024)",
            ShadowResolution::High => "High (2048)",
            ShadowResolution::Ultra => "Ultra (4096)",
        }
    }
}

/// Percentage Closer Filtering quality for shadow edge softness.
/// Controls the PCF kernel size in the shadow shader. Higher quality
/// produces softer shadow edges at the cost of more texture samples.
/// The `radius()` method returns the half-kernel size for the shader.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PcfQuality {
    Off,       // No filtering - sharp but aliased
    Soft,      // 3x3 kernel
    UltraSoft, // 5x5 kernel
}

impl PcfQuality {
    pub fn radius(self) -> i32 {
        match self {
            PcfQuality::Off => 0,
            PcfQuality::Soft => 1,
            PcfQuality::UltraSoft => 2,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PcfQuality::Off => "Off (Sharp)",
            PcfQuality::Soft => "3x3 Soft",
            PcfQuality::UltraSoft => "5x5 Ultra Soft",
        }
    }
}

/// Frame rate limit options for the render loop.
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
}

/// Atmospheric sky rendering quality levels.
/// Controls the complexity of the sky shader: from a simple gradient
/// to a physically-based Rayleigh/Mie scattering model.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SkyQuality {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl SkyQuality {
    pub fn label(self) -> &'static str {
        match self {
            SkyQuality::Low => "Low (Gradient)",
            SkyQuality::Medium => "Medium (Fast HDR)",
            SkyQuality::High => "High (Atmospheric)",
        }
    }
}

/// Aggregate runtime graphics configuration exposed in the Settings panel.
/// Controls shadow mapping, MSAA, bloom post-processing, FPS limiting,
/// sky atmosphere, sun parameters, and fog. Modified at runtime via
/// `EngineUiAction::UpdateGraphicsSettings`. All changes take effect
/// on the next frame without requiring pipeline recreation.
#[derive(Clone, Debug, PartialEq)]
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
    pub sun_disc_size: f32,
    pub sun_glow_strength: f32,

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
            atmosphere_density: 0.08, // Subtle but visible horizon atmosphere
            sun_disc_size: 0.05,
            sun_glow_strength: 0.15,

            fog_enabled: true,
            fog_distance: 800.0,
        }
    }
}
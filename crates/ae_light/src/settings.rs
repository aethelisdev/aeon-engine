// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Lighting, shadow quality, and procedural sky tier enumerations.
//!
//! Shared across renderer configuration, cascade computation, atmosphere evaluation,
//! and declarative graphics preference views without pulling `wgpu` into UI/data layers.

/// Shadow map resolution tiers controlling depth texture dimensions and shadow sharpness.
///
/// Higher tiers yield finer shadow edges at proportionally higher VRAM bandwidth and memory cost.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShadowResolution {
    /// 512x512 depth array per cascade.
    Low = 512,
    /// 1024x1024 depth array per cascade.
    Medium = 1024,
    /// 2048x2048 depth array per cascade.
    High = 2048,
    /// 4096x4096 depth array per cascade.
    Ultra = 4096,
}

impl ShadowResolution {
    /// Returns the square depth texture dimension in texels for this resolution tier.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Returns a human-readable display label suitable for settings UI dropdowns.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            ShadowResolution::Low => "Low (512)",
            ShadowResolution::Medium => "Medium (1024)",
            ShadowResolution::High => "High (2048)",
            ShadowResolution::Ultra => "Ultra (4096)",
        }
    }
}

/// Percentage-Closer Filtering (PCF) quality presets controlling shadow edge softness.
///
/// Higher tiers increase texture tap kernel radius in the PBR fragment shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PcfQuality {
    /// Single depth comparison tap (`radius = 0`).
    Off,
    /// 3x3 neighborhood filtering (`radius = 1`).
    Soft,
    /// 5x5 neighborhood filtering (`radius = 2`).
    UltraSoft,
}

impl PcfQuality {
    /// Returns the half-kernel sample radius consumed by `LightSpaceUniform::pcf_radius`.
    #[must_use]
    pub const fn radius(self) -> i32 {
        match self {
            PcfQuality::Off => 0,
            PcfQuality::Soft => 1,
            PcfQuality::UltraSoft => 2,
        }
    }

    /// Returns a human-readable display label suitable for settings UI dropdowns.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            PcfQuality::Off => "Off (Sharp)",
            PcfQuality::Soft => "3x3 Soft",
            PcfQuality::UltraSoft => "5x5 Ultra Soft",
        }
    }
}

/// Atmospheric sky shader quality tiers controlling scattering and cloud complexity.
///
/// Numeric discriminants match `SkyUniform::sky_quality_mode` consumed by `sky.wgsl`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SkyQuality {
    /// Simple two-stop sky gradient (`0`).
    Low = 0,
    /// Fast analytical HDR sky scattering (`1`).
    Medium = 1,
    /// Physical Rayleigh/Mie/ozone scattering with 2.5D procedural clouds (`2`).
    High = 2,
    /// Reserved 2D panoramic environment mode (`3`).
    Panoramic = 3,
    /// Reserved 3D volumetric raymarched clouds mode (`4`).
    Volumetric = 4,
}

impl SkyQuality {
    /// Returns a human-readable display label suitable for settings UI dropdowns.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            SkyQuality::Low => "Low (Gradient)",
            SkyQuality::Medium => "Medium (Fast HDR)",
            SkyQuality::High => "High (Atmospheric 2.5D)",
            SkyQuality::Panoramic => "2D Panoramic (Planned)",
            SkyQuality::Volumetric => "3D Volumetric (Planned)",
        }
    }
}
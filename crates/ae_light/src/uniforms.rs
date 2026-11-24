// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! WebGPU-aligned CPU-side uniform layouts for scene lighting, cascaded shadow maps, and sky shaders.
//!
//! All structures satisfy 16-byte WebGPU uniform alignment invariants and implement `bytemuck::Pod`
//! without requiring a `wgpu` dependency.

/// GPU uniform buffer layout for scene directional lighting, ambient fill, and atmospheric fog.
///
/// Matches the WGSL `Light` uniform layout across `shader.wgsl`, `grid_shader.wgsl`, and `sprite_shader.wgsl`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    /// Normalized vector pointing towards the primary directional light source.
    pub direction: [f32; 3],
    /// Viewport ground grid scale multiplier consumed by `grid_shader.wgsl`.
    pub grid_scale: f32,
    /// Direct directional light RGB color and intensity multiplier.
    pub color: [f32; 3],
    /// Explicit 16-byte WebGPU uniform padding.
    pub _padding2: u32,
    /// Ambient environment fill RGB color.
    pub ambient_color: [f32; 3],
    /// Explicit 16-byte WebGPU uniform padding.
    pub _padding3: u32,
    /// Distance fog parameters: `[r, g, b, distance]` (`distance == 0.0` disables fog).
    pub fog_params: [f32; 4],
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            direction: [0.0, 1.0, 0.0],
            grid_scale: 1.0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
            ambient_color: [0.1, 0.1, 0.15],
            _padding3: 0,
            fog_params: [0.0; 4],
        }
    }
}

/// GPU uniform buffer layout for Cascaded Shadow Map (CSM) matrices and sampling parameters.
///
/// Strictly 288 bytes (`16`-byte aligned) matching `LightSpaceUniform` in `shader.wgsl`.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightSpaceUniform {
    /// Light-space view-projection matrices (`OPENGL_TO_WGPU_MATRIX * proj * view`) for up to 4 cascades.
    pub matrices: [[[f32; 4]; 4]; 4],
    /// View-space Z split distances corresponding to each shadow cascade.
    pub cascade_splits: [f32; 4],
    /// Depth bias applied during shadow depth comparison to suppress shadow acne.
    pub shadow_bias: f32,
    /// Half-kernel PCF filter radius (`0`, `1`, or `2`).
    pub pcf_radius: i32,
    /// Boolean shadow toggle serialized as `0` or `1`.
    pub shadow_enabled: u32,
    /// Active cascade count (`1..=4`).
    pub cascade_count: u32,
}

impl Default for LightSpaceUniform {
    fn default() -> Self {
        Self {
            matrices: [[[0.0; 4]; 4]; 4],
            cascade_splits: [4.0, 15.0, 50.0, 150.0],
            shadow_bias: 0.005,
            pcf_radius: 1,
            shadow_enabled: 1,
            cascade_count: 4,
        }
    }
}

/// GPU uniform buffer layout for procedural sky scattering and 2.5D cloud layers (`sky.wgsl`).
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyUniform {
    /// Normalized vector pointing towards the sun (`xyz`), `w` reserved for alignment padding.
    pub sun_direction: [f32; 4],
    /// Solar disc RGB tint (`xyz`) and HDR intensity multiplier (`w`).
    pub sun_color: [f32; 4],
    /// Horizon tint RGB (`xyz`) and reserved `w`.
    pub horizon_color: [f32; 4],
    /// Zenith tint RGB (`xyz`) and reserved `w`.
    pub zenith_color: [f32; 4],
    /// Atmospheric Rayleigh/Mie scattering density multiplier.
    pub atmosphere_density: f32,
    /// Physical Chappuis ozone absorption multiplier.
    pub ozone_density: f32,
    /// Angular sun disc size multiplier.
    pub sun_disc_size: f32,
    /// Mie forward-scattering solar glow strength multiplier.
    pub sun_glow_strength: f32,
    /// Normalized procedural cloud coverage fraction (`0.0..=1.0`).
    pub cloud_coverage: f32,
    /// Cloud optical density multiplier.
    pub cloud_density: f32,
    /// Horizontal cloud wind drift speed multiplier.
    pub cloud_speed: f32,
    /// Temporal cloud noise evolution multiplier.
    pub cloud_evolution: f32,
    /// Cloud deck base altitude in meters.
    pub cloud_altitude: f32,
    /// Cloud deck vertical thickness in meters.
    pub cloud_thickness: f32,
    /// Monotonic elapsed seconds driving cloud animation.
    pub time: f32,
    /// Sky quality mode discriminant matching `SkyQuality as u32`.
    pub sky_quality_mode: u32,
}

impl Default for SkyUniform {
    fn default() -> Self {
        Self {
            sun_direction: [0.0, 1.0, 0.0, 0.0],
            sun_color: [1.0, 1.0, 1.0, 100.0],
            horizon_color: [0.5, 0.6, 0.7, 0.0],
            zenith_color: [0.1, 0.2, 0.4, 0.0],
            atmosphere_density: 1.0,
            ozone_density: 1.0,
            sun_disc_size: 1.0,
            sun_glow_strength: 1.0,
            cloud_coverage: 0.45,
            cloud_density: 1.0,
            cloud_speed: 1.0,
            cloud_evolution: 1.0,
            cloud_altitude: 1500.0,
            cloud_thickness: 3000.0,
            time: 0.0,
            sky_quality_mode: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LightSpaceUniform, LightUniform, SkyUniform};

    #[test]
    fn test_gpu_uniform_sizes_and_16_byte_alignment() {
        assert_eq!(
            std::mem::size_of::<LightUniform>(),
            64,
            "LightUniform must match 64-byte WGSL uniform layout"
        );
        assert_eq!(std::mem::size_of::<LightUniform>() % 16, 0);

        assert_eq!(
            std::mem::size_of::<LightSpaceUniform>(),
            288,
            "LightSpaceUniform must match 288-byte shader.wgsl layout"
        );
        assert_eq!(std::mem::size_of::<LightSpaceUniform>() % 16, 0);

        assert_eq!(
            std::mem::size_of::<SkyUniform>(),
            112,
            "SkyUniform must match 112-byte sky.wgsl uniform layout"
        );
        assert_eq!(std::mem::size_of::<SkyUniform>() % 16, 0);
    }
}
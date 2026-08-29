// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! # GPU Uniform Structures
//!
//! Defines memory-aligned GPU uniform buffers for scene lighting, shadow cascades, atmosphere, and depth textures.

/// Default depth stencil format across the rendering pipeline.
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// GPU uniform for scene lighting: directional sun, ambient fill, and fog parameters.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    /// A normalized vector pointing TOWARDS the light source.
    /// This convention is shared across Sky, PBR shading, and Shadow Cascades.
    pub direction: [f32; 3],
    pub _padding: u32,

    /// The direct sunlight color and intensity multiplier.
    pub color: [f32; 3],
    pub _padding2: u32,

    pub ambient_color: [f32; 3],
    pub _padding3: u32,

    /// Fog settings: r, g, b, w=distance (0.0 means disabled)
    pub fog_params: [f32; 4],
}

/// GPU uniform for Cascaded Shadow Map (CSM) data: 4 light-space matrices,
/// cascade split depths, and PCF/bias configuration.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightSpaceUniform {
    pub matrices: [[[f32; 4]; 4]; 4], // 4 matrices for 4 cascades (64 bytes * 4 = 256 bytes)
    pub cascade_splits: [f32; 4],     // Z view depths for splitting (16 bytes)
    pub shadow_bias: f32,
    pub pcf_radius: i32,
    pub shadow_enabled: u32,
    pub _pad: u32,
}

/// GPU uniform for the procedural sky and volumetric clouds shader: sun position,
/// physical atmosphere density, ozone absorption, volumetric cloud coverage and wind dynamics.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SkyUniform {
    pub sun_direction: [f32; 4], // xyz: direction, w: padding
    pub sun_color: [f32; 4],     // rgb: color, w: HDR intensity
    pub horizon_color: [f32; 4], // rgb: horizon tint, w: padding
    pub zenith_color: [f32; 4],  // rgb: zenith tint, w: padding
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
    pub time: f32,
    pub sky_quality_mode: u32, // 0=Low, 1=Medium, 2=High
}
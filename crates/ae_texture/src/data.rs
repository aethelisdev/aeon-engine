// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Texture data structures, color spaces, map types, and sampling configurations.

use crate::mipmap::{CpuMipmapLevel, generate_mipmap_chain};

/// Color space specification for texture data.
/// Defines whether a texture's pixel values are encoded in sRGB (e.g. Albedo/Diffuse maps)
/// or Linear color space (e.g. Normal maps, Roughness, Metallic, Ambient Occlusion).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// sRGB color space for visual albedo and diffuse color textures.
    Srgb,
    /// Linear color space for data textures like normal, roughness, or metallic maps.
    Linear,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self::Srgb
    }
}

/// Semantic PBR texture map type classification.
/// Used to automatically select the correct color space (sRGB vs Linear) and default sampler parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMapType {
    /// Base color / Diffuse texture map (sRGB).
    Albedo,
    /// Tangent-space normal map (Linear).
    Normal,
    /// Combined metallic and roughness map (Linear).
    MetallicRoughness,
    /// Self-emissive light map (sRGB).
    Emissive,
    /// Ambient occlusion map (Linear).
    AmbientOcclusion,
    /// Custom user-defined map type.
    Custom(ColorSpace),
}

impl TextureMapType {
    /// Returns the standard default color space associated with this map type.
    pub fn default_color_space(&self) -> ColorSpace {
        match self {
            Self::Albedo | Self::Emissive => ColorSpace::Srgb,
            Self::Normal | Self::MetallicRoughness | Self::AmbientOcclusion => ColorSpace::Linear,
            Self::Custom(space) => *space,
        }
    }
}

impl Default for TextureMapType {
    fn default() -> Self {
        Self::Albedo
    }
}

/// Texture address wrapping mode for texture coordinate sampling beyond [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WrapMode {
    /// Clamp texture coordinates to the edge pixels.
    ClampToEdge,
    /// Repeat the texture infinitely across coordinates.
    Repeat,
    /// Repeat the texture with mirrored reflections at integer boundaries.
    MirrorRepeat,
}

impl Default for WrapMode {
    fn default() -> Self {
        Self::ClampToEdge
    }
}

/// Texture filtering mode for magnification and minification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMode {
    /// Nearest-neighbor sampling (crisp, pixelated look).
    Nearest,
    /// Bilinear / Trilinear interpolation (smooth filtering).
    Linear,
}

impl Default for FilterMode {
    fn default() -> Self {
        Self::Linear
    }
}

/// Configuration descriptor for texture sampler parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplerConfig {
    /// Minification filter mode.
    pub min_filter: FilterMode,
    /// Magnification filter mode.
    pub mag_filter: FilterMode,
    /// Mipmap filter mode.
    pub mipmap_filter: FilterMode,
    /// Horizontal (U) coordinate wrapping mode.
    pub wrap_u: WrapMode,
    /// Vertical (V) coordinate wrapping mode.
    pub wrap_v: WrapMode,
    /// Anisotropic filtering clamp factor (1 = disabled, max 16).
    pub anisotropy_clamp: u16,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            wrap_u: WrapMode::ClampToEdge,
            wrap_v: WrapMode::ClampToEdge,
            anisotropy_clamp: 1,
        }
    }
}

/// Uncompressed CPU-side texture pixel data container.
/// Holds width, height, raw RGBA bytes, color space, mipmap chain levels, and source label.
/// Serves as the primary bridge between file loading/procedural generation and GPU uploading.
#[derive(Debug, Clone)]
pub struct CpuTextureData {
    /// Width of the texture image in pixels.
    pub width: u32,
    /// Height of the texture image in pixels.
    pub height: u32,
    /// Raw uncompressed RGBA8 pixel byte array (length MUST be width * height * 4).
    pub bytes: Vec<u8>,
    /// Target color space (sRGB or Linear).
    pub color_space: ColorSpace,
    /// Sampler configuration (wrapping, filtering, anisotropy).
    pub sampler_config: SamplerConfig,
    /// Last modification timestamp on disk for live hot-reloading.
    pub last_modified: Option<std::time::SystemTime>,
    /// Pre-generated CPU mipmap level chain (Level 0 = base image).
    pub mipmaps: Vec<CpuMipmapLevel>,
    /// Canonical local disk source path or label if procedurally generated.
    pub label: String,
}

impl CpuTextureData {
    /// Creates a new CPU texture data container with safety checks on byte buffer size.
    /// # Panics
    /// Panics in debug builds if `bytes.len()` does not match `width * height * 4`.
    pub fn new(
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        color_space: ColorSpace,
        label: impl Into<String>,
    ) -> Self {
        let expected_len = (width as usize) * (height as usize) * 4;
        debug_assert_eq!(
            bytes.len(),
            expected_len,
            "CpuTextureData buffer length mismatch: expected {}, got {}",
            expected_len,
            bytes.len()
        );
        Self {
            width,
            height,
            bytes,
            color_space,
            sampler_config: SamplerConfig::default(),
            last_modified: None,
            mipmaps: Vec::new(),
            label: label.into(),
        }
    }

    /// Builder pattern helper to set custom sampler configuration (wrapping, filtering).
    pub fn with_sampler_config(mut self, config: SamplerConfig) -> Self {
        self.sampler_config = config;
        self
    }

    /// Builder pattern helper to set modification timestamp for hot-reloading tracking.
    pub fn with_last_modified(mut self, time: Option<std::time::SystemTime>) -> Self {
        self.last_modified = time;
        self
    }

    /// Automatically generates and stores the CPU mipmap level chain down to 1x1.
    /// Returns a mutable reference to `self` for fluent builder usage.
    pub fn generate_mipmaps(&mut self) -> &mut Self {
        self.mipmaps = generate_mipmap_chain(self.width, self.height, &self.bytes);
        self
    }

    /// Builder pattern helper to generate mipmaps and return self by value.
    pub fn with_mipmaps(mut self) -> Self {
        self.generate_mipmaps();
        self
    }
}

/// Supported GPU Block Compression (BC) texture formats.
/// Block compression reduces VRAM footprint by 75-80% using 4x4 texel block encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressedTextureFormat {
    /// BC1 (DXT1) RGB/RGBA compression without alpha or 1-bit punchthrough alpha (8 bytes per 4x4 block).
    Bc1Unorm,
    /// BC1 (DXT1) sRGB encoded color space.
    Bc1Srgb,
    /// BC3 (DXT5) RGBA compression with full 8-bit interpolated alpha channel (16 bytes per 4x4 block).
    Bc3Unorm,
    /// BC3 (DXT5) sRGB encoded color space.
    Bc3Srgb,
    /// BC7 High-quality ARGB compression for  PBR materials (16 bytes per 4x4 block).
    Bc7Unorm,
    /// BC7 sRGB encoded color space.
    Bc7Srgb,
}

impl CompressedTextureFormat {
    /// Returns the block size in bytes for a 4x4 texel block.
    pub fn block_size(&self) -> u32 {
        match self {
            Self::Bc1Unorm | Self::Bc1Srgb => 8,
            Self::Bc3Unorm | Self::Bc3Srgb | Self::Bc7Unorm | Self::Bc7Srgb => 16,
        }
    }

    /// Returns whether this format uses sRGB encoding.
    pub fn is_srgb(&self) -> bool {
        matches!(self, Self::Bc1Srgb | Self::Bc3Srgb | Self::Bc7Srgb)
    }
}

/// Container for GPU block-compressed texture pixel data (BC1/BC3/BC7).
#[derive(Debug, Clone)]
pub struct CompressedTextureData {
    /// Width of the texture image in pixels.
    pub width: u32,
    /// Height of the texture image in pixels.
    pub height: u32,
    /// Block compression format (BC1/BC3/BC7).
    pub format: CompressedTextureFormat,
    /// Raw block-compressed bytes.
    pub bytes: Vec<u8>,
    /// Sampler configuration.
    pub sampler_config: SamplerConfig,
    /// Canonical disk source path or label.
    pub label: String,
}

impl CompressedTextureData {
    /// Creates a new compressed texture container.
    pub fn new(
        width: u32,
        height: u32,
        format: CompressedTextureFormat,
        bytes: Vec<u8>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            width,
            height,
            format,
            bytes,
            sampler_config: SamplerConfig::default(),
            label: label.into(),
        }
    }
}

/// Grouping container for multi-map PBR material texture asset handles.
/// Combines Albedo, Normal Map, Metallic-Roughness Map, Ambient Occlusion, and Emissive handles.
#[derive(Debug, Clone, Default)]
pub struct PbrMaterialTextures<H> {
    /// Base color / Albedo map handle (sRGB).
    pub albedo: H,
    /// Tangent-space normal map handle (Linear, optional).
    pub normal: Option<H>,
    /// Combined metallic and roughness map handle (Linear, optional).
    pub metallic_roughness: Option<H>,
    /// Ambient occlusion map handle (Linear, optional).
    pub ambient_occlusion: Option<H>,
    /// Self-emissive light map handle (sRGB, optional).
    pub emissive: Option<H>,
}

impl<H> PbrMaterialTextures<H> {
    /// Creates a new PBR material texture set with base albedo map.
    pub fn new(albedo: H) -> Self {
        Self {
            albedo,
            normal: None,
            metallic_roughness: None,
            ambient_occlusion: None,
            emissive: None,
        }
    }
}
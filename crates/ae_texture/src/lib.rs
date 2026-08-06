// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! `ae_texture` - Decoupled CPU-side Texture Processing and Asset Subsystem for Aeon Engine.
//!
//! Provides uncompressed texture pixel storage (`CpuTextureData`), CPU mipmap chain generation
//! (`CpuMipmapLevel`, `generate_mipmap_chain`), procedural fallback generation (`FallbackTextureGenerator`),
//! semantic texture map types (`TextureMapType`), path sanitization (`is_safe_path`),
//! image file loading (`parse_texture_file`), sampler configurations (`SamplerConfig`),
//! and generational asset handle storage (`TextureStorage`).
//!

pub mod asset;
pub mod data;
pub mod fallback;
pub mod loader;
pub mod mipmap;

#[cfg(test)]
mod tests;

pub use asset::{AssetHandle, TextureAssetData, TexturePathMap, TextureStorage};
pub use data::{ColorSpace, CpuTextureData, FilterMode, SamplerConfig, TextureMapType, WrapMode};
pub use fallback::FallbackTextureGenerator;
pub use loader::{MAX_TEXTURE_DIMENSION, is_safe_path, parse_texture_file};
pub use mipmap::{CpuMipmapLevel, generate_mipmap_chain};
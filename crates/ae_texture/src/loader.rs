// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! File and VFS loading, path security verification, image parsing, and [`CookedTexture`]
//! conversion utilities for CPU textures.

use crate::data::{ColorSpace, CpuTextureData};
use crate::mipmap::CpuMipmapLevel;
use ae_world::{
    AlignedBuffer, AssetGuid, COOKED_TEXTURE_MAGIC, CookedMipLevel, CookedTexture,
    CookedTextureFormat, TargetTextureProfile, VfsProvider, sanitize_virtual_path,
};
use std::path::Path;

/// Maximum allowable texture dimension in pixels (`16384x16384`) to prevent VRAM overflow.
pub const MAX_TEXTURE_DIMENSION: u32 = 16384;

/// Sanitizes file paths to prevent directory traversal vulnerabilities (e.g. `../..` manipulation).
///
/// Returns `false` if the path contains relative traversal sequences (`..`).
pub fn is_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    for component in p.components() {
        if let std::path::Component::ParentDir = component {
            return false;
        }
    }
    true
}

/// Parses raw texture bytes (either a pre-cooked [`CookedTexture`] blob or encoded PNG/JPEG/etc.)
/// into uncompressed RGBA8 [`CpuTextureData`].
///
/// # Errors
/// Returns an error string if decoding fails, a block-compressed cooked texture cannot be
/// unpacked on CPU, or dimensions exceed [`MAX_TEXTURE_DIMENSION`].
pub fn parse_texture_bytes(
    bytes: &[u8],
    label: &str,
    color_space: ColorSpace,
) -> Result<CpuTextureData, String> {
    if bytes.starts_with(&COOKED_TEXTURE_MAGIC) {
        let cooked = CookedTexture::decode(bytes)
            .map_err(|e| format!("[ERROR] Failed to decode cooked texture '{label}': {e}"))?;
        return cooked_to_cpu_texture(&cooked, label);
    }

    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("[ERROR] Failed to decode texture '{label}': {e}"))?;
    let (width, height) = (img.width(), img.height());
    if width > MAX_TEXTURE_DIMENSION || height > MAX_TEXTURE_DIMENSION {
        core::hint::cold_path();
        return Err(format!(
            "[ERROR] Texture dimensions {width}x{height} exceed maximum limit {MAX_TEXTURE_DIMENSION}x{MAX_TEXTURE_DIMENSION}"
        ));
    }

    let rgba = img.to_rgba8();
    Ok(CpuTextureData::new(width, height, rgba.into_raw(), color_space, label).with_mipmaps())
}

/// Loads and decodes a texture asset by virtual path from any [`VfsProvider`] without filesystem assumptions.
///
/// Supports both pre-cooked [`CookedTexture`] entries and raw PNG/JPEG payloads.
///
/// # Errors
/// Returns an error string on path traversal rejection, missing VFS entry, or decode failure.
pub fn load_texture_from_vfs(
    vfs: &dyn VfsProvider,
    virtual_path: &str,
    color_space: ColorSpace,
) -> Result<CpuTextureData, String> {
    sanitize_virtual_path(virtual_path).map_err(|e| {
        format!("[SECURITY ERROR] Blocked unsafe texture VFS path '{virtual_path}': {e}")
    })?;
    let bytes = vfs
        .read_bytes(virtual_path)
        .map_err(|e| format!("[ERROR] VFS failed to read texture '{virtual_path}': {e}"))?;
    parse_texture_bytes(&bytes, virtual_path, color_space)
}

/// Loads and decodes a texture asset by persistent [`AssetGuid`] from an indexed [`VfsProvider`].
///
/// # Errors
/// Returns an error string if the GUID is missing or decoding fails.
pub fn load_texture_by_guid_from_vfs(
    vfs: &dyn VfsProvider,
    guid: AssetGuid,
    label: &str,
    color_space: ColorSpace,
) -> Result<CpuTextureData, String> {
    let bytes = vfs
        .read_by_guid(guid)
        .map_err(|e| format!("[ERROR] VFS failed to read texture GUID {guid}: {e}"))?;
    parse_texture_bytes(&bytes, label, color_space)
}

/// Converts CPU RGBA8 texture data and its mipmaps into a platform-tagged [`CookedTexture`].
///
/// Uses uncompressed `Rgba8Srgb` / `Rgba8Unorm` until hardware block-compression (`Bc7` / `Astc4x4`)
/// encoders are connected in the export pipeline.
#[must_use]
pub fn cpu_texture_to_cooked(cpu: &CpuTextureData, profile: TargetTextureProfile) -> CookedTexture {
    let is_srgb = cpu.color_space == ColorSpace::Srgb;
    let format = if is_srgb {
        CookedTextureFormat::Rgba8Srgb
    } else {
        CookedTextureFormat::Rgba8Unorm
    };
    let mipmaps = if cpu.mipmaps.is_empty() {
        vec![CookedMipLevel {
            width: cpu.width,
            height: cpu.height,
            data: AlignedBuffer::from_slice(&cpu.bytes),
        }]
    } else {
        cpu.mipmaps
            .iter()
            .map(|mip| CookedMipLevel {
                width: mip.width,
                height: mip.height,
                data: AlignedBuffer::from_slice(&mip.bytes),
            })
            .collect()
    };

    CookedTexture {
        width: cpu.width,
        height: cpu.height,
        format,
        profile,
        is_srgb,
        mipmaps,
    }
}

/// Reconstructs uncompressed [`CpuTextureData`] from an uncompressed [`CookedTexture`].
///
/// # Errors
/// Returns an error string when dimensions exceed [`MAX_TEXTURE_DIMENSION`], mipmaps are missing,
/// or the cooked payload uses block compression (`Bc7`/`Astc4x4`/`Etc2`) that belongs directly on GPU.
pub fn cooked_to_cpu_texture(
    cooked: &CookedTexture,
    label: &str,
) -> Result<CpuTextureData, String> {
    if cooked.format.is_block_compressed() {
        return Err(format!(
            "[ERROR] Cooked texture '{label}' uses GPU block format {:?}; upload via GPU cooked path",
            cooked.format
        ));
    }
    if cooked.width > MAX_TEXTURE_DIMENSION || cooked.height > MAX_TEXTURE_DIMENSION {
        return Err(format!(
            "[ERROR] Cooked texture dimensions {}x{} exceed limit {}x{}",
            cooked.width, cooked.height, MAX_TEXTURE_DIMENSION, MAX_TEXTURE_DIMENSION
        ));
    }
    let base_mip = cooked
        .mipmaps
        .first()
        .ok_or_else(|| format!("[ERROR] Cooked texture '{label}' contains 0 mipmap levels"))?;
    let color_space = if cooked.is_srgb || cooked.format == CookedTextureFormat::Rgba8Srgb {
        ColorSpace::Srgb
    } else {
        ColorSpace::Linear
    };
    let mut cpu = CpuTextureData::new(
        cooked.width,
        cooked.height,
        base_mip.data.to_vec(),
        color_space,
        label,
    );
    cpu.mipmaps = cooked
        .mipmaps
        .iter()
        .map(|m| CpuMipmapLevel {
            width: m.width,
            height: m.height,
            bytes: m.data.to_vec(),
        })
        .collect();
    Ok(cpu)
}

/// Parses a texture file from local disk into uncompressed RGBA8 [`CpuTextureData`].
///
/// Performs security sanitization, reads raw bytes into memory, checks dimension bounds,
/// and decodes RGBA pixels + mipmaps via [`parse_texture_bytes`].
///
/// # Errors
/// Returns an error string if path is unsafe, file cannot be read, image format is unknown,
/// or dimensions exceed [`MAX_TEXTURE_DIMENSION`].
pub fn parse_texture_file(path: &str, color_space: ColorSpace) -> Result<CpuTextureData, String> {
    if !is_safe_path(path) {
        core::hint::cold_path();
        return Err(format!(
            "[SECURITY ERROR] Blocked unsafe texture path: {path}"
        ));
    }

    let bytes =
        std::fs::read(path).map_err(|e| format!("[ERROR] Failed to open texture '{path}': {e}"))?;
    let last_mod = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    let cpu = parse_texture_bytes(&bytes, path, color_space)?;
    Ok(cpu.with_last_modified(last_mod))
}

#[cfg(test)]
mod vfs_tests {
    use super::*;
    use ae_world::{AssetKind, PakBuilder, PakCodec, PakMemoryVfsProvider};

    #[test]
    fn test_vfs_loads_raw_and_cooked_textures_by_path_and_guid() {
        let mut img = image::RgbaImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgba([255, 10, 20, 255]));
        img.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
        img.put_pixel(0, 1, image::Rgba([0, 0, 255, 255]));
        img.put_pixel(1, 1, image::Rgba([255, 255, 255, 255]));

        let mut png_bytes = std::io::Cursor::new(Vec::new());
        img.write_to(&mut png_bytes, image::ImageFormat::Png)
            .expect("encode test png");

        let cpu_src =
            parse_texture_bytes(png_bytes.get_ref(), "textures/raw.png", ColorSpace::Srgb)
                .expect("decode png");
        let cooked = cpu_texture_to_cooked(&cpu_src, TargetTextureProfile::Desktop);
        let cooked_bytes = cooked.encode().expect("encode cooked texture");

        let mut builder = PakBuilder::new();
        let raw_guid = builder.add_asset(
            "textures/raw.png",
            AssetKind::Texture,
            PakCodec::Lz4Block,
            png_bytes.into_inner(),
        );
        let cooked_guid = builder.add_asset(
            "textures/cooked.cooktex",
            AssetKind::Texture,
            PakCodec::Raw,
            cooked_bytes,
        );
        let pak = builder.build_to_vec().expect("build pak");
        let vfs = PakMemoryVfsProvider::from_bytes(pak).expect("mount pak vfs");

        let from_raw =
            load_texture_by_guid_from_vfs(&vfs, raw_guid, "textures/raw.png", ColorSpace::Srgb)
                .expect("load raw png by guid");
        assert_eq!(from_raw.width, 2);
        assert_eq!(from_raw.height, 2);

        let from_cooked = load_texture_from_vfs(&vfs, "textures/cooked.cooktex", ColorSpace::Srgb)
            .expect("load cooked tex from vfs");
        assert_eq!(from_cooked.bytes, from_raw.bytes);
        assert_eq!(from_cooked.mipmaps.len(), from_raw.mipmaps.len());

        let from_cooked_guid = load_texture_by_guid_from_vfs(
            &vfs,
            cooked_guid,
            "textures/cooked.cooktex",
            ColorSpace::Srgb,
        )
        .expect("load cooked by guid");
        assert_eq!(from_cooked_guid.bytes, from_cooked.bytes);
    }
}
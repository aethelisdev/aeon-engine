// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cooked GPU/audio binary asset formats (`CookedMesh`, `CookedTexture`, `CookedAudio`)
//! and platform-keyed `Intermediate/` build cache helpers.
//!
//! Metadata descriptors use compact `postcard`, while vertex, index, mipmap, and PCM buffers are
//! stored as length-framed raw slices that deserialize directly into `16`-byte aligned
//! [`AlignedBuffer`] instances without per-element serialization overhead.

use crate::aligned_buffer::AlignedBuffer;
use crate::error::{WorldIoError, WorldResult};
use crate::header::{
    ByteReader, COOKED_AUDIO_MAGIC, COOKED_MESH_MAGIC, COOKED_TEXTURE_MAGIC,
    decode_checksummed_envelope, encode_checksummed_envelope,
};
use bytemuck::Pod;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Current binary format version for cooked mesh, texture, and audio blobs.
pub const COOKED_FORMAT_VERSION: u32 = 1;

const FNV_OFFSET_64: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME_64: u64 = 0x0100_0000_01b3;

/// Target GPU texture compression profile keyed in the `Intermediate/` build cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TargetTextureProfile {
    /// Desktop Vulkan/DX12/Metal target (prefers `BC7` compressed or `RGBA8` fallback).
    #[default]
    Desktop,
    /// Mobile Android Vulkan/GLES target (prefers `ASTC 4x4` or `ETC2`).
    MobileAndroid,
}

impl TargetTextureProfile {
    /// Returns the subdirectory name used inside `Intermediate/` to prevent cross-platform
    /// cooked cache collisions (`"desktop"` vs `"android"`).
    #[must_use]
    pub const fn cache_subdir(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::MobileAndroid => "android",
        }
    }

    /// Returns the preferred hardware-compressed format for this target profile.
    #[must_use]
    pub const fn preferred_compressed_format(self) -> CookedTextureFormat {
        match self {
            Self::Desktop => CookedTextureFormat::Bc7,
            Self::MobileAndroid => CookedTextureFormat::Astc4x4,
        }
    }

    /// Returns the active profile matching the host/target compilation OS.
    #[must_use]
    pub const fn current_target() -> Self {
        if cfg!(target_os = "android") {
            Self::MobileAndroid
        } else {
            Self::Desktop
        }
    }
}

/// GPU-native pixel or block-compression encoding stored inside [`CookedTexture`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CookedTextureFormat {
    /// Uncompressed linear `RGBA8` (`4` bytes/pixel).
    Rgba8Unorm,
    /// Uncompressed sRGB `RGBA8` (`4` bytes/pixel).
    #[default]
    Rgba8Srgb,
    /// Desktop `BC7` (`1` byte/pixel equivalent in 4x4 blocks).
    Bc7,
    /// Mobile `ASTC 4x4` (`1` byte/pixel equivalent in 4x4 blocks).
    Astc4x4,
    /// Mobile `ETC2 RGBA8` fallback (`1` byte/pixel equivalent in 4x4 blocks).
    Etc2,
}

impl CookedTextureFormat {
    /// Returns `true` when this format encodes block-compressed texels rather than raw RGBA pixels.
    #[must_use]
    pub const fn is_block_compressed(self) -> bool {
        matches!(self, Self::Bc7 | Self::Astc4x4 | Self::Etc2)
    }

    /// Returns uncompressed byte count per pixel when not block-compressed.
    #[must_use]
    pub const fn bytes_per_pixel_uncompressed(self) -> Option<u32> {
        match self {
            Self::Rgba8Unorm | Self::Rgba8Srgb => Some(4),
            Self::Bc7 | Self::Astc4x4 | Self::Etc2 => None,
        }
    }
}

/// Alpha compositing mode preserved inside [`CookedSubmesh`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CookedAlphaMode {
    /// Fully opaque surface with depth write enabled.
    #[default]
    Opaque,
    /// Alpha-tested cutout surface discarding fragments below `alpha_cutoff`.
    Mask,
    /// Alpha-blended surface sorted back-to-front.
    Blend,
}

/// Submesh material range inside a [`CookedMesh`] index buffer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CookedSubmesh {
    /// Starting index inside [`CookedMesh::indices`].
    pub start_index: u32,
    /// Number of indices consumed by this submesh draw call.
    pub index_count: u32,
    /// Optional embedded/external texture slot index.
    pub texture_index: Option<u32>,
    /// Linear base color multiplier `[r, g, b, a]`.
    pub base_color: [f32; 4],
    /// Alpha blending/masking mode.
    pub alpha_mode: CookedAlphaMode,
    /// Alpha cutout threshold used when `alpha_mode == CookedAlphaMode::Mask`.
    pub alpha_cutoff: f32,
    /// PBR metallic factor `[0.0 .. 1.0]`.
    pub metallic_factor: f32,
    /// PBR roughness factor `[0.0 .. 1.0]`.
    pub roughness_factor: f32,
    /// UV tiling multiplier `[u, v]`.
    pub uv_scale: [f32; 2],
}

#[derive(Serialize, Deserialize)]
struct CookedMeshMeta {
    min: [f32; 3],
    max: [f32; 3],
    vertex_stride: u32,
    vertex_count: u32,
    index_count: u32,
    submeshes: Vec<CookedSubmesh>,
}

/// Pre-processed mesh container pairing compact metadata with `16`-byte aligned vertex/index
/// buffers ready for safe zero-copy `bytemuck::try_cast_slice` WGPU buffer uploads.
#[derive(Debug, Clone, PartialEq)]
pub struct CookedMesh {
    /// Minimum local-space AABB corner `[x, y, z]`.
    pub min: [f32; 3],
    /// Maximum local-space AABB corner `[x, y, z]`.
    pub max: [f32; 3],
    /// Expected byte stride of one GPU vertex element (e.g., `size_of::<Vertex>()`).
    pub vertex_stride: u32,
    /// Total vertex count covered by [`Self::vertices`].
    pub vertex_count: u32,
    /// Total `u32` index count covered by [`Self::indices`].
    pub index_count: u32,
    /// Ordered submesh material segments.
    pub submeshes: Vec<CookedSubmesh>,
    /// `16`-byte aligned raw vertex byte payload.
    pub vertices: AlignedBuffer,
    /// `16`-byte aligned raw `u32` index byte payload.
    pub indices: AlignedBuffer,
}

impl CookedMesh {
    /// Safely reinterprets the vertex payload as a slice of `T` without allocation or copying.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if `size_of::<T>()` does not match
    /// [`Self::vertex_stride`] or if `bytemuck::try_cast_slice` rejects alignment/size.
    pub fn vertices_as<T: Pod>(&self) -> WorldResult<&[T]> {
        if core::mem::size_of::<T>() != self.vertex_stride as usize {
            return Err(WorldIoError::InvalidLayout(format!(
                "CookedMesh vertex stride {} mismatches requested type stride {}",
                self.vertex_stride,
                core::mem::size_of::<T>()
            )));
        }
        self.vertices
            .try_cast_slice()
            .map_err(|e| WorldIoError::InvalidLayout(format!("Vertex cast error: {e:?}")))
    }

    /// Safely reinterprets the index payload as `&[u32]` without allocation or copying.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if the byte length is not a multiple of `4`.
    pub fn indices_as_u32(&self) -> WorldResult<&[u32]> {
        self.indices
            .try_cast_slice()
            .map_err(|e| WorldIoError::InvalidLayout(format!("Index cast error: {e:?}")))
    }

    /// Serializes metadata and raw vertex/index slices inside an LZ4 + CRC32 [`COOKED_MESH_MAGIC`] envelope.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if metadata encoding fails or payload sizes exceed `u32::MAX`.
    pub fn encode(&self) -> WorldResult<Vec<u8>> {
        let meta = CookedMeshMeta {
            min: self.min,
            max: self.max,
            vertex_stride: self.vertex_stride,
            vertex_count: self.vertex_count,
            index_count: self.index_count,
            submeshes: self.submeshes.clone(),
        };
        let meta_bytes =
            postcard::to_stdvec(&meta).map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let raw = pack_sections(
            &meta_bytes,
            &[self.vertices.as_bytes(), self.indices.as_bytes()],
        )?;
        let compressed = lz4_flex::compress_prepend_size(&raw);
        Ok(encode_checksummed_envelope(
            COOKED_MESH_MAGIC,
            COOKED_FORMAT_VERSION,
            0,
            &compressed,
        ))
    }

    /// Validates, decompresses, and reconstructs a [`CookedMesh`] directly into `16`-byte aligned buffers.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on magic, version, CRC32, decompression, or layout mismatch.
    pub fn decode(bytes: &[u8]) -> WorldResult<Self> {
        let decompressed = decode_compressed_envelope(bytes, COOKED_MESH_MAGIC)?;
        let mut reader = ByteReader::new(&decompressed);
        let meta: CookedMeshMeta = postcard::from_bytes(read_len_slice(&mut reader, "mesh meta")?)
            .map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let vertices = read_aligned_section(&mut reader, "mesh vertices")?;
        let indices = read_aligned_section(&mut reader, "mesh indices")?;

        let expected_v_bytes =
            (meta.vertex_stride as usize).saturating_mul(meta.vertex_count as usize);
        let expected_i_bytes =
            (meta.index_count as usize).saturating_mul(core::mem::size_of::<u32>());
        if vertices.len() != expected_v_bytes || indices.len() != expected_i_bytes {
            return Err(WorldIoError::InvalidLayout(
                "CookedMesh buffer sizes do not match header vertex/index counts".to_string(),
            ));
        }

        Ok(Self {
            min: meta.min,
            max: meta.max,
            vertex_stride: meta.vertex_stride,
            vertex_count: meta.vertex_count,
            index_count: meta.index_count,
            submeshes: meta.submeshes,
            vertices,
            indices,
        })
    }
}

/// One pre-generated mipmap level inside [`CookedTexture`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookedMipLevel {
    /// Width of this mip level in pixels.
    pub width: u32,
    /// Height of this mip level in pixels.
    pub height: u32,
    /// `16`-byte aligned raw pixel or compressed block bytes for this mip level.
    pub data: AlignedBuffer,
}

#[derive(Serialize, Deserialize)]
struct CookedTextureMeta {
    width: u32,
    height: u32,
    format: CookedTextureFormat,
    profile: TargetTextureProfile,
    is_srgb: bool,
    mip_dims: Vec<(u32, u32)>,
}

/// Pre-processed mipmapped texture asset targeting either desktop or mobile GPU formats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookedTexture {
    /// Base level (`mip 0`) width in pixels.
    pub width: u32,
    /// Base level (`mip 0`) height in pixels.
    pub height: u32,
    /// Pixel or block-compressed GPU texture format.
    pub format: CookedTextureFormat,
    /// Platform profile this texture was cooked for.
    pub profile: TargetTextureProfile,
    /// Preserves sRGB vs Linear color intent independently of compressed container variants.
    pub is_srgb: bool,
    /// Ordered mipmap chain starting at full resolution (`level 0`).
    pub mipmaps: Vec<CookedMipLevel>,
}

impl CookedTexture {
    /// Serializes texture metadata and raw mipmap slices inside a CRC32 + LZ4 [`COOKED_TEXTURE_MAGIC`] envelope.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on metadata serialization failure or oversized buffers.
    pub fn encode(&self) -> WorldResult<Vec<u8>> {
        if self.width == 0 || self.height == 0 || self.mipmaps.is_empty() {
            return Err(WorldIoError::InvalidLayout(
                "CookedTexture must have positive dimensions and at least 1 mipmap level"
                    .to_string(),
            ));
        }
        let meta = CookedTextureMeta {
            width: self.width,
            height: self.height,
            format: self.format,
            profile: self.profile,
            is_srgb: self.is_srgb,
            mip_dims: self.mipmaps.iter().map(|m| (m.width, m.height)).collect(),
        };
        let meta_bytes =
            postcard::to_stdvec(&meta).map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let mip_slices: Vec<&[u8]> = self.mipmaps.iter().map(|m| m.data.as_bytes()).collect();
        let raw = pack_sections(&meta_bytes, &mip_slices)?;
        let compressed = lz4_flex::compress_prepend_size(&raw);
        Ok(encode_checksummed_envelope(
            COOKED_TEXTURE_MAGIC,
            COOKED_FORMAT_VERSION,
            0,
            &compressed,
        ))
    }

    /// Validates, decompresses, and reconstructs a [`CookedTexture`] with `16`-byte aligned mipmaps.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on corrupted header, CRC32 failure, decompression error, or empty mips.
    pub fn decode(bytes: &[u8]) -> WorldResult<Self> {
        let decompressed = decode_compressed_envelope(bytes, COOKED_TEXTURE_MAGIC)?;
        let mut reader = ByteReader::new(&decompressed);
        let meta: CookedTextureMeta =
            postcard::from_bytes(read_len_slice(&mut reader, "texture meta")?)
                .map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        if meta.width == 0 || meta.height == 0 || meta.mip_dims.is_empty() {
            return Err(WorldIoError::InvalidLayout(
                "CookedTexture has zero dimensions or zero mip levels".to_string(),
            ));
        }
        let mut mipmaps = Vec::with_capacity(meta.mip_dims.len());
        for (width, height) in meta.mip_dims {
            let data = read_aligned_section(&mut reader, "texture mipmap")?;
            mipmaps.push(CookedMipLevel {
                width,
                height,
                data,
            });
        }
        Ok(Self {
            width: meta.width,
            height: meta.height,
            format: meta.format,
            profile: meta.profile,
            is_srgb: meta.is_srgb,
            mipmaps,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct CookedAudioMeta {
    sample_rate: u32,
    channels: u16,
    sample_count: u64,
}

/// Pre-decoded PCM audio clip (`16`-bit interleaved samples) ready for zero-parse VFS playback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookedAudio {
    /// Sample rate in Hertz (e.g., `44_100` or `48_000`).
    pub sample_rate: u32,
    /// Interleaved channel count (`1` mono, `2` stereo).
    pub channels: u16,
    /// `16`-byte aligned interleaved `i16` PCM sample payload.
    pub pcm_i16: AlignedBuffer,
}

impl CookedAudio {
    /// Builds a [`CookedAudio`] clip from an interleaved `&[i16]` PCM buffer.
    #[must_use]
    pub fn from_samples_i16(sample_rate: u32, channels: u16, samples: &[i16]) -> Self {
        Self {
            sample_rate,
            channels,
            pcm_i16: AlignedBuffer::from_slice(bytemuck::cast_slice(samples)),
        }
    }

    /// Reinterprets the PCM buffer safely as `&[i16]` without extra allocations.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] when byte length is odd or misaligned.
    pub fn samples_i16(&self) -> WorldResult<&[i16]> {
        self.pcm_i16
            .try_cast_slice()
            .map_err(|e| WorldIoError::InvalidLayout(format!("Audio PCM cast error: {e:?}")))
    }

    /// Serializes audio metadata and PCM buffer inside a CRC32 + LZ4 [`COOKED_AUDIO_MAGIC`] envelope.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on invalid sample parameters or oversized payloads.
    pub fn encode(&self) -> WorldResult<Vec<u8>> {
        if self.sample_rate == 0 || self.channels == 0 {
            return Err(WorldIoError::InvalidLayout(
                "CookedAudio requires non-zero sample_rate and channels".to_string(),
            ));
        }
        let samples = self.samples_i16()?;
        let meta = CookedAudioMeta {
            sample_rate: self.sample_rate,
            channels: self.channels,
            sample_count: samples.len() as u64,
        };
        let meta_bytes =
            postcard::to_stdvec(&meta).map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let raw = pack_sections(&meta_bytes, &[self.pcm_i16.as_bytes()])?;
        let compressed = lz4_flex::compress_prepend_size(&raw);
        Ok(encode_checksummed_envelope(
            COOKED_AUDIO_MAGIC,
            COOKED_FORMAT_VERSION,
            0,
            &compressed,
        ))
    }

    /// Validates, decompresses, and reconstructs a [`CookedAudio`] buffer.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on corrupted envelopes or sample count mismatches.
    pub fn decode(bytes: &[u8]) -> WorldResult<Self> {
        let decompressed = decode_compressed_envelope(bytes, COOKED_AUDIO_MAGIC)?;
        let mut reader = ByteReader::new(&decompressed);
        let meta: CookedAudioMeta =
            postcard::from_bytes(read_len_slice(&mut reader, "audio meta")?)
                .map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        if meta.sample_rate == 0 || meta.channels == 0 {
            return Err(WorldIoError::InvalidLayout(
                "CookedAudio has zero sample_rate or channels".to_string(),
            ));
        }
        let pcm_i16 = read_aligned_section(&mut reader, "audio pcm")?;
        if (pcm_i16.len() as u64) != meta.sample_count.saturating_mul(2) {
            return Err(WorldIoError::InvalidLayout(
                "CookedAudio PCM byte size mismatches declared sample_count".to_string(),
            ));
        }
        Ok(Self {
            sample_rate: meta.sample_rate,
            channels: meta.channels,
            pcm_i16,
        })
    }
}

/// Computes a deterministic 64-bit content hash over source asset bytes for `Intermediate/` cache keying.
#[must_use]
pub fn compute_asset_content_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    hash ^ (bytes.len() as u64).rotate_left(32)
}

/// Returns the platform-isolated `Intermediate/<target>/<hash>.<ext>` cache file path.
///
/// Prevents desktop (`BC7`) and mobile (`ASTC`) cooked textures from colliding when switching build targets.
#[must_use]
pub fn intermediate_cook_path(
    intermediate_dir: &Path,
    profile: TargetTextureProfile,
    asset_hash: u64,
    ext: &str,
) -> PathBuf {
    let clean_ext = ext.trim_start_matches('.');
    intermediate_dir
        .join(profile.cache_subdir())
        .join(format!("{asset_hash:016x}.{clean_ext}"))
}

/// Attempts to read cached cooked asset bytes from `Intermediate/<target>/<hash>.<ext>`.
#[must_use]
pub fn read_cached_cook_bytes(
    intermediate_dir: &Path,
    profile: TargetTextureProfile,
    asset_hash: u64,
    ext: &str,
) -> Option<Vec<u8>> {
    let path = intermediate_cook_path(intermediate_dir, profile, asset_hash, ext);
    std::fs::read(path).ok()
}

/// Atomically stores cooked asset bytes inside `Intermediate/<target>/<hash>.<ext>`.
///
/// # Errors
/// Returns [`WorldIoError::Io`] if directory creation or atomic file replacement fails.
pub fn write_cached_cook_bytes(
    intermediate_dir: &Path,
    profile: TargetTextureProfile,
    asset_hash: u64,
    ext: &str,
    cooked_bytes: &[u8],
) -> WorldResult<PathBuf> {
    let target = intermediate_cook_path(intermediate_dir, profile, asset_hash, ext);
    if let Some(parent) = target.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = target.with_extension("cook.tmp");
    std::fs::write(&tmp, cooked_bytes)?;
    std::fs::rename(&tmp, &target)?;
    Ok(target)
}

fn pack_sections(meta_bytes: &[u8], data_sections: &[&[u8]]) -> WorldResult<Vec<u8>> {
    let meta_len = u32::try_from(meta_bytes.len())
        .map_err(|_| WorldIoError::InvalidLayout("Cooked metadata exceeds u32::MAX".to_string()))?;
    let total_data: usize = data_sections
        .iter()
        .try_fold(0usize, |acc, s| acc.checked_add(4 + s.len()))
        .ok_or_else(|| WorldIoError::InvalidLayout("Cooked payload overflows RAM".to_string()))?;
    let mut out = Vec::with_capacity(4 + meta_bytes.len() + total_data);
    out.extend_from_slice(&meta_len.to_le_bytes());
    out.extend_from_slice(meta_bytes);
    for section in data_sections {
        let len = u32::try_from(section.len()).map_err(|_| {
            WorldIoError::InvalidLayout("Cooked section exceeds u32::MAX".to_string())
        })?;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(section);
    }
    Ok(out)
}

fn decode_compressed_envelope(bytes: &[u8], expected_magic: [u8; 8]) -> WorldResult<Vec<u8>> {
    let envelope = decode_checksummed_envelope(bytes, expected_magic, COOKED_FORMAT_VERSION)?;
    lz4_flex::decompress_size_prepended(envelope.payload)
        .map_err(|e| WorldIoError::Compression(e.to_string()))
}

fn read_len_slice<'a>(reader: &mut ByteReader<'a>, context: &'static str) -> WorldResult<&'a [u8]> {
    let len = reader.read_u32(context)? as usize;
    reader.read_slice(len, context)
}

fn read_aligned_section(
    reader: &mut ByteReader<'_>,
    context: &'static str,
) -> WorldResult<AlignedBuffer> {
    let slice = read_len_slice(reader, context)?;
    Ok(AlignedBuffer::from_slice(slice))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
    struct TestVertex {
        pos: [f32; 3],
        uv: [f32; 2],
    }

    #[test]
    fn test_cooked_mesh_encode_decode_preserves_16_byte_aligned_zero_copy_slices() {
        let vertices = [
            TestVertex {
                pos: [0.0, 0.0, 0.0],
                uv: [0.0, 0.0],
            },
            TestVertex {
                pos: [1.0, 2.0, 0.0],
                uv: [0.5, 1.0],
            },
            TestVertex {
                pos: [-1.0, 0.0, 2.0],
                uv: [1.0, 0.0],
            },
        ];
        let indices = [0u32, 1, 2];

        let mesh = CookedMesh {
            min: [-1.0, 0.0, 0.0],
            max: [1.0, 2.0, 2.0],
            vertex_stride: core::mem::size_of::<TestVertex>() as u32,
            vertex_count: vertices.len() as u32,
            index_count: indices.len() as u32,
            submeshes: vec![CookedSubmesh {
                start_index: 0,
                index_count: 3,
                texture_index: Some(0),
                base_color: [1.0; 4],
                alpha_mode: CookedAlphaMode::Opaque,
                alpha_cutoff: 0.5,
                metallic_factor: 0.2,
                roughness_factor: 0.8,
                uv_scale: [1.0, 1.0],
            }],
            vertices: AlignedBuffer::from_slice(bytemuck::cast_slice(&vertices)),
            indices: AlignedBuffer::from_slice(bytemuck::cast_slice(&indices)),
        };

        let encoded = mesh.encode().expect("encode cooked mesh");
        let decoded = CookedMesh::decode(&encoded).expect("decode cooked mesh");
        assert_eq!(decoded, mesh);
        assert_eq!(
            decoded.vertices_as::<TestVertex>().expect("cast vertices"),
            &vertices
        );
        assert_eq!(decoded.indices_as_u32().expect("cast indices"), &indices);
    }

    #[test]
    fn test_cooked_texture_audio_and_intermediate_platform_cache_isolation() {
        let tex = CookedTexture {
            width: 4,
            height: 4,
            format: CookedTextureFormat::Bc7,
            profile: TargetTextureProfile::Desktop,
            is_srgb: true,
            mipmaps: vec![
                CookedMipLevel {
                    width: 4,
                    height: 4,
                    data: AlignedBuffer::from_slice(&[0xABu8; 16]),
                },
                CookedMipLevel {
                    width: 2,
                    height: 2,
                    data: AlignedBuffer::from_slice(&[0xCDu8; 16]),
                },
            ],
        };
        let tex_bytes = tex.encode().expect("encode cooked texture");
        let tex_decoded = CookedTexture::decode(&tex_bytes).expect("decode cooked texture");
        assert_eq!(tex_decoded, tex);
        assert_eq!(tex_decoded.format, CookedTextureFormat::Bc7);
        assert!(tex_decoded.format.is_block_compressed());

        let audio = CookedAudio::from_samples_i16(48_000, 2, &[0, 1024, -1024, 32767]);
        let audio_bytes = audio.encode().expect("encode cooked audio");
        let audio_decoded = CookedAudio::decode(&audio_bytes).expect("decode cooked audio");
        assert_eq!(
            audio_decoded.samples_i16().expect("pcm samples"),
            &[0, 1024, -1024, 32767]
        );

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let cache_dir = std::env::temp_dir().join(format!("ae_world_intermediate_{nanos}"));
        let hash = compute_asset_content_hash(b"albedo-source-png");

        write_cached_cook_bytes(
            &cache_dir,
            TargetTextureProfile::Desktop,
            hash,
            "cooktex",
            &tex_bytes,
        )
        .expect("write desktop cache");
        assert!(
            read_cached_cook_bytes(
                &cache_dir,
                TargetTextureProfile::MobileAndroid,
                hash,
                "cooktex"
            )
            .is_none()
        );
        assert_eq!(
            read_cached_cook_bytes(&cache_dir, TargetTextureProfile::Desktop, hash, "cooktex"),
            Some(tex_bytes)
        );
        let _ = std::fs::remove_dir_all(&cache_dir);
    }
}
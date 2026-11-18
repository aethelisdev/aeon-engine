// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Parallel asset cooking (`CookedMesh`, `CookedTexture`, `CookedAudio`), platform-keyed
//! `Intermediate/<target>/` caching, and `16`-byte aligned `.aepak` archive construction.

use crate::dependency_graph::{DependencyGraph, DiscoveredAsset};
use crate::error::{ExportError, ExportResult};
use ae_audio::audio_manager::decode_audio_bytes_to_cooked;
use ae_renderer::render::resources::model::loader::parse_gltf_bytes;
use ae_renderer::render::types::SubmeshAlphaMode;
use ae_texture::loader::{cpu_texture_to_cooked, parse_texture_bytes};
use ae_world::{
    AlignedBuffer, AssetGuid, AssetKind, CookedAlphaMode, CookedMesh, CookedSubmesh, PakBuilder,
    PakCodec, ProjectWorkspaceLayout, TargetTextureProfile, compute_asset_content_hash,
    read_cached_cook_bytes, write_cached_cook_bytes,
};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Compression strategy applied to asset blocks inside `game.aepak`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CookCompressionProfile {
    /// `64 KiB` independent LZ4 blocks enabling fast random-access decompression.
    #[default]
    FastLz4,
    /// Raw uncompressed `16`-byte aligned storage for minimum CPU decode overhead.
    UncompressedRaw,
}

impl CookCompressionProfile {
    /// Converts this profile to the underlying [`PakCodec`] used by `PakBuilder`.
    #[must_use]
    pub const fn to_pak_codec(self) -> PakCodec {
        match self {
            Self::FastLz4 => PakCodec::Lz4Block,
            Self::UncompressedRaw => PakCodec::Raw,
        }
    }
}

/// Quantitative metrics summarizing an incremental cook and pack execution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CookSummary {
    /// Total source assets processed from [`DependencyGraph`].
    pub total_assets: usize,
    /// Assets reused directly from `Intermediate/<target>/<hash>.cook`.
    pub cache_hits: usize,
    /// Assets decoded, cooked, and written into `Intermediate/` during this run.
    pub cooked_fresh: usize,
    /// Non-cooked assets (shaders, prefabs, world files) packed directly.
    pub raw_copied: usize,
    /// Final written `game.aepak` archive size in bytes.
    pub pak_bytes: u64,
}

struct PreparedPakEntry {
    guid: AssetGuid,
    virtual_path: String,
    kind: AssetKind,
    codec: PakCodec,
    bytes: Vec<u8>,
}

/// Incrementally cooks all discovered assets in parallel via `rayon`, reusing valid
/// `Intermediate/<target>/` cache blobs, and packs worlds, chunks, and project manifest into
/// a `16`-byte aligned `game.aepak` archive at `pak_output_path`.
///
/// # Errors
/// Returns [`ExportError`] if any asset fails to read/decode or archive writing fails.
pub fn cook_and_pack_project(
    graph: &DependencyGraph,
    pak_output_path: &Path,
    profile: TargetTextureProfile,
    compression: CookCompressionProfile,
    progress: &mut dyn FnMut(f32, &str),
) -> ExportResult<CookSummary> {
    let layout = ProjectWorkspaceLayout::new(&graph.project_root);
    let intermediate_dir = layout.intermediate_dir();
    let cache_hits = AtomicUsize::new(0);
    let cooked_fresh = AtomicUsize::new(0);
    let raw_copied = AtomicUsize::new(0);
    let codec = compression.to_pak_codec();

    progress(0.15, "Cooking project assets in parallel...");

    let prepared_assets: Vec<PreparedPakEntry> = graph
        .assets
        .par_iter()
        .map(|asset| {
            cook_single_asset(
                asset,
                &intermediate_dir,
                profile,
                codec,
                &cache_hits,
                &cooked_fresh,
                &raw_copied,
            )
        })
        .collect::<ExportResult<Vec<_>>>()?;

    progress(0.75, "Packing 16-byte aligned game.aepak...");

    let mut builder = PakBuilder::new();
    builder.add_asset(
        "aeon_project.json",
        AssetKind::Unknown,
        codec,
        graph.project_config_bytes.clone(),
    );

    for world in &graph.worlds {
        let world_bytes = std::fs::read(&world.disk_path)?;
        builder.add_asset(&world.virtual_path, AssetKind::Unknown, codec, world_bytes);
        for (chunk_virt, chunk_disk) in &world.chunk_files {
            let chunk_bytes = std::fs::read(chunk_disk)?;
            builder.add_asset(chunk_virt, AssetKind::Unknown, codec, chunk_bytes);
        }
    }

    for entry in prepared_assets {
        builder.add_asset_with_guid(
            entry.guid,
            &entry.virtual_path,
            entry.kind,
            entry.codec,
            entry.bytes,
        );
    }

    if let Some(parent) = pak_output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let tmp_pak = pak_output_path.with_extension("aepak.tmp");
    builder.write_to_file(&tmp_pak)?;
    std::fs::rename(&tmp_pak, pak_output_path)?;
    let pak_bytes = std::fs::metadata(pak_output_path)?.len();

    progress(1.0, "Packed game.aepak successfully.");

    Ok(CookSummary {
        total_assets: graph.assets.len(),
        cache_hits: cache_hits.load(Ordering::Relaxed),
        cooked_fresh: cooked_fresh.load(Ordering::Relaxed),
        raw_copied: raw_copied.load(Ordering::Relaxed),
        pak_bytes,
    })
}

fn cook_single_asset(
    asset: &DiscoveredAsset,
    intermediate_dir: &Path,
    profile: TargetTextureProfile,
    codec: PakCodec,
    cache_hits: &AtomicUsize,
    cooked_fresh: &AtomicUsize,
    raw_copied: &AtomicUsize,
) -> ExportResult<PreparedPakEntry> {
    let raw_bytes = std::fs::read(&asset.disk_path)?;
    let content_hash = compute_asset_content_hash(&raw_bytes);

    let (final_bytes, entry_codec) = match cook_ext_for_kind(asset.kind) {
        Some(ext) => {
            if let Some(cached) =
                read_cached_cook_bytes(intermediate_dir, profile, content_hash, ext)
            {
                cache_hits.fetch_add(1, Ordering::Relaxed);
                // Cooked envelopes already store LZ4-compressed sections; store Raw in pak to avoid double-compression
                (cached, PakCodec::Raw)
            } else {
                let cooked = encode_cooked_asset(asset, &raw_bytes, profile)?;
                let _ =
                    write_cached_cook_bytes(intermediate_dir, profile, content_hash, ext, &cooked)?;
                cooked_fresh.fetch_add(1, Ordering::Relaxed);
                (cooked, PakCodec::Raw)
            }
        }
        None => {
            raw_copied.fetch_add(1, Ordering::Relaxed);
            (raw_bytes, codec)
        }
    };

    Ok(PreparedPakEntry {
        guid: asset.guid,
        virtual_path: asset.virtual_path.clone(),
        kind: asset.kind,
        codec: entry_codec,
        bytes: final_bytes,
    })
}

const fn cook_ext_for_kind(kind: AssetKind) -> Option<&'static str> {
    match kind {
        AssetKind::Model => Some("ckmesh"),
        AssetKind::Texture => Some("cktex"),
        AssetKind::Audio => Some("ckaud"),
        _ => None,
    }
}

fn encode_cooked_asset(
    asset: &DiscoveredAsset,
    raw_bytes: &[u8],
    profile: TargetTextureProfile,
) -> ExportResult<Vec<u8>> {
    match asset.kind {
        AssetKind::Texture => {
            let cpu =
                parse_texture_bytes(raw_bytes, &asset.virtual_path, ae_texture::ColorSpace::Srgb)
                    .map_err(|reason| ExportError::AssetCookFailed {
                    asset: asset.virtual_path.clone(),
                    reason,
                })?;
            let cooked = cpu_texture_to_cooked(&cpu, profile);
            Ok(cooked.encode()?)
        }
        AssetKind::Model => {
            let parsed = parse_gltf_bytes(
                raw_bytes,
                asset.disk_path.clone(),
                &asset.virtual_path,
                asset.virtual_path.clone(),
            )
            .map_err(|reason| ExportError::AssetCookFailed {
                asset: asset.virtual_path.clone(),
                reason,
            })?;
            let cooked = parsed_model_to_cooked(&parsed);
            Ok(cooked.encode()?)
        }
        AssetKind::Audio => {
            let cooked = decode_audio_bytes_to_cooked(raw_bytes).map_err(|reason| {
                ExportError::AssetCookFailed {
                    asset: asset.virtual_path.clone(),
                    reason,
                }
            })?;
            Ok(cooked.encode()?)
        }
        _ => Ok(raw_bytes.to_vec()),
    }
}

fn parsed_model_to_cooked(parsed: &ae_renderer::asset::ParsedModelData) -> CookedMesh {
    let submeshes = parsed
        .submeshes
        .iter()
        .map(|s| CookedSubmesh {
            start_index: s.start_index,
            index_count: s.index_count,
            texture_index: s.texture_index.and_then(|i| u32::try_from(i).ok()),
            base_color: s.base_color,
            alpha_mode: match s.alpha_mode {
                SubmeshAlphaMode::Opaque => CookedAlphaMode::Opaque,
                SubmeshAlphaMode::Mask => CookedAlphaMode::Mask,
                SubmeshAlphaMode::Blend => CookedAlphaMode::Blend,
            },
            alpha_cutoff: s.alpha_cutoff,
            metallic_factor: s.metallic_factor,
            roughness_factor: s.roughness_factor,
            uv_scale: s.uv_scale,
        })
        .collect();

    CookedMesh {
        min: parsed.min,
        max: parsed.max,
        vertex_stride: std::mem::size_of::<ae_renderer::render::types::Vertex>() as u32,
        vertex_count: parsed.all_vertices.len() as u32,
        index_count: parsed.all_indices.len() as u32,
        submeshes,
        vertices: AlignedBuffer::from_slice(bytemuck::cast_slice(&parsed.all_vertices)),
        indices: AlignedBuffer::from_slice(bytemuck::cast_slice(&parsed.all_indices)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_world::{
        CookedTexture, EntityRecord, GuidGenerator, SCENE_ASSET_BINDING_TYPE, SceneAssetBinding,
        WorldDocument, open_pak_file,
    };

    #[test]
    fn test_cook_and_pack_project_caches_in_intermediate_and_builds_aligned_pak() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("ae_export_cook_{nanos}"));
        let layout = ProjectWorkspaceLayout::new(&root);
        layout.ensure_directories().expect("ensure dirs");

        std::fs::write(
            root.join("aeon_project.json"),
            br#"{"name":"CookTest","entry_scene":"Worlds/MainWorld/world.aeworld","is_2d":false}"#,
        )
        .expect("write project json");

        // Build a valid 2x2 RGBA PNG via image crate (re-exported through ae_texture or raw PNG)
        let tex_rel = "Content/textures/test.png";
        let tex_path = root.join(tex_rel);
        std::fs::create_dir_all(tex_path.parent().expect("parent")).expect("mkdir");
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 255]));
        img.save(&tex_path).expect("save png");

        let mut ids = GuidGenerator::from_seed(77);
        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, 128.0);
        let mut ent = EntityRecord::new(ids.next_entity_guid(), None);
        ent.components.push(
            ae_world::ComponentRecord::from_value(
                SCENE_ASSET_BINDING_TYPE,
                1,
                &SceneAssetBinding {
                    model_path: None,
                    sprite_path: Some(tex_rel.to_string()),
                    lod_group: None,
                },
            )
            .expect("binding"),
        );
        doc.insert_entity_at([0.0, 0.0, 0.0], ent);
        doc.save_partitioned_dir(&layout.worlds_dir().join("MainWorld"), None)
            .expect("save world");

        let graph = crate::collect_project_dependencies(&root, &[]).expect("collect deps");
        let pak_path = root.join("dist/game.aepak");

        let summary_1 = cook_and_pack_project(
            &graph,
            &pak_path,
            TargetTextureProfile::Desktop,
            CookCompressionProfile::FastLz4,
            &mut |_, _| {},
        )
        .expect("first cook");
        assert_eq!(summary_1.cooked_fresh, 1);
        assert_eq!(summary_1.cache_hits, 0);
        assert!(pak_path.is_file());

        // Second run must hit Intermediate/desktop/ cache cleanly
        let summary_2 = cook_and_pack_project(
            &graph,
            &pak_path,
            TargetTextureProfile::Desktop,
            CookCompressionProfile::FastLz4,
            &mut |_, _| {},
        )
        .expect("second cook");
        assert_eq!(summary_2.cooked_fresh, 0);
        assert_eq!(summary_2.cache_hits, 1);

        let mut reader = open_pak_file(&pak_path).expect("open game.aepak");
        let tex_bytes = reader.read_asset_by_path(tex_rel).expect("read cooked tex");
        let decoded = CookedTexture::decode(&tex_bytes).expect("decode cooked tex");
        assert_eq!(decoded.width, 2);
        assert_eq!(decoded.height, 2);

        let _ = std::fs::remove_dir_all(&root);
    }
}
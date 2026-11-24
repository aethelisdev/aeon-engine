// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Parallel Scene Asset Collector & Background Parsing Pipeline.
//!
//! Collects unique asset dependencies (`SceneAssetBinding`) from decoded world records
//! and parses GLTF geometry and RGBA texture payloads in parallel across `rayon` threads.

use ae_mesh::{ParsedModelData, is_safe_path, parse_gltf_file};
use ae_plugin_api::EntityGuid;
use ae_world::{
    EntityRecord, SCENE_ASSET_BINDING_TYPE, SceneAssetBinding, SpatialCellCoord,
    component_type_hash,
};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;

/// Parsed texture data bundled with its canonicalized filesystem path.
pub type ParsedTextureItem = (PathBuf, image::RgbaImage);

/// Tuple pairing a virtual asset path with its parallel background parse outcome.
pub type ParsedTextureEntry = (String, Result<ParsedTextureItem, String>);

/// Bundle of parsed asset data and binary `EntityRecord`s returned by background workers.
pub struct PendingSceneData {
    /// Persistent [`EntityGuid`] read from the loaded `.aeworld` header.
    pub world_guid: EntityGuid,
    /// Decoded binary entity records collected from persistent and partitioned world tables.
    pub entities: Vec<EntityRecord>,
    /// Partitioned world directory path when loaded from a modular `world.aeworld` directory.
    pub saved_world_dir: Option<PathBuf>,
    /// CRC32 map of loaded partition cells used for subsequent dirty-only saves.
    pub cell_crcs: HashMap<SpatialCellCoord, u32>,
    /// Pre-parsed 3D GLTF/GLB geometry and skinning payloads.
    pub parsed_models: Vec<(String, Result<ParsedModelData, String>)>,
    /// Pre-parsed 2D texture RGBA payloads.
    pub parsed_textures: Vec<ParsedTextureEntry>,
}

/// Parameters required to spawn parallel background scene asset parsing.
pub struct SceneAssetParseRequest {
    /// Persistent world GUID parsed from archive index.
    pub world_guid: EntityGuid,
    /// Decoded entity records from the archive.
    pub entities: Vec<EntityRecord>,
    /// Resolved partitioned world directory if loaded from modular partitions.
    pub saved_world_dir: Option<PathBuf>,
    /// Loaded spatial partition CRC32 map.
    pub cell_crcs: HashMap<SpatialCellCoord, u32>,
    /// Unique validated model paths to parse on worker threads.
    pub models_list: Vec<String>,
    /// Unique validated texture paths to decode on worker threads.
    pub textures_list: Vec<String>,
    /// Human-readable world path label for diagnostics logging.
    pub world_label: String,
}

/// Collects and validates unique GLTF/texture asset paths referenced by decoded entities.
pub fn collect_scene_asset_paths(
    entities: &[EntityRecord],
) -> std::io::Result<(Vec<String>, Vec<String>)> {
    let binding_hash = component_type_hash(SCENE_ASSET_BINDING_TYPE);
    let mut unique_models = HashSet::new();
    let mut unique_textures = HashSet::new();

    for rec in entities {
        for comp in &rec.components {
            if comp.type_hash != binding_hash {
                continue;
            }
            let binding: SceneAssetBinding = comp
                .decode_as()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

            if let Some(mp) = binding.model_path {
                if !is_safe_path(&mp) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("Security Error: Unsafe model path detected: {mp}"),
                    ));
                }
                unique_models.insert(mp);
            }
            if let Some(sp) = binding.sprite_path {
                if !is_safe_path(&sp) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!("Security Error: Unsafe sprite path detected: {sp}"),
                    ));
                }
                unique_textures.insert(sp);
            }
            if let Some(lg) = binding.lod_group {
                if is_safe_path(&lg.lod_0_path) {
                    unique_models.insert(lg.lod_0_path);
                }
                if let Some(p1) = lg.lod_1_path
                    && is_safe_path(&p1)
                {
                    unique_models.insert(p1);
                }
                if let Some(p2) = lg.lod_2_path
                    && is_safe_path(&p2)
                {
                    unique_models.insert(p2);
                }
            }
        }
    }

    Ok((
        unique_models.into_iter().collect(),
        unique_textures.into_iter().collect(),
    ))
}

/// Spawns a `rayon` background task that parses all models and textures referenced by a scene archive
/// in parallel and delivers a [`PendingSceneData`] bundle over the returned channel.
#[must_use]
pub fn spawn_scene_asset_parse(
    req: SceneAssetParseRequest,
) -> Receiver<Result<PendingSceneData, String>> {
    let (tx, rx) = std::sync::mpsc::channel();

    rayon::spawn(move || {
        let parsed_models: Vec<(String, Result<ParsedModelData, String>)> = req
            .models_list
            .into_par_iter()
            .map(|path| {
                let res = parse_gltf_file(&path, String::new());
                (path, res)
            })
            .collect();

        let parsed_textures: Vec<ParsedTextureEntry> = req
            .textures_list
            .into_par_iter()
            .map(|path| {
                let canonical = std::fs::canonicalize(&path)
                    .map_err(|e| format!("Failed to canonicalize texture path '{path}': {e}"));
                let res = canonical.and_then(|c_path| {
                    let cpu = ae_texture::parse_texture_file(&path, ae_texture::ColorSpace::Srgb)?;
                    image::RgbaImage::from_raw(cpu.width, cpu.height, cpu.bytes)
                        .map(|rgba| (c_path, rgba))
                        .ok_or_else(|| format!("Failed to construct RgbaImage for '{path}'"))
                });
                (path, res)
            })
            .collect();

        let _ = tx.send(Ok(PendingSceneData {
            world_guid: req.world_guid,
            entities: req.entities,
            saved_world_dir: req.saved_world_dir,
            cell_crcs: req.cell_crcs,
            parsed_models,
            parsed_textures,
        }));
        log::info!(
            "Finished parallel background parsing for world: {}",
            req.world_label
        );
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_world::{ComponentRecord, GuidGenerator, SavedLodPaths};

    #[test]
    fn test_collect_scene_asset_paths_deduplicates_and_rejects_traversal() {
        let mut ids = GuidGenerator::from_seed(77);
        let binding_a = SceneAssetBinding {
            model_path: Some("assets/models/hero.glb".to_string()),
            sprite_path: Some("assets/textures/albedo.png".to_string()),
            lod_group: Some(SavedLodPaths {
                lod_0_path: "assets/models/hero.glb".to_string(),
                lod_1_path: Some("assets/models/hero_lod1.glb".to_string()),
                lod_2_path: None,
                threshold_1: 20.0,
                threshold_2: 60.0,
            }),
        };
        let rec_a = EntityRecord {
            guid: ids.next_entity_guid(),
            parent_guid: None,
            components: vec![
                ComponentRecord::from_value(SCENE_ASSET_BINDING_TYPE, 1, &binding_a)
                    .expect("encode binding"),
            ],
        };

        let (models, textures) = collect_scene_asset_paths(&[rec_a]).expect("collect paths");
        assert_eq!(models.len(), 2);
        assert!(models.contains(&"assets/models/hero.glb".to_string()));
        assert!(models.contains(&"assets/models/hero_lod1.glb".to_string()));
        assert_eq!(textures, vec!["assets/textures/albedo.png".to_string()]);

        let unsafe_binding = SceneAssetBinding {
            model_path: Some("../../etc/passwd".to_string()),
            sprite_path: None,
            lod_group: None,
        };
        let bad_rec = EntityRecord {
            guid: ids.next_entity_guid(),
            parent_guid: None,
            components: vec![
                ComponentRecord::from_value(SCENE_ASSET_BINDING_TYPE, 1, &unsafe_binding)
                    .expect("encode unsafe binding"),
            ],
        };
        let err = collect_scene_asset_paths(&[bad_rec]).expect_err("reject unsafe traversal path");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
    }
}
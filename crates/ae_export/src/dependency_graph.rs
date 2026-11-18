// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Project dependency graph scanner resolving worlds, partitioned cells, `.aemeta` GUIDs,
//! and referenced models, textures, audio clips, shaders, and prefabs.

use crate::error::{ExportError, ExportResult};
use ae_world::pak::normalize_pak_path;
use ae_world::{
    AssetGuid, AssetKind, AssetMeta, ProjectWorkspaceLayout, SCENE_ASSET_BINDING_TYPE,
    SceneAssetBinding, WorldDocument, asset_guid_from_virtual_path, component_type_hash,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// Minimal `aeon_project.json` fields inspected and packaged during export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProjectManifest {
    /// Human-readable project title.
    #[serde(default = "default_project_name")]
    pub name: String,
    /// Relative virtual path to the entry `.aeworld` launched on startup.
    #[serde(default = "default_entry_scene")]
    pub entry_scene: String,
    /// Whether the project runs primarily in 2D mode.
    #[serde(default)]
    pub is_2d: bool,
}

fn default_project_name() -> String {
    "AeonGame".to_string()
}

fn default_entry_scene() -> String {
    ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL.to_string()
}

/// Discovered world manifest and its associated `.aechunk` files.
#[derive(Debug, Clone)]
pub struct DiscoveredWorld {
    /// Normalized virtual path inside `.aepak` (e.g., `"Worlds/MainWorld/world.aeworld"`).
    pub virtual_path: String,
    /// Absolute or workspace-relative path to the `.aeworld` file on disk.
    pub disk_path: PathBuf,
    /// `true` when the world stores entities across external `cells/*.aechunk` files.
    pub is_partitioned: bool,
    /// Partition cell files as `(virtual_path, disk_path)`.
    pub chunk_files: Vec<(String, PathBuf)>,
}

/// Discovered source asset referenced by one or more worlds to be cooked/packed.
#[derive(Debug, Clone)]
pub struct DiscoveredAsset {
    /// Persistent 128-bit GUID (from `.aemeta` when present, or seeded from virtual path).
    pub guid: AssetGuid,
    /// Normalized virtual path stored in `.aepak` Table of Contents.
    pub virtual_path: String,
    /// Source file location on disk.
    pub disk_path: PathBuf,
    /// Semantic asset kind driving the cooker pipeline.
    pub kind: AssetKind,
}

/// Complete dependency closure discovered for a project export run.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Root directory of the Aeon Engine project workspace.
    pub project_root: PathBuf,
    /// Serialized `aeon_project.json` bytes packaged at archive root.
    pub project_config_bytes: Vec<u8>,
    /// Normalized virtual path of the entry world.
    pub entry_world: String,
    /// All worlds and partition cells included in the export.
    pub worlds: Vec<DiscoveredWorld>,
    /// Deduplicated list of referenced assets on disk.
    pub assets: Vec<DiscoveredAsset>,
    /// Non-fatal warnings for referenced virtual paths missing on disk.
    pub warnings: Vec<String>,
}

/// Scans `project_root` and `selected_worlds` (always ensuring the `aeon_project.json` entry
/// world is included) to construct a deduplicated [`DependencyGraph`].
///
/// # Errors
/// Returns [`ExportError::ProjectNotFound`] if `project_root` does not exist or no valid world
/// can be loaded, or [`ExportError::WorldIo`] / [`ExportError::Io`] on decode failures.
pub fn collect_project_dependencies(
    project_root: &Path,
    selected_worlds: &[PathBuf],
) -> ExportResult<DependencyGraph> {
    if !project_root.is_dir() {
        return Err(ExportError::ProjectNotFound(format!(
            "Project directory does not exist: {}",
            project_root.display()
        )));
    }

    let layout = ProjectWorkspaceLayout::new(project_root);
    let proj_json_path = project_root.join("aeon_project.json");
    let (mut manifest, existing_config_bytes) = if proj_json_path.is_file() {
        let raw = std::fs::read(&proj_json_path)?;
        let parsed: ExportProjectManifest = serde_json::from_slice(&raw)
            .map_err(|e| ExportError::ProjectNotFound(format!("Invalid aeon_project.json: {e}")))?;
        (parsed, Some(raw))
    } else {
        (
            ExportProjectManifest {
                name: default_project_name(),
                entry_scene: default_entry_scene(),
                is_2d: false,
            },
            None,
        )
    };

    let mut entry_world = normalize_pak_path(&manifest.entry_scene);
    let mut world_candidates: Vec<PathBuf> = Vec::new();
    let entry_disk = resolve_world_disk_path(project_root, Path::new(&entry_world));
    let entry_exists_on_disk = entry_disk.is_file();
    if entry_exists_on_disk {
        world_candidates.push(entry_disk);
    }
    for extra in selected_worlds {
        let resolved = resolve_world_disk_path(project_root, extra);
        if resolved.is_file() && !world_candidates.contains(&resolved) {
            world_candidates.push(resolved);
        }
    }

    if world_candidates.is_empty() {
        return Err(ExportError::ProjectNotFound(format!(
            "No exportable .aeworld found in project '{}'",
            project_root.display()
        )));
    }

    let binding_hash = component_type_hash(SCENE_ASSET_BINDING_TYPE);
    let mut worlds = Vec::with_capacity(world_candidates.len());
    let mut asset_map: BTreeMap<String, (AssetKind, Option<AssetGuid>)> = BTreeMap::new();
    let mut first_world_is_2d = false;

    for (idx, world_disk) in world_candidates.into_iter().enumerate() {
        let (doc, _cell_crcs, is_partitioned) = WorldDocument::load_auto(&world_disk, None)?;
        if idx == 0 {
            first_world_is_2d = doc.is_2d;
        }
        let world_virt = to_project_virtual_path(project_root, &world_disk);
        let world_parent_virt = Path::new(&world_virt)
            .parent()
            .map_or_else(String::new, |p| normalize_pak_path(&p.to_string_lossy()));

        let mut chunk_files = Vec::new();
        if is_partitioned && let Some(dir) = world_disk.parent() {
            for coord in doc.cells.keys() {
                let rel_chunk = coord.relative_chunk_path();
                let disk_chunk = dir.join(&rel_chunk);
                if disk_chunk.is_file() {
                    let virt_chunk = if world_parent_virt.is_empty() {
                        rel_chunk
                    } else {
                        format!("{world_parent_virt}/{rel_chunk}")
                    };
                    chunk_files.push((normalize_pak_path(&virt_chunk), disk_chunk));
                }
            }
        }

        for asset_ref in &doc.asset_refs {
            if let Some(raw_path) = doc.strings.get(asset_ref.path_string_idx as usize) {
                let norm = normalize_pak_path(raw_path);
                if !norm.is_empty() {
                    asset_map
                        .entry(norm)
                        .or_insert((asset_ref.kind, Some(asset_ref.guid)));
                }
            }
        }

        let all_entities = doc
            .persistent_entities
            .iter()
            .chain(doc.cells.values().flat_map(|c| c.entities.iter()));
        for ent in all_entities {
            for comp in &ent.components {
                if comp.type_hash == binding_hash
                    && let Ok(binding) = comp.decode_as::<SceneAssetBinding>()
                {
                    collect_binding_paths(&binding, &mut asset_map);
                }
            }
        }

        worlds.push(DiscoveredWorld {
            virtual_path: world_virt,
            disk_path: world_disk,
            is_partitioned,
            chunk_files,
        });
    }

    let mut assets = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_paths = HashSet::new();

    for (virt_path, (kind, hinted_guid)) in asset_map {
        if let Some(disk_path) = resolve_asset_disk_path(project_root, &layout, &virt_path) {
            let canonical_virt = normalize_pak_path(&virt_path);
            if !seen_paths.insert(canonical_virt.clone()) {
                continue;
            }
            let guid = read_companion_guid(&disk_path)
                .or(hinted_guid)
                .unwrap_or_else(|| asset_guid_from_virtual_path(&canonical_virt));
            let resolved_kind = if kind == AssetKind::Unknown {
                classify_asset_extension(&disk_path)
            } else {
                kind
            };
            assets.push(DiscoveredAsset {
                guid,
                virtual_path: canonical_virt,
                disk_path,
                kind: resolved_kind,
            });
        } else {
            warnings.push(format!(
                "Referenced asset missing on disk and skipped: {virt_path}"
            ));
        }
    }

    let input_config_path = project_root.join("Config/input.json");
    if input_config_path.is_file() {
        assets.push(DiscoveredAsset {
            guid: asset_guid_from_virtual_path("Config/input.json"),
            virtual_path: "Config/input.json".to_string(),
            disk_path: input_config_path,
            kind: AssetKind::Unknown,
        });
    }

    let project_config_bytes = match existing_config_bytes {
        Some(raw) if entry_exists_on_disk => raw,
        _ => {
            entry_world = worlds[0].virtual_path.clone();
            manifest.entry_scene = entry_world.clone();
            manifest.is_2d = first_world_is_2d;
            serde_json::to_vec_pretty(&manifest)
                .map_err(|e| ExportError::ProjectNotFound(e.to_string()))?
        }
    };

    Ok(DependencyGraph {
        project_root: project_root.to_path_buf(),
        project_config_bytes,
        entry_world,
        worlds,
        assets,
        warnings,
    })
}

fn collect_binding_paths(
    binding: &SceneAssetBinding,
    asset_map: &mut BTreeMap<String, (AssetKind, Option<AssetGuid>)>,
) {
    if let Some(ref mp) = binding.model_path {
        let norm = normalize_pak_path(mp);
        if !norm.is_empty() {
            asset_map.entry(norm).or_insert((AssetKind::Model, None));
        }
    }
    if let Some(ref sp) = binding.sprite_path {
        let norm = normalize_pak_path(sp);
        if !norm.is_empty() {
            asset_map.entry(norm).or_insert((AssetKind::Texture, None));
        }
    }
    if let Some(ref lod) = binding.lod_group {
        for p in [
            Some(lod.lod_0_path.as_str()),
            lod.lod_1_path.as_deref(),
            lod.lod_2_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            let norm = normalize_pak_path(p);
            if !norm.is_empty() {
                asset_map.entry(norm).or_insert((AssetKind::Model, None));
            }
        }
    }
}

fn resolve_world_disk_path(project_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        project_root.join(candidate)
    }
}

fn to_project_virtual_path(project_root: &Path, disk_path: &Path) -> String {
    let rel = disk_path.strip_prefix(project_root).unwrap_or(disk_path);
    normalize_pak_path(&rel.to_string_lossy())
}

fn resolve_asset_disk_path(
    project_root: &Path,
    layout: &ProjectWorkspaceLayout,
    virt_path: &str,
) -> Option<PathBuf> {
    let clean = virt_path.trim_start_matches('/');
    let candidates = [
        project_root.join(clean),
        layout.content_dir().join(clean),
        project_root.join("assets").join(clean),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn read_companion_guid(asset_disk_path: &Path) -> Option<AssetGuid> {
    let meta_path = AssetMeta::companion_path(asset_disk_path);
    let bytes = std::fs::read(meta_path).ok()?;
    AssetMeta::decode(&bytes).ok().map(|m| m.guid)
}

/// Infers semantic [`AssetKind`] from file extension when world metadata omitted the kind.
#[must_use]
pub fn classify_asset_extension(path: &Path) -> AssetKind {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match ext.as_str() {
        "glb" | "gltf" | "obj" => AssetKind::Model,
        "png" | "jpg" | "jpeg" | "tga" | "bmp" | "webp" => AssetKind::Texture,
        "wav" | "ogg" | "mp3" | "flac" => AssetKind::Audio,
        "wgsl" => AssetKind::Shader,
        "aeprefab" => AssetKind::Prefab,
        _ => AssetKind::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_world::{EntityRecord, GuidGenerator};

    #[test]
    fn test_collect_project_dependencies_discovers_partitioned_chunks_and_aemeta_guids() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("ae_export_dep_{nanos}"));
        let layout = ProjectWorkspaceLayout::new(&root);
        layout.ensure_directories().expect("create project dirs");

        let proj_json = serde_json::json!({
            "name": "DepTest",
            "entry_scene": ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL,
            "is_2d": false
        });
        std::fs::write(
            root.join("aeon_project.json"),
            serde_json::to_vec_pretty(&proj_json).expect("serialize project json"),
        )
        .expect("write aeon_project.json");

        let tex_path = layout.content_dir().join("textures/crate.png");
        std::fs::create_dir_all(tex_path.parent().expect("tex parent")).expect("mkdir tex");
        std::fs::write(&tex_path, b"dummy_png_bytes").expect("write dummy png");
        let meta = AssetMeta::ensure_for_asset_file(
            &tex_path,
            AssetKind::Texture,
            "Content/textures/crate.png",
        )
        .expect("write .aemeta");

        let mut ids = GuidGenerator::from_seed(42);
        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, 128.0);
        let mut ent = EntityRecord::new(ids.next_entity_guid(), None);
        ent.components.push(
            ae_world::ComponentRecord::from_value(
                SCENE_ASSET_BINDING_TYPE,
                1,
                &SceneAssetBinding {
                    model_path: None,
                    sprite_path: Some("Content/textures/crate.png".to_string()),
                    lod_group: None,
                },
            )
            .expect("binding record"),
        );
        doc.insert_entity_at([10.0, 0.0, 10.0], ent);
        doc.register_asset_ref(meta.guid, AssetKind::Texture, "Content/textures/crate.png")
            .expect("register asset ref");

        let world_dir = layout.worlds_dir().join("MainWorld");
        doc.save_partitioned_dir(&world_dir, None)
            .expect("save partitioned world");

        let graph = collect_project_dependencies(&root, &[]).expect("collect deps");
        assert_eq!(graph.worlds.len(), 1);
        assert!(graph.worlds[0].is_partitioned);
        assert_eq!(graph.worlds[0].chunk_files.len(), 1);
        assert_eq!(graph.assets.len(), 1);
        assert_eq!(graph.assets[0].guid, meta.guid);
        assert_eq!(graph.assets[0].kind, AssetKind::Texture);
        assert!(graph.warnings.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_collect_project_dependencies_synthesizes_entry_scene_when_project_json_absent() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let root = std::env::temp_dir().join(format!("ae_export_loose_{nanos}"));
        let scenes_dir = root.join("assets/scenes");
        std::fs::create_dir_all(&scenes_dir).expect("mkdir scenes");

        let mut ids = GuidGenerator::from_seed(55);
        let doc = WorldDocument::new(ids.next_entity_guid(), true, 128.0);
        let scene_path = scenes_dir.join("demo.aeworld");
        doc.save_to_file(&scene_path).expect("save loose scene");

        let graph =
            collect_project_dependencies(&root, &[PathBuf::from("assets/scenes/demo.aeworld")])
                .expect("collect loose scene deps");
        assert_eq!(graph.entry_world, "assets/scenes/demo.aeworld");
        let parsed: ExportProjectManifest =
            serde_json::from_slice(&graph.project_config_bytes).expect("valid synthesized json");
        assert_eq!(parsed.entry_scene, "assets/scenes/demo.aeworld");
        assert!(parsed.is_2d);

        let _ = std::fs::remove_dir_all(&root);
    }
}
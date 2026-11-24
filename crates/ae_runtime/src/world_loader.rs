// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor-free runtime world loader resolving `.aepak` archives, `aeon_project.json` entry
//! worlds, and partitioned `cells/*.aechunk` directories through [`ae_world::VfsProvider`].

use ae_ecs::component_type_hash;
use ae_physics::PhysicsWorld;
use ae_plugin_api::EntityGuid;
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use ae_scene::{
    PrefixedWorldVfs, RuntimeStreamingManager, decode_world_document_from_vfs, spawn_world_records,
};
use ae_transform::update_hierarchy_transforms;
use ae_world::{
    DiskVfsProvider, PakFileVfsProvider, PakMemoryVfsProvider, ProjectWorkspaceLayout,
    SCENE_ASSET_BINDING_TYPE, SpatialCellCoord, VfsProvider, WORLD_MANIFEST_FILENAME,
};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Default streaming radius (in cubic cells) for standalone runtime playback.
pub const DEFAULT_RUNTIME_LOAD_RADIUS: i32 = 2;

/// Default hysteresis unload radius (in cubic cells) for standalone runtime playback.
pub const DEFAULT_RUNTIME_UNLOAD_RADIUS: i32 = 3;

#[derive(Deserialize)]
struct ProjectEntryConfig {
    entry_scene: Option<String>,
}

/// Runtime startup arguments accepted by the standalone `ae_runtime` desktop executable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeLaunchArgs {
    /// Optional explicit `.aepak` container path (`--pak <PATH>`).
    pub pak_path: Option<PathBuf>,
    /// Optional explicit `.aeworld` virtual or filesystem path (`--world <PATH>`).
    pub world_path: Option<String>,
    /// Optional writable directory storing `.aesave` slots (`--save-dir <PATH>`).
    pub save_dir: Option<PathBuf>,
}

impl RuntimeLaunchArgs {
    /// Parses CLI flags from `std::env::args()`.
    #[must_use]
    pub fn from_env() -> Self {
        let mut parsed = Self::default();
        let mut iter = std::env::args().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--pak" => parsed.pak_path = iter.next().map(PathBuf::from),
                "--world" => parsed.world_path = iter.next(),
                "--save-dir" => parsed.save_dir = iter.next().map(PathBuf::from),
                _ => {}
            }
        }
        parsed
    }
}

/// Metadata returned once a runtime world finishes booting from VFS.
#[derive(Debug, Clone)]
pub struct LoadedRuntimeWorld {
    /// Persistent [`EntityGuid`] identifying the loaded `.aeworld` manifest.
    pub world_guid: EntityGuid,
    /// `true` when the archive targets 2D orthographic gameplay.
    pub is_2d: bool,
    /// Partition cell size in world units.
    pub cell_size: f32,
    /// Directory where runtime `slot_<n>.aesave` files are persisted.
    pub save_games_dir: PathBuf,
}

/// Discovers the highest-priority runtime [`VfsProvider`] (`--pak`, adjacent `game.aepak`,
/// optional embedded bytes, or local workspace folder) and entry world virtual path.
///
/// # Errors
/// Returns a descriptive error string when a specified `.aepak` fails validation.
pub fn discover_vfs_and_entry(
    args: &RuntimeLaunchArgs,
    embedded_pak: Option<&'static [u8]>,
) -> Result<(Arc<dyn VfsProvider>, String, PathBuf), String> {
    let exe_pak = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|dir| dir.join("game.aepak")));
    let cwd_pak = PathBuf::from("game.aepak");

    let vfs: Arc<dyn VfsProvider> = if let Some(ref explicit) = args.pak_path {
        Arc::new(
            PakFileVfsProvider::open(explicit)
                .map_err(|e| format!("Failed to open pak '{}': {e}", explicit.display()))?,
        )
    } else if let Some(ref path) = exe_pak
        && path.is_file()
    {
        Arc::new(
            PakFileVfsProvider::open(path)
                .map_err(|e| format!("Failed to open adjacent pak '{}': {e}", path.display()))?,
        )
    } else if cwd_pak.is_file() {
        Arc::new(
            PakFileVfsProvider::open(&cwd_pak)
                .map_err(|e| format!("Failed to open 'game.aepak': {e}"))?,
        )
    } else if let Some(bytes) = embedded_pak {
        Arc::new(
            PakMemoryVfsProvider::from_bytes(bytes.to_vec())
                .map_err(|e| format!("Failed to mount embedded game pak: {e}"))?,
        )
    } else {
        Arc::new(DiskVfsProvider::new("."))
    };

    let entry_world = resolve_entry_world_path(vfs.as_ref(), args.world_path.as_deref());
    let save_dir = args
        .save_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("Saved").join("SaveGames"));
    Ok((vfs, entry_world, save_dir))
}

/// Resolves the starting `.aeworld` virtual path inside `vfs`.
///
/// Priority:
/// 1. Explicit `--world` argument
/// 2. `aeon_project.json::entry_scene` inside `vfs`
/// 3. First `*.aeworld` entry exposed by `vfs`
/// 4. Default [`ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL`]
#[must_use]
pub fn resolve_entry_world_path(vfs: &dyn VfsProvider, explicit_world: Option<&str>) -> String {
    if let Some(path) = explicit_world
        && !path.trim().is_empty()
    {
        return path.trim().replace('\\', "/");
    }

    if let Ok(proj_bytes) = vfs.read_bytes("aeon_project.json")
        && let Ok(cfg) = serde_json::from_slice::<ProjectEntryConfig>(&proj_bytes)
        && let Some(entry) = cfg.entry_scene
        && !entry.trim().is_empty()
    {
        return entry.trim().replace('\\', "/");
    }

    if let Some(found) = vfs
        .list_entries()
        .into_iter()
        .find(|p| p.ends_with(".aeworld"))
    {
        return found;
    }

    ProjectWorkspaceLayout::DEFAULT_ENTRY_WORLD_REL.to_string()
}

/// Loads a world from `vfs`, spawns its initial entities into `world`, binds VFS models/textures
/// via `render_state`, syncs physics colliders, and initializes [`RuntimeStreamingManager`].
///
/// # Errors
/// Returns an error string if the world cannot be read or decoded from `vfs`.
pub fn load_world_into_runtime(
    vfs: Arc<dyn VfsProvider>,
    world_virtual_path: &str,
    save_games_dir: PathBuf,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    world: &mut ae_ecs::World,
    physics_world: &mut PhysicsWorld,
) -> Result<(LoadedRuntimeWorld, RuntimeStreamingManager), String> {
    let mut allowed = ae_plugin_api::engine_component_registry().known_type_hashes();
    allowed.insert(component_type_hash(SCENE_ASSET_BINDING_TYPE));

    let (doc, is_partitioned) =
        decode_world_document_from_vfs(vfs.as_ref(), world_virtual_path, Some(&allowed))
            .map_err(|e| format!("Failed to decode world '{world_virtual_path}' from VFS: {e}"))?;

    let available_cells: HashSet<SpatialCellCoord> = doc.cells.keys().copied().collect();
    let world_guid = doc.world_guid;
    let is_2d = doc.is_2d;
    let cell_size = doc.cell_size;
    let records = doc.into_all_entities();

    spawn_world_records(
        records,
        Some(vfs.as_ref()),
        render_state,
        asset_manager,
        world,
    );
    update_hierarchy_transforms(world);
    physics_world.sync_ecs_to_physics(world, |handle| {
        asset_manager
            .get_physics_mesh_data(handle)
            .map(|(v, i)| (v.as_slice(), i.as_slice()))
    });

    let loaded_cells = if is_partitioned {
        available_cells.clone()
    } else {
        HashSet::new()
    };

    let streaming_vfs: Arc<dyn VfsProvider> = if is_partitioned {
        let prefix = Path::new(world_virtual_path)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        if prefix.is_empty() {
            vfs
        } else {
            Arc::new(PrefixedWorldVfs::new(vfs, prefix))
        }
    } else {
        vfs
    };

    let mut streaming = RuntimeStreamingManager::new(
        streaming_vfs,
        cell_size,
        DEFAULT_RUNTIME_LOAD_RADIUS,
        DEFAULT_RUNTIME_UNLOAD_RADIUS,
        available_cells,
    );
    streaming.seed_loaded_world(loaded_cells, world);

    Ok((
        LoadedRuntimeWorld {
            world_guid,
            is_2d,
            cell_size,
            save_games_dir,
        },
        streaming,
    ))
}

/// Ensures unused manifest constant stays referenced when inspecting default world paths.
#[must_use]
pub fn default_world_manifest_name() -> &'static str {
    WORLD_MANIFEST_FILENAME
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::Position;
    use ae_world::{
        AssetKind, ComponentRecord, EntityRecord, GuidGenerator, PakBuilder, PakCodec,
        WorldDocument,
    };

    #[test]
    fn test_pak_vfs_resolves_project_entry_and_decodes_partitioned_world() {
        let mut ids = GuidGenerator::from_seed(515);
        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, 128.0);
        let mut rec = EntityRecord::new(ids.next_entity_guid(), None);
        rec.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(15.0, 2.0, 20.0))
                .expect("encode pos"),
        );
        doc.insert_entity_at([15.0, 2.0, 20.0], rec);

        let dir = std::env::temp_dir().join("ae_runtime_pak_world_deterministic");
        let _ = std::fs::remove_dir_all(&dir);
        let world_dir = dir.join("Worlds/Dungeon");
        doc.save_partitioned_dir(&world_dir, None)
            .expect("save partitioned dir");

        let mut builder = PakBuilder::new();
        builder.add_asset(
            "aeon_project.json",
            AssetKind::Unknown,
            PakCodec::Raw,
            br#"{"entry_scene":"Worlds/Dungeon/world.aeworld"}"#.to_vec(),
        );
        builder.add_asset(
            "Worlds/Dungeon/world.aeworld",
            AssetKind::Unknown,
            PakCodec::Lz4Block,
            std::fs::read(world_dir.join("world.aeworld")).expect("read manifest"),
        );
        let chunk_rel = SpatialCellCoord::ZERO.relative_chunk_path();
        builder.add_asset(
            &format!("Worlds/Dungeon/{chunk_rel}"),
            AssetKind::Unknown,
            PakCodec::Lz4Block,
            std::fs::read(world_dir.join(&chunk_rel)).expect("read chunk"),
        );

        let pak_bytes = builder.build_to_vec().expect("build pak");
        let vfs = PakMemoryVfsProvider::from_bytes(pak_bytes).expect("mount pak");

        let entry = resolve_entry_world_path(&vfs, None);
        assert_eq!(entry, "Worlds/Dungeon/world.aeworld");

        let (decoded, is_partitioned) =
            decode_world_document_from_vfs(&vfs, &entry, None).expect("decode world from pak");
        assert!(is_partitioned);
        assert_eq!(decoded.world_guid, doc.world_guid);
        assert_eq!(decoded.total_entity_count(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
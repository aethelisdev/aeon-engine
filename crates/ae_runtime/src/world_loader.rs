// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Editor-free runtime world loader resolving `.aepak` archives, `aeon_project.json` entry
//! worlds, and partitioned `cells/*.aechunk` directories through [`ae_world::VfsProvider`].

use crate::streaming::{RuntimeStreamingManager, bind_entity_assets_from_binding};
use ae_core::ecs::{
    Children, ComponentRegistry, EntityGuid, Parent, TransformDirty, component_type_hash,
    update_hierarchy_transforms,
};
use ae_physics::PhysicsWorld;
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use ae_world::{
    DiskVfsProvider, EntityRecord, GuidGenerator, PakFileVfsProvider, PakMemoryVfsProvider,
    PartitionCell, ProjectWorkspaceLayout, SCENE_ASSET_BINDING_TYPE, SceneAssetBinding,
    SpatialCellCoord, VfsProvider, WORLD_MANIFEST_FILENAME, WorldDocument, WorldResult,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
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

/// Decodes a [`WorldDocument`] and all referenced spatial cells directly from `vfs` without OS
/// filesystem directory assumptions.
///
/// Handles both monolithic `.aeworld` archives and partitioned directories whose chunks live under
/// `<world_parent>/cells/<x>_<y>_<z>.aechunk`.
///
/// # Errors
/// Returns [`ae_world::WorldIoError`] on missing entries, corrupted headers, or CRC32 mismatches.
pub fn decode_world_document_from_vfs(
    vfs: &dyn VfsProvider,
    world_virtual_path: &str,
    known_hashes: Option<&HashSet<u64>>,
) -> WorldResult<(WorldDocument, bool)> {
    let normalized = world_virtual_path.replace('\\', "/");
    let manifest_bytes = vfs.read_bytes(&normalized)?;
    let mut cursor = Cursor::new(manifest_bytes.as_slice());
    let (index, is_partitioned) = WorldDocument::read_index_with_flags(&mut cursor, known_hashes)?;

    if !is_partitioned {
        return WorldDocument::decode_full(&manifest_bytes, known_hashes).map(|doc| (doc, false));
    }

    let world_parent = Path::new(&normalized)
        .parent()
        .map_or_else(String::new, |p| p.to_string_lossy().replace('\\', "/"));

    let mut cells = std::collections::BTreeMap::new();
    for part in &index.partitions {
        let rel = part.coord.relative_chunk_path();
        let full_rel = if world_parent.is_empty() {
            rel
        } else {
            format!("{world_parent}/{rel}")
        };
        let chunk_bytes = vfs.read_bytes(&full_rel)?;
        let cell = PartitionCell::decode_chunk(&chunk_bytes, known_hashes)?;
        cells.insert(part.coord, cell);
    }

    Ok((WorldDocument::from_index_and_cells(index, cells)?, true))
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
    world: &mut hecs::World,
    physics_world: &mut PhysicsWorld,
) -> Result<(LoadedRuntimeWorld, RuntimeStreamingManager), String> {
    let mut allowed = ComponentRegistry::global().known_type_hashes();
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
            Arc::new(PrefixedWorldVfs { inner: vfs, prefix })
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

struct PrefixedWorldVfs {
    inner: Arc<dyn VfsProvider>,
    prefix: String,
}

impl PrefixedWorldVfs {
    fn prefixed(&self, virtual_path: &str) -> String {
        let clean = virtual_path
            .trim_start_matches("./")
            .trim_start_matches('/');
        if clean.starts_with(&self.prefix) {
            clean.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), clean)
        }
    }
}

impl VfsProvider for PrefixedWorldVfs {
    fn exists(&self, virtual_path: &str) -> bool {
        self.inner.exists(&self.prefixed(virtual_path)) || self.inner.exists(virtual_path)
    }

    fn read_bytes(&self, virtual_path: &str) -> WorldResult<Vec<u8>> {
        self.inner
            .read_bytes(&self.prefixed(virtual_path))
            .or_else(|_| self.inner.read_bytes(virtual_path))
    }

    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<ae_world::AlignedBuffer> {
        self.inner
            .read_aligned_bytes(&self.prefixed(virtual_path))
            .or_else(|_| self.inner.read_aligned_bytes(virtual_path))
    }

    fn read_by_guid(&self, guid: ae_world::AssetGuid) -> WorldResult<Vec<u8>> {
        self.inner.read_by_guid(guid)
    }

    fn read_aligned_by_guid(
        &self,
        guid: ae_world::AssetGuid,
    ) -> WorldResult<ae_world::AlignedBuffer> {
        self.inner.read_aligned_by_guid(guid)
    }

    fn list_entries(&self) -> Vec<String> {
        self.inner.list_entries()
    }
}

fn spawn_world_records(
    records: Vec<EntityRecord>,
    vfs: Option<&dyn VfsProvider>,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    world: &mut hecs::World,
) {
    let registry = ComponentRegistry::global();
    let binding_hash = component_type_hash(SCENE_ASSET_BINDING_TYPE);
    let mut guid_to_entity: HashMap<EntityGuid, hecs::Entity> =
        HashMap::with_capacity(records.len());
    let mut pending_parents: Vec<(hecs::Entity, EntityGuid)> = Vec::new();
    let mut guid_gen = GuidGenerator::new();

    for rec in records {
        let ent = world.spawn(());
        let entity_guid = if rec.guid.is_nil() {
            guid_gen.next_entity_guid()
        } else {
            rec.guid
        };
        let _ = world.insert_one(ent, entity_guid);
        guid_to_entity.insert(entity_guid, ent);

        let mut has_physics = false;
        for comp in &rec.components {
            if comp.type_hash == binding_hash {
                if let Ok(binding) = comp.decode_as::<SceneAssetBinding>() {
                    bind_entity_assets_from_binding(
                        world,
                        render_state,
                        asset_manager,
                        vfs,
                        ent,
                        &binding,
                    );
                }
                continue;
            }
            if let Some(handler) = registry.get_by_type_hash(comp.type_hash)
                && handler
                    .apply_versioned(world, ent, comp.schema_ver, &comp.payload)
                    .is_ok()
                && matches!(
                    handler.type_name(),
                    "RigidBody" | "Collider" | "CharacterController"
                )
            {
                has_physics = true;
            }
        }
        if has_physics {
            let _ = world.insert_one(ent, TransformDirty);
        }
        if let Some(parent_guid) = rec.parent_guid {
            pending_parents.push((ent, parent_guid));
        }
    }

    for (child, parent_guid) in pending_parents {
        if let Some(&parent) = guid_to_entity.get(&parent_guid) {
            let _ = world.insert_one(child, Parent(parent));
            if let Ok(mut children) = world.get::<&mut Children>(parent) {
                if !children.0.contains(&child) {
                    children.0.push(child);
                }
            } else {
                let _ = world.insert_one(parent, Children(vec![child]));
            }
        }
    }
}

/// Ensures unused manifest constant stays referenced when inspecting default world paths.
#[must_use]
pub fn default_world_manifest_name() -> &'static str {
    WORLD_MANIFEST_FILENAME
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_core::ecs::Position;
    use ae_world::{AssetKind, ComponentRecord, PakBuilder, PakCodec};

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

        // Save partitioned tree into temporary dir, then pack all files into an in-memory .aepak
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let dir = std::env::temp_dir().join(format!("ae_runtime_pak_world_{nanos}"));
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
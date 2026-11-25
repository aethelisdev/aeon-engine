// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Partitioned `.aeworld` archive decoding, background asset preparation, and unified ECS
//! world spawning shared by both the editor and the standalone runtime player.

use crate::serialization::DEFAULT_WORLD_CELL_SIZE;
use crate::streaming::{RuntimeStreamingManager, bind_entity_assets_from_binding};
use ae_assets::{PendingSceneData, SceneAssetParseRequest};
use ae_ecs::{Entity, World, component_type_hash};
use ae_plugin_api::{Children, EntityGuid, Parent, TransformDirty};
use ae_renderer::asset::AssetManager;
use ae_renderer::render::RenderState;
use ae_transform::update_hierarchy_transforms;
use ae_world::{
    AlignedBuffer, AssetGuid, EntityRecord, GuidGenerator, PartitionCell, SCENE_ASSET_BINDING_TYPE,
    SceneAssetBinding, SpatialCellCoord, VfsProvider, WORLD_MANIFEST_FILENAME, WorldDocument,
    WorldResult, resolve_partitioned_world_paths,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;

/// Maximum binary world archive manifest size accepted when opening a scene (256 MiB).
pub const MAX_SCENE_FILE_SIZE: u64 = 256 * 1024 * 1024;

/// Maximum total entity count permitted inside a single decoded `.aeworld` scene.
pub const MAX_SCENE_ENTITIES: usize = 500_000;

/// Prefixed virtual filesystem view scoping chunk reads to `<world_parent>/cells/*.aechunk`.
pub struct PrefixedWorldVfs {
    inner: Arc<dyn VfsProvider>,
    prefix: String,
}

impl PrefixedWorldVfs {
    /// Wraps `inner` so relative requests resolve beneath `prefix`.
    #[must_use]
    pub fn new(inner: Arc<dyn VfsProvider>, prefix: String) -> Self {
        Self { inner, prefix }
    }

    fn prefixed(&self, rel: &str) -> String {
        let clean = rel.trim_start_matches("./").trim_start_matches('/');
        if self.prefix.is_empty() || clean.starts_with(&self.prefix) {
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

    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<AlignedBuffer> {
        self.inner
            .read_aligned_bytes(&self.prefixed(virtual_path))
            .or_else(|_| self.inner.read_aligned_bytes(virtual_path))
    }

    fn read_by_guid(&self, guid: AssetGuid) -> WorldResult<Vec<u8>> {
        self.inner.read_by_guid(guid)
    }

    fn read_aligned_by_guid(&self, guid: AssetGuid) -> WorldResult<AlignedBuffer> {
        self.inner.read_aligned_by_guid(guid)
    }

    fn list_entries(&self) -> Vec<String> {
        self.inner.list_entries()
    }
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

    let mut cells = BTreeMap::new();
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

    WorldDocument::from_index_and_cells(index, cells).map(|doc| (doc, true))
}

/// Validates and decodes an `.aeworld` archive or partitioned folder on disk and builds a
/// [`SceneAssetParseRequest`] ready for background asset decoding.
///
/// # Errors
/// Returns `PermissionDenied` when `index.is_2d != expected_is_2d`, `InvalidData` on corrupted
/// archives or size/entity limit violations, or I/O errors when reading from disk.
pub fn prepare_world_load_request(
    target_path: &Path,
    expected_is_2d: bool,
) -> std::io::Result<SceneAssetParseRequest> {
    let manifest_to_check = if target_path.is_dir() {
        target_path.join(WORLD_MANIFEST_FILENAME)
    } else {
        target_path.to_path_buf()
    };
    let metadata = std::fs::metadata(&manifest_to_check)?;
    if metadata.len() > MAX_SCENE_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "World archive too large ({} bytes, max {} bytes)",
                metadata.len(),
                MAX_SCENE_FILE_SIZE
            ),
        ));
    }

    let file = File::open(&manifest_to_check)?;
    let mut reader = BufReader::new(file);
    let index = WorldDocument::read_index(&mut reader, None)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    if expected_is_2d && !index.is_2d {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "Blocked loading 3D world '{}' while running in 2D mode. Switch to 3D mode to open this world.",
                target_path.display()
            ),
        ));
    }

    if !expected_is_2d && index.is_2d {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "Blocked loading 2D world '{}' while running in 3D mode. Switch to 2D mode to open this world.",
                target_path.display()
            ),
        ));
    }

    let registry = crate::scene_component_registry();
    let mut allowed_hashes = registry.known_type_hashes();
    allowed_hashes.insert(component_type_hash(SCENE_ASSET_BINDING_TYPE));

    let (world_doc, cell_crcs, is_partitioned_dir) =
        WorldDocument::load_auto(target_path, Some(&allowed_hashes))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let resolved_world_dir = if is_partitioned_dir {
        if target_path.is_dir() {
            Some(target_path.to_path_buf())
        } else {
            Some(resolve_partitioned_world_paths(&manifest_to_check).0)
        }
    } else {
        None
    };

    let total_entities = world_doc.total_entity_count();
    if total_entities > MAX_SCENE_ENTITIES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("World has too many entities ({total_entities}, max {MAX_SCENE_ENTITIES})"),
        ));
    }

    let world_guid = world_doc.world_guid;
    let all_entities = world_doc.into_all_entities();
    let (models_list, textures_list) = ae_assets::collect_scene_asset_paths(&all_entities)?;

    Ok(SceneAssetParseRequest {
        world_guid,
        entities: all_entities,
        saved_world_dir: resolved_world_dir,
        cell_crcs,
        models_list,
        textures_list,
        world_label: target_path.to_string_lossy().into_owned(),
    })
}

/// Validates and starts background parsing of an `.aeworld` scene archive and its referenced
/// model/texture assets, returning the completion receiver.
///
/// # Errors
/// Propagates validation and filesystem errors from [`prepare_world_load_request`].
pub fn begin_async_scene_load(
    target_path: &Path,
    expected_is_2d: bool,
) -> std::io::Result<Receiver<Result<PendingSceneData, String>>> {
    let request = prepare_world_load_request(target_path, expected_is_2d)?;
    Ok(ae_assets::spawn_scene_asset_parse(request))
}

/// Spawns decoded [`EntityRecord`] entries into `world`, binding registered component payloads,
/// scene asset bindings (`ModelId`/`SpriteId`/`LodGroup`), `TransformDirty` physics flags, and
/// persistent `EntityGuid` `Parent`/`Children` hierarchies.
pub fn spawn_world_records(
    records: Vec<EntityRecord>,
    vfs: Option<&dyn VfsProvider>,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    world: &mut World,
) {
    let registry = crate::scene_component_registry();
    let binding_hash = component_type_hash(SCENE_ASSET_BINDING_TYPE);
    let mut guid_to_entity: HashMap<EntityGuid, Entity> = HashMap::with_capacity(records.len());
    let mut pending_parents: Vec<(Entity, EntityGuid)> = Vec::new();
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
            if let Ok(children) = world.get_mut::<Children>(parent) {
                if !children.0.contains(&child) {
                    children.0.push(child);
                }
            } else {
                let _ = world.insert_one(parent, Children(vec![child]));
            }
        }
    }
}

/// World state metadata produced once [`apply_loaded_scene_data`] finishes uploading GPU assets
/// and reconstructing all entities inside the live ECS world.
pub struct AppliedWorldLoad {
    /// Active partitioned world folder on disk when opened from a partitioned archive.
    pub saved_world_dir: Option<PathBuf>,
    /// Persistent world identifier decoded from the archive header.
    pub world_guid: EntityGuid,
    /// Per-cell CRC32 checksums recorded for differential chunk saving.
    pub cell_crcs: HashMap<SpatialCellCoord, u32>,
    /// Pre-seeded chunk streaming manager when loaded from a partitioned directory.
    pub streaming: Option<RuntimeStreamingManager>,
}

/// Uploads GPU assets from completed `scene_data`, resets `world`, spawns all entities with
/// reconstructed hierarchies, updates transforms, and initializes partition streaming when applicable.
pub fn apply_loaded_scene_data(
    scene_data: PendingSceneData,
    world: &mut World,
    render_state: &RenderState,
    asset_manager: &mut AssetManager,
    active_vfs: Option<&dyn VfsProvider>,
) -> AppliedWorldLoad {
    for (orig_path, model_res) in scene_data.parsed_models {
        match model_res {
            Ok(parsed_data) => {
                render_state.upload_model_data(asset_manager, parsed_data);
            }
            Err(e) => log::error!("Failed to parse model '{orig_path}' during world load: {e}"),
        }
    }

    for (orig_path, tex_res) in scene_data.parsed_textures {
        match tex_res {
            Ok((c_path, rgba)) => {
                render_state.upload_texture_data(asset_manager, c_path, rgba, &orig_path);
            }
            Err(e) => log::error!("Failed to parse texture '{orig_path}' during world load: {e}"),
        }
    }

    *world = World::new();
    spawn_world_records(
        scene_data.entities,
        active_vfs,
        render_state,
        asset_manager,
        world,
    );
    update_hierarchy_transforms(world);

    let streaming = scene_data.saved_world_dir.as_ref().map(|world_dir| {
        let cells: HashSet<SpatialCellCoord> = scene_data.cell_crcs.keys().copied().collect();
        RuntimeStreamingManager::from_world_dir(
            world_dir,
            DEFAULT_WORLD_CELL_SIZE,
            2,
            3,
            cells.clone(),
            cells,
            world,
        )
    });

    AppliedWorldLoad {
        saved_world_dir: scene_data.saved_world_dir,
        world_guid: scene_data.world_guid,
        cell_crcs: scene_data.cell_crcs,
        streaming,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ae_plugin_api::Position;
    use ae_world::{AssetKind, ComponentRecord, PakBuilder, PakCodec, PakMemoryVfsProvider};

    #[test]
    fn test_decode_world_document_from_pak_vfs_and_reject_dimension_mismatch() {
        let mut ids = GuidGenerator::from_seed(515);
        let mut doc = WorldDocument::new(ids.next_entity_guid(), false, DEFAULT_WORLD_CELL_SIZE);
        let mut rec = EntityRecord::new(ids.next_entity_guid(), None);
        rec.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(15.0, 2.0, 20.0))
                .expect("encode pos"),
        );
        doc.insert_entity_at([15.0, 2.0, 20.0], rec);

        let dir = std::env::temp_dir().join("ae_scene_test_loader_deterministic");
        let _ = std::fs::remove_dir_all(&dir);
        let world_dir = dir.join("Worlds/Dungeon");
        doc.save_partitioned_dir(&world_dir, None)
            .expect("save partitioned dir");

        // Negative boundary check: loading a 3D world when expecting 2D fails with PermissionDenied
        let err = prepare_world_load_request(&world_dir, true)
            .err()
            .expect("3D world must fail 2D check");
        assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);

        let mut builder = PakBuilder::new();
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

        let (decoded, is_partitioned) =
            decode_world_document_from_vfs(&vfs, "Worlds/Dungeon/world.aeworld", None)
                .expect("decode world from pak");
        assert!(is_partitioned);
        assert_eq!(decoded.world_guid, doc.world_guid);
        assert_eq!(decoded.total_entity_count(), 1);

        // Prefixed VFS cleanly exposes chunk relative to world root
        let prefixed = PrefixedWorldVfs::new(Arc::new(vfs), "Worlds/Dungeon".to_string());
        assert!(prefixed.exists(&chunk_rel));
        assert!(prefixed.read_bytes(&chunk_rel).is_ok());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
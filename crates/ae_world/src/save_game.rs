// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Runtime player SaveGame (`.aesave`) delta snapshot format with spatial cell indexing,
//! CRC32 validation, and crash-safe atomic write / `.bak` fallback recovery.

use crate::component_record::EntityRecord;
use crate::error::{WorldIoError, WorldResult};
use crate::guid::EntityGuid;
use crate::header::{SAVE_MAGIC, decode_checksummed_envelope, encode_checksummed_envelope};
use crate::partition::SpatialCellCoord;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Current `.aesave` cell-indexed delta snapshot binary format version.
pub const SAVE_FORMAT_VERSION: u32 = 2;

/// Header metadata summarizing a player save slot without needing to apply ECS deltas first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotMetadata {
    /// Save slot number (`0` for autosave/quicksave, `1..N` for manual slots).
    pub slot_index: u16,
    /// Persistent [`EntityGuid`] of the `.aeworld` level this save delta belongs to.
    pub world_guid: EntityGuid,
    /// Save creation time in UNIX epoch seconds.
    pub timestamp_unix: u64,
    /// Accumulated player session duration in seconds.
    pub play_time_secs: u64,
    /// Human-readable slot title (e.g., `"Chapter 2 - Foundry"`).
    pub label: String,
}

/// Strongly-typed binary value stored in [`WorldDeltaSnapshot::variables`] without JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameplayValue {
    /// Boolean quest/switch flag.
    Bool(bool),
    /// Signed 64-bit integer counter, score, or currency.
    I64(i64),
    /// Double-precision scalar statistic or timer.
    F64(f64),
    /// UTF-8 string value (checkpoint name, dialogue state).
    Text(String),
    /// Opaque binary blob for custom game plugin save state.
    Bytes(Vec<u8>),
}

/// Scoped runtime changes relative to either global persistent entities or one [`SpatialCellCoord`].
///
/// Organizing save deltas per spatial cell keeps chunk-streaming lookups `O(log C)` regardless of
/// total world size and resolves asymmetric boundary crossings where a destination cell loads before
/// an entity's origin chunk.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellDelta {
    /// Chunk-native entities destroyed, picked up, or relocated out of this cell.
    pub destroyed_entities: Vec<EntityGuid>,
    /// Chunk-native entities whose state or components diverged from base `.aechunk` data.
    pub modified_entities: Vec<EntityRecord>,
    /// Dynamically spawned runtime entities or entities relocated into this cell from elsewhere.
    pub spawned_entities: Vec<EntityRecord>,
}

impl CellDelta {
    /// Returns `true` when this cell contains zero destroyed, modified, or spawned records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.destroyed_entities.is_empty()
            && self.modified_entities.is_empty()
            && self.spawned_entities.is_empty()
    }

    /// Marks `guid` destroyed in this cell, removing any conflicting modified or spawned entry.
    pub fn mark_destroyed(&mut self, guid: EntityGuid) {
        self.modified_entities.retain(|r| r.guid != guid);
        self.spawned_entities.retain(|r| r.guid != guid);
        if !self.destroyed_entities.contains(&guid) {
            self.destroyed_entities.push(guid);
        }
    }

    /// Inserts or updates a modified entity record inside this cell.
    ///
    /// If `record.guid` currently lives in [`Self::spawned_entities`], updates that spawned snapshot
    /// in-place so duplicate records never accumulate; otherwise clears any destroyed marker and
    /// upserts into [`Self::modified_entities`].
    pub fn upsert_modified(&mut self, record: EntityRecord) {
        if let Some(existing) = self
            .spawned_entities
            .iter_mut()
            .find(|r| r.guid == record.guid)
        {
            *existing = record;
            return;
        }
        self.destroyed_entities.retain(|g| *g != record.guid);
        upsert_record_vec(&mut self.modified_entities, record);
    }

    /// Inserts or updates a dynamically spawned or relocated entity snapshot inside this cell.
    ///
    /// Clears any conflicting destroyed or modified entry for `record.guid`.
    pub fn upsert_spawned(&mut self, record: EntityRecord) {
        self.destroyed_entities.retain(|g| *g != record.guid);
        self.modified_entities.retain(|r| r.guid != record.guid);
        upsert_record_vec(&mut self.spawned_entities, record);
    }

    /// Removes `guid` from [`Self::spawned_entities`], returning `true` if a spawned entry existed.
    fn take_spawned(&mut self, guid: EntityGuid) -> bool {
        let before = self.spawned_entities.len();
        self.spawned_entities.retain(|r| r.guid != guid);
        self.spawned_entities.len() != before
    }
}

/// Cell-indexed runtime SaveGame payload storing only changes relative to immutable world chunks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldDeltaSnapshot {
    /// Slot metadata and world affinity identifier.
    pub metadata: SaveSlotMetadata,
    /// Delta entries for global persistent entities not partitioned into spatial cells.
    pub global_delta: CellDelta,
    /// Per-cell delta map keyed deterministically by [`SpatialCellCoord`] for targeted chunk loading.
    pub cell_deltas: BTreeMap<SpatialCellCoord, CellDelta>,
    /// Named binary gameplay variables (quests, inventory flags, global counters).
    pub variables: BTreeMap<String, GameplayValue>,
}

#[derive(Serialize, Deserialize)]
struct SerializedCellDelta {
    destroyed_entities: Vec<EntityGuid>,
    modified_stream: Vec<u8>,
    spawned_stream: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SavePayloadHeader {
    metadata: SaveSlotMetadata,
    global_delta: SerializedCellDelta,
    cell_deltas: Vec<(SpatialCellCoord, SerializedCellDelta)>,
    variables: BTreeMap<String, GameplayValue>,
}

impl WorldDeltaSnapshot {
    /// Creates an empty cell-indexed delta snapshot targeting `metadata`.
    #[must_use]
    pub fn new(metadata: SaveSlotMetadata) -> Self {
        Self {
            metadata,
            global_delta: CellDelta::default(),
            cell_deltas: BTreeMap::new(),
            variables: BTreeMap::new(),
        }
    }

    /// Returns the recorded [`CellDelta`] for `coord`, if any modifications exist for that chunk.
    ///
    /// Executes an $O(\log C)$ lookup without inspecting unrelated cells.
    #[must_use]
    pub fn delta_for_cell(&self, coord: SpatialCellCoord) -> Option<&CellDelta> {
        self.cell_deltas.get(&coord)
    }

    /// Returns a mutable [`CellDelta`] entry for `coord`, inserting an empty delta if absent.
    pub fn cell_delta_mut(&mut self, coord: SpatialCellCoord) -> &mut CellDelta {
        self.cell_deltas.entry(coord).or_default()
    }

    /// Records a destroyed chunk-native entity under `coord`.
    pub fn record_cell_destroyed(&mut self, coord: SpatialCellCoord, guid: EntityGuid) {
        self.cell_delta_mut(coord).mark_destroyed(guid);
    }

    /// Records a modified entity under `coord`.
    pub fn record_cell_modified(&mut self, coord: SpatialCellCoord, record: EntityRecord) {
        self.cell_delta_mut(coord).upsert_modified(record);
    }

    /// Records a dynamically spawned entity under `coord`.
    pub fn record_cell_spawned(&mut self, coord: SpatialCellCoord, record: EntityRecord) {
        self.cell_delta_mut(coord).upsert_spawned(record);
    }

    /// Records a cross-cell entity boundary migration from `from_cell` to `to_cell`.
    ///
    /// Ensures asymmetric stream-in order remains deterministic:
    /// - If `record.guid` originated in `from_cell`'s immutable chunk, `from_cell` marks it
    ///   destroyed (preventing duplicate spawn when `from_cell` later streams in) while `to_cell`
    ///   receives the full snapshot in `spawned_entities`.
    /// - If `record.guid` already lived transiently in `from_cell.spawned_entities`, the stale copy
    ///   is removed cleanly and empty deltas are pruned.
    /// - If `to_cell` previously suppressed `record.guid` (returning to its original home chunk),
    ///   its destroyed flag is cleared and `record` moves back to `modified_entities`.
    pub fn relocate_entity(
        &mut self,
        from_cell: SpatialCellCoord,
        to_cell: SpatialCellCoord,
        record: EntityRecord,
    ) {
        if from_cell == to_cell {
            self.record_cell_modified(to_cell, record);
            return;
        }

        let guid = record.guid;
        let removed_transient = if let Some(from_delta) = self.cell_deltas.get_mut(&from_cell) {
            let took = from_delta.take_spawned(guid);
            if took && from_delta.is_empty() {
                self.cell_deltas.remove(&from_cell);
            }
            took
        } else {
            false
        };

        if !removed_transient {
            self.record_cell_destroyed(from_cell, guid);
        }

        let returning_home = self
            .cell_deltas
            .get(&to_cell)
            .is_some_and(|delta| delta.destroyed_entities.contains(&guid));

        if returning_home {
            let dest = self.cell_delta_mut(to_cell);
            dest.destroyed_entities.retain(|g| *g != guid);
            dest.upsert_modified(record);
        } else {
            self.record_cell_spawned(to_cell, record);
        }
    }

    /// Removes cell delta entries whose destroyed, modified, and spawned lists are all empty.
    pub fn prune_empty_cells(&mut self) {
        self.cell_deltas.retain(|_, delta| !delta.is_empty());
    }

    /// Serializes and LZ4-compresses this savegame inside a CRC32-checked [`SAVE_MAGIC`] envelope.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if component stream or `postcard` encoding fails.
    pub fn encode(&self) -> WorldResult<Vec<u8>> {
        let global_delta = encode_cell_delta(&self.global_delta)?;
        let mut cell_deltas = Vec::with_capacity(self.cell_deltas.len());
        for (&coord, delta) in &self.cell_deltas {
            cell_deltas.push((coord, encode_cell_delta(delta)?));
        }

        let payload = SavePayloadHeader {
            metadata: self.metadata.clone(),
            global_delta,
            cell_deltas,
            variables: self.variables.clone(),
        };
        let raw = postcard::to_stdvec(&payload)
            .map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let compressed = lz4_flex::compress_prepend_size(&raw);

        Ok(encode_checksummed_envelope(
            SAVE_MAGIC,
            SAVE_FORMAT_VERSION,
            0,
            &compressed,
        ))
    }

    /// Validates, decompresses, and decodes a `.aesave` binary buffer with forward component filtering.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on bad magic, version mismatch, CRC32 failure, or truncated data.
    pub fn decode(bytes: &[u8], known_hashes: Option<&HashSet<u64>>) -> WorldResult<Self> {
        let envelope = decode_checksummed_envelope(bytes, SAVE_MAGIC, SAVE_FORMAT_VERSION)?;
        let decompressed = lz4_flex::decompress_size_prepended(envelope.payload)
            .map_err(|e| WorldIoError::Compression(e.to_string()))?;
        let body: SavePayloadHeader = postcard::from_bytes(&decompressed)
            .map_err(|e| WorldIoError::Serialization(e.to_string()))?;

        let global_delta = decode_cell_delta(body.global_delta, known_hashes)?;
        let mut cell_deltas = BTreeMap::new();
        for (coord, serialized) in body.cell_deltas {
            cell_deltas.insert(coord, decode_cell_delta(serialized, known_hashes)?);
        }

        Ok(Self {
            metadata: body.metadata,
            global_delta,
            cell_deltas,
            variables: body.variables,
        })
    }
}

fn upsert_record_vec(list: &mut Vec<EntityRecord>, record: EntityRecord) {
    if let Some(existing) = list.iter_mut().find(|r| r.guid == record.guid) {
        *existing = record;
    } else {
        list.push(record);
    }
}

fn encode_cell_delta(delta: &CellDelta) -> WorldResult<SerializedCellDelta> {
    Ok(SerializedCellDelta {
        destroyed_entities: delta.destroyed_entities.clone(),
        modified_stream: EntityRecord::encode_stream(&delta.modified_entities)?,
        spawned_stream: EntityRecord::encode_stream(&delta.spawned_entities)?,
    })
}

fn decode_cell_delta(
    serialized: SerializedCellDelta,
    known_hashes: Option<&HashSet<u64>>,
) -> WorldResult<CellDelta> {
    Ok(CellDelta {
        destroyed_entities: serialized.destroyed_entities,
        modified_entities: EntityRecord::decode_stream(&serialized.modified_stream, known_hashes)?,
        spawned_entities: EntityRecord::decode_stream(&serialized.spawned_stream, known_hashes)?,
    })
}

/// Computes the `.bak` sibling path (`slot_1.aesave` -> `slot_1.aesave.bak`).
#[must_use]
pub fn backup_path_for(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".bak");
    PathBuf::from(os)
}

/// Computes the `.tmp` staging path (`slot_1.aesave` -> `slot_1.aesave.tmp`).
#[must_use]
pub fn temp_path_for(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

/// Writes `snapshot` atomically to `path` via `.tmp` + `sync_all` + `.bak` backup swap.
///
/// # Errors
/// Returns [`WorldIoError`] on serialization or filesystem write/sync/rename failure.
pub fn atomic_write_save(path: &Path, snapshot: &WorldDeltaSnapshot) -> WorldResult<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let encoded = snapshot.encode()?;
    let tmp_path = temp_path_for(path);
    let bak_path = backup_path_for(path);

    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(&encoded)?;
        file.flush()?;
        file.sync_all()?;
    }

    if path.exists() {
        let _ = std::fs::remove_file(&bak_path);
        std::fs::rename(path, &bak_path)?;
    }

    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

/// Loads `path`, automatically falling back to `path.bak` when the primary save file is missing,
/// truncated, or fails CRC32 integrity checks.
///
/// # Errors
/// Returns [`WorldIoError`] when neither the primary save nor a `.bak` backup can be validated.
pub fn load_save_with_fallback(
    path: &Path,
    known_hashes: Option<&HashSet<u64>>,
) -> WorldResult<WorldDeltaSnapshot> {
    let primary_err = match std::fs::read(path) {
        Ok(bytes) => match WorldDeltaSnapshot::decode(&bytes, known_hashes) {
            Ok(snapshot) => return Ok(snapshot),
            Err(err) => err,
        },
        Err(err) => WorldIoError::Io(err),
    };

    let bak_path = backup_path_for(path);
    if bak_path.is_file() {
        let bak_bytes = std::fs::read(&bak_path)?;
        return WorldDeltaSnapshot::decode(&bak_bytes, known_hashes);
    }

    Err(primary_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_record::ComponentRecord;
    use crate::guid::GuidGenerator;
    use ae_plugin_api::Position;

    fn unique_save_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!("ae_world_save_{label}_{nanos}"))
    }

    #[test]
    fn test_save_game_atomic_write_and_corrupt_primary_backup_recovery() {
        let dir = unique_save_dir("atomic_bak");
        let _ = std::fs::remove_dir_all(&dir);
        let save_path = dir.join("slot_1.aesave");

        let mut ids = GuidGenerator::from_seed(55);
        let world_guid = ids.next_entity_guid();
        let door_guid = ids.next_entity_guid();
        let cell_origin = SpatialCellCoord::ZERO;

        let mut first = WorldDeltaSnapshot::new(SaveSlotMetadata {
            slot_index: 1,
            world_guid,
            timestamp_unix: 1_700_000_000,
            play_time_secs: 120,
            label: "Checkpoint Alpha".to_string(),
        });
        first
            .variables
            .insert("score".to_string(), GameplayValue::I64(450));
        atomic_write_save(&save_path, &first).expect("write initial save");

        let mut second = WorldDeltaSnapshot::new(SaveSlotMetadata {
            slot_index: 1,
            world_guid,
            timestamp_unix: 1_700_000_100,
            play_time_secs: 240,
            label: "Checkpoint Beta".to_string(),
        });
        second.record_cell_destroyed(cell_origin, door_guid);
        let mut mod_ent = EntityRecord::new(ids.next_entity_guid(), None);
        mod_ent.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(4.0, 2.0, -8.0))
                .expect("encode position"),
        );
        second.record_cell_modified(cell_origin, mod_ent);
        second
            .variables
            .insert("score".to_string(), GameplayValue::I64(900));
        atomic_write_save(&save_path, &second).expect("overwrite save atomically");

        assert!(backup_path_for(&save_path).is_file());
        let loaded = load_save_with_fallback(&save_path, None).expect("load primary save");
        assert_eq!(loaded, second);
        assert!(loaded.delta_for_cell(cell_origin).is_some());
        assert!(
            loaded
                .delta_for_cell(SpatialCellCoord::new(5, 0, -3))
                .is_none()
        );

        std::fs::write(&save_path, b"corrupted-incomplete-power-loss").expect("corrupt primary");
        let recovered =
            load_save_with_fallback(&save_path, None).expect("recover from .bak fallback");
        assert_eq!(recovered, first);

        std::fs::write(backup_path_for(&save_path), b"corrupt-bak").expect("corrupt bak");
        assert!(load_save_with_fallback(&save_path, None).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_asymmetric_relocate_entity_across_cells_and_return_home() {
        let mut ids = GuidGenerator::from_seed(909);
        let world_guid = ids.next_entity_guid();
        let npc_guid = ids.next_entity_guid();
        let dyn_guid = ids.next_entity_guid();
        let cell_a = SpatialCellCoord::new(0, 0, 0);
        let cell_b = SpatialCellCoord::new(1, 0, 0);
        let cell_c = SpatialCellCoord::new(2, 0, 0);

        let mut snap = WorldDeltaSnapshot::new(SaveSlotMetadata {
            slot_index: 2,
            world_guid,
            timestamp_unix: 1_700_000_200,
            play_time_secs: 600,
            label: "Frontier Crossing".to_string(),
        });

        let mut npc_rec = EntityRecord::new(npc_guid, None);
        npc_rec.components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(66.0, 0.0, 0.0))
                .expect("encode npc pos"),
        );

        // Chunk-native NPC walks from cell_a -> cell_b: suppressed in cell_a, spawned in cell_b
        snap.relocate_entity(cell_a, cell_b, npc_rec.clone());
        assert_eq!(
            snap.delta_for_cell(cell_a)
                .map(|d| d.destroyed_entities.as_slice()),
            Some(&[npc_guid][..])
        );
        assert_eq!(
            snap.delta_for_cell(cell_b)
                .map(|d| d.spawned_entities.len()),
            Some(1)
        );

        // NPC continues cell_b -> cell_c: intermediate cell_b delta prunes completely
        snap.relocate_entity(cell_b, cell_c, npc_rec.clone());
        assert!(snap.delta_for_cell(cell_b).is_none());
        assert_eq!(
            snap.delta_for_cell(cell_a)
                .map(|d| d.destroyed_entities.as_slice()),
            Some(&[npc_guid][..])
        );
        assert_eq!(
            snap.delta_for_cell(cell_c)
                .map(|d| d.spawned_entities.len()),
            Some(1)
        );

        // Roundtrip persistence keeps asymmetric lookup intact
        let bytes = snap.encode().expect("encode relocated save");
        let mut decoded = WorldDeltaSnapshot::decode(&bytes, None).expect("decode save");
        assert_eq!(decoded, snap);

        // Dynamic entity spawned in cell_a moving to cell_b must NOT mark cell_a destroyed
        let dyn_rec = EntityRecord::new(dyn_guid, None);
        decoded.record_cell_spawned(cell_a, dyn_rec.clone());
        decoded.relocate_entity(cell_a, cell_b, dyn_rec);
        assert!(
            !decoded
                .delta_for_cell(cell_a)
                .expect("cell_a remains for npc")
                .destroyed_entities
                .contains(&dyn_guid)
        );

        // Chunk-native NPC walks back home cell_c -> cell_a: cell_c clears, cell_a restores modified
        decoded.relocate_entity(cell_c, cell_a, npc_rec);
        assert!(decoded.delta_for_cell(cell_c).is_none());
        let home_delta = decoded.delta_for_cell(cell_a).expect("home cell modified");
        assert!(!home_delta.destroyed_entities.contains(&npc_guid));
        assert_eq!(home_delta.modified_entities.len(), 1);
        assert_eq!(home_delta.modified_entities[0].guid, npc_guid);
    }
}
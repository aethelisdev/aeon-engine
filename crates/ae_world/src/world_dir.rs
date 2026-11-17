// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Partitioned directory-based world authoring (`Worlds/<WorldName>/world.aeworld` +
//! `cells/x_y_z.aechunk`) with content-hashed dirty-only parallel chunk writes.

use crate::component_record::EntityRecord;
use crate::error::{WorldIoError, WorldResult};
use crate::header::{WORLD_MAGIC, compute_crc32, encode_checksummed_envelope};
use crate::partition::{PartitionCell, SpatialCellCoord};
use crate::world_archive::{
    PartitionIndexEntry, WORLD_FLAG_2D, WORLD_FORMAT_VERSION, WorldDocument, WorldIndex,
};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

/// Canonical manifest filename inside every partitioned world directory.
pub const WORLD_MANIFEST_FILENAME: &str = "world.aeworld";

/// Header flag bit identifying a partitioned directory world whose chunks reside externally
/// inside `cells/x_y_z.aechunk` rather than concatenated inside `world.aeworld`.
pub const WORLD_FLAG_PARTITIONED_DIR: u32 = 1 << 1;

/// Summary metrics and updated chunk checksums produced by [`WorldDocument::save_partitioned_dir`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartitionedSaveStats {
    /// Number of new or modified `cells/x_y_z.aechunk` files written to disk.
    pub written_cells: usize,
    /// Number of unchanged cells skipped without touching disk.
    pub skipped_clean_cells: usize,
    /// Number of obsolete chunk files deleted because those cells are now empty.
    pub removed_cells: usize,
    /// Updated CRC32 map for all resident cells after save, passed to subsequent saves.
    pub cell_crcs: HashMap<SpatialCellCoord, u32>,
}

/// Resolves `(world_dir, manifest_path)` cleanly from either a world folder (`Worlds/MainWorld`),
/// a manifest path (`Worlds/MainWorld/world.aeworld`), or a standalone name (`Worlds/Level2.aeworld`).
#[must_use]
pub fn resolve_partitioned_world_paths(target: &Path) -> (PathBuf, PathBuf) {
    if target.is_dir() {
        return (target.to_path_buf(), target.join(WORLD_MANIFEST_FILENAME));
    }
    if target
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case(WORLD_MANIFEST_FILENAME))
    {
        let dir = target
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        return (dir, target.to_path_buf());
    }
    let stem = target
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("MainWorld");
    let parent = target.parent().unwrap_or_else(|| Path::new(""));
    let world_dir = parent.join(stem);
    let manifest = world_dir.join(WORLD_MANIFEST_FILENAME);
    (world_dir, manifest)
}

impl WorldDocument {
    /// Saves the world as a modular partitioned directory (`world_dir/world.aeworld` +
    /// `world_dir/cells/x_y_z.aechunk`).
    ///
    /// When `previous_cell_crcs` is supplied, each spatial cell is encoded in parallel and its
    /// CRC32 is compared against the previous saved checksum: **only modified or new cells** are
    /// written atomically (`*.aechunk.tmp -> sync_all -> rename`), leaving untouched chunks on disk
    /// completely unmodified. Cells present in `previous_cell_crcs` but absent from `self.cells`
    /// have their stale chunk files removed.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on serialization or filesystem errors.
    pub fn save_partitioned_dir(
        &self,
        world_dir: &Path,
        previous_cell_crcs: Option<&HashMap<SpatialCellCoord, u32>>,
    ) -> WorldResult<PartitionedSaveStats> {
        let cells_dir = world_dir.join("cells");
        std::fs::create_dir_all(&cells_dir)?;

        let cells_vec: Vec<&PartitionCell> = self.cells.values().collect();
        let encoded_cells: Vec<(SpatialCellCoord, u64, u32, Vec<u8>, u32)> = cells_vec
            .par_iter()
            .map(|cell| {
                let bytes = cell.encode_chunk()?;
                let crc = compute_crc32(&bytes);
                let count = u32::try_from(cell.entities.len()).map_err(|_| {
                    WorldIoError::InvalidLayout("Chunk entity count exceeds u32::MAX".to_string())
                })?;
                Ok((cell.coord, cell.revision, count, bytes, crc))
            })
            .collect::<WorldResult<Vec<_>>>()?;

        let mut partitions = Vec::with_capacity(encoded_cells.len());
        let mut to_write: Vec<(SpatialCellCoord, &[u8])> = Vec::new();
        let mut next_crcs = HashMap::with_capacity(encoded_cells.len());
        let mut skipped_clean_cells = 0usize;

        for (coord, revision, entity_count, chunk_bytes, crc) in &encoded_cells {
            partitions.push(PartitionIndexEntry {
                coord: *coord,
                revision: *revision,
                entity_count: *entity_count,
                offset: 0,
                len: chunk_bytes.len() as u64,
                crc32: *crc,
            });
            next_crcs.insert(*coord, *crc);

            let chunk_path = world_dir.join(coord.relative_chunk_path());
            let unchanged_on_disk = previous_cell_crcs
                .and_then(|map| map.get(coord))
                .is_some_and(|prev| *prev == *crc)
                && chunk_path.is_file();

            if unchanged_on_disk {
                skipped_clean_cells += 1;
            } else {
                to_write.push((*coord, chunk_bytes.as_slice()));
            }
        }

        // Parallel atomic writes strictly for dirty/new cells
        to_write
            .par_iter()
            .try_for_each(|(coord, bytes)| -> WorldResult<()> {
                let final_path = world_dir.join(coord.relative_chunk_path());
                let mut tmp_os = final_path.as_os_str().to_os_string();
                tmp_os.push(".tmp");
                let tmp_path = PathBuf::from(tmp_os);
                {
                    let file = File::create(&tmp_path)?;
                    let mut writer = BufWriter::new(file);
                    writer.write_all(bytes)?;
                    writer.flush()?;
                    writer.get_ref().sync_all()?;
                }
                std::fs::rename(&tmp_path, &final_path)?;
                Ok(())
            })?;

        // Remove stale .aechunk files for cells that were previously saved but are now empty
        let mut removed_cells = 0usize;
        if let Some(prev_map) = previous_cell_crcs {
            for &old_coord in prev_map.keys() {
                if !self.cells.contains_key(&old_coord) {
                    let stale_path = world_dir.join(old_coord.relative_chunk_path());
                    if stale_path.is_file() {
                        std::fs::remove_file(&stale_path)?;
                        removed_cells += 1;
                    }
                }
            }
        }

        // Write light world.aeworld manifest (WORLD_FLAG_PARTITIONED_DIR set)
        let header_payload = self.encode_header_body(partitions)?;
        let flags = WORLD_FLAG_PARTITIONED_DIR | if self.is_2d { WORLD_FLAG_2D } else { 0 };
        let envelope =
            encode_checksummed_envelope(WORLD_MAGIC, WORLD_FORMAT_VERSION, flags, &header_payload);

        let manifest_path = world_dir.join(WORLD_MANIFEST_FILENAME);
        let tmp_manifest = world_dir.join("world.aeworld.tmp");
        {
            let file = File::create(&tmp_manifest)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&envelope)?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        std::fs::rename(&tmp_manifest, &manifest_path)?;

        Ok(PartitionedSaveStats {
            written_cells: to_write.len(),
            skipped_clean_cells,
            removed_cells,
            cell_crcs: next_crcs,
        })
    }

    /// Reads ONLY the `world.aeworld` manifest index from a partitioned world directory without
    /// touching `cells/*.aechunk`.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if `world.aeworld` is missing or invalid.
    pub fn read_partitioned_dir_index(
        world_dir: &Path,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<(WorldIndex, bool)> {
        let manifest_path = world_dir.join(WORLD_MANIFEST_FILENAME);
        let file = File::open(&manifest_path)?;
        let mut reader = BufReader::new(file);
        Self::read_index_with_flags(&mut reader, known_hashes)
    }

    /// Loads a partitioned world directory (`world.aeworld` + `cells/*.aechunk`), reading and
    /// verifying all chunk files in parallel via `rayon`, and returns `(WorldDocument, cell_crcs)`.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if the manifest or any referenced `.aechunk` file fails validation.
    pub fn load_partitioned_dir(
        world_dir: &Path,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<(Self, HashMap<SpatialCellCoord, u32>)> {
        let (index, _) = Self::read_partitioned_dir_index(world_dir, known_hashes)?;

        let loaded_cells: Vec<(PartitionCell, u32)> = index
            .partitions
            .par_iter()
            .map(|entry| {
                let chunk_path = world_dir.join(entry.coord.relative_chunk_path());
                let bytes = std::fs::read(&chunk_path).map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        WorldIoError::EntryNotFound(chunk_path.to_string_lossy().into_owned())
                    } else {
                        WorldIoError::Io(e)
                    }
                })?;
                let actual_crc = compute_crc32(&bytes);
                if actual_crc != entry.crc32 {
                    return Err(WorldIoError::ChecksumMismatch {
                        expected: entry.crc32,
                        actual: actual_crc,
                    });
                }
                let cell = PartitionCell::decode_chunk(&bytes, known_hashes)?;
                Ok((cell, actual_crc))
            })
            .collect::<WorldResult<Vec<_>>>()?;

        let mut cells = BTreeMap::new();
        let mut cell_crcs = HashMap::with_capacity(loaded_cells.len());
        for (cell, crc) in loaded_cells {
            cell_crcs.insert(cell.coord, crc);
            cells.insert(cell.coord, cell);
        }

        let doc = Self::from_index_and_cells(index, cells)?;
        Ok((doc, cell_crcs))
    }

    /// Loads either a partitioned world directory (`Worlds/<Name>` or `world.aeworld` with
    /// `WORLD_FLAG_PARTITIONED_DIR`) or a standalone monolithic `.aeworld` archive transparently.
    ///
    /// Returns `(WorldDocument, cell_crcs, resolved_is_partitioned_dir)`.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on validation, I/O, or decompression errors.
    pub fn load_auto(
        path: &Path,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<(Self, HashMap<SpatialCellCoord, u32>, bool)> {
        if path.is_dir() {
            let (doc, crcs) = Self::load_partitioned_dir(path, known_hashes)?;
            return Ok((doc, crcs, true));
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let (_, is_partitioned_dir) = Self::read_index_with_flags(&mut reader, known_hashes)?;
        if is_partitioned_dir {
            let world_dir = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            let (doc, crcs) = Self::load_partitioned_dir(world_dir, known_hashes)?;
            Ok((doc, crcs, true))
        } else {
            let doc = Self::load_from_file(path, known_hashes)?;
            Ok((doc, HashMap::new(), false))
        }
    }

    /// Helper computing serialized entity count across both persistent globals and cells.
    #[must_use]
    pub fn total_entity_count(&self) -> usize {
        self.persistent_entities.len()
            + self.cells.values().map(|c| c.entities.len()).sum::<usize>()
    }

    /// Collects all [`EntityRecord`]s across persistent globals and cells into one vector.
    #[must_use]
    pub fn into_all_entities(self) -> Vec<EntityRecord> {
        let total = self.total_entity_count();
        let mut out = Vec::with_capacity(total);
        out.extend(self.persistent_entities);
        for cell in self.cells.into_values() {
            out.extend(cell.entities);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_record::ComponentRecord;
    use crate::guid::GuidGenerator;
    use ae_plugin_api::{Name, Position};

    fn unique_world_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!("ae_world_dir_{label}_{nanos}"))
    }

    #[test]
    fn test_partitioned_world_dir_dirty_only_saves_leave_clean_chunks_untouched() {
        let world_dir = unique_world_dir("dirty_only");
        let _ = std::fs::remove_dir_all(&world_dir);

        let mut ids = GuidGenerator::from_seed(909);
        let mut world = WorldDocument::new(ids.next_entity_guid(), false, 100.0);

        let mut ent_a = EntityRecord::new(ids.next_entity_guid(), None);
        ent_a.components.push(
            ComponentRecord::from_value("Name", 1, &Name("RockCell0".to_string()))
                .expect("encode name A"),
        );
        world.insert_entity_at([10.0, 0.0, 10.0], ent_a);

        let mut ent_b = EntityRecord::new(ids.next_entity_guid(), None);
        ent_b.components.push(
            ComponentRecord::from_value("Name", 1, &Name("RockCell2".to_string()))
                .expect("encode name B"),
        );
        world.insert_entity_at([250.0, 0.0, 10.0], ent_b);

        // Initial full save writes both chunks
        let stats1 = world
            .save_partitioned_dir(&world_dir, None)
            .expect("initial partitioned save");
        assert_eq!(stats1.written_cells, 2);
        assert_eq!(stats1.skipped_clean_cells, 0);

        let cell0_path = world_dir.join(SpatialCellCoord::new(0, 0, 0).relative_chunk_path());
        let cell2_path = world_dir.join(SpatialCellCoord::new(2, 0, 0).relative_chunk_path());
        assert!(world_dir.join("world.aeworld").is_file());
        assert!(cell0_path.is_file());
        assert!(cell2_path.is_file());

        let mtime_cell0_before = std::fs::metadata(&cell0_path)
            .expect("meta 0")
            .modified()
            .expect("mtime 0");

        // Tiny delay guarantees filesystem mtime granularity sees any write to cell2
        std::thread::sleep(std::time::Duration::from_millis(20));

        // Mutate ONLY cell 2 (250, 0, 10)
        let cell2 = world
            .cells
            .get_mut(&SpatialCellCoord::new(2, 0, 0))
            .expect("cell 2 exists");
        cell2.entities[0].components.push(
            ComponentRecord::from_value("Position", 1, &Position::new(255.0, 3.0, 10.0))
                .expect("encode position"),
        );

        let stats2 = world
            .save_partitioned_dir(&world_dir, Some(&stats1.cell_crcs))
            .expect("dirty-only save");
        assert_eq!(stats2.written_cells, 1);
        assert_eq!(stats2.skipped_clean_cells, 1);

        let mtime_cell0_after = std::fs::metadata(&cell0_path)
            .expect("meta 0 after")
            .modified()
            .expect("mtime 0 after");
        assert_eq!(
            mtime_cell0_before, mtime_cell0_after,
            "Clean chunk file must NOT be rewritten on disk"
        );

        // Load back via load_auto on world.aeworld manifest
        let (reloaded, reloaded_crcs, is_dir) =
            WorldDocument::load_auto(&world_dir.join("world.aeworld"), None)
                .expect("load_auto partitioned world");
        assert!(is_dir);
        assert_eq!(reloaded_crcs.len(), 2);
        assert_eq!(reloaded.total_entity_count(), 2);

        // Delete cell 0 and re-save: stale chunk file must be removed
        world.cells.remove(&SpatialCellCoord::new(0, 0, 0));
        let stats3 = world
            .save_partitioned_dir(&world_dir, Some(&stats2.cell_crcs))
            .expect("delete cell save");
        assert_eq!(stats3.removed_cells, 1);
        assert!(!cell0_path.exists());

        let _ = std::fs::remove_dir_all(&world_dir);
    }
}
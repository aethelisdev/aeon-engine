// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Binary world archive (`.aeworld`) with string/asset tables, persistent globals,
//! parallel chunk compression, and seek-based random-access partition streaming.
//!
//! Layout on disk/stream:
//! `[WORLD_MAGIC Envelope (28B header + CRC32'd TOC payload)][Concatenated .aechunk blocks...]`

use crate::component_record::EntityRecord;
use crate::error::{WorldIoError, WorldResult};
use crate::guid::{AssetGuid, EntityGuid};
use crate::header::{
    ByteReader, ENVELOPE_HEADER_BYTES, WORLD_MAGIC, compute_crc32, decode_checksummed_envelope,
    encode_checksummed_envelope, verify_crc32,
};
use crate::partition::{PartitionCell, SpatialCellCoord};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Current `.aeworld` binary format version.
pub const WORLD_FORMAT_VERSION: u32 = 1;

/// Flag bit indicating a 2D coordinate world (`0` denotes 3D).
pub const WORLD_FLAG_2D: u32 = 1 << 0;

/// Semantic classification for referenced assets stored in [`AssetRefEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetKind {
    /// 3D mesh/model asset.
    Model,
    /// 2D image/texture asset.
    Texture,
    /// Sound effect or music stream.
    Audio,
    /// Reusable entity hierarchy prefab.
    Prefab,
    /// Material parameter set.
    Material,
    /// Shader program source or bytecode.
    Shader,
    /// Fallback or extension asset category.
    Unknown,
}

/// Deduplicated asset reference mapping an [`AssetGuid`] to an interned path index in [`WorldDocument::strings`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRefEntry {
    /// Persistent 128-bit asset identifier.
    pub guid: AssetGuid,
    /// Semantic asset kind.
    pub kind: AssetKind,
    /// Index into the world's `strings` table holding the normalized virtual asset path.
    pub path_string_idx: u32,
}

/// Directory entry locating a single compressed `.aechunk` partition inside a `.aeworld` stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionIndexEntry {
    /// Spatial cell coordinates covered by this chunk.
    pub coord: SpatialCellCoord,
    /// Chunk edit revision.
    pub revision: u64,
    /// Number of entities encoded inside this chunk.
    pub entity_count: u32,
    /// Byte offset relative to the start of the `.aeworld` chunk data region.
    pub offset: u64,
    /// Byte length of the `.aechunk` envelope on disk.
    pub len: u64,
    /// IEEE CRC32 checksum of the `.aechunk` envelope bytes (`offset..offset + len`).
    pub crc32: u32,
}

/// Validated metadata and partition index read from the header of a `.aeworld` stream without
/// decompressing any spatial cells.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldIndex {
    /// Persistent unique identifier of the world.
    pub world_guid: EntityGuid,
    /// Whether this world targets the 2D (`true`) or 3D (`false`) domain.
    pub is_2d: bool,
    /// World partition cubic grid size in world units.
    pub cell_size: f32,
    /// Deduplicated string pool (entity names, asset virtual paths, tags).
    pub strings: Vec<String>,
    /// Asset GUID references used by this world.
    pub asset_refs: Vec<AssetRefEntry>,
    /// Always-loaded global/persistent entities outside spatial chunk streaming.
    pub persistent_entities: Vec<EntityRecord>,
    /// Partition table mapping cell coordinates to byte offsets inside the chunk region.
    pub partitions: Vec<PartitionIndexEntry>,
    /// Absolute byte offset within the `.aeworld` file/buffer where the chunk data region begins.
    pub chunk_region_offset: u64,
}

#[derive(Serialize, Deserialize)]
struct WorldHeaderBody {
    world_guid: EntityGuid,
    is_2d: bool,
    cell_size: f32,
    strings: Vec<String>,
    asset_refs: Vec<AssetRefEntry>,
    persistent_entities_stream: Vec<u8>,
    partitions: Vec<PartitionIndexEntry>,
}

/// Complete authoring and runtime representation of a binary partitioned world (`.aeworld`).
#[derive(Debug, Clone, PartialEq)]
pub struct WorldDocument {
    /// Persistent unique identifier of the world.
    pub world_guid: EntityGuid,
    /// Whether this world uses 2D (`true`) or 3D (`false`) simulation systems.
    pub is_2d: bool,
    /// Cubic grid cell side length in world units.
    pub cell_size: f32,
    /// Interned string table referenced by index across world tables.
    pub strings: Vec<String>,
    /// Asset reference table mapping [`AssetGuid`]s to interned paths.
    pub asset_refs: Vec<AssetRefEntry>,
    /// Entities always kept resident regardless of camera streaming distance.
    pub persistent_entities: Vec<EntityRecord>,
    /// Spatial partition cells keyed deterministically by [`SpatialCellCoord`].
    pub cells: BTreeMap<SpatialCellCoord, PartitionCell>,
    string_lookup: HashMap<String, u32>,
}

impl WorldDocument {
    /// Creates an empty [`WorldDocument`] with the specified world GUID, dimensionality, and cell size.
    #[must_use]
    pub fn new(world_guid: EntityGuid, is_2d: bool, cell_size: f32) -> Self {
        let valid_cell_size = if cell_size.is_finite() && cell_size > 0.0 {
            cell_size
        } else {
            128.0
        };
        Self {
            world_guid,
            is_2d,
            cell_size: valid_cell_size,
            strings: Vec::new(),
            asset_refs: Vec::new(),
            persistent_entities: Vec::new(),
            cells: BTreeMap::new(),
            string_lookup: HashMap::new(),
        }
    }

    /// Interns `value` in the world's string table, returning its compact `u32` index.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if the string table exceeds `u32::MAX` entries.
    pub fn intern_string(&mut self, value: &str) -> WorldResult<u32> {
        if let Some(&existing) = self.string_lookup.get(value) {
            return Ok(existing);
        }
        let idx = u32::try_from(self.strings.len()).map_err(|_| {
            WorldIoError::InvalidLayout("World string table exceeded u32::MAX entries".to_string())
        })?;
        let owned = value.to_string();
        self.strings.push(owned.clone());
        self.string_lookup.insert(owned, idx);
        Ok(idx)
    }

    /// Inserts or updates a referenced asset entry, deduplicating identical [`AssetGuid`]s.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if string table capacity is exceeded.
    pub fn register_asset_ref(
        &mut self,
        guid: AssetGuid,
        kind: AssetKind,
        virtual_path: &str,
    ) -> WorldResult<()> {
        let str_idx = self.intern_string(virtual_path)?;
        if let Some(existing) = self.asset_refs.iter_mut().find(|entry| entry.guid == guid) {
            existing.kind = kind;
            existing.path_string_idx = str_idx;
            return Ok(());
        }
        self.asset_refs.push(AssetRefEntry {
            guid,
            kind,
            path_string_idx: str_idx,
        });
        Ok(())
    }

    /// Inserts an [`EntityRecord`] directly into the spatial cell corresponding to `world_pos`.
    pub fn insert_entity_at(&mut self, world_pos: [f32; 3], entity: EntityRecord) {
        let coord = SpatialCellCoord::from_world_pos(world_pos, self.cell_size);
        let cell = self
            .cells
            .entry(coord)
            .or_insert_with(|| PartitionCell::new(coord));
        cell.entities.push(entity);
    }

    /// Encodes the entire world into a `.aeworld` binary buffer, compressing all spatial chunks
    /// in parallel across `rayon` worker threads.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if entity streams or `postcard` encoding fails.
    pub fn encode_full(&self) -> WorldResult<Vec<u8>> {
        let cells_vec: Vec<&PartitionCell> = self.cells.values().collect();
        let encoded_chunks: Vec<(SpatialCellCoord, u64, u32, Vec<u8>)> = cells_vec
            .par_iter()
            .map(|cell| {
                let bytes = cell.encode_chunk()?;
                let count = u32::try_from(cell.entities.len()).map_err(|_| {
                    WorldIoError::InvalidLayout("Chunk entity count exceeds u32::MAX".to_string())
                })?;
                Ok((cell.coord, cell.revision, count, bytes))
            })
            .collect::<WorldResult<Vec<_>>>()?;

        let mut partitions = Vec::with_capacity(encoded_chunks.len());
        let mut chunk_region_bytes = 0usize;
        let mut cursor = 0u64;
        for (coord, revision, entity_count, chunk_bytes) in &encoded_chunks {
            let len = chunk_bytes.len() as u64;
            let crc32 = compute_crc32(chunk_bytes);
            partitions.push(PartitionIndexEntry {
                coord: *coord,
                revision: *revision,
                entity_count: *entity_count,
                offset: cursor,
                len,
                crc32,
            });
            cursor = cursor.saturating_add(len);
            chunk_region_bytes = chunk_region_bytes.saturating_add(chunk_bytes.len());
        }

        let header_payload = self.encode_header_body(partitions)?;

        let flags = if self.is_2d { WORLD_FLAG_2D } else { 0 };
        let envelope =
            encode_checksummed_envelope(WORLD_MAGIC, WORLD_FORMAT_VERSION, flags, &header_payload);

        let mut out = Vec::with_capacity(envelope.len() + chunk_region_bytes);
        out.extend_from_slice(&envelope);
        for (_, _, _, chunk_bytes) in encoded_chunks {
            out.extend_from_slice(&chunk_bytes);
        }
        Ok(out)
    }

    /// Serializes the world metadata header body (`WorldHeaderBody`) for either monolithic
    /// archives or partitioned directory manifests.
    pub(crate) fn encode_header_body(
        &self,
        partitions: Vec<PartitionIndexEntry>,
    ) -> WorldResult<Vec<u8>> {
        let persistent_stream = EntityRecord::encode_stream(&self.persistent_entities)?;
        let header_body = WorldHeaderBody {
            world_guid: self.world_guid,
            is_2d: self.is_2d,
            cell_size: self.cell_size,
            strings: self.strings.clone(),
            asset_refs: self.asset_refs.clone(),
            persistent_entities_stream: persistent_stream,
            partitions,
        };
        postcard::to_stdvec(&header_body).map_err(|e| WorldIoError::Serialization(e.to_string()))
    }

    /// Reads and validates ONLY the `.aeworld` header, string/asset tables, persistent globals,
    /// and partition index from any seekable stream without decompressing spatial chunks.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on malformed headers, unsupported versions, or CRC32 mismatches.
    pub fn read_index<R: Read + Seek>(
        reader: &mut R,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<WorldIndex> {
        let (index, _) = Self::read_index_with_flags(reader, known_hashes)?;
        Ok(index)
    }

    /// Reads the world header index along with whether the partitioned-directory flag is active.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if the envelope header or body fails validation.
    pub fn read_index_with_flags<R: Read + Seek>(
        reader: &mut R,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<(WorldIndex, bool)> {
        let mut fixed = [0u8; ENVELOPE_HEADER_BYTES];
        reader.seek(SeekFrom::Start(0))?;
        reader.read_exact(&mut fixed).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                WorldIoError::TruncatedData {
                    context: "world envelope header",
                    expected: ENVELOPE_HEADER_BYTES,
                    actual: 0,
                }
            } else {
                WorldIoError::Io(e)
            }
        })?;

        let mut hdr_reader = ByteReader::new(&fixed);
        let magic = hdr_reader.read_fixed_8("world magic")?;
        if magic != WORLD_MAGIC {
            return Err(WorldIoError::InvalidMagic {
                expected: WORLD_MAGIC,
                found: magic,
            });
        }
        let format_version = hdr_reader.read_u32("world format_version")?;
        if format_version == 0 || format_version > WORLD_FORMAT_VERSION {
            return Err(WorldIoError::UnsupportedVersion {
                found: format_version,
                max_supported: WORLD_FORMAT_VERSION,
            });
        }
        let _flags = hdr_reader.read_u32("world flags")?;
        let payload_len = hdr_reader.read_u64("world payload_len")?;
        let _payload_crc = hdr_reader.read_u32("world payload_crc32")?;

        let payload_usize = usize::try_from(payload_len).map_err(|_| {
            WorldIoError::InvalidLayout(format!(
                "World header payload {payload_len} exceeds addressable RAM"
            ))
        })?;

        let mut full_envelope = Vec::with_capacity(ENVELOPE_HEADER_BYTES + payload_usize);
        full_envelope.extend_from_slice(&fixed);
        full_envelope.resize(ENVELOPE_HEADER_BYTES + payload_usize, 0);
        reader
            .read_exact(&mut full_envelope[ENVELOPE_HEADER_BYTES..])
            .map_err(|_| WorldIoError::TruncatedData {
                context: "world header payload",
                expected: payload_usize,
                actual: 0,
            })?;

        let decoded =
            decode_checksummed_envelope(&full_envelope, WORLD_MAGIC, WORLD_FORMAT_VERSION)?;
        let is_partitioned_dir =
            (decoded.header.flags & crate::world_dir::WORLD_FLAG_PARTITIONED_DIR) != 0;
        let body: WorldHeaderBody = postcard::from_bytes(decoded.payload)
            .map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let persistent_entities =
            EntityRecord::decode_stream(&body.persistent_entities_stream, known_hashes)?;

        let index = WorldIndex {
            world_guid: body.world_guid,
            is_2d: body.is_2d,
            cell_size: body.cell_size,
            strings: body.strings,
            asset_refs: body.asset_refs,
            persistent_entities,
            partitions: body.partitions,
            chunk_region_offset: full_envelope.len() as u64,
        };
        Ok((index, is_partitioned_dir))
    }

    /// Reads, verifies CRC32, and decodes a single spatial partition cell via direct seek I/O.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on I/O errors, CRC32 mismatch, or malformed chunk payloads.
    pub fn read_partition_cell<R: Read + Seek>(
        reader: &mut R,
        chunk_region_offset: u64,
        entry: &PartitionIndexEntry,
        known_hashes: Option<&HashSet<u64>>,
    ) -> WorldResult<PartitionCell> {
        let abs_offset = chunk_region_offset
            .checked_add(entry.offset)
            .ok_or_else(|| WorldIoError::InvalidLayout("Chunk offset overflow".to_string()))?;
        let chunk_usize = usize::try_from(entry.len).map_err(|_| {
            WorldIoError::InvalidLayout(format!("Chunk length {} exceeds RAM", entry.len))
        })?;

        reader.seek(SeekFrom::Start(abs_offset))?;
        let mut buf = vec![0u8; chunk_usize];
        reader
            .read_exact(&mut buf)
            .map_err(|_| WorldIoError::TruncatedData {
                context: "partition chunk payload",
                expected: chunk_usize,
                actual: 0,
            })?;
        verify_crc32(&buf, entry.crc32)?;
        PartitionCell::decode_chunk(&buf, known_hashes)
    }

    /// Decodes a complete `.aeworld` byte buffer including all spatial chunks in parallel.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if headers or any partition chunk fail validation.
    pub fn decode_full(bytes: &[u8], known_hashes: Option<&HashSet<u64>>) -> WorldResult<Self> {
        let mut cursor = Cursor::new(bytes);
        let index = Self::read_index(&mut cursor, known_hashes)?;

        let chunk_slices: Vec<&[u8]> = index
            .partitions
            .iter()
            .map(|entry| {
                let start = index
                    .chunk_region_offset
                    .checked_add(entry.offset)
                    .and_then(|v| usize::try_from(v).ok())
                    .ok_or_else(|| {
                        WorldIoError::InvalidLayout("Partition chunk offset overflow".to_string())
                    })?;
                let len = usize::try_from(entry.len).map_err(|_| {
                    WorldIoError::InvalidLayout("Partition chunk length overflow".to_string())
                })?;
                let end = start.checked_add(len).ok_or_else(|| {
                    WorldIoError::InvalidLayout("Partition chunk end overflow".to_string())
                })?;
                if end > bytes.len() {
                    return Err(WorldIoError::TruncatedData {
                        context: "world partition chunk",
                        expected: end,
                        actual: bytes.len(),
                    });
                }
                let slice = &bytes[start..end];
                verify_crc32(slice, entry.crc32)?;
                Ok(slice)
            })
            .collect::<WorldResult<Vec<_>>>()?;

        let decoded_cells = PartitionCell::decode_chunks_parallel(&chunk_slices, known_hashes)?;
        let mut cells = BTreeMap::new();
        for cell in decoded_cells {
            cells.insert(cell.coord, cell);
        }

        Self::from_index_and_cells(index, cells)
    }

    /// Reconstructs a [`WorldDocument`] and its interned string table from a [`WorldIndex`] and cells.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if the string pool exceeds `u32::MAX`.
    pub fn from_index_and_cells(
        index: WorldIndex,
        cells: BTreeMap<SpatialCellCoord, PartitionCell>,
    ) -> WorldResult<Self> {
        let mut string_lookup = HashMap::with_capacity(index.strings.len());
        for (idx, s) in index.strings.iter().enumerate() {
            let idx_u32 = u32::try_from(idx)
                .map_err(|_| WorldIoError::InvalidLayout("String index overflow".to_string()))?;
            string_lookup.insert(s.clone(), idx_u32);
        }

        Ok(Self {
            world_guid: index.world_guid,
            is_2d: index.is_2d,
            cell_size: index.cell_size,
            strings: index.strings,
            asset_refs: index.asset_refs,
            persistent_entities: index.persistent_entities,
            cells,
            string_lookup,
        })
    }

    /// Writes this world archive to `path` via an atomic temporary file swap.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on serialization or filesystem failure.
    pub fn save_to_file(&self, path: &Path) -> WorldResult<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let encoded = self.encode_full()?;
        let tmp_path = path.with_extension("aeworld.tmp");
        {
            let file = File::create(&tmp_path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(&encoded)?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Loads a full `.aeworld` binary archive from disk.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if reading or binary validation fails.
    pub fn load_from_file(path: &Path, known_hashes: Option<&HashSet<u64>>) -> WorldResult<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Self::decode_full(&bytes, known_hashes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_record::ComponentRecord;
    use crate::guid::{GuidGenerator, asset_guid_from_virtual_path};
    use ae_plugin_api::{Name, Position};

    #[test]
    fn test_world_archive_10k_entities_parallel_roundtrip_and_seek_streaming() {
        let mut ids = GuidGenerator::from_seed(1337);
        let mut world = WorldDocument::new(ids.next_entity_guid(), false, 100.0);
        let model_guid = asset_guid_from_virtual_path("models/rock.glb");
        world
            .register_asset_ref(model_guid, AssetKind::Model, "models/rock.glb")
            .expect("register asset ref");
        // Re-register same GUID updates cleanly without duplicating
        world
            .register_asset_ref(model_guid, AssetKind::Model, "models/rock.glb")
            .expect("deduplicate asset ref");
        assert_eq!(world.asset_refs.len(), 1);

        let mut sun = EntityRecord::new(ids.next_entity_guid(), None);
        sun.components.push(
            ComponentRecord::from_value("Name", 1, &Name("SunLight".to_string()))
                .expect("encode sun name"),
        );
        world.persistent_entities.push(sun);

        for i in 0..10_000 {
            let x = ((i % 20) as f32 - 10.0) * 60.0;
            let z = (((i / 20) % 20) as f32 - 10.0) * 60.0;
            let mut rec = EntityRecord::new(ids.next_entity_guid(), None);
            rec.components.push(
                ComponentRecord::from_value("Position", 1, &Position::new(x, 0.0, z))
                    .expect("encode position"),
            );
            world.insert_entity_at([x, 0.0, z], rec);
        }

        let bytes = world.encode_full().expect("encode 10k world");
        let restored = WorldDocument::decode_full(&bytes, None).expect("decode 10k world");
        assert_eq!(restored.world_guid, world.world_guid);
        assert_eq!(restored.persistent_entities.len(), 1);
        let total_partition_entities: usize =
            restored.cells.values().map(|c| c.entities.len()).sum();
        assert_eq!(total_partition_entities, 10_000);

        // Verify seek-based single cell streaming without loading remaining cells
        let mut cursor = Cursor::new(&bytes);
        let index = WorldDocument::read_index(&mut cursor, None).expect("read world index");
        let target_entry = index.partitions.first().expect("at least one partition");
        let single_cell = WorldDocument::read_partition_cell(
            &mut cursor,
            index.chunk_region_offset,
            target_entry,
            None,
        )
        .expect("stream single partition cell");
        assert_eq!(single_cell.coord, target_entry.coord);
        assert_eq!(
            single_cell.entities.len(),
            target_entry.entity_count as usize
        );
    }

    #[test]
    fn test_world_archive_detects_truncated_and_corrupted_chunks() {
        let mut ids = GuidGenerator::from_seed(7);
        let mut world = WorldDocument::new(ids.next_entity_guid(), true, 64.0);
        world.insert_entity_at(
            [12.0, 4.0, 0.0],
            EntityRecord::new(ids.next_entity_guid(), None),
        );

        let bytes = world.encode_full().expect("encode world");
        let truncated = &bytes[..bytes.len() - 4];
        assert!(matches!(
            WorldDocument::decode_full(truncated, None),
            Err(WorldIoError::TruncatedData { .. })
        ));

        let mut corrupted = bytes.clone();
        if let Some(last) = corrupted.last_mut() {
            *last ^= 0xaa;
        }
        assert!(matches!(
            WorldDocument::decode_full(&corrupted, None),
            Err(WorldIoError::ChecksumMismatch { .. })
        ));
    }
}
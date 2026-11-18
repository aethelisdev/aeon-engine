// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Block-compressed indexed asset container (`.aepak`) supporting parallel packing and
//! random-access byte-range decompression over `64 KiB` LZ4 blocks.
//!
//! Layout:
//! `[PAK_MAGIC Header (32B)] [Compressed LZ4 Data Blocks ...] [Postcard TOC] [TOC CRC32 (4B)]`

use crate::aligned_buffer::AlignedBuffer;
use crate::error::{WorldIoError, WorldResult};
use crate::guid::{AssetGuid, asset_guid_from_virtual_path};
use crate::header::{ByteReader, PAK_MAGIC, compute_crc32, verify_crc32};
use crate::world_archive::AssetKind;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Current `.aepak` container format version (`2` enforces 16-byte aligned block offsets).
pub const PAK_FORMAT_VERSION: u32 = 2;

/// Fixed 32-byte header written at offset `0` of every `.aepak` archive.
pub const PAK_HEADER_BYTES: usize = 32;

/// Required byte alignment (`16` bytes) for data block offsets inside `.aepak` archives.
pub const PAK_BLOCK_ALIGNMENT: u64 = 16;

/// Default independent LZ4 block size (`64 KiB`) enabling sub-asset random reads.
pub const DEFAULT_PAK_BLOCK_SIZE: usize = 64 * 1024;

/// Compression algorithm applied to individual asset blocks inside a `.aepak` archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PakCodec {
    /// Stored uncompressed (e.g., already compressed PCM/streaming formats).
    Raw,
    /// Compressed independently per fixed-size block using `lz4_flex`.
    Lz4Block,
}

/// Physical location and compressed sizing metadata for one `<= block_size` slice of an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PakBlockMeta {
    /// Absolute byte offset from the start of the `.aepak` archive.
    pub offset: u64,
    /// Stored byte length in the `.aepak` archive.
    pub stored_len: u32,
    /// Uncompressed byte length of this block.
    pub raw_len: u32,
}

/// Table of Contents entry describing a packed asset inside `.aepak`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PakTocEntry {
    /// Persistent 128-bit asset GUID.
    pub guid: AssetGuid,
    /// Normalized forward-slash virtual path (e.g., `"textures/stone.png"`).
    pub virtual_path: String,
    /// Semantic asset kind.
    pub kind: AssetKind,
    /// Compression codec applied to blocks in this entry.
    pub codec: PakCodec,
    /// Total uncompressed byte size of the asset.
    pub raw_size: u64,
    /// IEEE CRC32 checksum computed over the complete uncompressed asset bytes.
    pub raw_crc32: u32,
    /// Ordered list of fixed-capacity blocks comprising this asset.
    pub blocks: Vec<PakBlockMeta>,
}

/// Parsed `.aepak` Table of Contents with $O(1)$ lookup indices by [`AssetGuid`] and virtual path.
#[derive(Debug, Clone)]
pub struct PakIndex {
    /// Format version decoded from the `.aepak` header.
    pub format_version: u32,
    /// Block capacity used to slice entries during packing.
    pub block_size: u32,
    /// Ordered TOC entries.
    pub entries: Vec<PakTocEntry>,
    guid_map: HashMap<AssetGuid, usize>,
    path_map: HashMap<String, usize>,
}

impl PakIndex {
    fn from_entries(format_version: u32, block_size: u32, entries: Vec<PakTocEntry>) -> Self {
        let mut guid_map = HashMap::with_capacity(entries.len());
        let mut path_map = HashMap::with_capacity(entries.len());
        for (idx, entry) in entries.iter().enumerate() {
            guid_map.insert(entry.guid, idx);
            path_map.insert(entry.virtual_path.clone(), idx);
        }
        Self {
            format_version,
            block_size,
            entries,
            guid_map,
            path_map,
        }
    }

    /// Resolves a TOC entry by [`AssetGuid`].
    #[must_use]
    pub fn get_by_guid(&self, guid: AssetGuid) -> Option<&PakTocEntry> {
        self.guid_map.get(&guid).map(|&idx| &self.entries[idx])
    }

    /// Resolves a TOC entry by normalized virtual path.
    #[must_use]
    pub fn get_by_path(&self, virtual_path: &str) -> Option<&PakTocEntry> {
        let norm = normalize_pak_path(virtual_path);
        self.path_map.get(&norm).map(|&idx| &self.entries[idx])
    }

    pub(crate) fn require_guid(&self, guid: AssetGuid) -> WorldResult<&PakTocEntry> {
        self.get_by_guid(guid)
            .ok_or_else(|| WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))
    }

    pub(crate) fn require_path(&self, virtual_path: &str) -> WorldResult<&PakTocEntry> {
        self.get_by_path(virtual_path)
            .ok_or_else(|| WorldIoError::EntryNotFound(virtual_path.to_string()))
    }
}

struct PendingPakInput {
    guid: AssetGuid,
    virtual_path: String,
    kind: AssetKind,
    codec: PakCodec,
    raw_bytes: Vec<u8>,
}

/// Parallel `.aepak` builder that slices assets into blocks, compresses via `rayon` + `lz4_flex`,
/// and writes a checksum-protected Table of Contents.
pub struct PakBuilder {
    block_size: usize,
    items: Vec<PendingPakInput>,
}

impl PakBuilder {
    /// Creates a new builder using the default `64 KiB` block size.
    #[must_use]
    pub fn new() -> Self {
        Self::with_block_size(DEFAULT_PAK_BLOCK_SIZE)
    }

    /// Creates a builder with an explicit positive block size in bytes.
    #[must_use]
    pub fn with_block_size(block_size: usize) -> Self {
        Self {
            block_size: block_size.max(512),
            items: Vec::new(),
        }
    }

    /// Stages an asset for packing using an automatically derived deterministic [`AssetGuid`]
    /// from its normalized `virtual_path`.
    pub fn add_asset(
        &mut self,
        virtual_path: &str,
        kind: AssetKind,
        codec: PakCodec,
        raw_bytes: Vec<u8>,
    ) -> AssetGuid {
        let normalized = normalize_pak_path(virtual_path);
        let guid = asset_guid_from_virtual_path(&normalized);
        self.add_asset_with_guid(guid, &normalized, kind, codec, raw_bytes);
        guid
    }

    /// Stages an asset for packing with an explicit [`AssetGuid`].
    pub fn add_asset_with_guid(
        &mut self,
        guid: AssetGuid,
        virtual_path: &str,
        kind: AssetKind,
        codec: PakCodec,
        raw_bytes: Vec<u8>,
    ) {
        self.items.push(PendingPakInput {
            guid,
            virtual_path: normalize_pak_path(virtual_path),
            kind,
            codec,
            raw_bytes,
        });
    }

    /// Compresses all staged asset blocks in parallel and returns the complete `.aepak` archive bytes.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if block counts or metadata serialization exceed format bounds.
    pub fn build_to_vec(&self) -> WorldResult<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::new());
        self.write_to(&mut cursor)?;
        Ok(cursor.into_inner())
    }

    /// Compresses all staged asset blocks in parallel and writes the `.aepak` container to `path`.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on I/O or encoding failures.
    pub fn write_to_file(&self, path: &Path) -> WorldResult<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let tmp_path = path.with_extension("aepak.tmp");
        {
            let file = File::create(&tmp_path)?;
            let mut writer = BufWriter::new(file);
            self.write_to(&mut writer)?;
            writer.flush()?;
            writer.get_ref().sync_all()?;
        }
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Compresses all staged asset blocks in parallel and writes the `.aepak` archive into `writer`.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on stream or serialization failure.
    pub fn write_to<W: Write + Seek>(&self, writer: &mut W) -> WorldResult<()> {
        let block_size = self.block_size;
        let block_size_u32 = u32::try_from(block_size).map_err(|_| {
            WorldIoError::InvalidLayout("Pak block size exceeds u32::MAX".to_string())
        })?;

        let prepared: Vec<(PakTocEntry, Vec<Vec<u8>>)> = self
            .items
            .par_iter()
            .map(|item| compress_pending_item(item, block_size))
            .collect::<WorldResult<Vec<_>>>()?;

        // Reserve header space first so block offsets are exact.
        writer.seek(SeekFrom::Start(0))?;
        writer.write_all(&[0u8; PAK_HEADER_BYTES])?;

        let mut cursor_offset = PAK_HEADER_BYTES as u64;
        let mut toc_entries = Vec::with_capacity(prepared.len());
        const ZERO_PAD: [u8; 16] = [0u8; 16];

        for (mut entry, encoded_blocks) in prepared {
            for (block_idx, block_bytes) in encoded_blocks.iter().enumerate() {
                let misalignment = cursor_offset % PAK_BLOCK_ALIGNMENT;
                if misalignment != 0 {
                    let pad = (PAK_BLOCK_ALIGNMENT - misalignment) as usize;
                    writer.write_all(&ZERO_PAD[..pad])?;
                    cursor_offset = cursor_offset.saturating_add(pad as u64);
                }
                writer.write_all(block_bytes)?;
                entry.blocks[block_idx].offset = cursor_offset;
                cursor_offset = cursor_offset.saturating_add(block_bytes.len() as u64);
            }
            toc_entries.push(entry);
        }

        let toc_offset = cursor_offset;
        let toc_bytes = postcard::to_stdvec(&toc_entries)
            .map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        let toc_len = toc_bytes.len() as u64;
        let toc_crc32 = compute_crc32(&toc_bytes);
        writer.write_all(&toc_bytes)?;
        writer.write_all(&toc_crc32.to_le_bytes())?;

        // Patch final 32-byte header at offset 0.
        writer.seek(SeekFrom::Start(0))?;
        writer.write_all(&PAK_MAGIC)?;
        writer.write_all(&PAK_FORMAT_VERSION.to_le_bytes())?;
        writer.write_all(&block_size_u32.to_le_bytes())?;
        writer.write_all(&toc_offset.to_le_bytes())?;
        writer.write_all(&toc_len.to_le_bytes())?;
        Ok(())
    }
}

impl Default for PakBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Random-access `.aepak` reader validating TOC checksums and reading only the blocks needed.
pub struct PakReader<R: Read + Seek> {
    reader: R,
    index: PakIndex,
}

impl<R: Read + Seek> PakReader<R> {
    /// Opens an `.aepak` stream, validates the header and TOC CRC32, and builds the $O(1)$ lookup index.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] when magic, format version, TOC bounds, or CRC32 checks fail.
    pub fn open(mut reader: R) -> WorldResult<Self> {
        let index = read_pak_index(&mut reader)?;
        Ok(Self { reader, index })
    }

    /// Returns a reference to the validated [`PakIndex`].
    #[must_use]
    pub fn index(&self) -> &PakIndex {
        &self.index
    }

    /// Reads, decompresses, and CRC32-verifies an entire asset entry by [`AssetGuid`].
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`], I/O errors, decompression errors, or checksum mismatches.
    pub fn read_asset_by_guid(&mut self, guid: AssetGuid) -> WorldResult<Vec<u8>> {
        let entry = self.index.require_guid(guid)?;
        read_full_entry(&mut self.reader, entry)
    }

    /// Reads, decompresses in-place, and CRC32-verifies an entire asset directly into a 16-byte
    /// aligned [`AlignedBuffer`] suitable for zero-copy `bytemuck::try_cast_slice` GPU uploads.
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`], I/O errors, decompression errors, or checksum mismatches.
    pub fn read_asset_aligned_by_guid(&mut self, guid: AssetGuid) -> WorldResult<AlignedBuffer> {
        let entry = self.index.require_guid(guid)?;
        read_full_entry_aligned(&mut self.reader, entry)
    }

    /// Reads, decompresses, and CRC32-verifies an entire asset entry by virtual path.
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`], decompression errors, or checksum mismatches.
    pub fn read_asset_by_path(&mut self, virtual_path: &str) -> WorldResult<Vec<u8>> {
        let entry = self.index.require_path(virtual_path)?;
        read_full_entry(&mut self.reader, entry)
    }

    /// Reads, decompresses in-place, and CRC32-verifies an asset by virtual path directly into a
    /// 16-byte aligned [`AlignedBuffer`].
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`], decompression errors, or checksum mismatches.
    pub fn read_asset_aligned_by_path(&mut self, virtual_path: &str) -> WorldResult<AlignedBuffer> {
        let entry = self.index.require_path(virtual_path)?;
        read_full_entry_aligned(&mut self.reader, entry)
    }

    /// Reads an arbitrary byte slice `[offset .. offset + len)` from an entry by decompressing
    /// ONLY the intersecting `block_size` blocks.
    ///
    /// # Errors
    /// Returns [`WorldIoError::InvalidLayout`] if the requested range exceeds `entry.raw_size`,
    /// or [`WorldIoError`] on I/O/decompression failures.
    pub fn read_asset_range(
        &mut self,
        guid: AssetGuid,
        offset: u64,
        len: usize,
    ) -> WorldResult<Vec<u8>> {
        let entry = self.index.require_guid(guid)?;
        read_entry_range(
            &mut self.reader,
            entry,
            self.index.block_size as u64,
            offset,
            len,
        )
    }
}

/// Parses and CRC32-verifies the Table of Contents of an `.aepak` stream.
pub(crate) fn read_pak_index<R: Read + Seek>(reader: &mut R) -> WorldResult<PakIndex> {
    reader.seek(SeekFrom::Start(0))?;
    let mut header_buf = [0u8; PAK_HEADER_BYTES];
    read_exact_checked(reader, &mut header_buf, "pak header")?;

    let mut hdr = ByteReader::new(&header_buf);
    let magic = hdr.read_fixed_8("pak magic")?;
    if magic != PAK_MAGIC {
        return Err(WorldIoError::InvalidMagic {
            expected: PAK_MAGIC,
            found: magic,
        });
    }
    let format_version = hdr.read_u32("pak format_version")?;
    if format_version == 0 || format_version > PAK_FORMAT_VERSION {
        return Err(WorldIoError::UnsupportedVersion {
            found: format_version,
            max_supported: PAK_FORMAT_VERSION,
        });
    }
    let block_size = hdr.read_u32("pak block_size")?;
    if block_size == 0 {
        return Err(WorldIoError::InvalidLayout(
            "Pak block_size must be positive".to_string(),
        ));
    }
    let toc_offset = hdr.read_u64("pak toc_offset")?;
    let toc_len = hdr.read_u64("pak toc_len")?;
    let toc_usize = usize::try_from(toc_len).map_err(|_| {
        WorldIoError::InvalidLayout(format!("Pak TOC length {toc_len} exceeds RAM"))
    })?;

    reader.seek(SeekFrom::Start(toc_offset))?;
    let mut toc_bytes = vec![0u8; toc_usize];
    read_exact_checked(reader, &mut toc_bytes, "pak TOC payload")?;
    let mut crc_buf = [0u8; 4];
    read_exact_checked(reader, &mut crc_buf, "pak TOC crc32")?;
    verify_crc32(&toc_bytes, u32::from_le_bytes(crc_buf))?;

    let entries: Vec<PakTocEntry> =
        postcard::from_bytes(&toc_bytes).map_err(|e| WorldIoError::Serialization(e.to_string()))?;
    Ok(PakIndex::from_entries(format_version, block_size, entries))
}

fn read_exact_checked<R: Read>(
    reader: &mut R,
    buf: &mut [u8],
    context: &'static str,
) -> WorldResult<()> {
    reader
        .read_exact(buf)
        .map_err(|_| WorldIoError::TruncatedData {
            context,
            expected: buf.len(),
            actual: 0,
        })
}

fn checked_raw_size(entry: &PakTocEntry) -> WorldResult<usize> {
    usize::try_from(entry.raw_size).map_err(|_| {
        WorldIoError::InvalidLayout(format!("Asset size {} exceeds RAM", entry.raw_size))
    })
}

pub(crate) fn read_full_entry<R: Read + Seek>(
    reader: &mut R,
    entry: &PakTocEntry,
) -> WorldResult<Vec<u8>> {
    let mut out = vec![0u8; checked_raw_size(entry)?];
    decode_entry_into(reader, entry, &mut out)?;
    Ok(out)
}

pub(crate) fn read_full_entry_aligned<R: Read + Seek>(
    reader: &mut R,
    entry: &PakTocEntry,
) -> WorldResult<AlignedBuffer> {
    let mut out = AlignedBuffer::with_zeroed_len(checked_raw_size(entry)?);
    decode_entry_into(reader, entry, out.as_mut_bytes())?;
    Ok(out)
}

fn decode_entry_into<R: Read + Seek>(
    reader: &mut R,
    entry: &PakTocEntry,
    dest: &mut [u8],
) -> WorldResult<()> {
    let mut scratch = Vec::new();
    let mut cursor = 0usize;
    for block in &entry.blocks {
        let block_raw_len = block.raw_len as usize;
        let end = cursor.checked_add(block_raw_len).ok_or_else(|| {
            WorldIoError::InvalidLayout("Block raw lengths overflow target buffer".to_string())
        })?;
        let target_slice = dest.get_mut(cursor..end).ok_or_else(|| {
            WorldIoError::InvalidLayout("Block raw lengths exceed entry raw_size".to_string())
        })?;
        decode_block_into(reader, block, entry.codec, &mut scratch, target_slice)?;
        cursor = end;
    }
    if cursor != dest.len() {
        return Err(WorldIoError::InvalidLayout(format!(
            "Decoded {} bytes, expected entry raw_size {}",
            cursor,
            dest.len()
        )));
    }
    verify_crc32(dest, entry.raw_crc32)
}

fn read_entry_range<R: Read + Seek>(
    reader: &mut R,
    entry: &PakTocEntry,
    block_size: u64,
    offset: u64,
    len: usize,
) -> WorldResult<Vec<u8>> {
    let end = offset
        .checked_add(len as u64)
        .ok_or_else(|| WorldIoError::InvalidLayout("Range end overflow".to_string()))?;
    if end > entry.raw_size {
        return Err(WorldIoError::InvalidLayout(format!(
            "Requested range [{offset}..{end}) exceeds asset size {}",
            entry.raw_size
        )));
    }
    if len == 0 {
        return Ok(Vec::new());
    }

    let first_block = (offset / block_size) as usize;
    let last_block = ((end - 1) / block_size) as usize;
    let mut out = Vec::with_capacity(len);
    let mut scratch = Vec::new();
    let mut block_data = Vec::new();
    for block_idx in first_block..=last_block {
        let meta = entry.blocks.get(block_idx).ok_or_else(|| {
            WorldIoError::InvalidLayout(format!("Missing pak block index {block_idx}"))
        })?;
        block_data.resize(meta.raw_len as usize, 0);
        decode_block_into(reader, meta, entry.codec, &mut scratch, &mut block_data)?;
        let block_start = (block_idx as u64) * block_size;
        let slice_start = offset.saturating_sub(block_start) as usize;
        let slice_end = (end.saturating_sub(block_start) as usize).min(block_data.len());
        out.extend_from_slice(&block_data[slice_start..slice_end]);
    }
    Ok(out)
}

fn decode_block_into<R: Read + Seek>(
    reader: &mut R,
    block: &PakBlockMeta,
    codec: PakCodec,
    scratch: &mut Vec<u8>,
    target_slice: &mut [u8],
) -> WorldResult<()> {
    reader.seek(SeekFrom::Start(block.offset))?;
    match codec {
        PakCodec::Raw => {
            if block.stored_len != block.raw_len {
                return Err(WorldIoError::InvalidLayout(
                    "Raw pak block stored_len mismatches raw_len".to_string(),
                ));
            }
            read_exact_checked(reader, target_slice, "pak raw block")?;
        }
        PakCodec::Lz4Block => {
            scratch.resize(block.stored_len as usize, 0);
            read_exact_checked(reader, scratch, "pak compressed block")?;
            let written = lz4_flex::block::decompress_into(scratch, target_slice)
                .map_err(|e| WorldIoError::Compression(e.to_string()))?;
            if written != block.raw_len as usize {
                return Err(WorldIoError::Compression(format!(
                    "LZ4 block decompressed {written} bytes, expected {}",
                    block.raw_len
                )));
            }
        }
    }
    Ok(())
}

fn compress_pending_item(
    item: &PendingPakInput,
    block_size: usize,
) -> WorldResult<(PakTocEntry, Vec<Vec<u8>>)> {
    let raw_crc32 = compute_crc32(&item.raw_bytes);
    let raw_size = item.raw_bytes.len() as u64;

    let mut metas = Vec::new();
    let mut encoded_blocks = Vec::new();
    for chunk in item.raw_bytes.chunks(block_size) {
        let raw_len = u32::try_from(chunk.len())
            .map_err(|_| WorldIoError::InvalidLayout("Block size exceeds u32::MAX".to_string()))?;
        let stored = match item.codec {
            PakCodec::Raw => chunk.to_vec(),
            PakCodec::Lz4Block => lz4_flex::compress(chunk),
        };
        let stored_len = u32::try_from(stored.len()).map_err(|_| {
            WorldIoError::InvalidLayout("Compressed block size exceeds u32::MAX".to_string())
        })?;
        metas.push(PakBlockMeta {
            offset: 0,
            stored_len,
            raw_len,
        });
        encoded_blocks.push(stored);
    }

    Ok((
        PakTocEntry {
            guid: item.guid,
            virtual_path: item.virtual_path.clone(),
            kind: item.kind,
            codec: item.codec,
            raw_size,
            raw_crc32,
            blocks: metas,
        },
        encoded_blocks,
    ))
}

pub fn normalize_pak_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}

/// Opens an `.aepak` file on disk inside a buffered [`PakReader`].
///
/// # Errors
/// Returns [`WorldIoError`] on I/O or TOC validation failures.
pub fn open_pak_file(path: &Path) -> WorldResult<PakReader<BufReader<File>>> {
    let file = File::open(path)?;
    PakReader::open(BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pak_parallel_build_and_block_level_random_access() {
        let mut builder = PakBuilder::with_block_size(1024);
        let payload_a: Vec<u8> = (0..5000u32).map(|i| (i % 251) as u8).collect();
        let payload_b = b"hello-raw-shader-bytes".to_vec();

        let guid_a = builder.add_asset(
            "./textures\\terrain_albedo.bin",
            AssetKind::Texture,
            PakCodec::Lz4Block,
            payload_a.clone(),
        );
        let guid_b = builder.add_asset(
            "shaders/pbr.wgsl",
            AssetKind::Shader,
            PakCodec::Raw,
            payload_b.clone(),
        );

        let pak_bytes = builder.build_to_vec().expect("build pak bytes");
        let mut reader = PakReader::open(Cursor::new(&pak_bytes)).expect("open pak");

        assert_eq!(reader.index().entries.len(), 2);
        assert_eq!(
            reader
                .read_asset_by_path("textures/terrain_albedo.bin")
                .expect("read by normalized path"),
            payload_a
        );
        assert_eq!(
            reader
                .read_asset_by_guid(guid_b)
                .expect("read shader by guid"),
            payload_b
        );

        // Random range crossing block boundary (block 0 ends at 1024, block 1 starts at 1024)
        let sub = reader
            .read_asset_range(guid_a, 1000, 120)
            .expect("random-access range read");
        assert_eq!(sub, &payload_a[1000..1120]);

        // Out-of-bounds range must error cleanly
        assert!(matches!(
            reader.read_asset_range(guid_a, 4950, 100),
            Err(WorldIoError::InvalidLayout(_))
        ));
    }

    #[test]
    fn test_pak_rejects_corrupted_toc_or_data() {
        let mut builder = PakBuilder::new();
        builder.add_asset(
            "audio/click.pcm",
            AssetKind::Audio,
            PakCodec::Lz4Block,
            vec![7u8; 4096],
        );
        let pak = builder.build_to_vec().expect("build pak");

        let mut bad_toc = pak;
        if let Some(last) = bad_toc.last_mut() {
            *last ^= 0x01;
        }
        assert!(matches!(
            PakReader::open(Cursor::new(&bad_toc)),
            Err(WorldIoError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn test_pak_16_byte_block_offsets_and_aligned_gpu_decompress() {
        let mut builder = PakBuilder::with_block_size(512);
        let unaligned_pad_asset = vec![1u8; 7];
        let verts: Vec<[f32; 4]> = (0..256u32).map(|i| [i as f32, 1.0, 2.0, 3.0]).collect();
        let verts_raw = bytemuck::cast_slice(&verts).to_vec();

        builder.add_asset(
            "misc/odd.bin",
            AssetKind::Unknown,
            PakCodec::Raw,
            unaligned_pad_asset,
        );
        let lz4_guid = builder.add_asset(
            "meshes/cube.cooked",
            AssetKind::Model,
            PakCodec::Lz4Block,
            verts_raw.clone(),
        );
        builder.add_asset(
            "meshes/quad.raw",
            AssetKind::Model,
            PakCodec::Raw,
            verts_raw,
        );

        let pak_bytes = builder.build_to_vec().expect("build aligned pak");
        let mut reader = PakReader::open(Cursor::new(&pak_bytes)).expect("open aligned pak");
        for entry in &reader.index().entries {
            for block in &entry.blocks {
                assert_eq!(block.offset % PAK_BLOCK_ALIGNMENT, 0);
            }
        }
        let lz4_buf = reader
            .read_asset_aligned_by_guid(lz4_guid)
            .expect("aligned lz4");
        let raw_buf = reader
            .read_asset_aligned_by_path("meshes/quad.raw")
            .expect("aligned raw");
        assert_eq!(
            lz4_buf
                .try_cast_slice::<[f32; 4]>()
                .expect("cast lz4 verts"),
            verts.as_slice()
        );
        assert_eq!(
            raw_buf
                .try_cast_slice::<[f32; 4]>()
                .expect("cast raw verts"),
            verts.as_slice()
        );
    }
}
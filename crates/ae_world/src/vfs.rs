// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Cross-platform Virtual File System (`VfsProvider`) abstracting project disk folders,
//! memory-resident `.aepak` bundles, and file-backed random-access `.aepak` archives.

use crate::aligned_buffer::AlignedBuffer;
use crate::error::{WorldIoError, WorldResult};
use crate::guid::AssetGuid;
use crate::pak::{
    PakIndex, PakTocEntry, normalize_pak_path, read_full_entry, read_full_entry_aligned,
    read_pak_index,
};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

/// Thread-safe asset and chunk byte source implemented identically across editor disk layouts,
/// standalone `.aepak` containers, and embedded/mobile bundles.
pub trait VfsProvider: Send + Sync {
    /// Returns `true` if an entry exists at `virtual_path`.
    fn exists(&self, virtual_path: &str) -> bool;

    /// Loads the complete uncompressed payload for `virtual_path`.
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`] when missing or [`WorldIoError`] on I/O/integrity failure.
    fn read_bytes(&self, virtual_path: &str) -> WorldResult<Vec<u8>>;

    /// Loads the complete uncompressed payload for `virtual_path` directly into a `16`-byte
    /// aligned [`AlignedBuffer`] suitable for safe `bytemuck::try_cast_slice` GPU uploads.
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`] when missing or [`WorldIoError`] on I/O/integrity failure.
    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<AlignedBuffer>;

    /// Optionally loads an entry directly by [`AssetGuid`] when backed by an indexed archive.
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`] when the provider has no GUID index entry for `guid`.
    fn read_by_guid(&self, guid: AssetGuid) -> WorldResult<Vec<u8>> {
        Err(WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))
    }

    /// Optionally loads an entry directly by [`AssetGuid`] into a `16`-byte aligned [`AlignedBuffer`].
    ///
    /// # Errors
    /// Returns [`WorldIoError::EntryNotFound`] when the provider has no GUID index entry for `guid`.
    fn read_aligned_by_guid(&self, guid: AssetGuid) -> WorldResult<AlignedBuffer> {
        Err(WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))
    }

    /// Lists normalized virtual paths exposed by this provider.
    fn list_entries(&self) -> Vec<String>;
}

/// Validates and normalizes a virtual asset path, rejecting parent-directory (`..`) escape attempts.
///
/// # Errors
/// Returns [`WorldIoError::InvalidLayout`] if the path is empty or contains `..` / root prefixes.
pub fn sanitize_virtual_path(raw: &str) -> WorldResult<PathBuf> {
    let normalized = normalize_pak_path(raw);
    if normalized.is_empty() {
        return Err(WorldIoError::InvalidLayout(
            "Virtual path must not be empty".to_string(),
        ));
    }
    let mut clean = PathBuf::new();
    for comp in Path::new(&normalized).components() {
        match comp {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(WorldIoError::InvalidLayout(format!(
                    "Illegal path traversal segment in virtual path '{raw}'"
                )));
            }
        }
    }
    Ok(clean)
}

/// Editor/development [`VfsProvider`] reading loose files directly from a root directory on disk.
///
/// Enforces path traversal isolation so virtual paths cannot escape `root_dir`.
#[derive(Debug, Clone)]
pub struct DiskVfsProvider {
    root_dir: PathBuf,
}

impl DiskVfsProvider {
    /// Creates a disk-backed VFS provider rooted at `root_dir`.
    #[must_use]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    /// Returns the filesystem root directory wrapped by this provider.
    #[must_use]
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn resolve_safe(&self, virtual_path: &str) -> WorldResult<PathBuf> {
        let rel = sanitize_virtual_path(virtual_path)?;
        Ok(self.root_dir.join(rel))
    }
}

impl VfsProvider for DiskVfsProvider {
    fn exists(&self, virtual_path: &str) -> bool {
        self.resolve_safe(virtual_path)
            .is_ok_and(|path| path.is_file())
    }

    fn read_bytes(&self, virtual_path: &str) -> WorldResult<Vec<u8>> {
        let target = self.resolve_safe(virtual_path)?;
        if !target.is_file() {
            return Err(WorldIoError::EntryNotFound(virtual_path.to_string()));
        }
        Ok(std::fs::read(target)?)
    }

    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<AlignedBuffer> {
        let target = self.resolve_safe(virtual_path)?;
        if !target.is_file() {
            return Err(WorldIoError::EntryNotFound(virtual_path.to_string()));
        }
        let mut file = File::open(target)?;
        let len_u64 = file.metadata()?.len();
        let len = usize::try_from(len_u64)
            .map_err(|_| WorldIoError::InvalidLayout(format!("File size {len_u64} exceeds RAM")))?;
        let mut out = AlignedBuffer::with_zeroed_len(len);
        file.read_exact(out.as_mut_bytes())?;
        Ok(out)
    }

    fn list_entries(&self) -> Vec<String> {
        let mut out = Vec::new();
        collect_disk_entries(&self.root_dir, &self.root_dir, &mut out);
        out.sort();
        out
    }
}

fn collect_disk_entries(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_disk_entries(root, &path, out);
        } else if path.is_file()
            && let Ok(rel) = path.strip_prefix(root)
        {
            out.push(normalize_pak_path(&rel.to_string_lossy()));
        }
    }
}

/// Lock-free, thread-safe `.aepak` [`VfsProvider`] backed by an in-memory archive buffer.
///
/// Ideal for embedded packages and mobile asset streams loaded once into shared memory.
#[derive(Debug, Clone)]
pub struct PakMemoryVfsProvider {
    bytes: Arc<[u8]>,
    index: Arc<PakIndex>,
}

impl PakMemoryVfsProvider {
    /// Parses and validates an `.aepak` buffer into a lock-free concurrent [`VfsProvider`].
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if the header or TOC CRC32 verification fails.
    pub fn from_bytes(bytes: Vec<u8>) -> WorldResult<Self> {
        let arc_bytes: Arc<[u8]> = Arc::from(bytes);
        let mut cursor = Cursor::new(&*arc_bytes);
        let index = read_pak_index(&mut cursor)?;
        Ok(Self {
            bytes: arc_bytes,
            index: Arc::new(index),
        })
    }

    fn read_toc_entry(&self, entry: &PakTocEntry) -> WorldResult<Vec<u8>> {
        let mut cursor = Cursor::new(&*self.bytes);
        read_full_entry(&mut cursor, entry)
    }

    fn read_toc_entry_aligned(&self, entry: &PakTocEntry) -> WorldResult<AlignedBuffer> {
        let mut cursor = Cursor::new(&*self.bytes);
        read_full_entry_aligned(&mut cursor, entry)
    }
}

impl VfsProvider for PakMemoryVfsProvider {
    fn exists(&self, virtual_path: &str) -> bool {
        self.index.get_by_path(virtual_path).is_some()
    }

    fn read_bytes(&self, virtual_path: &str) -> WorldResult<Vec<u8>> {
        let entry = self
            .index
            .get_by_path(virtual_path)
            .ok_or_else(|| WorldIoError::EntryNotFound(virtual_path.to_string()))?;
        self.read_toc_entry(entry)
    }

    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<AlignedBuffer> {
        let entry = self
            .index
            .get_by_path(virtual_path)
            .ok_or_else(|| WorldIoError::EntryNotFound(virtual_path.to_string()))?;
        self.read_toc_entry_aligned(entry)
    }

    fn read_by_guid(&self, guid: AssetGuid) -> WorldResult<Vec<u8>> {
        let entry = self
            .index
            .get_by_guid(guid)
            .ok_or_else(|| WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))?;
        self.read_toc_entry(entry)
    }

    fn read_aligned_by_guid(&self, guid: AssetGuid) -> WorldResult<AlignedBuffer> {
        let entry = self
            .index
            .get_by_guid(guid)
            .ok_or_else(|| WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))?;
        self.read_toc_entry_aligned(entry)
    }

    fn list_entries(&self) -> Vec<String> {
        self.index
            .entries
            .iter()
            .map(|e| e.virtual_path.clone())
            .collect()
    }
}

/// Concurrent file-backed `.aepak` [`VfsProvider`] caching the validated TOC in memory
/// while allowing parallel worker threads to read compressed asset blocks independently.
#[derive(Debug, Clone)]
pub struct PakFileVfsProvider {
    pak_path: PathBuf,
    index: Arc<PakIndex>,
}

impl PakFileVfsProvider {
    /// Opens an `.aepak` file on disk, validates its TOC CRC32 once, and caches the index.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on I/O or TOC corruption failures.
    pub fn open(pak_path: impl Into<PathBuf>) -> WorldResult<Self> {
        let path = pak_path.into();
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let index = read_pak_index(&mut reader)?;
        Ok(Self {
            pak_path: path,
            index: Arc::new(index),
        })
    }

    fn read_toc_entry(&self, entry: &PakTocEntry) -> WorldResult<Vec<u8>> {
        let file = File::open(&self.pak_path)?;
        let mut reader = BufReader::new(file);
        read_full_entry(&mut reader, entry)
    }

    fn read_toc_entry_aligned(&self, entry: &PakTocEntry) -> WorldResult<AlignedBuffer> {
        let file = File::open(&self.pak_path)?;
        let mut reader = BufReader::new(file);
        read_full_entry_aligned(&mut reader, entry)
    }
}

impl VfsProvider for PakFileVfsProvider {
    fn exists(&self, virtual_path: &str) -> bool {
        self.index.get_by_path(virtual_path).is_some()
    }

    fn read_bytes(&self, virtual_path: &str) -> WorldResult<Vec<u8>> {
        let entry = self
            .index
            .get_by_path(virtual_path)
            .ok_or_else(|| WorldIoError::EntryNotFound(virtual_path.to_string()))?;
        self.read_toc_entry(entry)
    }

    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<AlignedBuffer> {
        let entry = self
            .index
            .get_by_path(virtual_path)
            .ok_or_else(|| WorldIoError::EntryNotFound(virtual_path.to_string()))?;
        self.read_toc_entry_aligned(entry)
    }

    fn read_by_guid(&self, guid: AssetGuid) -> WorldResult<Vec<u8>> {
        let entry = self
            .index
            .get_by_guid(guid)
            .ok_or_else(|| WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))?;
        self.read_toc_entry(entry)
    }

    fn read_aligned_by_guid(&self, guid: AssetGuid) -> WorldResult<AlignedBuffer> {
        let entry = self
            .index
            .get_by_guid(guid)
            .ok_or_else(|| WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))?;
        self.read_toc_entry_aligned(entry)
    }

    fn list_entries(&self) -> Vec<String> {
        self.index
            .entries
            .iter()
            .map(|e| e.virtual_path.clone())
            .collect()
    }
}

/// Ordered mount stack checking higher-priority [`VfsProvider`] layers first
/// (e.g., patch archive -> base archive -> loose workspace override).
#[derive(Default)]
pub struct CompositeVfs {
    mounts: Vec<Arc<dyn VfsProvider>>,
}

impl CompositeVfs {
    /// Creates an empty composite mount stack.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a provider layer evaluated after any previously mounted providers.
    pub fn mount(&mut self, provider: Arc<dyn VfsProvider>) {
        self.mounts.push(provider);
    }
}

impl VfsProvider for CompositeVfs {
    fn exists(&self, virtual_path: &str) -> bool {
        self.mounts.iter().any(|m| m.exists(virtual_path))
    }

    fn read_bytes(&self, virtual_path: &str) -> WorldResult<Vec<u8>> {
        for mount in &self.mounts {
            match mount.read_bytes(virtual_path) {
                Ok(bytes) => return Ok(bytes),
                Err(WorldIoError::EntryNotFound(_)) => continue,
                Err(err) => return Err(err),
            }
        }
        Err(WorldIoError::EntryNotFound(virtual_path.to_string()))
    }

    fn read_aligned_bytes(&self, virtual_path: &str) -> WorldResult<AlignedBuffer> {
        for mount in &self.mounts {
            match mount.read_aligned_bytes(virtual_path) {
                Ok(bytes) => return Ok(bytes),
                Err(WorldIoError::EntryNotFound(_)) => continue,
                Err(err) => return Err(err),
            }
        }
        Err(WorldIoError::EntryNotFound(virtual_path.to_string()))
    }

    fn read_by_guid(&self, guid: AssetGuid) -> WorldResult<Vec<u8>> {
        for mount in &self.mounts {
            match mount.read_by_guid(guid) {
                Ok(bytes) => return Ok(bytes),
                Err(WorldIoError::EntryNotFound(_)) => continue,
                Err(err) => return Err(err),
            }
        }
        Err(WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))
    }

    fn read_aligned_by_guid(&self, guid: AssetGuid) -> WorldResult<AlignedBuffer> {
        for mount in &self.mounts {
            match mount.read_aligned_by_guid(guid) {
                Ok(bytes) => return Ok(bytes),
                Err(WorldIoError::EntryNotFound(_)) => continue,
                Err(err) => return Err(err),
            }
        }
        Err(WorldIoError::EntryNotFound(format!("AssetGuid {guid}")))
    }

    fn list_entries(&self) -> Vec<String> {
        let mut unique = BTreeSet::new();
        for mount in &self.mounts {
            for entry in mount.list_entries() {
                unique.insert(entry);
            }
        }
        unique.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pak::{PakBuilder, PakCodec};
    use crate::world_archive::AssetKind;
    use rayon::prelude::*;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!("ae_world_vfs_{label}_{nanos}"))
    }

    #[test]
    fn test_composite_vfs_priority_and_traversal_guard() {
        let root = unique_temp_dir("priority");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("configs")).expect("mkdir");
        std::fs::write(root.join("configs/game.bin"), b"disk-override").expect("write disk file");

        let mut builder = PakBuilder::new();
        builder.add_asset(
            "configs/game.bin",
            AssetKind::Unknown,
            PakCodec::Lz4Block,
            b"pak-base".to_vec(),
        );
        let guid_only = builder.add_asset(
            "scenes/intro.aeworld",
            AssetKind::Unknown,
            PakCodec::Lz4Block,
            b"pak-world".to_vec(),
        );
        let pak_bytes = builder.build_to_vec().expect("build pak");

        let mut composite = CompositeVfs::new();
        composite.mount(Arc::new(DiskVfsProvider::new(&root)));
        composite.mount(Arc::new(
            PakMemoryVfsProvider::from_bytes(pak_bytes).expect("mount memory pak"),
        ));

        // Higher priority disk layer overrides pak
        assert_eq!(
            composite
                .read_bytes("configs/game.bin")
                .expect("read override"),
            b"disk-override"
        );
        // Missing on disk resolves  from pak by path and GUID
        assert_eq!(
            composite
                .read_bytes("scenes/intro.aeworld")
                .expect("read from pak"),
            b"pak-world"
        );
        assert_eq!(
            composite.read_by_guid(guid_only).expect("read by guid"),
            b"pak-world"
        );
        assert_eq!(
            composite
                .read_aligned_bytes("configs/game.bin")
                .expect("read aligned override")
                .as_bytes(),
            b"disk-override"
        );
        assert_eq!(
            composite
                .read_aligned_by_guid(guid_only)
                .expect("read aligned by guid")
                .as_bytes(),
            b"pak-world"
        );

        // Traversal attempt must be rejected
        assert!(matches!(
            DiskVfsProvider::new(&root).read_bytes("../secret.txt"),
            Err(WorldIoError::InvalidLayout(_))
        ));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_pak_file_vfs_parallel_reads() {
        let dir = unique_temp_dir("pakfile");
        let _ = std::fs::remove_dir_all(&dir);
        let pak_path = dir.join("game.aepak");

        let mut builder = PakBuilder::with_block_size(1024);
        let expected: Vec<(String, Vec<u8>)> = (0..16)
            .map(|i| {
                let path = format!("chunks/c_{i}.bin");
                let bytes = vec![i as u8; 2500];
                builder.add_asset(&path, AssetKind::Unknown, PakCodec::Lz4Block, bytes.clone());
                (path, bytes)
            })
            .collect();
        builder.write_to_file(&pak_path).expect("write pak file");

        let provider = PakFileVfsProvider::open(&pak_path).expect("open pak vfs");
        expected.par_iter().for_each(|(path, bytes)| {
            let loaded = provider
                .read_aligned_bytes(path)
                .expect("parallel vfs aligned read");
            assert_eq!(loaded.as_bytes(), bytes.as_slice());
            assert_eq!((loaded.as_bytes().as_ptr() as usize) % 16, 0);
        });

        let _ = std::fs::remove_dir_all(&dir);
    }
}
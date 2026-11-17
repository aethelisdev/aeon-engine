// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Modular project workspace directory layout (`Config/`, `Content/`, `Worlds/`,
//! `Intermediate/`, `Saved/`) and binary companion asset metadata (`.aemeta`).

use crate::error::{WorldIoError, WorldResult};
use crate::guid::{AssetGuid, asset_guid_from_virtual_path};
use crate::header::{compute_crc32, decode_checksummed_envelope, encode_checksummed_envelope};
use crate::pak::normalize_pak_path;
use crate::world_archive::AssetKind;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Magic signature identifying binary `.aemeta` companion asset metadata files.
pub const AEMETA_MAGIC: [u8; 8] = *b"AEMET001";

/// Current `.aemeta` binary format version.
pub const AEMETA_FORMAT_VERSION: u32 = 1;

/// Companion binary metadata stored beside each source asset inside `Content/` (`<asset>.aemeta`).
///
/// Preserves persistent [`AssetGuid`] identity across directory moves or renames so worlds
/// and prefabs do not lose references when assets are reorganized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetMeta {
    /// Persistent 128-bit asset identifier referenced by worlds, prefabs, and paks.
    pub guid: AssetGuid,
    /// Semantic asset classification.
    pub kind: AssetKind,
    /// Source asset byte length recorded when metadata was last synchronized.
    pub source_size: u64,
    /// Source asset CRC32 recorded when metadata was last synchronized.
    pub source_crc32: u32,
}

impl AssetMeta {
    /// Returns the companion metadata path for `asset_path` (e.g., `hero.glb` -> `hero.glb.aemeta`).
    #[must_use]
    pub fn companion_path(asset_path: &Path) -> PathBuf {
        let mut os = asset_path.as_os_str().to_os_string();
        os.push(".aemeta");
        PathBuf::from(os)
    }

    /// Returns `true` if `path` is itself a `.aemeta` or staging `.tmp` file that should not
    /// receive its own nested metadata file.
    #[must_use]
    pub fn is_metadata_or_staging(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                ext.eq_ignore_ascii_case("aemeta") || ext.eq_ignore_ascii_case("tmp")
            })
    }

    /// Encodes this metadata into a CRC32-verified [`AEMETA_MAGIC`] binary buffer.
    ///
    /// # Errors
    /// Returns [`WorldIoError::Serialization`] if `postcard` encoding fails.
    pub fn encode(&self) -> WorldResult<Vec<u8>> {
        let payload =
            postcard::to_stdvec(self).map_err(|e| WorldIoError::Serialization(e.to_string()))?;
        Ok(encode_checksummed_envelope(
            AEMETA_MAGIC,
            AEMETA_FORMAT_VERSION,
            0,
            &payload,
        ))
    }

    /// Validates and decodes an `.aemeta` binary buffer.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on magic, version, CRC32, or `postcard` errors.
    pub fn decode(bytes: &[u8]) -> WorldResult<Self> {
        let env = decode_checksummed_envelope(bytes, AEMETA_MAGIC, AEMETA_FORMAT_VERSION)?;
        postcard::from_bytes(env.payload).map_err(|e| WorldIoError::Serialization(e.to_string()))
    }

    /// Reads an existing companion `.aemeta` file or creates/updates one atomically for `asset_path`.
    ///
    /// Existing valid `.aemeta` files keep their original [`AssetGuid`] verbatim, refreshing only
    /// `source_size` and `source_crc32` when the underlying file changed on disk.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] if `asset_path` cannot be read or metadata cannot be persisted.
    pub fn ensure_for_asset_file(
        asset_path: &Path,
        kind: AssetKind,
        virtual_path_for_seed: &str,
    ) -> WorldResult<Self> {
        let raw_bytes = std::fs::read(asset_path)?;
        let source_size = raw_bytes.len() as u64;
        let source_crc32 = compute_crc32(&raw_bytes);
        let meta_path = Self::companion_path(asset_path);

        if let Ok(existing_bytes) = std::fs::read(&meta_path)
            && let Ok(mut existing) = Self::decode(&existing_bytes)
        {
            if existing.source_size != source_size
                || existing.source_crc32 != source_crc32
                || existing.kind != kind
            {
                existing.kind = kind;
                existing.source_size = source_size;
                existing.source_crc32 = source_crc32;
                existing.write_atomic(&meta_path)?;
            }
            return Ok(existing);
        }

        let created = Self {
            guid: asset_guid_from_virtual_path(virtual_path_for_seed),
            kind,
            source_size,
            source_crc32,
        };
        created.write_atomic(&meta_path)?;
        Ok(created)
    }

    /// Writes this `.aemeta` file atomically via a `.tmp` + `sync_all` + `rename` swap.
    ///
    /// # Errors
    /// Returns [`WorldIoError`] on I/O or encoding errors.
    pub fn write_atomic(&self, meta_path: &Path) -> WorldResult<()> {
        if let Some(parent) = meta_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = self.encode()?;
        let mut tmp_os = meta_path.as_os_str().to_os_string();
        tmp_os.push(".tmp");
        let tmp_path = PathBuf::from(tmp_os);
        {
            let mut f = File::create(&tmp_path)?;
            f.write_all(&bytes)?;
            f.flush()?;
            f.sync_all()?;
        }
        std::fs::rename(&tmp_path, meta_path)?;
        Ok(())
    }
}

/// Scans `content_dir` recursively and ensures every classified asset file has an up-to-date
/// `.aemeta` companion file preserving its persistent [`AssetGuid`].
///
/// # Errors
/// Returns [`WorldIoError`] if writing metadata fails.
pub fn sync_content_metadata(
    content_dir: &Path,
    classify: impl Fn(&Path) -> Option<AssetKind> + Copy,
) -> WorldResult<Vec<(PathBuf, AssetMeta)>> {
    let mut results = Vec::new();
    if !content_dir.is_dir() {
        return Ok(results);
    }
    sync_content_recursive(content_dir, content_dir, classify, &mut results)?;
    Ok(results)
}

fn sync_content_recursive(
    root: &Path,
    dir: &Path,
    classify: impl Fn(&Path) -> Option<AssetKind> + Copy,
    out: &mut Vec<(PathBuf, AssetMeta)>,
) -> WorldResult<()> {
    let entries = std::fs::read_dir(dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sync_content_recursive(root, &path, classify, out)?;
        } else if path.is_file()
            && !AssetMeta::is_metadata_or_staging(&path)
            && let Some(kind) = classify(&path)
        {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let virt = normalize_pak_path(&rel.to_string_lossy());
            let meta = AssetMeta::ensure_for_asset_file(&path, kind, &virt)?;
            out.push((path, meta));
        }
    }
    Ok(())
}

/// Canonical directory layout for an Aeon Engine project workspace:
/// `Config/`, `Content/`, `Worlds/`, `Intermediate/`, and `Saved/{Autosaves,Logs,SaveGames}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectWorkspaceLayout {
    root: PathBuf,
}

impl ProjectWorkspaceLayout {
    /// Canonical relative path to the default initial world manifest inside a new project.
    pub const DEFAULT_ENTRY_WORLD_REL: &'static str = "Worlds/MainWorld/world.aeworld";

    /// Creates a workspace layout rooted at `root`.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the project root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns `<root>/Config` for engine, physics, render, and input configuration files.
    #[must_use]
    pub fn config_dir(&self) -> PathBuf {
        self.root.join("Config")
    }

    /// Returns `<root>/Content` for source models, textures, audio, shaders, and prefabs.
    #[must_use]
    pub fn content_dir(&self) -> PathBuf {
        self.root.join("Content")
    }

    /// Returns `<root>/Worlds` containing partitioned world subdirectories.
    #[must_use]
    pub fn worlds_dir(&self) -> PathBuf {
        self.root.join("Worlds")
    }

    /// Returns `<root>/Worlds/<world_name>` holding `world.aeworld` and `cells/*.aechunk`.
    #[must_use]
    pub fn world_dir(&self, world_name: &str) -> PathBuf {
        self.worlds_dir().join(world_name)
    }

    /// Returns `<root>/Intermediate` for derived build and cooked asset caches.
    #[must_use]
    pub fn intermediate_dir(&self) -> PathBuf {
        self.root.join("Intermediate")
    }

    /// Returns `<root>/Saved` for autosaves, logs, and player `.aesave` slots.
    #[must_use]
    pub fn saved_dir(&self) -> PathBuf {
        self.root.join("Saved")
    }

    /// Returns `<root>/Saved/Autosaves`.
    #[must_use]
    pub fn autosaves_dir(&self) -> PathBuf {
        self.saved_dir().join("Autosaves")
    }

    /// Returns `<root>/Saved/Logs`.
    #[must_use]
    pub fn logs_dir(&self) -> PathBuf {
        self.saved_dir().join("Logs")
    }

    /// Returns `<root>/Saved/SaveGames`.
    #[must_use]
    pub fn savegames_dir(&self) -> PathBuf {
        self.saved_dir().join("SaveGames")
    }

    /// Creates the entire modular workspace folder hierarchy (`Config/`, `Content/`, `Worlds/`,
    /// `Intermediate/`, `Saved/{Autosaves,Logs,SaveGames}`) and adds `.gitignore` guards inside
    /// derived/transient directories (`Intermediate/`, `Saved/`) if missing.
    ///
    /// # Errors
    /// Returns [`WorldIoError::Io`] if directory creation fails.
    pub fn ensure_directories(&self) -> WorldResult<()> {
        for dir in [
            self.config_dir(),
            self.content_dir(),
            self.worlds_dir(),
            self.intermediate_dir(),
            self.saved_dir(),
            self.autosaves_dir(),
            self.logs_dir(),
            self.savegames_dir(),
        ] {
            std::fs::create_dir_all(&dir)?;
        }

        for transient in [self.intermediate_dir(), self.saved_dir()] {
            let ignore_file = transient.join(".gitignore");
            if !ignore_file.exists() {
                let _ = std::fs::write(&ignore_file, "*\n!.gitignore\n");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_proj_dir(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        std::env::temp_dir().join(format!("ae_world_proj_{label}_{nanos}"))
    }

    #[test]
    fn test_project_workspace_layout_creates_expected_tree() {
        let root = unique_proj_dir("tree");
        let _ = std::fs::remove_dir_all(&root);
        let layout = ProjectWorkspaceLayout::new(&root);
        layout.ensure_directories().expect("create workspace tree");

        assert!(layout.config_dir().is_dir());
        assert!(layout.content_dir().is_dir());
        assert!(layout.worlds_dir().is_dir());
        assert!(layout.intermediate_dir().is_dir());
        assert!(layout.autosaves_dir().is_dir());
        assert!(layout.logs_dir().is_dir());
        assert!(layout.savegames_dir().is_dir());
        assert!(layout.intermediate_dir().join(".gitignore").is_file());
        assert!(layout.saved_dir().join(".gitignore").is_file());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn test_asset_meta_preserves_guid_across_move_and_content_sync() {
        let root = unique_proj_dir("meta");
        let _ = std::fs::remove_dir_all(&root);
        let layout = ProjectWorkspaceLayout::new(&root);
        layout.ensure_directories().expect("create layout");

        let src_dir = layout.content_dir().join("Props");
        std::fs::create_dir_all(&src_dir).expect("mkdir Props");
        let asset_file = src_dir.join("crate.glb");
        std::fs::write(&asset_file, b"binary-mesh-placeholder").expect("write asset");

        let synced = sync_content_metadata(&layout.content_dir(), |p| {
            let ext = p.extension()?.to_str()?;
            if ext.eq_ignore_ascii_case("glb") {
                Some(AssetKind::Model)
            } else {
                None
            }
        })
        .expect("sync content metadata");
        assert_eq!(synced.len(), 1);
        let original_guid = synced[0].1.guid;
        assert!(AssetMeta::companion_path(&asset_file).is_file());

        // Move both asset and its .aemeta to a different subfolder: GUID must remain identical!
        let dst_dir = layout.content_dir().join("Environment");
        std::fs::create_dir_all(&dst_dir).expect("mkdir Environment");
        let moved_asset = dst_dir.join("crate.glb");
        std::fs::rename(&asset_file, &moved_asset).expect("move asset");
        std::fs::rename(
            AssetMeta::companion_path(&asset_file),
            AssetMeta::companion_path(&moved_asset),
        )
        .expect("move .aemeta");

        let re_synced = sync_content_metadata(&layout.content_dir(), |p| {
            let ext = p.extension()?.to_str()?;
            if ext.eq_ignore_ascii_case("glb") {
                Some(AssetKind::Model)
            } else {
                None
            }
        })
        .expect("re-sync moved metadata");
        assert_eq!(re_synced.len(), 1);
        assert_eq!(re_synced[0].1.guid, original_guid);

        let _ = std::fs::remove_dir_all(&root);
    }
}
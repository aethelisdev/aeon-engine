// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.

//! Texture asset storage, generational handle management, and path deduplication.

pub use ae_plugin_api::AssetHandle;
use slotmap::SlotMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::data::CpuTextureData;

/// CPU texture asset metadata and pixel data container registered in storage.
#[derive(Debug, Clone)]
pub struct TextureAssetData {
    /// Inner uncompressed CPU texture pixel data.
    pub data: CpuTextureData,
    /// Absolute canonical path on disk for memory deduplication.
    pub canonical_path: Option<PathBuf>,
}

impl TextureAssetData {
    /// Creates a new texture asset data wrapper from CPU texture data.
    pub fn new(data: CpuTextureData, canonical_path: Option<PathBuf>) -> Self {
        Self {
            data,
            canonical_path,
        }
    }
}

/// Generic generational slotmap-backed asset container for texture handles.
pub struct TextureStorage<T> {
    inner: SlotMap<AssetHandle, T>,
}

impl<T> TextureStorage<T> {
    /// Creates a new empty texture storage container.
    pub fn new() -> Self {
        Self {
            inner: SlotMap::default(),
        }
    }

    /// Creates a new storage container with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SlotMap::with_capacity_and_key(capacity),
        }
    }

    /// Inserts a new texture asset value into storage and returns its generational handle.
    pub fn insert(&mut self, value: T) -> AssetHandle {
        self.inner.insert(value)
    }

    /// Retrieves an immutable reference to the asset associated with the handle.
    pub fn get(&self, handle: AssetHandle) -> Option<&T> {
        self.inner.get(handle)
    }

    /// Retrieves a mutable reference to the asset associated with the handle.
    pub fn get_mut(&mut self, handle: AssetHandle) -> Option<&mut T> {
        self.inner.get_mut(handle)
    }

    /// Removes an asset from storage by handle, returning the removed value if present.
    pub fn remove(&mut self, handle: AssetHandle) -> Option<T> {
        self.inner.remove(handle)
    }

    /// Returns the number of assets stored.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if storage contains no assets.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clears all stored assets.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<T> Default for TextureStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Path deduplication map to prevent loading duplicate textures from the same canonical disk path.
#[derive(Debug, Default)]
pub struct TexturePathMap {
    map: HashMap<PathBuf, AssetHandle>,
}

impl TexturePathMap {
    /// Creates a new empty path deduplication map.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Inserts a mapping between canonical disk path and asset handle.
    pub fn insert(&mut self, path: PathBuf, handle: AssetHandle) {
        self.map.insert(path, handle);
    }

    /// Checks if a texture has already been loaded for a canonical path.
    pub fn get(&self, path: &Path) -> Option<AssetHandle> {
        self.map.get(path).copied()
    }

    /// Removes a path mapping.
    pub fn remove(&mut self, path: &Path) -> Option<AssetHandle> {
        self.map.remove(path)
    }

    /// Clears all path mappings.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}
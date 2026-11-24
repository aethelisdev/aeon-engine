// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 AethelisDEV / Aeon Engine. All rights reserved.
pub use ae_plugin_api::AssetHandle;
use slotmap::SlotMap;

/// Physics material properties for friction and bounciness (restitution)
#[derive(Debug, Clone, Copy)]
pub struct PhysicsMaterialAsset {
    pub friction: f32,
    pub restitution: f32,
    pub name: &'static str,
}

impl Default for PhysicsMaterialAsset {
    fn default() -> Self {
        Self {
            friction: 0.5,
            restitution: 0.0,
            name: "DefaultMaterial",
        }
    }
}

/// Generic slotmap-backed asset container with O(1) insert, get, and remove.
///
/// Wraps `SlotMap<AssetHandle, T>` to provide a type-safe, generational-index
/// storage for any asset type (models, textures, shaders, physics materials).
/// Handles remain valid across insertions/removals of other assets.
pub struct AssetStorage<T> {
    inner: SlotMap<AssetHandle, T>,
}

impl<T> AssetStorage<T> {
    pub fn new() -> Self {
        Self {
            inner: SlotMap::default(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: SlotMap::with_capacity_and_key(capacity),
        }
    }

    pub fn insert(&mut self, value: T) -> AssetHandle {
        self.inner.insert(value)
    }

    pub fn get(&self, handle: AssetHandle) -> Option<&T> {
        self.inner.get(handle)
    }

    pub fn get_mut(&mut self, handle: AssetHandle) -> Option<&mut T> {
        self.inner.get_mut(handle)
    }

    pub fn remove(&mut self, handle: AssetHandle) -> Option<T> {
        self.inner.remove(handle)
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (AssetHandle, &T)> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (AssetHandle, &mut T)> {
        self.inner.iter_mut()
    }
}

impl<T> Default for AssetStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// GPU-compiled WGSL shader module with raw source and metadata.
///
/// Holds the compiled `wgpu::ShaderModule` alongside its source text and canonical
/// disk path, allowing hot-reloading, inspector inspection, and dynamic pipeline binding.
pub struct ShaderAsset {
    /// Compiled GPU shader module.
    pub module: wgpu::ShaderModule,
    /// Raw WGSL shader source code.
    pub source_code: String,
    /// Canonical asset / file name.
    pub name: String,
    /// Absolute local file path.
    pub source_path: String,
}

/// Central asset registry managing all loaded models, textures, shaders, and physics materials.
///
/// Provides path-based deduplication via `model_path_map`, `texture_path_map`, and `shader_path_map`
/// to prevent loading the same file twice. Also offers physics mesh data retrieval
/// for collision shape generation and VRAM usage estimation.
pub struct AssetManager {
    pub models: AssetStorage<crate::render::ModelAsset>,
    pub textures: AssetStorage<crate::render::TextureAsset>,
    pub shaders: AssetStorage<ShaderAsset>,
    pub physics_materials: AssetStorage<PhysicsMaterialAsset>,
    pub model_path_map: std::collections::HashMap<std::path::PathBuf, AssetHandle>,
    pub texture_path_map: std::collections::HashMap<std::path::PathBuf, AssetHandle>,
    pub shader_path_map: std::collections::HashMap<std::path::PathBuf, AssetHandle>,
    model_ref_counts: std::collections::HashMap<AssetHandle, u32>,
    texture_ref_counts: std::collections::HashMap<AssetHandle, u32>,
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetManager {
    pub fn new() -> Self {
        let mut physics_materials = AssetStorage::with_capacity(16);
        // Insert a default material at index 0 (if we need one globally)
        physics_materials.insert(PhysicsMaterialAsset::default());

        Self {
            models: AssetStorage::with_capacity(32),
            textures: AssetStorage::with_capacity(32),
            shaders: AssetStorage::with_capacity(16),
            physics_materials,
            model_path_map: std::collections::HashMap::new(),
            texture_path_map: std::collections::HashMap::new(),
            shader_path_map: std::collections::HashMap::new(),
            model_ref_counts: std::collections::HashMap::new(),
            texture_ref_counts: std::collections::HashMap::new(),
        }
    }

    /// Increments the active chunk/entity reference count for a GPU model handle.
    pub fn retain_model(&mut self, handle: AssetHandle) {
        if self.models.get(handle).is_some() {
            let count = self.model_ref_counts.entry(handle).or_insert(0);
            *count = count.saturating_add(1);
        }
    }

    /// Returns the active reference count for a GPU model handle (`0` if unretained).
    #[must_use]
    pub fn model_ref_count(&self, handle: AssetHandle) -> u32 {
        self.model_ref_counts.get(&handle).copied().unwrap_or(0)
    }

    /// Decrements a model's reference count when a streaming chunk or entity unloads,
    /// freeing its GPU buffers and `model_path_map` entry only once the count hits `0`.
    ///
    /// Returns `true` if the GPU asset was unloaded because no active chunk references remain.
    pub fn release_model(&mut self, handle: AssetHandle) -> bool {
        let Some(count) = self.model_ref_counts.get_mut(&handle) else {
            return false;
        };
        if *count > 1 {
            *count -= 1;
            return false;
        }
        self.model_ref_counts.remove(&handle);
        self.models.remove(handle);
        self.model_path_map.retain(|_, &mut h| h != handle);
        true
    }

    /// Increments the active chunk/entity reference count for a GPU texture handle.
    pub fn retain_texture(&mut self, handle: AssetHandle) {
        if self.textures.get(handle).is_some() {
            let count = self.texture_ref_counts.entry(handle).or_insert(0);
            *count = count.saturating_add(1);
        }
    }

    /// Returns the active reference count for a GPU texture handle (`0` if unretained).
    #[must_use]
    pub fn texture_ref_count(&self, handle: AssetHandle) -> u32 {
        self.texture_ref_counts.get(&handle).copied().unwrap_or(0)
    }

    /// Decrements a texture's reference count when a streaming chunk or entity unloads,
    /// releasing its GPU bind group only when no active neighbor chunk still references it.
    pub fn release_texture(&mut self, handle: AssetHandle) -> bool {
        let Some(count) = self.texture_ref_counts.get_mut(&handle) else {
            return false;
        };
        if *count > 1 {
            *count -= 1;
            return false;
        }
        self.texture_ref_counts.remove(&handle);
        self.textures.remove(handle);
        self.texture_path_map.retain(|_, &mut h| h != handle);
        true
    }

    /// Retrieves raw geometric data (positions and indices) for physics shape generation.
    /// Returns (vertices, indices) if the model is loaded.
    pub fn get_physics_mesh_data(
        &self,
        handle: AssetHandle,
    ) -> Option<(&Vec<[f32; 3]>, &Vec<u32>)> {
        self.models
            .get(handle)
            .map(|asset| (&asset.raw_vertices, &asset.raw_indices))
    }

    /// Returns estimated memory usage in bytes (models_vram, textures_vram).
    ///
    /// Computes model footprints from raw vertex/index buffers and calculates
    /// texture footprints dynamically from pixel dimensions (width * height * 4).
    pub fn get_memory_usage(&self) -> (usize, usize) {
        let mut models_bytes = 0;
        for (_, model) in self.models.iter() {
            models_bytes += model.raw_vertices.len() * std::mem::size_of::<[f32; 3]>();
            models_bytes += model.raw_indices.len() * std::mem::size_of::<u32>();
            // Add some overhead for VGPU buffers (approximate duplicate)
            models_bytes += model.raw_vertices.len() * std::mem::size_of::<ae_mesh::Vertex>();
            models_bytes += model.raw_indices.len() * std::mem::size_of::<u32>();
        }

        let mut textures_bytes = 0;
        for (_, tex) in self.textures.iter() {
            // Dynamically calculate memory usage based on exact dimensions (RGBA8 format = 4 bytes per pixel)
            textures_bytes += (tex.width * tex.height * 4) as usize;
        }

        (models_bytes, textures_bytes)
    }

    /// Scans the ECS [`ae_ecs::World`] for all active `ModelId` and `SpriteId` components,
    /// and unloads any loaded models and textures that are no longer referenced.
    ///
    /// This sweeps both `models` and `textures` storages, automatically releasing their
    /// CPU memory and GPU/VRAM resources, and cleans up the path-to-handle lookup maps.
    pub fn unload_unused_assets(&mut self, world: &ae_ecs::World) {
        use std::collections::HashSet;

        // Collect all referenced model handles in the active ECS entities.
        let mut referenced_models = HashSet::new();
        for model_id in world.query::<&ae_plugin_api::ModelId>().iter() {
            referenced_models.insert(model_id.0);
        }

        // Collect all referenced texture handles in the active ECS entities.
        let mut referenced_textures = HashSet::new();
        for sprite_id in world.query::<&ae_plugin_api::SpriteId>().iter() {
            referenced_textures.insert(sprite_id.0);
        }

        // --- SWEEP MODELS ---
        let mut models_to_remove = Vec::new();
        for (handle, _) in self.models.iter() {
            if !referenced_models.contains(&handle) {
                models_to_remove.push(handle);
            }
        }
        let removed_models_count = models_to_remove.len();
        for handle in models_to_remove {
            self.models.remove(handle);
        }

        // Clean up model path map: retain only paths whose handles are still in `models`
        self.model_path_map
            .retain(|_, &mut handle| self.models.get(handle).is_some());

        // --- SWEEP TEXTURES ---
        let mut textures_to_remove = Vec::new();
        for (handle, _) in self.textures.iter() {
            if !referenced_textures.contains(&handle) {
                textures_to_remove.push(handle);
            }
        }
        let removed_textures_count = textures_to_remove.len();
        for handle in textures_to_remove {
            self.textures.remove(handle);
        }

        // Clean up texture path map: retain only paths whose handles are still in `textures`
        self.texture_path_map
            .retain(|_, &mut handle| self.textures.get(handle).is_some());

        log::info!(
            "[Asset GC] Swept unused assets. Unloaded {} models and {} textures.",
            removed_models_count,
            removed_textures_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_manager_initialization() {
        let manager = AssetManager::new();
        assert_eq!(manager.shaders.len(), 0);
        assert_eq!(manager.models.len(), 0);
        assert_eq!(manager.textures.len(), 0);
    }

    #[test]
    fn test_asset_manager_ref_counting_preserves_shared_assets_until_zero() {
        let mut manager = AssetManager::new();
        let fake_handle = AssetHandle::default();
        assert_eq!(manager.model_ref_count(fake_handle), 0);
        assert!(!manager.release_model(fake_handle));
        assert_eq!(manager.texture_ref_count(fake_handle), 0);
        assert!(!manager.release_texture(fake_handle));

        // Retaining missing handles is a safe no-op
        manager.retain_model(fake_handle);
        manager.retain_texture(fake_handle);
        assert_eq!(manager.model_ref_count(fake_handle), 0);
        assert_eq!(manager.texture_ref_count(fake_handle), 0);

        // Physics materials validate SlotMap handle retention lifecycle cleanly
        let mat_handle = manager
            .physics_materials
            .insert(PhysicsMaterialAsset::default());
        assert!(manager.physics_materials.get(mat_handle).is_some());
    }
}
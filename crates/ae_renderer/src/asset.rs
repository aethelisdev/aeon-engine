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
/// Wraps `SlotMap<AssetHandle, T>` to provide a type-safe, generational-index
/// storage for any asset type (models, textures, physics materials).
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

/// Central asset registry managing all loaded models, textures, and physics materials.
/// Provides path-based deduplication via `model_path_map` and `texture_path_map`
/// to prevent loading the same file twice. Also offers physics mesh data retrieval
/// for collision shape generation and VRAM usage estimation.
pub struct AssetManager {
    pub models: AssetStorage<crate::render::ModelAsset>,
    pub textures: AssetStorage<crate::render::TextureAsset>,
    pub physics_materials: AssetStorage<PhysicsMaterialAsset>,
    pub model_path_map: std::collections::HashMap<std::path::PathBuf, AssetHandle>,
    pub texture_path_map: std::collections::HashMap<std::path::PathBuf, AssetHandle>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut physics_materials = AssetStorage::with_capacity(16);
        // Insert a default material at index 0 (if we need one globally)
        physics_materials.insert(PhysicsMaterialAsset::default());

        Self {
            models: AssetStorage::with_capacity(32),
            textures: AssetStorage::with_capacity(32),
            physics_materials,
            model_path_map: std::collections::HashMap::new(),
            texture_path_map: std::collections::HashMap::new(),
        }
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
    /// Computes model footprints from raw vertex/index buffers and calculates
    /// texture footprints dynamically from pixel dimensions (width * height * 4).
    pub fn get_memory_usage(&self) -> (usize, usize) {
        let mut models_bytes = 0;
        for (_, model) in self.models.iter() {
            models_bytes += model.raw_vertices.len() * std::mem::size_of::<[f32; 3]>();
            models_bytes += model.raw_indices.len() * std::mem::size_of::<u32>();
            // Add some overhead for VGPU buffers (approximate duplicate)
            models_bytes += model.raw_vertices.len() * std::mem::size_of::<crate::render::Vertex>();
            models_bytes += model.raw_indices.len() * std::mem::size_of::<u32>();
        }

        let mut textures_bytes = 0;
        for (_, tex) in self.textures.iter() {
            // Dynamically calculate memory usage based on exact dimensions (RGBA8 format = 4 bytes per pixel)
            textures_bytes += (tex.width * tex.height * 4) as usize;
        }

        (models_bytes, textures_bytes)
    }

    /// Scans the ECS `hecs::World` for all active `ModelId` and `SpriteId` components,
    /// and unloads any loaded models and textures that are no longer referenced.
    /// This sweeps both `models` and `textures` storages, automatically releasing their
    /// CPU memory and GPU/VRAM resources, and cleans up the path-to-handle lookup maps.
    pub fn unload_unused_assets(&mut self, world: &hecs::World) {
        use std::collections::HashSet;

        // Collect all referenced model handles in the active ECS entities.
        let mut referenced_models = HashSet::new();
        for model_id in world.query::<&ae_core::ecs::ModelId>().iter() {
            referenced_models.insert(model_id.0);
        }

        // Collect all referenced texture handles in the active ECS entities.
        let mut referenced_textures = HashSet::new();
        for sprite_id in world.query::<&ae_core::ecs::SpriteId>().iter() {
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

impl<T> Default for AssetStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Intermediate result of async glTF/GLB file parsing.
/// Contains all vertex/index data ready for GPU upload, AABB bounds for
/// auto-scaling and culling, the canonical disk path for deduplication,
/// and the display name for the ECS entity.
pub struct ParsedModelData {
    pub all_vertices: Vec<crate::render::types::Vertex>,
    pub all_indices: Vec<u32>,
    pub raw_positions: Vec<[f32; 3]>,
    pub raw_skin_vertices: Vec<crate::render::types::SkinVertex>,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub canonical_path: std::path::PathBuf,
    pub original_path: String,
    pub final_name: String,
    pub skeleton: Option<ae_animation::Skeleton>,
    pub animations: Vec<ae_animation::AnimationClip>,
}

/// Verifies if a given file path is secure for the engine to load.
/// Performs multi-layered validation to block relative path traversal,
/// UNC network paths, protocol schemes, and null bytes.
pub fn is_safe_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.starts_with("\\\\") {
        return false;
    }
    if trimmed.contains("://") {
        return false;
    }
    if trimmed.contains('\0') {
        return false;
    }
    let p = std::path::Path::new(trimmed);
    for component in p.components() {
        if component == std::path::Component::ParentDir {
            return false;
        }
    }
    true
}